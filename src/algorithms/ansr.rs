use rand::{SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, Normal, Uniform};

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::fit_in_bounds,
};

pub struct ANSR {
    pub popsize: usize,
    pub tol: f32,
    pub sigma: f32,
    pub self_instead_neighbour: f32,
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
        F: Fn(&[f32]) -> f32 + Sync,
    {
        let params = bounds.len();
        let popsize = self.popsize;
        let max_epoch = f64::round(maxiter as f64 / popsize as f64) as u64;
        let mut range_min: Vec<f32> = vec![0.0; params];
        let mut range_max: Vec<f32> = vec![0.0; params];
        for i in 0..params {
            range_min[i] = bounds[i][0];
            range_max[i] = bounds[i][1];
        }
        let mut current_positions: Vec<Vec<f32>> = vec![vec![0.0; params]; popsize];
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
        let random_distr = Uniform::new_inclusive(0.0, 1.0).unwrap();
        for p in 0..popsize {
            for d in 0..params {
                current_positions[p][d] = random_distr.sample(&mut rng);
            }
        }
        let mut best_positions: Vec<Vec<f32>> = vec![vec![0.0; params]; popsize];
        let mut best_errors: Vec<f32> = vec![f32::INFINITY; popsize];
        let tol = self.tol;
        let sigma = self.sigma;
        let normal = Normal::new(0.0, sigma).unwrap();
        let mut current_errors: Vec<f32> = vec![0.0; popsize];
        let mut ind = 0;
        let mut history = OptimizationHistory {
            x: Vec::new(),
            f_x: Vec::new(),
        };
        if use_history {
            history.x.push(current_positions.clone());
            history.f_x.push(current_errors.clone());
        }
        let mut current_epoch = 0;
        let popsize_distr = Uniform::new(0, popsize).unwrap();
        let self_instead_neighbour = self.self_instead_neighbour;
        for epoch in 0..max_epoch {
            for p in 0..popsize {
                current_errors[p] = func(&fit_in_bounds(
                    &current_positions[p],
                    &range_min,
                    &range_max,
                ))
            }
            for p in 0..popsize {
                if current_errors[p] < best_errors[p] {
                    best_errors[p] = current_errors[p];
                    best_positions[p] = current_positions[p].clone();
                    if best_errors[p] < best_errors[ind] {
                        ind = p;
                    }
                }
            }
            if use_history {
                history.x.push(best_positions.clone());
                history.f_x.push(best_errors.clone());
            }
            current_epoch = epoch;
            if early_stop_callback.should_stop(&fit_in_bounds(
                &best_positions[ind],
                &range_min,
                &range_max,
            )) {
                break;
            }
            for p in 0..popsize {
                for r in (p + 1)..popsize {
                    if best_errors[p] != f32::INFINITY
                        && best_errors[r] != f32::INFINITY
                        && f32::max(best_errors[p], best_errors[r]) != 0.0
                        && f32::abs(
                            (best_errors[p] - best_errors[r])
                                / f32::max(best_errors[p].abs(), best_errors[r].abs()),
                        ) < tol
                    {
                        for d in 0..params {
                            best_positions[r][d] = random_distr.sample(&mut rng);
                            best_errors[r] = f32::INFINITY
                        }
                    }
                }
            }
            for p in 0..popsize {
                for d in 0..params {
                    if self_instead_neighbour <= random_distr.sample(&mut rng) {
                        current_positions[p][d] = f32::min(
                            f32::max(
                                best_positions[p][d]
                                    + normal.sample(&mut rng)
                                        * f32::abs(best_positions[p][d] - current_positions[p][d]),
                                0.0,
                            ),
                            1.0,
                        )
                    } else {
                        let mut r = popsize_distr.sample(&mut rng);
                        while r == p {
                            r = popsize_distr.sample(&mut rng);
                        }
                        current_positions[p][d] = f32::min(
                            f32::max(
                                best_positions[r][d]
                                    + normal.sample(&mut rng)
                                        * f32::abs(best_positions[r][d] - current_positions[p][d]),
                                0.0,
                            ),
                            1.0,
                        )
                    }
                }
            }
        }
        let result = OptimizerResult {
            x: best_positions[ind].clone(),
            f_x: best_errors[ind],
            nfev: current_epoch * popsize as u64,
            history: if use_history { Some(history) } else { None },
        };
        return result;
    }
}
