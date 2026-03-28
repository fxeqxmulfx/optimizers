use std::{collections::BTreeMap, mem::swap};

use rand::SeedableRng;
use rand_distr::{Distribution, Normal, Uniform};
use rand_pcg::Pcg64Mcg;
use simd_vector::Vec8;

use crate::{
    early_stop_callback::EarlyStopCallback,
    optimizer::{OptimizationHistory, Optimizer, OptimizerResult},
    utils::{fit_in_bounds, fit_in_bounds_simd},
};

fn wrap_to_unit_cube(mut v: f32) -> f32 {
    v = v % 1.0;
    if v < 0.0 { v += 1.0; }
    v
}

/// AnsrV2 — Adaptive Neighbourhood Search with Restarts V2
///
/// # Parameter bounds and justification
///
/// All operations are performed in the unit cube [0, 1]^D. Positions are
/// clamped via `clamp_to_unit_cube` and mapped to the original bounds only
/// for function evaluation via `fit_in_bounds`.
///
/// ## popsize ∈ {2, 3, ...}  (natural number ≥ 2), tune range: [D/2, 2D]
///   The restart mechanism compares pairs (lhs, rhs) where lhs < rhs < popsize,
///   so popsize ≥ 2 is required for at least one pair. The neighbour perturbation
///   samples r ≠ p from Uniform(0, popsize), which requires popsize ≥ 2.
///   Computational cost per epoch is O(popsize² · D) from the restart check
///   and O(popsize · D) from the perturbation step.
///   Tune range scales with dimension: D/2 provides minimal coverage,
///   2D provides thorough per-dimension exploration.
///
/// ## restart_tolerance ∈ (0, ∞), default: 1e-8
///   Controls when two particles are considered converged to the same basin:
///     |f_max − f_min| / |f_max| < restart_tolerance  →  restart worse particle.
///   Must be > 0: at 0 no restart ever triggers (relative diff is always ≥ 0
///   but never strictly < 0 for distinct finite values). Larger values restart
///   more aggressively, increasing exploration but losing exploitation.
///   Tune range: [1e-7, 1e-6, 1e-5] — near-tight values, avoids over-aggressive restarts.
///
/// ## sigma ∈ (0, 1]
///   Standard deviation of the Gaussian perturbation N(0, σ) scaled by
///   |best_d − current_d|. The perturbation formula is:
///     new_d = best_d + N(0, σ) × |best_d − current_d|
///   Since positions lie in [0, 1], the distance |best_d − current_d| ≤ 1,
///   so σ acts as a relative scaling factor on the step size:
///   - σ = 0.01: fine local search (~1% of distance)
///   - σ = 1.0: aggressive exploration (~100% of distance, ~32% of samples
///     exceed ±1 and get clamped to boundaries)
///   Must be > 0: at σ = 0 the Normal constructor panics.
///   σ > 1 wastes evaluations — most perturbations are clamped to 0 or 1,
///   biasing toward boundaries with no benefit.
///
/// ## self_instead_neighbour ∈ [0, 1]
///   Probability that a particle perturbs around its own best vs a random
///   neighbour's best. Being a probability, it is bounded to [0, 1].
///   At 0.0: pure social (always use neighbour) — fast convergence, risk of
///   premature collapse. At 1.0: pure individual — independent random searches,
///   no information sharing.
///
/// ## restart_decay_power ∈ (0, ∞), default: 6.0
///   Exponent for the restart tolerance decay schedule: effective_rt =
///   restart_tolerance × (1 − t)^restart_decay_power. Higher values cause
///   the tolerance to shrink faster, reducing restarts in later epochs.
///
pub struct AnsrV2 {
    pub popsize: usize,
    pub restart_tolerance: f32,
    pub sigma: f32,
    pub self_instead_neighbour: f32,
    pub restart_decay_power: f32,
    pub neighbour_multiplier: f32,
}

pub fn new_ansr_v2(params: &BTreeMap<String, f32>) -> AnsrV2 {
    AnsrV2 {
        popsize: params["popsize"] as usize,
        restart_tolerance: params["restart_tolerance"],
        sigma: params["sigma"],
        self_instead_neighbour: params["self_instead_neighbour"],
        restart_decay_power: params["restart_decay_power"],
        neighbour_multiplier: params["neighbour_multiplier"],
    }
}

impl Optimizer for AnsrV2 {
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
        let mut rng: Pcg64Mcg = SeedableRng::seed_from_u64(seed);
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
        let restart_decay_power = self.restart_decay_power;
        let normal = Normal::new(0.0, 1.0).unwrap();
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
            let t = epoch as f32 / max_epoch as f32;
            let effective_rt = restart_tolerance * (1.0 - t).powf(restart_decay_power);
            for lhs in 0..popsize {
                for rhs in (lhs + 1)..popsize {
                    let mut min_residual = best_residuals[lhs];
                    let mut max_residual = best_residuals[rhs];
                    if min_residual > max_residual {
                        swap(&mut min_residual, &mut max_residual);
                    }
                    if min_residual != f32::INFINITY
                        && max_residual != f32::INFINITY
                        && max_residual != 0.0
                        && f32::abs((max_residual - min_residual) / max_residual)
                            < effective_rt
                    {
                        let worse = if lhs != ind && rhs != ind {
                            if best_residuals[lhs] < best_residuals[rhs] {
                                rhs
                            } else {
                                lhs
                            }
                        } else if lhs != ind {
                            lhs
                        } else {
                            rhs
                        };
                        let better = if worse == lhs { rhs } else { lhs };
                        best_residuals[worse] = f32::INFINITY;
                        for d in 0..params {
                            best_positions[worse][d] = 1.0 - best_positions[better][d];
                            current_positions[worse][d] = 1.0 - best_positions[better][d];
                        }
                    }
                }
            }
            let effective_sigma = sigma * 0.5 * (1.0 + f32::cos(std::f32::consts::PI * t));
            for p in 0..popsize {
                let mut r = popsize_distr.sample(&mut rng);
                while r == p {
                    r = popsize_distr.sample(&mut rng);
                }
                for d in 0..params {
                    if random.sample(&mut rng) <= self_instead_neighbour {
                        let dist = f32::abs(best_positions[p][d] - current_positions[p][d]);
                        current_positions[p][d] = wrap_to_unit_cube(
                            best_positions[p][d]
                                + normal.sample(&mut rng) * effective_sigma * dist,
                        )
                    } else {
                        current_positions[p][d] = wrap_to_unit_cube(
                            best_positions[r][d]
                                + normal.sample(&mut rng) * effective_sigma * (1.0 + self.neighbour_multiplier * (1.0 + f32::cos(std::f32::consts::PI * t)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::early_stop_callback::EarlyStopCallback;
    use crate::utils::broadcast_simd;

    fn sphere(x: Vec8, y: Vec8) -> Vec8 {
        x * x + y * y
    }

    fn default_ansr_v2() -> AnsrV2 {
        AnsrV2 {
            popsize: 8,
            restart_tolerance: 0.01,
            sigma: 0.05,
            self_instead_neighbour: 0.9,
            restart_decay_power: 6.0,
            neighbour_multiplier: 1.125,
        }
    }

    fn run_ansr_v2(ansr: &AnsrV2, maxiter: u64) -> OptimizerResult {
        let bounds = vec![[-5.0f32, 5.0]; 16];
        let func = broadcast_simd(sphere);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        ansr.find_infimum(&func, &bounds, maxiter, 42, false, &early_stop)
    }

    // ── popsize ≥ 2 ─────────────────────────────────────────────────

    #[test]
    fn test_popsize_2_runs() {
        let ansr = AnsrV2 { popsize: 2, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 10_000);
        assert!(result.f_x.is_finite());
    }

    #[test]
    fn test_popsize_large_converges() {
        let ansr = AnsrV2 { popsize: 64, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 100_000);
        assert!(result.f_x <= 0.01, "f_x={} > 0.01", result.f_x);
    }

    // ── restart_tolerance > 0 ────────────────────────────────────────

    #[test]
    fn test_restart_tolerance_zero_no_restart() {
        let ansr = AnsrV2 { restart_tolerance: 0.0, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 50_000);
        assert!(result.f_x.is_finite());
    }

    #[test]
    fn test_restart_tolerance_positive_converges() {
        let result = run_ansr_v2(&default_ansr_v2(), 100_000);
        assert!(result.f_x <= 0.01, "f_x={} > 0.01", result.f_x);
    }

    // ── sigma > 0 ────────────────────────────────────────────────────

    #[test]
    fn test_sigma_zero_stalls() {
        let ansr = AnsrV2 {
            sigma: 0.0,
            self_instead_neighbour: 1.0,
            ..default_ansr_v2()
        };
        let result = run_ansr_v2(&ansr, 50_000);
        assert!(result.f_x.is_finite());
    }

    #[test]
    fn test_sigma_positive_converges() {
        let ansr = AnsrV2 { sigma: 0.3, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 100_000);
        assert!(result.f_x <= 0.01, "f_x={} > 0.01", result.f_x);
    }

    // ── self_instead_neighbour ∈ [0, 1] ──────────────────────────────

    #[test]
    fn test_self_instead_neighbour_zero_social_only() {
        let ansr = AnsrV2 { self_instead_neighbour: 0.0, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 100_000);
        assert!(result.f_x.is_finite());
    }

    #[test]
    fn test_self_instead_neighbour_one_individual_only() {
        let ansr = AnsrV2 { self_instead_neighbour: 1.0, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 100_000);
        assert!(result.f_x.is_finite());
    }

    #[test]
    fn test_self_instead_neighbour_mid_converges() {
        let ansr = AnsrV2 { self_instead_neighbour: 0.5, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 100_000);
        assert!(result.f_x <= 0.01, "f_x={} > 0.01", result.f_x);
    }

    // ── Output bounds ────────────────────────────────────────────────

    #[test]
    fn test_output_within_bounds() {
        let bounds: Vec<[f32; 2]> = vec![[-5.0, 5.0]; 16];
        let ansr = AnsrV2 { sigma: 0.5, ..default_ansr_v2() };
        let func = broadcast_simd(sphere);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = ansr.find_infimum(&func, &bounds, 10_000, 42, false, &early_stop);
        for (i, &xi) in result.x.iter().enumerate() {
            assert!(
                xi >= bounds[i][0] && xi <= bounds[i][1],
                "x[{}]={} outside [{}, {}]",
                i,
                xi,
                bounds[i][0],
                bounds[i][1]
            );
        }
    }

    // ── nfev bound ───────────────────────────────────────────────────

    #[test]
    fn test_nfev_upper_bound() {
        let popsize = 8;
        let maxiter = 1000;
        let ansr = default_ansr_v2();
        let result = run_ansr_v2(&ansr, maxiter);
        let max_epoch = (maxiter as f64 / popsize as f64).ceil() as u64;
        let upper_bound = max_epoch * popsize as u64;
        assert!(
            result.nfev <= upper_bound,
            "nfev={} > upper_bound={}",
            result.nfev,
            upper_bound
        );
    }

    #[test]
    fn test_nfev_at_least_one_epoch() {
        let ansr = AnsrV2 { popsize: 4, ..default_ansr_v2() };
        let result = run_ansr_v2(&ansr, 100);
        assert!(result.nfev >= 4, "nfev={} < popsize=4", result.nfev);
    }

    #[test]
    fn test_new_ansr_v2_from_params() {
        let mut params = BTreeMap::new();
        params.insert("popsize".to_string(), 16.0);
        params.insert("restart_tolerance".to_string(), 0.001);
        params.insert("sigma".to_string(), 0.1);
        params.insert("self_instead_neighbour".to_string(), 0.8);
        params.insert("restart_decay_power".to_string(), 4.0);
        params.insert("neighbour_multiplier".to_string(), 2.0);
        let ansr = new_ansr_v2(&params);
        assert_eq!(ansr.popsize, 16);
        assert_eq!(ansr.restart_tolerance, 0.001);
        assert_eq!(ansr.sigma, 0.1);
        assert_eq!(ansr.self_instead_neighbour, 0.8);
        assert_eq!(ansr.restart_decay_power, 4.0);
        assert_eq!(ansr.neighbour_multiplier, 2.0);
    }

    #[test]
    fn test_early_stop_fires() {
        let ansr = default_ansr_v2();
        let bounds = vec![[-5.0f32, 5.0]; 16];
        let func = broadcast_simd(sphere);
        let early_stop = EarlyStopCallback::new(&func, 100.0);
        let result = ansr.find_infimum(&func, &bounds, 100_000, 42, false, &early_stop);
        let max_nfev = (100_000f64 / 8.0).ceil() as u64 * 8;
        assert!(result.nfev < max_nfev, "nfev={} should be < {}", result.nfev, max_nfev);
    }

    #[test]
    fn test_with_history() {
        let ansr = AnsrV2 { popsize: 4, ..default_ansr_v2() };
        let bounds = vec![[-5.0f32, 5.0]; 16];
        let func = broadcast_simd(sphere);
        let early_stop = EarlyStopCallback::new(&func, 0.01);
        let result = ansr.find_infimum(&func, &bounds, 200, 42, true, &early_stop);
        let history = result.history.as_ref().unwrap();
        assert!(!history.x.is_empty());
        assert!(!history.f_x.is_empty());
        assert_eq!(history.x.len(), history.f_x.len());
    }

    #[test]
    fn test_wrap_to_unit_cube() {
        assert!((wrap_to_unit_cube(0.5) - 0.5).abs() < 1e-6);
        assert!((wrap_to_unit_cube(1.5) - 0.5).abs() < 1e-6);
        assert!(wrap_to_unit_cube(-0.3) >= 0.0);
        assert!(wrap_to_unit_cube(-0.3) <= 1.0);
    }
}
