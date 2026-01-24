use crate::algorithms::ansr::ANSR;

pub struct AlgorithmDescriptor {
    pub name: &'static str,
    pub algorithm: &'static ANSR,
}

pub static DEFAULT_ANSR: ANSR = ANSR {
    popsize: 8,
    tol: 0.01,
    sigma: 0.05,
    self_instead_neighbour: 0.8,
};

pub const ALGORITHMS: [AlgorithmDescriptor; 1] = [AlgorithmDescriptor {
    name: "ANSR",
    algorithm: &DEFAULT_ANSR,
}];
