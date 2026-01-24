use crate::early_stop_callback::EarlyStopCallback;

#[derive(Debug)]
pub struct OptimizationHistory {
    pub x: Vec<Vec<Vec<f32>>>,
    pub f_x: Vec<Vec<f32>>,
}

#[derive(Debug)]
pub struct OptimizerResult {
    pub x: Vec<f32>,
    pub f_x: f32,
    pub nfev: u64,
    pub history: Option<OptimizationHistory>,
}

pub trait Optimizer {
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
        F: Fn(&[f32]) -> f32 + Sync;
}
