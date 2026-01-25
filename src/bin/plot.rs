use optimizers::{
    default_algorithms_params::ALGORITHMS,
    early_stop_callback::EarlyStopCallback,
    functions::TEST_FUNCTIONS,
    optimizer::Optimizer,
    plot::save_video_h264,
    utils::{broadcast, format_best_f_x_history, format_x_history},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(optimizer) = ALGORITHMS[1].algorithm.as_zero_gradient() {
        let function = &TEST_FUNCTIONS[2];
        let dimension_count = 10;
        let maxiter = 10_000;
        let stop_residual = 0.01;
        let func = &broadcast(&function.func);
        let bounds = &function.bounds.repeat(dimension_count / 2);
        let early_stop_callback = EarlyStopCallback::new(func, stop_residual);
        let result = optimizer.find_infimum(func, bounds, maxiter, 42, true, &early_stop_callback);
        println!("f({:?}) == {:?}, nfev={}", result.x, result.f_x, result.nfev);
        if let Some(history) = result.history {
            save_video_h264(
                func,
                &format_x_history(&history.x, bounds),
                &format_best_f_x_history(&history.f_x),
                "result.mp4",
                bounds[0],
                bounds[1],
                1024,
                1024,
                1024,
                30,
            )?;
        }
    }
    return Ok(());
}
