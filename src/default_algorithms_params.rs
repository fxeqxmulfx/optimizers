use std::collections::BTreeMap;

use once_cell::sync::Lazy;

use crate::algorithms::{ansr::ANSR, zero_gradient::ZeroGradient};

fn frange(start: f32, step: f32, end: f32) -> Vec<f32> {
    let n = ((end - start) / step).round() as usize + 1;
    (0..n).map(|i| start + step * i as f32).collect()
}

fn log10_range(exp_start: i32, exp_end: i32) -> Vec<f32> {
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
