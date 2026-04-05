use std::{collections::BTreeMap, fs::File, io::Write};

use optimizers::{
    benchmark_params::get_params,
    functions::{EASY_TEST_FUNCTIONS, HARD_TEST_FUNCTIONS, HARD_DISCRETE_FUNCTIONS, MEDIUM_PERIODIC_FUNCTIONS, LMMAES_TEST_FUNCTIONS, MAIN_TEST_FUNCTIONS, TERRAIN_TEST_FUNCTIONS, MINI_TEST_FUNCTIONS, WEIERSTRASS_TEST_FUNCTIONS, TestFunction},
    optimizer::Optimizer,
    runner::{run_multiple_optimizations_detailed, SeedResult},
};

fn run_algo<T: Optimizer + Sync>(
    name: &str,
    test_set: &str,
    dim: usize,
    optimizer: &T,
    functions: &BTreeMap<String, TestFunction>,
    maxiter: u64,
    csv: &mut File,
) {
    let results = run_multiple_optimizations_detailed(
        optimizer, functions, dim, maxiter, 200, 0.01, true, true,
    );
    for r in &results {
        writeln!(csv, "{},{},{},{},{},{},{}", test_set, dim, name, r.function, r.seed, r.f_x, r.nfev).unwrap();
    }
    let mut by_func: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for r in &results {
        let entry = by_func.entry(r.function.clone()).or_insert((0, 0));
        if r.f_x <= 0.01 {
            entry.0 += r.nfev;
            entry.1 += 1;
        }
    }
    print!("{name}:");
    for (func, (total_nfev, count)) in &by_func {
        if *count == 200 {
            print!(" {func}={:.2}", *total_nfev as f32 / *count as f32);
        } else {
            print!(" {func}=inf");
        }
    }
    println!();
}

fn run_all(test_set: &str, functions: &BTreeMap<String, TestFunction>, dim: usize, maxiter: u64, csv: &mut File) {
    let p = get_params(test_set, dim);
    run_algo("ans", test_set, dim, &p.ans, functions, maxiter, csv);
    run_algo("ans_sort", test_set, dim, &p.ans_sort, functions, maxiter, csv);
    run_algo("ansr", test_set, dim, &p.ansr, functions, maxiter, csv);
    run_algo("ansr_dpnm", test_set, dim, &p.ansr_dpnm, functions, maxiter, csv);
    run_algo("de", test_set, dim, &p.de, functions, maxiter, csv);
    run_algo("shade", test_set, dim, &p.shade, functions, maxiter, csv);
    run_algo("zero_gradient", test_set, dim, &p.zero_gradient, functions, maxiter, csv);
}

fn main() {
    let mut csv = File::create("benchmark_results.csv").unwrap();
    writeln!(csv, "test_set,dim,algorithm,function,seed,f_x,nfev").unwrap();

    for d in [64, 128, 256, 512, 1024] {
        println!("\n=== easy test {d}D ===");
        run_all("easy", &EASY_TEST_FUNCTIONS, d, 50_000, &mut csv);
    }

    for d in [64, 128, 256] {
        println!("\n=== medium_terrain test {d}D ===");
        run_all("medium_terrain", &TERRAIN_TEST_FUNCTIONS, d, 500_000, &mut csv);
    }

    for d in [16, 32, 64] {
        println!("\n=== medium_periodic test {d}D ===");
        run_all("medium_periodic", &MEDIUM_PERIODIC_FUNCTIONS, d, 500_000, &mut csv);
    }

    for d in [16, 32, 64] {
        println!("\n=== hard_discrete test {d}D ===");
        run_all("hard_discrete", &HARD_DISCRETE_FUNCTIONS, d, 500_000, &mut csv);
    }

    println!("\nResults saved to benchmark_results.csv");
}
