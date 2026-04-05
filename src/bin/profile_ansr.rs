use std::time::Instant;

use optimizers::{
    algorithms::ansr::ANSR,
    early_stop_callback::EarlyStopCallback,
    functions::LMMAES_TEST_FUNCTIONS,
    optimizer::Optimizer,
    utils::broadcast_simd,
};

fn main() {
    let optimizer = ANSR {
        popsize: 64,
        restart_tolerance: 1e-5,
        sigma: 0.1,
        self_instead_neighbour: 0.0,
    };
    let dim = 1024;
    let maxiter = 100_000u64;
    let stop_residual = 0.01;

    for (name, tf) in LMMAES_TEST_FUNCTIONS.iter() {
        let func = broadcast_simd(tf.func);
        let bounds = tf.bounds.repeat(dim / 2);
        let early_stop = EarlyStopCallback::new(&func, stop_residual);

        let start = Instant::now();
        let result = optimizer.find_infimum(&func, &bounds, maxiter, 0, false, &early_stop);
        let elapsed = start.elapsed();

        let evals_per_sec = result.nfev as f64 / elapsed.as_secs_f64();
        println!(
            "{:20} f_x={:12.4} nfev={:8} time={:8.3}s  evals/s={:.0}",
            name, result.f_x, result.nfev, elapsed.as_secs_f64(), evals_per_sec
        );
    }

    // Now profile individual phases on sphere
    println!("\n--- Phase breakdown on sphere 1024D (1 epoch = 64 evals) ---");
    let tf = &LMMAES_TEST_FUNCTIONS["sphere"];
    let func = broadcast_simd(tf.func);
    let bounds = tf.bounds.repeat(dim / 2);

    // Time just function evals
    use rand::SeedableRng;
    use rand_distr::{Distribution, Uniform};
    use rand_pcg::Pcg64Mcg;
    use simd_vector::Vec8;
    use optimizers::utils::BoundsSimd;

    let params = bounds.len();
    let popsize = 64usize;
    let mut rng: Pcg64Mcg = SeedableRng::seed_from_u64(0);
    let random = Uniform::new_inclusive(0.0f32, 1.0).unwrap();
    let mut positions: Vec<Vec<f32>> = (0..popsize)
        .map(|_| (0..params).map(|_| random.sample(&mut rng)).collect())
        .collect();

    let range_min: Vec<f32> = bounds.iter().map(|b| b[0]).collect();
    let range_max: Vec<f32> = bounds.iter().map(|b| b[1]).collect();
    let bounds_simd = BoundsSimd::new(&range_min, &range_max);
    let mut simd_buf = vec![Vec8::ZERO; bounds_simd.output_len()];

    let n_epochs = 1000;

    // Time transform_into
    let start = Instant::now();
    for _ in 0..n_epochs {
        for p in 0..popsize {
            bounds_simd.transform_into(&positions[p], &mut simd_buf);
        }
    }
    let t_transform = start.elapsed();

    // Time func eval
    let start = Instant::now();
    for _ in 0..n_epochs {
        for p in 0..popsize {
            bounds_simd.transform_into(&positions[p], &mut simd_buf);
            std::hint::black_box(func(&simd_buf));
        }
    }
    let t_eval = start.elapsed();

    // Time perturbation (the inner loop over dims)
    let normal = rand_distr::Normal::new(0.0f32, 0.1).unwrap();
    let popsize_distr = Uniform::new(0usize, popsize).unwrap();
    let mut best_positions = positions.clone();
    let start = Instant::now();
    for _ in 0..n_epochs {
        for p in 0..popsize {
            for d in 0..params {
                // neighbour path (sin=0.0)
                let mut r = popsize_distr.sample(&mut rng);
                while r == p {
                    r = popsize_distr.sample(&mut rng);
                }
                let val = best_positions[r][d]
                    + normal.sample(&mut rng)
                        * f32::abs(best_positions[r][d] - positions[p][d]);
                positions[p][d] = val.clamp(0.0, 1.0);
            }
        }
    }
    let t_perturb = start.elapsed();

    let total_evals = (n_epochs * popsize) as f64;
    println!(
        "transform_into:  {:8.3}s ({:.1} us/eval)",
        t_transform.as_secs_f64(),
        t_transform.as_secs_f64() / total_evals * 1e6
    );
    println!(
        "eval (incl xform):{:8.3}s ({:.1} us/eval)",
        t_eval.as_secs_f64(),
        t_eval.as_secs_f64() / total_evals * 1e6
    );
    println!(
        "func only:       {:8.3}s ({:.1} us/eval)",
        (t_eval - t_transform).as_secs_f64(),
        (t_eval - t_transform).as_secs_f64() / total_evals * 1e6
    );
    println!(
        "perturbation:    {:8.3}s ({:.1} us/eval)",
        t_perturb.as_secs_f64(),
        t_perturb.as_secs_f64() / total_evals * 1e6
    );
    println!(
        "perturb/eval ratio: {:.1}x",
        t_perturb.as_secs_f64() / t_eval.as_secs_f64()
    );
    println!("params (dim): {}", params);
}
