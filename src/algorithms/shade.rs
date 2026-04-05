use std::collections::BTreeMap;

use rand::SeedableRng;
use rand_distr::{Cauchy, Distribution, Normal, Uniform};
use rand_pcg::Pcg64Mcg;
use simd_vector::Vec8;

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::{fit_in_bounds, BoundsSimd},
};

/// Midpoint boundary repair (JADE/SHADE standard):
/// if v < 0: v = (0 + x_parent) / 2
/// if v > 1: v = (1 + x_parent) / 2
#[inline]
fn midpoint_repair(v: f32, parent: f32) -> f32 {
    if v < 0.0 {
        parent / 2.0
    } else if v > 1.0 {
        (1.0 + parent) / 2.0
    } else {
        v
    }
}

/// SHADE: Success-History based Adaptive Differential Evolution
/// (Tanabe & Fukunaga, CEC 2013)
/// Uses current-to-pbest/1 mutation with adaptive F and CR,
/// external archive, midpoint boundary repair, and generational replacement.
pub struct SHADE {
    pub popsize: usize,
    pub h: usize,
    pub p_best_rate: f32,
}

pub fn new_shade(params: &BTreeMap<String, f32>) -> SHADE {
    SHADE {
        popsize: params["popsize"] as usize,
        h: params["h"] as usize,
        p_best_rate: params["p_best_rate"],
    }
}

impl Optimizer for SHADE {
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
        let h_distr = Uniform::new(0, self.h).unwrap();

        // p_i = rand[p_min, p_max] per paper eq. (20)
        // paper uses p_max=0.2, we use p_best_rate as tunable upper bound
        let p_min = 2.0 / popsize as f32;
        let p_max = self.p_best_rate.max(p_min);
        let p_distr = Uniform::new_inclusive(p_min, p_max).unwrap();

        // Flat storage: popsize * dims
        let mut pop = vec![0.0f32; popsize * dims];
        for v in &mut pop {
            *v = uniform_init.sample(&mut rng);
        }
        let mut fitness: Vec<f32> = vec![f32::INFINITY; popsize];

        for p in 0..popsize {
            bounds_simd.transform_into(&pop[p * dims..(p + 1) * dims], &mut simd_buf);
            fitness[p] = func(&simd_buf);
        }

        let mut best_idx = 0;
        for p in 1..popsize {
            if fitness[p] < fitness[best_idx] {
                best_idx = p;
            }
        }

        // History of successful F and CR
        let h = self.h;
        let mut m_f: Vec<f32> = vec![0.5; h];
        let mut m_cr: Vec<f32> = vec![0.5; h];
        let mut k = 0usize;

        // External archive of replaced individuals (bounded to popsize)
        let mut archive: Vec<f32> = Vec::with_capacity(popsize * dims);
        let mut archive_len: usize = 0;

        let stop_residual = early_stop_callback.stop_residual();
        let mut history = OptimizationHistory {
            x: Vec::new(),
            f_x: Vec::new(),
        };
        if use_history {
            history
                .x
                .push((0..popsize).map(|p| pop[p * dims..(p + 1) * dims].to_vec()).collect());
            history.f_x.push(fitness.clone());
        }

        let mut current_epoch = 0;
        let mut sorted_idx: Vec<usize> = (0..popsize).collect();
        let mut s_f: Vec<f32> = Vec::with_capacity(popsize);
        let mut s_cr: Vec<f32> = Vec::with_capacity(popsize);
        let mut s_delta: Vec<f32> = Vec::with_capacity(popsize);

        // Generational replacement buffers
        let mut trials = vec![0.0f32; popsize * dims];
        let mut trial_fitness: Vec<f32> = vec![f32::INFINITY; popsize];
        let mut trial_f: Vec<f32> = vec![0.0; popsize];
        let mut trial_cr: Vec<f32> = vec![0.0; popsize];

        for epoch in 0..max_epoch {
            current_epoch = epoch;
            if fitness[best_idx] <= stop_residual {
                break;
            }

            sorted_idx.iter_mut().enumerate().for_each(|(i, v)| *v = i);
            sorted_idx.sort_by(|&a, &b| fitness[a].total_cmp(&fitness[b]));

            // Phase 1: Generate all trial vectors from current generation
            for i in 0..popsize {
                // Sample F and CR from history
                let r_idx = h_distr.sample(&mut rng);
                let cauchy = Cauchy::new(m_f[r_idx] as f64, 0.1).unwrap();
                let mut fi = cauchy.sample(&mut rng) as f32;
                while fi <= 0.0 {
                    fi = cauchy.sample(&mut rng) as f32;
                }
                if fi > 1.0 {
                    fi = 1.0;
                }

                let normal = Normal::new(m_cr[r_idx] as f64, 0.1).unwrap();
                let cri = (normal.sample(&mut rng) as f32).clamp(0.0, 1.0);

                trial_f[i] = fi;
                trial_cr[i] = cri;

                // Random p per individual (paper eq. 20)
                let pi = p_distr.sample(&mut rng);
                let p_num = ((popsize as f32 * pi).ceil() as usize).max(2);
                let pbest = sorted_idx[Uniform::new(0, p_num).unwrap().sample(&mut rng)];

                // Select r1 != i from population
                let mut r1 = pop_distr.sample(&mut rng);
                while r1 == i {
                    r1 = pop_distr.sample(&mut rng);
                }

                // Select r2 != i, r1 from population ∪ archive
                let union_size = popsize + archive_len;
                let mut r2 = Uniform::new(0, union_size).unwrap().sample(&mut rng);
                while r2 == i || r2 == r1 {
                    r2 = Uniform::new(0, union_size).unwrap().sample(&mut rng);
                }

                // current-to-pbest/1 mutation + binomial crossover
                let j_rand = dim_distr.sample(&mut rng);
                let io = i * dims;
                for d in 0..dims {
                    if d == j_rand || uniform01.sample(&mut rng) < cri {
                        let r2_d = if r2 < popsize {
                            pop[r2 * dims + d]
                        } else {
                            archive[(r2 - popsize) * dims + d]
                        };
                        trials[io + d] = midpoint_repair(
                            pop[io + d]
                                + fi * (pop[pbest * dims + d] - pop[io + d])
                                + fi * (pop[r1 * dims + d] - r2_d),
                            pop[io + d],
                        );
                    } else {
                        trials[io + d] = pop[io + d];
                    }
                }

                // Evaluate trial
                bounds_simd.transform_into(&trials[io..io + dims], &mut simd_buf);
                trial_fitness[i] = func(&simd_buf);
            }

            // Phase 2: Selection (generational replacement)
            s_f.clear();
            s_cr.clear();
            s_delta.clear();

            for i in 0..popsize {
                if trial_fitness[i] <= fitness[i] {
                    if trial_fitness[i] < fitness[i] {
                        // Strict improvement: record success and archive parent
                        s_f.push(trial_f[i]);
                        s_cr.push(trial_cr[i]);
                        s_delta.push(fitness[i] - trial_fitness[i]);
                        if archive_len < popsize {
                            archive.extend_from_slice(&pop[i * dims..(i + 1) * dims]);
                            archive_len += 1;
                        } else {
                            let arc_idx =
                                Uniform::new(0, archive_len).unwrap().sample(&mut rng);
                            archive[arc_idx * dims..(arc_idx + 1) * dims]
                                .copy_from_slice(&pop[i * dims..(i + 1) * dims]);
                        }
                    }
                    // Replace (including equal fitness)
                    pop[i * dims..(i + 1) * dims]
                        .copy_from_slice(&trials[i * dims..(i + 1) * dims]);
                    fitness[i] = trial_fitness[i];
                }
            }

            // Update best
            best_idx = 0;
            for p in 1..popsize {
                if fitness[p] < fitness[best_idx] {
                    best_idx = p;
                }
            }

            // Update history with weighted Lehmer mean
            if !s_f.is_empty() {
                let sum_delta: f32 = s_delta.iter().sum();
                let weights: Vec<f32> = s_delta.iter().map(|d| d / sum_delta).collect();

                let num: f32 = weights.iter().zip(&s_f).map(|(w, f)| w * f * f).sum();
                let den: f32 = weights.iter().zip(&s_f).map(|(w, f)| w * f).sum();
                if den > 0.0 {
                    m_f[k] = num / den;
                }

                let mean_cr: f32 = weights.iter().zip(&s_cr).map(|(w, cr)| w * cr).sum();
                m_cr[k] = mean_cr;

                k = (k + 1) % h;
            }

            if use_history {
                history.x.push(
                    (0..popsize)
                        .map(|p| pop[p * dims..(p + 1) * dims].to_vec())
                        .collect(),
                );
                history.f_x.push(fitness.clone());
            }
        }

        OptimizerResult {
            x: fit_in_bounds(&pop[best_idx * dims..(best_idx + 1) * dims], &range_min, &range_max),
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
    fn test_shade_finds_minimum() {
        let optimizer = SHADE {
            popsize: 32,
            h: 10,
            p_best_rate: 0.1,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 100_000, 0, false, &early_stop);
        assert!(result.f_x <= 0.01, "SHADE did not converge: f_x={}", result.f_x);
        assert!(result.nfev > 0);
    }

    #[test]
    fn test_shade_with_history() {
        let optimizer = SHADE {
            popsize: 16,
            h: 5,
            p_best_rate: 0.2,
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
    fn test_new_shade() {
        let mut params = BTreeMap::new();
        params.insert("popsize".to_string(), 50.0);
        params.insert("h".to_string(), 10.0);
        params.insert("p_best_rate".to_string(), 0.1);
        let optimizer = new_shade(&params);
        assert_eq!(optimizer.popsize, 50);
        assert_eq!(optimizer.h, 10);
        assert_eq!(optimizer.p_best_rate, 0.1);
    }

    #[test]
    fn test_shade_deterministic() {
        let optimizer = SHADE {
            popsize: 16,
            h: 10,
            p_best_rate: 0.1,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let r1 = optimizer.find_infimum(&func, &bounds, 10_000, 42, false, &early_stop);
        let r2 = optimizer.find_infimum(&func, &bounds, 10_000, 42, false, &early_stop);
        assert_eq!(r1.f_x, r2.f_x);
        assert_eq!(r1.nfev, r2.nfev);
    }

    #[test]
    fn test_midpoint_repair() {
        assert_eq!(midpoint_repair(0.5, 0.3), 0.5); // in bounds
        assert_eq!(midpoint_repair(-0.2, 0.4), 0.2); // below: (0 + 0.4) / 2
        assert_eq!(midpoint_repair(1.3, 0.6), 0.8);  // above: (1 + 0.6) / 2
    }
}
