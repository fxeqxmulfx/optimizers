use core::f32;
use std::{collections::BTreeMap, env, sync::atomic::Ordering};

use atomic_float::AtomicF32;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use optimizers::{
    algorithms::{ansr::new_ansr, ansr_v2::new_ansr_v2},
    default_algorithms_params::{ansr_v2_params, ANSR_PARAMS},
    functions::MINI_TEST_FUNCTIONS,
    optimizer::Optimizer,
    runner::run_multiple_optimizaions,
    utils::{all_combinations, f32_to_i64, group_by_key, mean_and_mad, summarize_group},
};

fn tune<T, F>(
    name: &str,
    params_grid: &BTreeMap<String, Vec<f32>>,
    make_optimizer: F,
    functions: &BTreeMap<String, optimizers::functions::TestFunction>,
    dimension_count: usize,
    maxiter: u64,
    seed_count: u64,
    stop_residual: f32,
) where
    T: Optimizer + Sync,
    F: Fn(&BTreeMap<String, f32>) -> T + Sync,
{
    let all_combinations = all_combinations(params_grid);
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}|{eta_precise}|mean={msg}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");
    println!("\n=== Tuning {} ({} combinations, {}D) ===", name, all_combinations.len(), dimension_count);
    let pb = ProgressBar::new(all_combinations.len() as u64);
    pb.set_style(sty.clone());
    let global_mean = AtomicF32::new(f32::INFINITY);
    let mut results: Vec<(i64, BTreeMap<String, f32>, BTreeMap<String, f32>)> = all_combinations
        .par_iter()
        .map(|params| {
            let optimizer = make_optimizer(params);
            let result = run_multiple_optimizaions(
                &optimizer,
                functions,
                dimension_count,
                maxiter,
                seed_count,
                stop_residual,
                false,
                false,
            );
            let mean = result["mean"];
            pb.inc(1);
            if pb.position() % 100 == 0 {
                println!(
                    "[{}/{}] mean={}",
                    pb.position(),
                    pb.length().unwrap_or(0),
                    mean
                );
            }
            pb.set_message(format!(
                "{}",
                global_mean.fetch_min(mean, Ordering::Relaxed)
            ));
            (f32_to_i64(mean), params.clone(), result)
        })
        .collect();
    results.sort_by_key(|&(mean, _, _)| mean);

    let grouped = group_by_key(&results, "popsize");

    let all_means: Vec<f32> = results.iter().map(|(_, _, r)| r["mean"]).collect();
    let (global_mean, global_mad) = mean_and_mad(&all_means);
    let best = &results[0];
    println!("\n=== {} Summary ===", name);
    println!("Total combinations: {}", results.len());
    println!("Global mean: {:.4}, mad: {:.4}", global_mean, global_mad);
    println!("Best:  mean={:.4} params={:?}", best.2["mean"], best.1);

    println!("\n{:-<90}", "");
    println!(
        "{:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>5} | {:>6}",
        "popsize", "best", "mean", "mad", "worst", "count", "finite"
    );
    println!("{:-<90}", "");
    for (popsize, entries) in &grouped {
        let means: Vec<f32> = entries.iter().map(|(_, _, r)| r["mean"]).collect();
        let s = summarize_group(&means);
        println!(
            "{:>8} | {:>8.2} | {:>8.2} | {:>8.2} | {:>8.2} | {:>5} | {:>6}",
            popsize, s.best, s.mean, s.mad, s.worst, s.count, s.finite_count
        );
    }
    println!("{:-<90}", "");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let algo = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    let dimension_count = 16;
    let maxiter = 100_000;
    let seed_count = 10;
    let stop_residual = 0.01;
    let functions = &MINI_TEST_FUNCTIONS;

    if algo == "ansr" || algo == "all" {
        tune(
            "ANSR",
            &ANSR_PARAMS,
            |params| new_ansr(params),
            functions,
            dimension_count,
            maxiter,
            seed_count,
            stop_residual,
        );
    }

    if algo == "ansr_v2" || algo == "all" {
        let v2_params = ansr_v2_params(dimension_count);
        tune(
            "ANSR V2",
            &v2_params,
            |params| new_ansr_v2(params),
            functions,
            dimension_count,
            maxiter,
            seed_count,
            stop_residual,
        );
    }
}
