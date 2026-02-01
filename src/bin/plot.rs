use optimizers::{
    default_algorithms_params::DEFAULT_ANSR,
    early_stop_callback::EarlyStopCallback,
    functions::TEST_FUNCTIONS,
    optimizer::Optimizer,
    plot::save_video_h264,
    utils::{broadcast_scalar, broadcast_simd, format_best_f_x_history, format_x_history},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let optimizer = &DEFAULT_ANSR;
    let function = &TEST_FUNCTIONS[4];
    let dimension_count = 16;
    let maxiter = 100_000;
    let stop_residual = 0.01;
    let func = &broadcast_simd(&function.func);
    let bounds = &function.bounds.repeat(dimension_count / 2);
    let early_stop_callback = EarlyStopCallback::new(func, stop_residual);
    let result = optimizer.find_infimum(func, bounds, maxiter, 42, true, &early_stop_callback);
    println!("f() == {:?}, nfev={}", result.f_x, result.nfev);
    if let Some(history) = result.history {
        save_video_h264(
            &broadcast_scalar(&function.func),
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
    return Ok(());
}
