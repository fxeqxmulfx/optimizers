use std::collections::BTreeMap;

use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64Mcg;
use simd_vector::Vec8;

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::{clamp_to_unit_cube, fit_in_bounds, BoundsSimd},
};

/// Classic Differential Evolution (DE/rand/1/bin)
pub struct DE {
    pub popsize: usize,
    pub f: f32,
    pub cr: f32,
}

pub fn new_de(params: &BTreeMap<String, f32>) -> DE {
    DE {
        popsize: params["popsize"] as usize,
        f: params["f"],
        cr: params["cr"],
    }
}

impl Optimizer for DE {
    fn find_infimum<F>(
        &self,
        func: &F,
        bounds: &[[f32; 2]],
        maxiter: u64,
        seed: u64,
        use_history: bool,
        early_stop_callback: &EarlyStopCallback<&F>,
    ) -> OptimizerResult
    where
        F: Fn(&[Vec8]) -> f32 + Sync,
    {
        let dims = bounds.len();
        let popsize = self.popsize;
        let max_epoch = f64::ceil(maxiter as f64 / popsize as f64) as u64;
        let mut range_min = vec![0.0f32; dims];
        let mut range_max = vec![0.0f32; dims];
        for i in 0..dims {
            range_min[i] = bounds[i][0];
            range_max[i] = bounds[i][1];
        }
        let bounds_simd = BoundsSimd::new(&range_min, &range_max);
        let mut simd_buf = vec![Vec8::ZERO; bounds_simd.output_len()];

        let mut rng: Pcg64Mcg = SeedableRng::seed_from_u64(seed);
        let uniform_init = Uniform::new_inclusive(0.0f32, 1.0).unwrap();
        let uniform01 = Uniform::new(0.0f32, 1.0).unwrap();
        let dim_distr = Uniform::new(0, dims).unwrap();
        let pop_distr = Uniform::new(0, popsize).unwrap();

        // Flat storage: popsize * dims
        let mut pop = vec![0.0f32; popsize * dims];
        for v in &mut pop { *v = uniform_init.sample(&mut rng); }
        let mut fitness = vec![f32::INFINITY; popsize];

        for p in 0..popsize {
            bounds_simd.transform_into(&pop[p*dims..(p+1)*dims], &mut simd_buf);
            fitness[p] = func(&simd_buf);
        }

        let mut best_idx = 0;
        for p in 1..popsize {
            if fitness[p] < fitness[best_idx] {
                best_idx = p;
            }
        }

        let stop_residual = early_stop_callback.stop_residual();
        let mut history = OptimizationHistory {
            x: Vec::new(),
            f_x: Vec::new(),
        };
        if use_history {
            history.x.push((0..popsize).map(|p| pop[p*dims..(p+1)*dims].to_vec()).collect());
            history.f_x.push(fitness.clone());
        }

        let f_scale = self.f;
        let cr = self.cr;
        let mut trial = vec![0.0f32; dims];
        let mut current_epoch = 0;
        // Next generation buffers for generational replacement
        let mut next_pop = vec![0.0f32; popsize * dims];
        let mut next_fitness = fitness.clone();

        for epoch in 0..max_epoch {
            current_epoch = epoch;
            if fitness[best_idx] <= stop_residual {
                break;
            }

            for i in 0..popsize {
                // Select 3 distinct indices != i
                let mut r1 = pop_distr.sample(&mut rng);
                while r1 == i {
                    r1 = pop_distr.sample(&mut rng);
                }
                let mut r2 = pop_distr.sample(&mut rng);
                while r2 == i || r2 == r1 {
                    r2 = pop_distr.sample(&mut rng);
                }
                let mut r3 = pop_distr.sample(&mut rng);
                while r3 == i || r3 == r1 || r3 == r2 {
                    r3 = pop_distr.sample(&mut rng);
                }

                // Mutation + binomial crossover (read from old generation)
                let j_rand = dim_distr.sample(&mut rng);
                for d in 0..dims {
                    if d == j_rand || uniform01.sample(&mut rng) < cr {
                        trial[d] = clamp_to_unit_cube(
                            pop[r1 * dims + d] + f_scale * (pop[r2 * dims + d] - pop[r3 * dims + d]),
                        );
                    } else {
                        trial[d] = pop[i * dims + d];
                    }
                }

                // Selection — write to next generation
                bounds_simd.transform_into(&trial, &mut simd_buf);
                let trial_fitness = func(&simd_buf);
                if trial_fitness <= fitness[i] {
                    next_pop[i*dims..(i+1)*dims].copy_from_slice(&trial);
                    next_fitness[i] = trial_fitness;
                } else {
                    next_pop[i*dims..(i+1)*dims].copy_from_slice(&pop[i*dims..(i+1)*dims]);
                    next_fitness[i] = fitness[i];
                }
            }

            // Swap generations
            std::mem::swap(&mut pop, &mut next_pop);
            std::mem::swap(&mut fitness, &mut next_fitness);

            // Update best
            best_idx = 0;
            for p in 1..popsize {
                if fitness[p] < fitness[best_idx] {
                    best_idx = p;
                }
            }

            if use_history {
                history.x.push((0..popsize).map(|p| pop[p*dims..(p+1)*dims].to_vec()).collect());
                history.f_x.push(fitness.clone());
            }
        }

        OptimizerResult {
            x: fit_in_bounds(&pop[best_idx*dims..(best_idx+1)*dims], &range_min, &range_max),
            f_x: fitness[best_idx],
            nfev: (current_epoch + 1) * popsize as u64,
            history: if use_history { Some(history) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        early_stop_callback::EarlyStopCallback,
        functions::{shifted_sphere, SHIFTED_SPHERE_BOUNDS},
        utils::broadcast_simd,
    };

    #[test]
    fn test_de_finds_minimum() {
        let optimizer = DE {
            popsize: 32,
            f: 0.8,
            cr: 0.9,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 100_000, 0, false, &early_stop);
        assert!(result.f_x <= 0.01, "DE did not converge: f_x={}", result.f_x);
        assert!(result.nfev > 0);
    }

    #[test]
    fn test_de_with_history() {
        let optimizer = DE {
            popsize: 16,
            f: 0.8,
            cr: 0.9,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 1_000, 0, true, &early_stop);
        let history = result.history.unwrap();
        assert!(!history.x.is_empty());
        assert_eq!(history.x.len(), history.f_x.len());
    }

    #[test]
    fn test_new_de() {
        let mut params = BTreeMap::new();
        params.insert("popsize".to_string(), 50.0);
        params.insert("f".to_string(), 0.5);
        params.insert("cr".to_string(), 0.9);
        let optimizer = new_de(&params);
        assert_eq!(optimizer.popsize, 50);
        assert_eq!(optimizer.f, 0.5);
        assert_eq!(optimizer.cr, 0.9);
    }

    #[test]
    fn test_de_deterministic() {
        let optimizer = DE {
            popsize: 16,
            f: 0.8,
            cr: 0.9,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let r1 = optimizer.find_infimum(&func, &bounds, 10_000, 42, false, &early_stop);
        let r2 = optimizer.find_infimum(&func, &bounds, 10_000, 42, false, &early_stop);
        assert_eq!(r1.f_x, r2.f_x);
        assert_eq!(r1.nfev, r2.nfev);
    }
}
