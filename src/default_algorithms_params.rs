use crate::algorithms::{ansr::ANSR, zero_gradient::ZeroGradient};

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 8,
    restart_tolerance: 0.01,
    sigma: 0.05,
    self_instead_neighbour: 0.9,
};

pub static DEFAULT_ZERO_GRADIENT: ZeroGradient = ZeroGradient { init_jump: 0.1 };
