use std::collections::BTreeMap;

use glam::Vec4;
use rand::{SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, Normal, Uniform};

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::{clamp_to_unit_cube, fit_in_bounds, fit_in_bounds_simd},
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
        F: Fn(&[Vec4]) -> f32 + Sync,
    {
        let params = bounds.len();
        let popsize = self.popsize;
        let max_epoch = f64::ceil(maxiter as f64 / popsize as f64) as u64;
        let mut range_min: Vec<f32> = vec![0.0; params];
        let mut range_max: Vec<f32> = vec![0.0; params];
        for i in 0..params {
            range_min[i] = bounds[i][0];
            range_max[i] = bounds[i][1];
        }
        let mut current_positions: Vec<Vec<f32>> = vec![vec![0.0; params]; popsize];
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
        let random = Uniform::new_inclusive(0.0, 1.0).unwrap();
        for p in 0..popsize {
            for d in 0..params {
                current_positions[p][d] = random.sample(&mut rng);
            }
        }
        let mut best_positions: Vec<Vec<f32>> = vec![vec![0.0; params]; popsize];
        let mut best_residuals: Vec<f32> = vec![f32::INFINITY; popsize];
        let restart_tolerance = self.restart_tolerance;
        let sigma = self.sigma;
        let normal = Normal::new(0.0, sigma).unwrap();
        let mut current_residuals: Vec<f32> = vec![f32::INFINITY; popsize];
        let mut ind = 0;
        let mut history = OptimizationHistory {
            x: Vec::new(),
            f_x: Vec::new(),
        };
        if use_history {
            history.x.push(current_positions.clone());
            history.f_x.push(current_residuals.clone());
        }
        let mut current_epoch = 0;
        let popsize_distr = Uniform::new(0, popsize).unwrap();
        let self_instead_neighbour = self.self_instead_neighbour;
        for epoch in 0..max_epoch {
            for p in 0..popsize {
                current_residuals[p] = func(&fit_in_bounds_simd(
                    &current_positions[p],
                    &range_min,
                    &range_max,
                ))
            }
            for p in 0..popsize {
                if current_residuals[p] < best_residuals[p] {
                    best_residuals[p] = current_residuals[p];
                    best_positions[p] = current_positions[p].clone();
                    if best_residuals[p] < best_residuals[ind] {
                        ind = p;
                    }
                }
            }
            if use_history {
                history.x.push(best_positions.clone());
                history.f_x.push(best_residuals.clone());
            }
            current_epoch = epoch;
            if early_stop_callback.should_stop(&fit_in_bounds_simd(
                &best_positions[ind],
                &range_min,
                &range_max,
            )) {
                break;
            }
            for lhs in 0..popsize {
                for rhs in (lhs + 1)..popsize {
                    if best_residuals[lhs] != f32::INFINITY
                        && best_residuals[rhs] != f32::INFINITY
                        && f32::max(best_residuals[lhs].abs(), best_residuals[rhs].abs()) != 0.0
                        && f32::abs(
                            (best_residuals[lhs] - best_residuals[rhs])
                                / f32::max(best_residuals[lhs].abs(), best_residuals[rhs].abs()),
                        ) < restart_tolerance
                    {
                        if lhs != ind && rhs != ind {
                            if best_residuals[lhs] < best_residuals[rhs] {
                                best_residuals[rhs] = f32::INFINITY;
                                for d in 0..params {
                                    best_positions[rhs][d] = random.sample(&mut rng);
                                    current_positions[rhs][d] = random.sample(&mut rng);
                                }
                            } else {
                                best_residuals[lhs] = f32::INFINITY;
                                for d in 0..params {
                                    best_positions[lhs][d] = random.sample(&mut rng);
                                    current_positions[lhs][d] = random.sample(&mut rng);
                                }
                            }
                        } else if lhs != ind {
                            best_residuals[lhs] = f32::INFINITY;
                            for d in 0..params {
                                best_positions[lhs][d] = random.sample(&mut rng);
                                current_positions[lhs][d] = random.sample(&mut rng);
                            }
                        } else {
                            best_residuals[rhs] = f32::INFINITY;
                            for d in 0..params {
                                best_positions[rhs][d] = random.sample(&mut rng);
                                current_positions[rhs][d] = random.sample(&mut rng);
                            }
                        }
                    }
                }
            }
            for p in 0..popsize {
                for d in 0..params {
                    if random.sample(&mut rng) <= self_instead_neighbour {
                        current_positions[p][d] = clamp_to_unit_cube(
                            best_positions[p][d]
                                + normal.sample(&mut rng)
                                    * f32::abs(best_positions[p][d] - current_positions[p][d]),
                        )
                    } else {
                        let mut r = popsize_distr.sample(&mut rng);
                        while r == p {
                            r = popsize_distr.sample(&mut rng);
                        }
                        current_positions[p][d] = clamp_to_unit_cube(
                            best_positions[r][d]
                                + normal.sample(&mut rng)
                                    * f32::abs(best_positions[r][d] - current_positions[p][d]),
                        )
                    }
                }
            }
        }
        let result = OptimizerResult {
            x: fit_in_bounds(&best_positions[ind], &range_min, &range_max),
            f_x: best_residuals[ind],
            nfev: (current_epoch + 1) * popsize as u64,
            history: if use_history { Some(history) } else { None },
        };
        return result;
    }
}
