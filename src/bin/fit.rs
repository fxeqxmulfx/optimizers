use std::collections::BTreeMap;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use optimizers::{
    algorithms::ansr::new_ansr,
    default_algorithms_params::ANSR_PARAMS,
    functions::TEST_FUNCTIONS,
    runner::run_multiple_optimizaions,
    utils::{all_combinations, f32_to_i64},
};

fn main() {
    let ansr_params = ANSR_PARAMS.clone();
    let functions = TEST_FUNCTIONS;
    let dimension_count = 16;
    let maxiter = 300_000;
    let seed_count = 10;
    let stop_residual = 0.01;
    let all_combinations = all_combinations(&ansr_params);
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}|{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");
    let pb = ProgressBar::new(all_combinations.len() as u64);
    pb.set_style(sty.clone());
    let single_test_function = &functions[1..2];
    let maxiter_single_test_function = 40_000.0;
    let mut results: Vec<(i64, &BTreeMap<String, f32>)> = all_combinations
        .par_iter()
        .map(|params| {
            let optimizer = new_ansr(params);
            let mean_single_test_function = run_multiple_optimizaions(
                &optimizer,
                single_test_function,
                dimension_count,
                maxiter_single_test_function as u64,
                seed_count,
                stop_residual,
                false,
                false,
            )["mean"];
            if mean_single_test_function > maxiter_single_test_function {
                pb.inc(1);
                (i64::MAX, params)
            } else {
                let mean = run_multiple_optimizaions(
                    &optimizer,
                    &functions,
                    dimension_count,
                    maxiter,
                    seed_count,
                    stop_residual,
                    false,
                    false,
                )["mean"];
                pb.inc(1);
                (f32_to_i64(mean), params)
            }
        })
        .collect();
    results.sort_by_key(|&(mean, _)| mean);
    println!("Top 10 (lowest) mean values:");
    for (rank, (mean, params)) in results.iter().take(10).enumerate() {
        println!("{}: mean = {}   params = {:?}", rank + 1, mean, params);
    }
}
