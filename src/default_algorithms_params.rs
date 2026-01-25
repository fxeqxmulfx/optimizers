use crate::algorithms::{ansr::ANSR, zero_gradient::ZeroGradient};

pub enum OptimizerKind {
    ANSR(&'static ANSR),
    ZeroGradient(&'static ZeroGradient),
}

pub struct AlgorithmDescriptor {
    pub name: &'static str,
    pub algorithm: OptimizerKind,
}

impl OptimizerKind {
    pub fn as_ansr(&self) -> Option<&ANSR> {
        if let OptimizerKind::ANSR(ansr) = self {
            Some(ansr)
        } else {
            None
        }
    }

    pub fn as_zero_gradient(&self) -> Option<&ZeroGradient> {
        if let OptimizerKind::ZeroGradient(zero_gradient) = self {
            Some(zero_gradient)
        } else {
            None
        }
    }
}

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 8,
    restart_tolerance: 0.01,
    sigma: 0.05,
    self_instead_neighbour: 0.9,
};

pub static DEFAULT_ZERO_GRADIENT: ZeroGradient = ZeroGradient { init_jump: 0.1 };

pub const ALGORITHMS: [AlgorithmDescriptor; 2] = [
    AlgorithmDescriptor {
        name: "ANSR",
        algorithm: OptimizerKind::ANSR(&DEFAULT_ANSR),
    },
    AlgorithmDescriptor {
        name: "zero_gradient",
        algorithm: OptimizerKind::ZeroGradient(&DEFAULT_ZERO_GRADIENT),
    },
];
