use std::collections::BTreeMap;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::{
    early_stop_callback::EarlyStopCallback,
    functions::TestFunction,
    optimizer::{Optimizer, OptimizerResult},
    utils::broadcast_simd,
};

pub fn run_multiple_optimizaions<T>(
    optimizer: &T,
    functions: &BTreeMap<String, TestFunction>,
    dimension_count: usize,
    maxiter: u64,
    seed_count: u64,
    stop_residual: f32,
    use_progress_bar: bool,
    use_par_iter: bool,
) -> BTreeMap<String, f32>
where
    T: Optimizer + Sync,
{
    let mut total_result: BTreeMap<String, f32> = BTreeMap::new();
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");
    let optional_pb = if use_progress_bar {
        let pb = m.add(ProgressBar::new(functions.len() as u64));
        pb.set_style(sty.clone());
        Some(pb)
    } else {
        None
    };
    for (function_name, function) in functions {
        let optional_seed_pb = if use_progress_bar {
            let seed_pb = m.add(ProgressBar::new(seed_count));
            seed_pb.set_style(sty.clone());
            seed_pb.set_message(function_name.clone());
            Some(seed_pb)
        } else {
            None
        };
        let func = &broadcast_simd(&function.func);
        let bounds = &function.bounds.repeat(dimension_count / 2);
        let mut total_nfev = 0;
        let early_stop_callback = EarlyStopCallback::new(func, stop_residual);
        let compute = |seed: u64| {
            let result =
                optimizer.find_infimum(func, bounds, maxiter, seed, false, &early_stop_callback);

            if let Some(seed_pb) = &optional_seed_pb {
                seed_pb.inc(1);
            }
            result
        };
        let results: Vec<OptimizerResult> = if use_par_iter {
            (0..seed_count).into_par_iter().map(compute).collect()
        } else {
            (0..seed_count).into_iter().map(compute).collect()
        };
        if let Some(seed_pb) = &optional_seed_pb {
            seed_pb.finish_and_clear();
        }
        for result in results {
            if result.f_x <= stop_residual {
                total_nfev += result.nfev;
            } else {
                total_nfev = u64::MAX;
                break;
            }
        }
        if total_nfev != u64::MAX {
            total_result.insert(function_name.clone(), total_nfev as f32 / seed_count as f32);
        } else {
            total_result.insert(function_name.clone(), f32::INFINITY);
        }
        if let Some(pb) = &optional_pb {
            pb.inc(1);
        }
    }
    total_result.insert(
        "mean".to_string(),
        total_result.values().sum::<f32>() / total_result.len() as f32,
    );
    total_result
}
