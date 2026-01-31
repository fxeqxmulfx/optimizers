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

#[cfg(test)]
mod tests {
    use super::*;

    fn sum_of_squares(x: &[f32]) -> f32 {
        x.iter().map(|v| v * v).sum()
    }

    #[test]
    fn test_new_stores_values() {
        let cb = EarlyStopCallback::new(sum_of_squares, 1.0);
        assert!(!cb.should_stop(&[2.0, 0.0]));
        assert!(cb.should_stop(&[1.0, 0.0]));
    }

    #[test]
    fn test_should_stop_exact_threshold() {
        let cb = EarlyStopCallback::new(sum_of_squares, 5.0);
        let x = [2.0, 1.0];
        assert!(
            cb.should_stop(&x),
            "should stop when value equals threshold"
        );
    }

    #[test]
    fn test_should_stop_above_threshold() {
        let cb = EarlyStopCallback::new(sum_of_squares, 3.0);
        let x = [2.0, 1.0];
        assert!(
            !cb.should_stop(&x),
            "should NOT stop when value is greater than threshold"
        );
    }

    #[test]
    fn test_should_stop_below_threshold() {
        let cb = EarlyStopCallback::new(sum_of_squares, 10.0);
        let x = [1.0, 2.0];
        assert!(
            cb.should_stop(&x),
            "should stop when value is below threshold"
        );
    }

    #[test]
    fn test_using_closure_that_captures_state() {
        let scale = 2.0_f32;
        let cb = EarlyStopCallback::new(move |x: &[f32]| scale * sum_of_squares(x), 8.0);
        let x = [2.0, 0.0];
        assert!(
            cb.should_stop(&x),
            "closure that captures state should work"
        );
    }

    #[test]
    fn test_using_function_pointer() {
        let cb = EarlyStopCallback::new(sum_of_squares as fn(&[f32]) -> f32, 4.0);
        let x = [2.0, 0.0];
        assert!(cb.should_stop(&x));
    }

    #[test]
    #[should_panic(expected = "panic")]
    fn test_should_stop_panics_if_function_panics() {
        let cb = EarlyStopCallback::new(|_x: &[f32]| panic!("panic!"), 0.0);
        cb.should_stop(&[0.0, 0.0]);
    }
}
