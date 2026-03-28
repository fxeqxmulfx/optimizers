use simd_vector::Vec8;

pub struct EarlyStopCallback<F>
where
    F: Fn(&[Vec8]) -> f32,
{
    function: F,
    stop_residual: f32,
}

impl<F> EarlyStopCallback<F>
where
    F: Fn(&[Vec8]) -> f32,
{
    pub fn new(function: F, stop_residual: f32) -> Self {
        Self {
            function,
            stop_residual,
        }
    }

    pub fn should_stop(&self, x: &[Vec8]) -> bool {
        let function = &self.function;
        function(x) <= self.stop_residual
    }

    pub fn stop_residual(&self) -> f32 {
        self.stop_residual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_stop_below_threshold() {
        let callback = EarlyStopCallback::new(|_x: &[Vec8]| 0.005, 0.01);
        assert!(callback.should_stop(&[]));
    }

    #[test]
    fn test_should_stop_equal_threshold() {
        let callback = EarlyStopCallback::new(|_x: &[Vec8]| 0.01, 0.01);
        assert!(callback.should_stop(&[]));
    }

    #[test]
    fn test_should_not_stop_above_threshold() {
        let callback = EarlyStopCallback::new(|_x: &[Vec8]| 0.02, 0.01);
        assert!(!callback.should_stop(&[]));
    }

    #[test]
    fn test_stop_residual_getter() {
        let callback = EarlyStopCallback::new(|_x: &[Vec8]| 0.0, 0.05);
        assert_eq!(callback.stop_residual(), 0.05);
    }
}
