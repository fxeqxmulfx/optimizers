pub struct EarlyStopCallback<F>
where
    F: Fn(&[f32]) -> f32,
{
    function: F,
    stop_residual: f32,
}

impl<F> EarlyStopCallback<F>
where
    F: Fn(&[f32]) -> f32,
{
    pub fn new(function: F, stop_residual: f32) -> Self {
        Self {
            function,
            stop_residual,
        }
    }

    pub fn should_stop(&self, x: &[f32]) -> bool {
        let function = &self.function;
        function(x) <= self.stop_residual
    }
}
