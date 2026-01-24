use crate::algorithms::ansr::ANSR;

pub struct AlgorithmDescriptor {
    pub name: &'static str,
    pub algorithm: &'static ANSR,
}

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 16,
    tol: 0.001,
    sigma: 0.1,
    self_instead_neighbour: 0.3,
};

pub const ALGORITHMS: [AlgorithmDescriptor; 1] = [AlgorithmDescriptor {
    name: "ANSR",
    algorithm: &DEFAULT_ANSR,
}];
