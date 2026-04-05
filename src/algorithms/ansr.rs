use std::collections::BTreeMap;

use rand::SeedableRng;
use rand_distr::{Distribution, Normal, Uniform};
use rand_pcg::Pcg64Mcg;
use simd_vector::Vec8;

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::{clamp_to_unit_cube, fit_in_bounds, BoundsSimd},
};

pub struct ANSR {
    pub popsize: usize,
    pub restart_tolerance: f32,
    pub sigma: f32,
    pub self_instead_neighbour: f32,
}

pub fn new_ansr(params: &BTreeMap<String, f32>) -> ANSR {
    ANSR {
        popsize: params["popsize"] as usize,
        restart_tolerance: params["restart_tolerance"],
        sigma: params["sigma"],
        self_instead_neighbour: params["self_instead_neighbour"],
    }
}

impl Optimizer for ANSR {
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
        let mut range_min: Vec<f32> = vec![0.0; dims];
        let mut range_max: Vec<f32> = vec![0.0; dims];
        for i in 0..dims {
            range_min[i] = bounds[i][0];
            range_max[i] = bounds[i][1];
        }
        let bounds_simd = BoundsSimd::new(&range_min, &range_max);
        let mut simd_buf = vec![Vec8::ZERO; bounds_simd.output_len()];
        let mut rng: Pcg64Mcg = SeedableRng::seed_from_u64(seed);
        let random = Uniform::new_inclusive(0.0, 1.0).unwrap();

        // Flat storage: popsize * dims
        let mut cur = vec![0.0f32; popsize * dims];
        for v in &mut cur { *v = random.sample(&mut rng); }
        let mut best = vec![0.0f32; popsize * dims];
        let mut best_f = vec![f32::INFINITY; popsize];
        let mut cur_f = vec![f32::INFINITY; popsize];

        let restart_tolerance = self.restart_tolerance;
        let sigma = self.sigma;
        let normal = Normal::new(0.0, sigma).unwrap();
        let mut ind = 0;
        let stop_residual = early_stop_callback.stop_residual();
        let mut history = OptimizationHistory { x: Vec::new(), f_x: Vec::new() };
        if use_history {
            history.x.push((0..popsize).map(|p| cur[p*dims..(p+1)*dims].to_vec()).collect());
            history.f_x.push(cur_f.clone());
        }
        let mut current_epoch = 0;
        let popsize_distr = Uniform::new(0, popsize).unwrap();
        let self_instead_neighbour = self.self_instead_neighbour;

        for epoch in 0..max_epoch {
            for p in 0..popsize {
                bounds_simd.transform_into(&cur[p*dims..(p+1)*dims], &mut simd_buf);
                cur_f[p] = func(&simd_buf);
            }
            for p in 0..popsize {
                if cur_f[p] < best_f[p] {
                    best_f[p] = cur_f[p];
                    best[p*dims..(p+1)*dims].copy_from_slice(&cur[p*dims..(p+1)*dims]);
                    if best_f[p] < best_f[ind] {
                        ind = p;
                    }
                }
            }
            if use_history {
                history.x.push((0..popsize).map(|p| best[p*dims..(p+1)*dims].to_vec()).collect());
                history.f_x.push(best_f.clone());
            }
            current_epoch = epoch;
            if best_f[ind] <= stop_residual {
                break;
            }
            for lhs in 0..popsize {
                if best_f[lhs] == f32::INFINITY {
                    continue;
                }
                for rhs in (lhs + 1)..popsize {
                    if best_f[rhs] == f32::INFINITY {
                        continue;
                    }
                    let (min_residual, max_residual) = if best_f[lhs] <= best_f[rhs] {
                        (best_f[lhs], best_f[rhs])
                    } else {
                        (best_f[rhs], best_f[lhs])
                    };
                    if max_residual != 0.0
                        && (max_residual - min_residual) / max_residual < restart_tolerance
                    {
                        let loser = if lhs == ind || (rhs != ind && best_f[lhs] < best_f[rhs]) {
                            rhs
                        } else {
                            lhs
                        };
                        best_f[loser] = f32::INFINITY;
                        let lo = loser * dims;
                        for d in 0..dims {
                            best[lo + d] = random.sample(&mut rng);
                            cur[lo + d] = random.sample(&mut rng);
                        }
                    }
                }
            }
            for p in 0..popsize {
                let po = p * dims;
                for d in 0..dims {
                    if random.sample(&mut rng) <= self_instead_neighbour {
                        cur[po + d] = clamp_to_unit_cube(
                            best[po + d]
                                + normal.sample(&mut rng)
                                    * f32::abs(best[po + d] - cur[po + d]),
                        )
                    } else {
                        let mut r = popsize_distr.sample(&mut rng);
                        while r == p { r = popsize_distr.sample(&mut rng); }
                        let ro = r * dims;
                        cur[po + d] = clamp_to_unit_cube(
                            best[ro + d]
                                + normal.sample(&mut rng)
                                    * f32::abs(best[ro + d] - cur[po + d]),
                        )
                    }
                }
            }
        }
        OptimizerResult {
            x: fit_in_bounds(&best[ind*dims..(ind+1)*dims], &range_min, &range_max),
            f_x: best_f[ind],
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
    fn test_ansr_finds_minimum() {
        let optimizer = ANSR {
            popsize: 4,
            restart_tolerance: 0.01,
            sigma: 0.05,
            self_instead_neighbour: 0.9,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 100_000, 0, false, &early_stop);
        assert!(result.f_x <= 0.01, "ANSR did not converge: f_x={}", result.f_x);
        assert!(result.nfev > 0);
        assert!(result.history.is_none());
    }

    #[test]
    fn test_ansr_with_history() {
        let optimizer = ANSR {
            popsize: 4,
            restart_tolerance: 0.01,
            sigma: 0.05,
            self_instead_neighbour: 0.9,
        };
        let func = broadcast_simd(shifted_sphere);
        let bounds = SHIFTED_SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 1_000, 0, true, &early_stop);
        let history = result.history.unwrap();
        assert!(!history.x.is_empty());
        assert!(!history.f_x.is_empty());
        assert_eq!(history.x.len(), history.f_x.len());
    }

    #[test]
    fn test_new_ansr() {
        let mut params = BTreeMap::new();
        params.insert("popsize".to_string(), 8.0);
        params.insert("restart_tolerance".to_string(), 0.001);
        params.insert("sigma".to_string(), 0.1);
        params.insert("self_instead_neighbour".to_string(), 0.5);
        let optimizer = new_ansr(&params);
        assert_eq!(optimizer.popsize, 8);
        assert_eq!(optimizer.sigma, 0.1);
    }

    #[test]
    fn test_ansr_deterministic() {
        let optimizer = ANSR {
            popsize: 4,
            restart_tolerance: 0.01,
            sigma: 0.05,
            self_instead_neighbour: 0.9,
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
