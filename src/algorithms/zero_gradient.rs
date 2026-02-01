use glam::Vec4;
use rand::{SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, Uniform};

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::{clamp_to_unit_cube, fit_in_bounds, fit_in_bounds_simd},
};

pub fn zero_gradient<F>(
    func: &F,
    range_min: &Vec<f32>,
    range_max: &Vec<f32>,
    current_positions: &Vec<f32>,
    init_jump: f32,
    use_history: bool,
) -> OptimizerResult
where
    F: Fn(&[Vec4]) -> f32 + Sync,
{
    let mut history = OptimizationHistory {
        x: Vec::new(),
        f_x: Vec::new(),
    };
    let mut current_positions = current_positions.clone();
    let mut current_residual = func(&fit_in_bounds_simd(
        &current_positions,
        range_min,
        range_max,
    ));
    if use_history {
        history.x.push([current_positions.clone()].to_vec());
        history.f_x.push([current_residual].to_vec());
    }
    let mut nfev = 1;
    for p in 0..current_positions.len() {
        let current_coordinate = current_positions[p];
        let mut multiplicator = 1.0;
        let lhs_coordinate = clamp_to_unit_cube(current_coordinate - init_jump * multiplicator);
        current_positions[p] = lhs_coordinate;
        let lhs_residual = func(&fit_in_bounds_simd(
            &current_positions,
            range_min,
            range_max,
        ));
        if use_history {
            history.x.push([current_positions.clone()].to_vec());
            history.f_x.push([lhs_residual].to_vec());
        }
        nfev += 1;
        let rhs_coordinate = clamp_to_unit_cube(current_coordinate + init_jump * multiplicator);
        current_positions[p] = rhs_coordinate;
        let rhs_residual = func(&fit_in_bounds_simd(
            &current_positions,
            range_min,
            range_max,
        ));
        if use_history {
            history.x.push([current_positions.clone()].to_vec());
            history.f_x.push([rhs_residual].to_vec());
        }
        nfev += 1;
        if current_residual < lhs_residual && current_residual < rhs_residual {
            current_positions[p] = current_coordinate;
            continue;
        }
        let mut turn = if lhs_residual < rhs_residual {
            current_positions[p] = lhs_coordinate;
            current_residual = lhs_residual;
            -1.0
        } else {
            current_positions[p] = rhs_coordinate;
            current_residual = rhs_residual;
            1.0
        };
        multiplicator *= 2.0;
        loop {
            let current_coordinate = current_positions[p];
            let new_coordinate =
                clamp_to_unit_cube(current_coordinate + init_jump * multiplicator * turn);
            current_positions[p] = new_coordinate;
            let new_residual = func(&fit_in_bounds_simd(
                &current_positions,
                range_min,
                range_max,
            ));
            if use_history {
                history.x.push([current_positions.clone()].to_vec());
                history.f_x.push([new_residual].to_vec());
            }
            nfev += 1;
            if new_residual > current_residual {
                current_positions[p] = current_coordinate;
                break;
            }
            if new_coordinate == 0.0 || new_coordinate == 1.0 {
                break;
            }
            current_residual = new_residual;
            multiplicator *= 2.0;
        }
        multiplicator /= 2.0;
        let current_coordinate = current_positions[p];
        let new_coordinate =
            clamp_to_unit_cube(current_coordinate + init_jump * multiplicator * turn);
        current_positions[p] = new_coordinate;
        let new_residual = func(&fit_in_bounds_simd(
            &current_positions,
            range_min,
            range_max,
        ));
        if use_history {
            history.x.push([current_positions.clone()].to_vec());
            history.f_x.push([new_residual].to_vec());
        }
        nfev += 1;
        if new_residual > current_residual {
            current_positions[p] = current_coordinate;
        } else {
            current_residual = new_residual;
        }
        multiplicator /= 2.0;
        loop {
            let current_coordinate = current_positions[p];
            let add = init_jump * multiplicator;
            if add < f32::EPSILON {
                break;
            }
            let lhs_coordinate = clamp_to_unit_cube(current_coordinate - add);
            current_positions[p] = lhs_coordinate;
            let lhs_residual = func(&fit_in_bounds_simd(
                &current_positions,
                range_min,
                range_max,
            ));
            if use_history {
                history.x.push([current_positions.clone()].to_vec());
                history.f_x.push([lhs_residual].to_vec());
            }
            nfev += 1;
            let rhs_coordinate = clamp_to_unit_cube(current_coordinate + add);
            current_positions[p] = rhs_coordinate;
            let rhs_residual = func(&fit_in_bounds_simd(
                &current_positions,
                range_min,
                range_max,
            ));
            if use_history {
                history.x.push([current_positions.clone()].to_vec());
                history.f_x.push([rhs_residual].to_vec());
            }
            nfev += 1;
            multiplicator /= 2.0;
            if current_residual < lhs_residual && current_residual < rhs_residual {
                current_positions[p] = current_coordinate;
                continue;
            }
            if lhs_residual < rhs_residual {
                current_positions[p] = lhs_coordinate;
                current_residual = lhs_residual;
                turn = 1.0;
            } else {
                current_positions[p] = rhs_coordinate;
                current_residual = rhs_residual;
                turn = -1.0;
            }
            break;
        }
        loop {
            let current_coordinate = current_positions[p];
            let add = init_jump * multiplicator * turn;
            if add.abs() < f32::EPSILON {
                break;
            }
            let new_coordinate =
                clamp_to_unit_cube(current_coordinate + init_jump * multiplicator * turn);
            current_positions[p] = new_coordinate;
            let new_residual = func(&fit_in_bounds_simd(
                &current_positions,
                range_min,
                range_max,
            ));
            if use_history {
                history.x.push([current_positions.clone()].to_vec());
                history.f_x.push([new_residual].to_vec());
            }
            nfev += 1;
            multiplicator /= 2.0;
            if new_residual > current_residual {
                current_positions[p] = current_coordinate;
                continue;
            }
            current_residual = new_residual;
            turn = -turn;
        }
    }
    OptimizerResult {
        x: fit_in_bounds(&current_positions, range_min, range_max),
        f_x: current_residual,
        nfev: nfev,
        history: if use_history { Some(history) } else { None },
    }
}

pub struct ZeroGradient {
    pub init_jump: f32,
}

impl Optimizer for ZeroGradient {
    fn find_infimum<F>(
        &self,
        func: &F,
        bounds: &[[f32; 2]],
        _maxiter: u64,
        seed: u64,
        use_history: bool,
        _early_stop_callback: &EarlyStopCallback<&F>,
    ) -> OptimizerResult
    where
        F: Fn(&[Vec4]) -> f32 + Sync,
    {
        let params = bounds.len();
        let mut range_min: Vec<f32> = vec![0.0; params];
        let mut range_max: Vec<f32> = vec![0.0; params];
        for i in 0..params {
            range_min[i] = bounds[i][0];
            range_max[i] = bounds[i][1];
        }
        let mut current_positions: Vec<f32> = vec![0.0; params];
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
        let random = Uniform::new_inclusive(0.0, 1.0).unwrap();
        for d in 0..params {
            current_positions[d] = random.sample(&mut rng);
        }
        let init_jump = self.init_jump;
        let result = zero_gradient(
            func,
            &range_min,
            &range_max,
            &current_positions,
            init_jump,
            use_history,
        );
        result
    }
}
