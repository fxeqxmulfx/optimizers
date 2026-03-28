use std::collections::BTreeMap;

use once_cell::sync::Lazy;

use crate::algorithms::{ansr::ANSR, zero_gradient::ZeroGradient};

pub(crate) fn frange(start: f32, step: f32, end: f32) -> Vec<f32> {
    let n = ((end - start) / step).round() as usize + 1;
    (0..n).map(|i| start + step * i as f32).collect()
}

pub(crate) fn log10_range(exp_start: i32, exp_end: i32) -> Vec<f32> {
    if exp_start <= exp_end {
        (exp_start..=exp_end).map(|e| 10f32.powi(e)).collect()
    } else {
        (exp_end..=exp_start).rev().map(|e| 10f32.powi(e)).collect()
    }
}

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 4,
    restart_tolerance: 0.01,
    sigma: 0.05,
    self_instead_neighbour: 0.9,
};

pub static ANSR_PARAMS: Lazy<BTreeMap<String, Vec<f32>>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert("popsize".to_string(), [64.0].to_vec());
    m.insert("restart_tolerance".to_string(), log10_range(-1, -7));
    m.insert(
        "sigma".to_string(),
        [vec![0.01], frange(0.05, 0.05, 1.0)].concat(),
    );
    m.insert("self_instead_neighbour".to_string(), frange(0.0, 0.05, 1.0));
    m
});

pub static DEFAULT_ZERO_GRADIENT: ZeroGradient = ZeroGradient { init_jump: 0.1 };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frange() {
        let r = frange(0.0, 0.5, 1.0);
        assert_eq!(r, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_frange_single() {
        let r = frange(1.0, 1.0, 1.0);
        assert_eq!(r, vec![1.0]);
    }

    #[test]
    fn test_log10_range_ascending() {
        let r = log10_range(-2, 1);
        assert_eq!(r.len(), 4);
        assert!((r[0] - 0.01).abs() < 1e-6);
        assert!((r[3] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_log10_range_descending() {
        let r = log10_range(1, -2);
        assert_eq!(r.len(), 4);
        assert!((r[0] - 10.0).abs() < 1e-6);
        assert!((r[3] - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_default_ansr() {
        assert_eq!(DEFAULT_ANSR.popsize, 4);
        assert_eq!(DEFAULT_ANSR.sigma, 0.05);
    }

    #[test]
    fn test_default_zero_gradient() {
        assert_eq!(DEFAULT_ZERO_GRADIENT.init_jump, 0.1);
    }

    #[test]
    fn test_ansr_params() {
        let p = &*ANSR_PARAMS;
        assert_eq!(p["popsize"], vec![64.0]);
        assert_eq!(p["restart_tolerance"].len(), 7);
        assert!(p["sigma"].len() > 1);
        assert!(p["self_instead_neighbour"].len() > 1);
    }
}
