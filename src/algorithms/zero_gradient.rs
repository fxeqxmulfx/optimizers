use simd_vector::Vec8;
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
    F: Fn(&[Vec8]) -> f32 + Sync,
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
        F: Fn(&[Vec8]) -> f32 + Sync,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        early_stop_callback::EarlyStopCallback,
        functions::{sphere, SPHERE_BOUNDS},
        utils::broadcast_simd,
    };

    #[test]
    fn test_zero_gradient_finds_minimum() {
        let optimizer = ZeroGradient { init_jump: 0.1 };
        let func = broadcast_simd(sphere);
        let bounds = SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 1_000_000, 0, false, &early_stop);
        assert!(result.f_x < 0.1, "ZeroGradient f_x={}", result.f_x);
        assert!(result.nfev > 0);
    }

    #[test]
    fn test_zero_gradient_with_history() {
        let optimizer = ZeroGradient { init_jump: 0.1 };
        let func = broadcast_simd(sphere);
        let bounds = SPHERE_BOUNDS.repeat(8);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = optimizer.find_infimum(&func, &bounds, 1_000_000, 0, true, &early_stop);
        let history = result.history.unwrap();
        assert!(!history.x.is_empty());
        assert!(!history.f_x.is_empty());
    }

    #[test]
    fn test_zero_gradient_bisection_exhaustion() {
        // Staircase function: floor(x*100)/100 — has flat regions.
        // From a position on a flat region, both +/- small steps give same or worse value.
        // The doubling loop will find improvement by jumping to a different stair,
        // then the bisection loop will exhaust because near the stair edge both
        // directions are equally flat → continue until epsilon.
        use simd_vector::fast::FastMath;
        let func = |x: &[Vec8]| -> f32 {
            x.iter()
                .map(|v| ((*v * Vec8::splat(100.0)).floor() / Vec8::splat(100.0)).sum())
                .sum::<f32>()
                / (x.len() * 8) as f32
        };
        let range_min = vec![0.0; 16];
        let range_max = vec![10.0; 16];
        let positions = vec![0.55; 16];
        let result = zero_gradient(&func, &range_min, &range_max, &positions, 0.1, false);
        assert!(result.f_x.is_finite());
    }

    #[test]
    fn test_zero_gradient_standalone() {
        let func = broadcast_simd(sphere);
        let range_min = vec![-5.0; 16];
        let range_max = vec![5.0; 16];
        let positions = vec![0.5; 16];
        let result = zero_gradient(&func, &range_min, &range_max, &positions, 0.1, false);
        assert!(result.f_x < 0.1);
        assert!(result.history.is_none());
    }

    #[test]
    fn test_zero_gradient_boundary_clamp() {
        // f(x) = sum(x_i): monotonically decreasing toward boundary 0.
        // Start at 0.5, init_jump=0.4: the doubling loop will push coordinates
        // past 0 where clamp_to_unit_cube returns exactly 0.0, hitting the break.
        use simd_vector::fast::FastMath;
        let func = |x: &[Vec8]| -> f32 {
            x.iter().map(|v| v.sum()).sum::<f32>() / (x.len() * 8) as f32
        };
        let range_min = vec![0.0; 16];
        let range_max = vec![1.0; 16];
        let positions = vec![0.5; 16];
        let result = zero_gradient(&func, &range_min, &range_max, &positions, 0.4, true);
        assert!(result.f_x < 0.05);
    }

    #[test]
    fn test_zero_gradient_standalone_with_history() {
        let func = broadcast_simd(sphere);
        let range_min = vec![-5.0; 16];
        let range_max = vec![5.0; 16];
        let positions = vec![0.5; 16];
        let result = zero_gradient(&func, &range_min, &range_max, &positions, 0.1, true);
        assert!(result.history.is_some());
    }
}
