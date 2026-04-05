use std::collections::BTreeMap;

use crate::algorithms::{ans::ANS, ans_sort::AnsSorted, ansr::ANSR, ansr_dpnm::AnsrDpnm, de::DE, shade::SHADE, zero_gradient::ZeroGradient};

pub(crate) fn frange(start: f32, step: f32, end: f32) -> Vec<f32> {
    let n = ((end - start) / step).round() as usize + 1;
    (0..n).map(|i| start + step * i as f32).collect()
}

#[allow(dead_code)]
pub(crate) fn log10_range(exp_start: i32, exp_end: i32) -> Vec<f32> {
    if exp_start <= exp_end {
        (exp_start..=exp_end).map(|e| 10f32.powi(e)).collect()
    } else {
        (exp_end..=exp_start).rev().map(|e| 10f32.powi(e)).collect()
    }
}

fn popsize_grid(_dimension_count: usize) -> Vec<f32> {
    vec![64.0]
}

pub static DEFAULT_ANS: ANS = ANS {
    popsize: 64,
    sigma: 0.15,
    self_instead_neighbour: 0.7,
};

pub fn ans_params(dimension_count: usize) -> BTreeMap<String, Vec<f32>> {
    let mut m = ansr_params(dimension_count);
    m.remove("restart_tolerance");
    m
}

pub static DEFAULT_ANS_SORT: AnsSorted = AnsSorted {
    popsize: 64,
    sigma: 0.15,
    self_instead_neighbour: 0.7,
};

pub fn ans_sort_params(dimension_count: usize) -> BTreeMap<String, Vec<f32>> {
    ans_params(dimension_count)
}

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 16,
    restart_tolerance: 0.01,
    sigma: 0.01,
    self_instead_neighbour: 0.9,
};

pub fn ansr_params(dimension_count: usize) -> BTreeMap<String, Vec<f32>> {
    let mut m = BTreeMap::new();
    m.insert("popsize".to_string(), popsize_grid(dimension_count));
    m.insert(
        "restart_tolerance".to_string(),
        vec![1e-8],
    );
    m.insert(
        "sigma".to_string(),
        frange(0.04, 0.04, 0.96),  // sigma > 0 required for Normal distribution
    );
    m.insert(
        "self_instead_neighbour".to_string(),
        frange(0.0, 0.04, 0.96),
    );
    m
}

pub static DEFAULT_ANSR_DPNM: AnsrDpnm = AnsrDpnm {
    popsize: 8,
    restart_tolerance: 0.01,
    sigma: 0.3,
    self_instead_neighbour: 0.85,
    restart_decay_power: 2.0,
    neighbour_multiplier: 0.5,
};

pub fn ansr_dpnm_params(dimension_count: usize) -> BTreeMap<String, Vec<f32>> {
    let mut m = BTreeMap::new();
    m.insert("popsize".to_string(), popsize_grid(dimension_count));
    m.insert("restart_tolerance".to_string(), vec![1e-8]);
    m.insert("sigma".to_string(), frange(0.2, 0.2, 1.0));  // sigma > 0 required
    m.insert("self_instead_neighbour".to_string(), frange(0.0, 0.2, 1.0));
    m.insert("restart_decay_power".to_string(), vec![2.0, 4.0, 6.0, 8.0]);
    m.insert("neighbour_multiplier".to_string(), vec![0.5, 0.75, 1.0, 1.125, 1.5]);
    m
}

pub static DEFAULT_DE: DE = DE {
    popsize: 64,
    f: 0.6,
    cr: 0.1,
};

pub fn de_params(dimension_count: usize) -> BTreeMap<String, Vec<f32>> {
    let mut m = BTreeMap::new();
    m.insert("popsize".to_string(), popsize_grid(dimension_count));
    m.insert("f".to_string(), frange(0.04, 0.04, 0.96));  // F > 0 required
    m.insert("cr".to_string(), frange(0.0, 0.04, 0.96));
    m
}

pub static DEFAULT_SHADE: SHADE = SHADE {
    popsize: 56,
    h: 20,
    p_best_rate: 0.3,
};

pub fn shade_params(dimension_count: usize) -> BTreeMap<String, Vec<f32>> {
    let mut m = BTreeMap::new();
    m.insert("popsize".to_string(), popsize_grid(dimension_count));
    m.insert("h".to_string(), frange(1.0, 1.0, 24.0));
    m.insert("p_best_rate".to_string(), frange(0.04, 0.04, 1.0));
    m
}

pub static DEFAULT_ZERO_GRADIENT: ZeroGradient = ZeroGradient { init_jump: 0.1 };

pub fn zero_gradient_params() -> BTreeMap<String, Vec<f32>> {
    let mut m = BTreeMap::new();
    m.insert(
        "init_jump".to_string(),
        [log10_range(-8, -1), frange(0.15, 0.05, 1.0)].concat(),
    );
    m
}

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
    fn test_default_ans() {
        assert_eq!(DEFAULT_ANS.popsize, 64);
        assert_eq!(DEFAULT_ANS.sigma, 0.15);
        assert_eq!(DEFAULT_ANS.self_instead_neighbour, 0.7);
    }

    #[test]
    fn test_ans_params() {
        let p = ans_params(16);
        assert_eq!(p["popsize"], vec![64.0]);
        assert!(!p.contains_key("restart_tolerance"));
        assert_eq!(p["sigma"].len(), 24);
        assert_eq!(p["self_instead_neighbour"].len(), 25);
    }

    #[test]
    fn test_default_ansr() {
        assert_eq!(DEFAULT_ANSR.popsize, 16);
        assert_eq!(DEFAULT_ANSR.sigma, 0.01);
        assert_eq!(DEFAULT_ANSR.self_instead_neighbour, 0.9);
        assert!((DEFAULT_ANSR.restart_tolerance - 0.01).abs() < 1e-8);
    }

    #[test]
    fn test_default_zero_gradient() {
        assert_eq!(DEFAULT_ZERO_GRADIENT.init_jump, 0.1);
    }

    #[test]
    fn test_ansr_params() {
        let p = ansr_params(16);
        assert_eq!(p["popsize"], vec![64.0]);
        assert_eq!(p["restart_tolerance"].len(), 1);
        assert_eq!(p["sigma"].len(), 24); // 0.04 to 0.96 step 0.04
        assert_eq!(p["self_instead_neighbour"].len(), 25);
    }

    #[test]
    fn test_default_ansr_dpnm() {
        assert_eq!(DEFAULT_ANSR_DPNM.popsize, 8);
        assert_eq!(DEFAULT_ANSR_DPNM.sigma, 0.3);
        assert_eq!(DEFAULT_ANSR_DPNM.self_instead_neighbour, 0.85);
        assert!((DEFAULT_ANSR_DPNM.restart_tolerance - 0.01).abs() < 1e-8);
        assert_eq!(DEFAULT_ANSR_DPNM.restart_decay_power, 2.0);
        assert_eq!(DEFAULT_ANSR_DPNM.neighbour_multiplier, 0.5);
    }

    #[test]
    fn test_ansr_dpnm_params_has_all_keys() {
        let p = ansr_dpnm_params(10);
        assert!(p.contains_key("popsize"));
        assert!(p.contains_key("restart_tolerance"));
        assert!(p.contains_key("sigma"));
        assert!(p.contains_key("self_instead_neighbour"));
        assert!(p.contains_key("restart_decay_power"));
    }

#[test]
    fn test_ansr_dpnm_params_sigma_range() {
        let p = ansr_dpnm_params(10);
        let sigma = &p["sigma"];
        assert!(sigma.first().unwrap() < sigma.last().unwrap());
    }

    #[test]
    fn test_ansr_dpnm_params_self_instead_neighbour_range() {
        let p = ansr_dpnm_params(10);
        let sin = &p["self_instead_neighbour"];
        assert_eq!(*sin.first().unwrap(), 0.0);
        assert_eq!(*sin.last().unwrap(), 1.0);
    }
}
