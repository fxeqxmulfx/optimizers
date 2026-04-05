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

pub struct SeedResult {
    pub function: String,
    pub seed: u64,
    pub f_x: f32,
    pub nfev: u64,
}

pub fn run_multiple_optimizations_detailed<T>(
    optimizer: &T,
    functions: &BTreeMap<String, TestFunction>,
    dimension_count: usize,
    maxiter: u64,
    seed_count: u64,
    stop_residual: f32,
    use_progress_bar: bool,
    use_par_iter: bool,
) -> Vec<SeedResult>
where
    T: Optimizer + Sync,
{
    let mut all_results: Vec<SeedResult> = Vec::new();
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
        for (seed, result) in results.into_iter().enumerate() {
            all_results.push(SeedResult {
                function: function_name.clone(),
                seed: seed as u64,
                f_x: result.f_x,
                nfev: result.nfev,
            });
        }
        if let Some(pb) = &optional_pb {
            pb.inc(1);
        }
    }
    all_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        default_algorithms_params::DEFAULT_ANSR,
        functions::MINI_TEST_FUNCTIONS,
    };

    #[test]
    fn test_run_multiple_sequential() {
        let result = run_multiple_optimizaions(
            &DEFAULT_ANSR,
            &MINI_TEST_FUNCTIONS,
            16,
            10_000,
            2,
            0.1,
            false,
            false,
        );
        assert!(result.contains_key("mean"));
        assert!(result.contains_key("shifted_sphere"));
        assert!(result.contains_key("hilly"));
        assert!(result.contains_key("forest"));
    }

    #[test]
    fn test_run_multiple_parallel() {
        let result = run_multiple_optimizaions(
            &DEFAULT_ANSR,
            &MINI_TEST_FUNCTIONS,
            16,
            10_000,
            2,
            0.1,
            false,
            true,
        );
        assert!(result.contains_key("mean"));
    }

    #[test]
    fn test_run_multiple_with_progress_bar() {
        let result = run_multiple_optimizaions(
            &DEFAULT_ANSR,
            &MINI_TEST_FUNCTIONS,
            16,
            10_000,
            1,
            0.1,
            true,
            false,
        );
        assert!(result.contains_key("mean"));
    }

    #[test]
    fn test_run_multiple_impossible_target() {
        let result = run_multiple_optimizaions(
            &DEFAULT_ANSR,
            &MINI_TEST_FUNCTIONS,
            16,
            100,
            1,
            0.0,
            false,
            false,
        );
        assert_eq!(result["shifted_sphere"], f32::INFINITY);
    }
}
