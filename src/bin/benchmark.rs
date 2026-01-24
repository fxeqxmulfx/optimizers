use optimizers::{
    functions::TEST_FUNCTIONS, optimizers::ALGORITHMS, runner::run_multiple_optimizaions,
};

fn main() {
    let optimizer = ALGORITHMS[0].algorithm;
    let functions = &TEST_FUNCTIONS;
    let dimension_count = 16;
    let maxiter = 10_000_000;
    let seed_count = 200;
    let stop_residual = 0.01;
    let result = run_multiple_optimizaions(
        optimizer,
        functions,
        dimension_count,
        maxiter,
        seed_count,
        stop_residual,
    );
    println!(
        "mean: {}, shifted_sphere: {}, shifted_weierstrass: {}, hilly: {}, forest: {}, megacity: {}",
        result["mean"],
        result["shifted_sphere"],
        result["shifted_weierstrass"],
        result["hilly"],
        result["forest"],
        result["megacity"]
    );
}
