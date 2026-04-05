import numpy as np
from dataclasses import dataclass


@dataclass
class OptimizerResult:
    x: np.ndarray
    f_x: float
    nfev: int


def ansr(
    func,
    bounds: np.ndarray,
    maxiter: int = 100_000,
    seed: int = 0,
    stop_residual: float = 1e-6,
    popsize: int = 4,
    restart_tolerance: float = 0.01,
    sigma: float = 0.05,
    self_instead_neighbour: float = 0.9,
) -> OptimizerResult:
    rng = np.random.default_rng(seed)
    params = len(bounds)
    range_min = bounds[:, 0]
    range_max = bounds[:, 1]
    max_epoch = int(np.ceil(maxiter / popsize))

    current_positions = rng.uniform(0.0, 1.0, size=(popsize, params))
    best_positions = np.zeros_like(current_positions)
    best_residuals = np.full(popsize, np.inf)
    current_residuals = np.full(popsize, np.inf)
    ind = 0
    current_epoch = 0

    for epoch in range(max_epoch):
        # evaluate
        for p in range(popsize):
            x = range_min + current_positions[p] * (range_max - range_min)
            current_residuals[p] = func(x)

        # update best
        for p in range(popsize):
            if current_residuals[p] < best_residuals[p]:
                best_residuals[p] = current_residuals[p]
                best_positions[p] = current_positions[p].copy()
                if best_residuals[p] < best_residuals[ind]:
                    ind = p

        current_epoch = epoch
        if best_residuals[ind] <= stop_residual:
            break

        # restart mechanism
        for lhs in range(popsize):
            if best_residuals[lhs] == np.inf:
                continue
            for rhs in range(lhs + 1, popsize):
                if best_residuals[rhs] == np.inf:
                    continue
                mn = min(best_residuals[lhs], best_residuals[rhs])
                mx = max(best_residuals[lhs], best_residuals[rhs])
                if np.isfinite(mx) and mx != 0.0 and (mx - mn) / mx < restart_tolerance:
                    if lhs == ind or (rhs != ind and best_residuals[lhs] < best_residuals[rhs]):
                        loser = rhs
                    else:
                        loser = lhs
                    best_residuals[loser] = np.inf
                    best_positions[loser] = rng.uniform(0.0, 1.0, size=params)
                    current_positions[loser] = rng.uniform(0.0, 1.0, size=params)

        # perturbation
        for p in range(popsize):
            for d in range(params):
                if rng.uniform() <= self_instead_neighbour:
                    current_positions[p, d] = np.clip(
                        best_positions[p, d]
                        + rng.normal(0.0, sigma)
                        * abs(best_positions[p, d] - current_positions[p, d]),
                        0.0,
                        1.0,
                    )
                else:
                    r = rng.integers(0, popsize)
                    while r == p:
                        r = rng.integers(0, popsize)
                    current_positions[p, d] = np.clip(
                        best_positions[r, d]
                        + rng.normal(0.0, sigma)
                        * abs(best_positions[r, d] - current_positions[p, d]),
                        0.0,
                        1.0,
                    )

    x_best = range_min + best_positions[ind] * (range_max - range_min)
    return OptimizerResult(x=x_best, f_x=best_residuals[ind], nfev=(current_epoch + 1) * popsize)


# ---------------------------------------------------------------------------
# LMMAES test functions (2D, scaled to [0,1])
# ---------------------------------------------------------------------------

def _scale(v, in_min, in_max):
    return (v - in_min) / (in_max - in_min)


def sphere(x, y):
    return _scale(x**2 + y**2, 0.0, 50.0)


def ellipsoid(x, y):
    return _scale(x**2 + 1_000_000.0 * y**2, 0.0, 25_000_025.0)


def rosenbrock(x, y):
    return _scale(100.0 * (x**2 - y) ** 2 + (x - 1.0) ** 2, 0.0, 90_036.0)


def discus(x, y):
    return _scale(1_000_000.0 * x**2 + y**2, 0.0, 25_000_025.0)


def different_powers(x, y):
    return _scale(x**2 + y**6, 0.0, 15_650.0)


LMMAES_TEST_FUNCTIONS = {
    "sphere":           (sphere,           np.array([[-5, 5], [-5, 5]], dtype=float)),
    "ellipsoid":        (ellipsoid,        np.array([[-5, 5], [-5, 5]], dtype=float)),
    "rosenbrock":       (rosenbrock,       np.array([[-5, 5], [-5, 5]], dtype=float)),
    "discus":           (discus,           np.array([[-5, 5], [-5, 5]], dtype=float)),
    "different_powers": (different_powers, np.array([[-5, 5], [-5, 5]], dtype=float)),
}


def broadcast(func_2d):
    """Wrap a 2D test function for multi-dimensional optimization.

    Takes pairs of coordinates, evaluates func_2d on each pair, returns the mean.
    """
    def wrapper(x):
        total = 0.0
        n_pairs = len(x) // 2
        for i in range(n_pairs):
            total += func_2d(x[2 * i], x[2 * i + 1])
        return total / n_pairs
    return wrapper


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    passed = 0
    failed = 0

    def check(name, cond, msg=""):
        global passed, failed
        if cond:
            passed += 1
            print(f"  PASS  {name}")
        else:
            failed += 1
            print(f"  FAIL  {name}  {msg}")

    # test function extrema
    print("--- function extrema ---")
    check("sphere min",           abs(sphere(0, 0))           < 1e-6)
    check("sphere max",           abs(sphere(5, 5) - 1)       < 1e-6)
    check("ellipsoid min",        abs(ellipsoid(0, 0))        < 1e-6)
    check("ellipsoid max",        abs(ellipsoid(5, 5) - 1)    < 1e-6)
    check("rosenbrock min",       abs(rosenbrock(1, 1))       < 1e-6)
    check("rosenbrock max",       abs(rosenbrock(-5, -5) - 1) < 1e-6)
    check("discus min",           abs(discus(0, 0))           < 1e-6)
    check("discus max",           abs(discus(5, 5) - 1)       < 1e-6)
    check("different_powers min", abs(different_powers(0, 0)) < 1e-6)
    check("different_powers max", abs(different_powers(5, 5) - 1) < 1e-3)

    # ANSR convergence on each lmmaes function
    print("\n--- ANSR convergence (16D, 100k evals) ---")
    for name, (fn2d, bounds_2d) in LMMAES_TEST_FUNCTIONS.items():
        func = broadcast(fn2d)
        bounds = np.tile(bounds_2d, (8, 1))  # 16D
        result = ansr(func, bounds, maxiter=100_000, seed=0, stop_residual=0.01,
                      popsize=4, restart_tolerance=0.01, sigma=0.05,
                      self_instead_neighbour=0.9)
        check(f"{name:20s} f_x={result.f_x:.6f}  nfev={result.nfev}",
              result.f_x <= 0.01,
              f"did not converge")

    # determinism
    print("\n--- determinism ---")
    func = broadcast(sphere)
    bounds = np.tile(np.array([[-5, 5], [-5, 5]], dtype=float), (8, 1))
    r1 = ansr(func, bounds, maxiter=10_000, seed=42, stop_residual=0.01)
    r2 = ansr(func, bounds, maxiter=10_000, seed=42, stop_residual=0.01)
    check("same seed -> same f_x",  r1.f_x == r2.f_x)
    check("same seed -> same nfev", r1.nfev == r2.nfev)

    print(f"\n{passed} passed, {failed} failed")
