use core::f32;
use std::{collections::BTreeMap, env, fs::File, io::Write, sync::atomic::Ordering};

use atomic_float::AtomicF32;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use optimizers::{
    algorithms::{ans::new_ans, ansr::new_ansr, ansr_dpnm::new_ansr_dpnm, de::new_de, shade::new_shade, zero_gradient::new_zero_gradient},
    default_algorithms_params::{ans_params, ansr_params, ansr_dpnm_params, de_params, shade_params, zero_gradient_params},
    functions::{EASY_TEST_FUNCTIONS, HARD_TEST_FUNCTIONS, HARD_DISCRETE_FUNCTIONS, MEDIUM_PERIODIC_FUNCTIONS, LMMAES_TEST_FUNCTIONS, MAIN_TEST_FUNCTIONS, TERRAIN_TEST_FUNCTIONS, MINI_TEST_FUNCTIONS, WEIERSTRASS_TEST_FUNCTIONS, TestFunction},
    optimizer::Optimizer,
    runner::run_multiple_optimizaions,
    utils::{all_combinations, f32_to_i64, group_by_key, mean_and_mad, summarize_group},
};

fn tune<T, F>(
    name: &str,
    test_set: &str,
    params_grid: &BTreeMap<String, Vec<f32>>,
    make_optimizer: F,
    functions: &BTreeMap<String, TestFunction>,
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
            let best = global_mean.fetch_min(mean, Ordering::Relaxed);
            if pb.position() % 100 == 0 {
                println!(
                    "[{}/{}] best={}",
                    pb.position(),
                    pb.length().unwrap_or(0),
                    best.min(mean)
                );
            }
            pb.set_message(format!("{}", best.min(mean)));
            (f32_to_i64(mean), params.clone(), result)
        })
        .collect();
    results.sort_by_key(|&(mean, _, _)| mean);

    // Write CSV: tune_results/<test_set>_<dim>D_<algo>.csv
    {
        let dir = "tune_results";
        std::fs::create_dir_all(dir).ok();
        let algo_slug = name.to_lowercase().replace(' ', "_");
        let path = format!("{}/{}_{:}d_{}.csv", dir, test_set, dimension_count, algo_slug);
        let param_keys: Vec<&String> = params_grid.keys().collect();
        let func_keys: Vec<&String> = functions.keys().collect();
        let mut f = File::create(&path).expect("Failed to create CSV");
        // header
        let mut header = param_keys.iter().map(|k| k.as_str()).collect::<Vec<_>>();
        for fk in &func_keys {
            header.push(fk.as_str());
        }
        header.push("mean");
        header.push("mad");
        writeln!(f, "{}", header.join(",")).unwrap();
        // rows
        for (_, params, result) in &results {
            let mut row: Vec<String> = param_keys.iter().map(|k| format!("{}", params[*k])).collect();
            let func_values: Vec<f32> = func_keys.iter().map(|fk| result.get(*fk).copied().unwrap_or(f32::INFINITY)).collect();
            for &v in &func_values {
                row.push(format!("{}", v));
            }
            let mean = result["mean"];
            let mad = if func_values.iter().all(|v| v.is_finite()) {
                func_values.iter().map(|v| (v - mean).abs()).sum::<f32>() / func_values.len() as f32
            } else {
                f32::INFINITY
            };
            row.push(format!("{}", mean));
            row.push(format!("{}", mad));
            writeln!(f, "{}", row.join(",")).unwrap();
        }
        println!("Saved {}", path);
    }

    let group_key = params_grid.keys().next().unwrap();
    let grouped = group_by_key(&results, group_key);

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
        group_key, "best", "mean", "mad", "worst", "count", "finite"
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

fn tune_all(
    algo: &str,
    test_set: &str,
    functions: &BTreeMap<String, TestFunction>,
    dimension_count: usize,
    maxiter: u64,
    seed_count: u64,
    stop_residual: f32,
) {
    if algo == "ans" || algo == "all" {
        tune("ANS", test_set, &ans_params(dimension_count), |p| new_ans(p), functions, dimension_count, maxiter, seed_count, stop_residual);
    }
    if algo == "ansr" || algo == "all" {
        tune("ANSR", test_set, &ansr_params(dimension_count), |p| new_ansr(p), functions, dimension_count, maxiter, seed_count, stop_residual);
    }
    if algo == "ansr_dpnm" || algo == "all" {
        tune("ANSR DPNM", test_set, &ansr_dpnm_params(dimension_count), |p| new_ansr_dpnm(p), functions, dimension_count, maxiter, seed_count, stop_residual);
    }
    if algo == "de" || algo == "all" {
        tune("DE", test_set, &de_params(dimension_count), |p| new_de(p), functions, dimension_count, maxiter, seed_count, stop_residual);
    }
    if algo == "shade" || algo == "all" {
        tune("SHADE", test_set, &shade_params(dimension_count), |p| new_shade(p), functions, dimension_count, maxiter, seed_count, stop_residual);
    }
    if algo == "zero_gradient" || algo == "all" {
        tune("Zero Gradient", test_set, &zero_gradient_params(), |p| new_zero_gradient(p), functions, dimension_count, maxiter, seed_count, stop_residual);
    }
}

// Usage: cargo run --bin tune -r -- <test_set> [algo]
// test_set: main | mini | lmmaes
// algo: ans | ansr | ansr_dpnm | de | shade | zero_gradient | all (default)
fn main() {
    let args: Vec<String> = env::args().collect();
    let test_set = args.get(1).map(|s| s.as_str()).unwrap_or("main");
    let algo = args.get(2).map(|s| s.as_str()).unwrap_or("all");

    let seed_count = 10;
    let stop_residual = 0.01;

    match test_set {
        "main" => {
            println!(">>> Test set: MAIN (16D)");
            tune_all(algo, "main", &MAIN_TEST_FUNCTIONS, 16, 50_000, seed_count, stop_residual);
        }
        "mini" => {
            println!(">>> Test set: MINI (64D)");
            tune_all(algo, "mini", &MINI_TEST_FUNCTIONS, 64, 50_000, seed_count, stop_residual);
        }
        "medium_terrain" => {
            for d in [64, 128, 256] {
                println!("\n>>> Test set: MEDIUM_TERRAIN ({d}D)");
                tune_all(algo, "medium_terrain", &TERRAIN_TEST_FUNCTIONS, d, 500_000, seed_count, stop_residual);
            }
        }
        "medium_weierstrass" => {
            for d in [16, 64, 128, 256] {
                println!("\n>>> Test set: MEDIUM_WEIERSTRASS ({d}D)");
                tune_all(algo, "medium_weierstrass", &WEIERSTRASS_TEST_FUNCTIONS, d, 500_000, seed_count, stop_residual);
            }
        }
        "lmmaes" => {
            for d in [64] {
                println!("\n>>> Test set: LMMAES ({d}D)");
                tune_all(algo, "lmmaes", &LMMAES_TEST_FUNCTIONS, d, 50_000, seed_count, stop_residual);
            }
        }
        "easy" => {
            for d in [64, 128, 256, 512, 1024] {
                println!("\n>>> Test set: EASY ({d}D)");
                tune_all(algo, "easy", &EASY_TEST_FUNCTIONS, d, 50_000, seed_count, stop_residual);
            }
        }
        "hard" => {
            for d in [8, 16] {
                println!("\n>>> Test set: HARD ({d}D)");
                tune_all(algo, "hard", &HARD_TEST_FUNCTIONS, d, 500_000, seed_count, stop_residual);
            }
        }
        "hard_discrete" => {
            for d in [16, 32, 64] {
                println!("\n>>> Test set: HARD_DISCRETE ({d}D)");
                tune_all(algo, "hard_discrete", &HARD_DISCRETE_FUNCTIONS, d, 500_000, seed_count, stop_residual);
            }
        }
        "medium_periodic" => {
            for d in [16, 32, 64] {
                println!("\n>>> Test set: HARD_PERIODIC ({d}D)");
                tune_all(algo, "medium_periodic", &MEDIUM_PERIODIC_FUNCTIONS, d, 500_000, seed_count, stop_residual);
            }
        }
        "each" => {
            let dim = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(64);
            let maxiter_each = 50_000;
            let mut all_funcs = BTreeMap::new();
            for (k, v) in &*MAIN_TEST_FUNCTIONS { all_funcs.insert(k.clone(), v.clone()); }
            for (k, v) in &*LMMAES_TEST_FUNCTIONS { all_funcs.insert(k.clone(), v.clone()); }
            for (name, tf) in &all_funcs {
                let mut single = BTreeMap::new();
                single.insert(name.clone(), tf.clone());
                println!("\n>>> Function: {name} ({dim}D)");
                tune_all(algo, &format!("each_{name}"), &single, dim, maxiter_each, seed_count, stop_residual);
            }
        }
        _ => {
            eprintln!("Unknown test set: {}. Use: main | mini | medium_terrain | medium_weierstrass | easy | hard | lmmaes | each", test_set);
            std::process::exit(1);
        }
    }
}
