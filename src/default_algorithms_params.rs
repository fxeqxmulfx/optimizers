use std::collections::{BTreeMap};

use once_cell::sync::Lazy;

use crate::algorithms::{ansr::ANSR, zero_gradient::ZeroGradient};

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 8,
    restart_tolerance: 0.01,
    sigma: 0.05,
    self_instead_neighbour: 0.9,
};

pub static ANSR_PARAMS: Lazy<BTreeMap<String, Vec<f32>>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert("popsize".to_string(), vec![4., 8., 16., 24., 32., 40., 48.]);
    m.insert(
        "restart_tolerance".to_string(),
        vec![1e-1, 1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7, 1e-8],
    );
    m.insert(
        "sigma".to_string(),
        vec![0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5],
    );
    m.insert(
        "self_instead_neighbour".to_string(),
        vec![
            0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75,
            0.8, 0.85, 0.9, 0.95, 1.0,
        ],
    );
    m
});

pub static DEFAULT_ZERO_GRADIENT: ZeroGradient = ZeroGradient { init_jump: 0.1 };
