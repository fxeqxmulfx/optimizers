use optimizers::{
    early_stop_callback::EarlyStopCallback,
    functions::TEST_FUNCTIONS,
    optimizer::Optimizer,
    optimizers::ALGORITHMS,
    plot::save_video_h264,
    utils::{broadcast, format_best_f_x_history, format_x_history},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let optimizer = ALGORITHMS[0].algorithm;
    let function = &TEST_FUNCTIONS[3];
    let dimension_count = 40;
    let maxiter = 10_000;
    let stop_residual = 0.01;
    let func = &broadcast(&function.func);
    let bounds = &function.bounds.repeat(dimension_count / 2);
    let early_stop_callback = EarlyStopCallback::new(func, stop_residual);
    let result = optimizer.find_infimum(func, bounds, maxiter, 42, true, &early_stop_callback);
    println!("f({:?}) == {:?}", result.x, result.f_x);
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
    return Ok(());
}
