# Optimizers

Rust implementations of population-based optimization algorithms, benchmarked on functions spanning 16--1024 dimensions with 200 independent runs per configuration.

## Algorithms

| Algorithm | Description | Parameters |
|-----------|-------------|------------|
| **ANS** | Across Neighbourhood Search (Wu 2016). Each particle perturbs around its own best or a random neighbour's best. | m, sigma, p_self |
| **ANSR** | ANS + pairwise convergence detection with random restart. | m, sigma, p_self, tau |
| **ANSR DPNM** | Adaptive ANSR with cosine sigma annealing, power-law restart decay, opposition-based restart, toroidal boundaries. | m, sigma, p_self, tau, p, m_n |
| **DE** | Differential Evolution (DE/rand/1/bin). | m, F, CR |
| **SHADE** | Success-History Adaptive DE with current-to-pbest/1 mutation and external archive. | m, H, p_best |
| **Zero-Gradient** | Coordinate-wise line search with doubling steps and bisection refinement. | init_jump |

All population-based algorithms use a fixed population size of 64.

## Benchmark functions

| Group | Functions | Dims | Maxiter |
|-------|-----------|------|---------|
| Easy | sphere, shifted_sphere, ellipsoid, discus, different_powers, rosenbrock | 64--1024 | 50k |
| Medium terrain | hilly, forest | 64, 128, 256 | 500k |
| Medium periodic | shubert | 16, 32, 64 | 500k |
| Hard discrete | megacity | 16, 32, 64 | 500k |

All functions operate on coordinate pairs and are scaled to [0, 1] output range. A run is successful if residual f(x) <= 0.01.

## Results

### Easy functions (64--1024D)

Median of per-function median nfev across 6 functions. All algorithms 100% success rate except SHADE (fails shifted_sphere at 1024D) and ZG (fails discus at 128D).

| Dim | ANS | ANSR | DPNM | DE | SHADE | ZG |
|-----|-----|------|------|----|-------|----|
| 64 | 1856 | 1856 | 3072 | 2528 | **1856** | 1380 |
| 128 | 2832 | 2848 | 5632 | 3968 | **2176** | 2669 |
| 256 | 4352 | 4352 | 10112 | 6272 | **3264** | 5340 |
| 512 | 6976 | 6960 | 16864 | 10048 | **5376** | 10689 |
| 1024 | **12288** | **12288** | 23776 | 16832 | ---* | 21350 |

\*SHADE fails on shifted_sphere at 1024D (0% success rate).

### Medium terrain (64--256D)

Median nfev (success rate if < 100%). ZG fails entirely.

| Dim | Func | ANS | ANSR | DPNM | DE | SHADE |
|-----|------|-----|------|------|----|-------|
| 64 | hilly | 70.7k (97.5%) | 70.7k (97%) | 61.8k (99.5%) | **42.4k** (72.5%) | 286k (96.5%) |
| 64 | forest | 47.3k | 47.3k | 47.1k | **29.5k** (96%) | 86.0k (92%) |
| 128 | hilly | 159k (96%) | 111k (83.5%) | **101k** (97.5%) | 137k (99%) | --- |
| 128 | forest | 101k | 73.1k | **69.6k** | 85.6k | 196k (99.5%) |
| 256 | hilly | 274k (94.5%) | 334k (98%) | --- | **202k** (95%) | --- |
| 256 | forest | 175k | 209k | --- | **128k** | 450k (94%) |

### Medium periodic: Shubert (16--64D)

| Dim | ANS | ANSR | DPNM | DE | SHADE |
|-----|-----|------|------|----|-------|
| 16 | 13.8k (98%) | **9.1k** (100%) | 120k (100%) | 16.6k (100%) | 41.1k (100%) |
| 32 | 44.7k (87%) | **28.4k** (100%) | 403k (100%) | 37.6k (100%) | 103k (100%) |
| 64 | 187k (77.5%) | **60.3k** (100%) | --- (0%) | 81.8k (100%) | 245k (100%) |

### Hard discrete: Megacity (16--64D)

| Dim | ANS | ANSR | DPNM | DE | SHADE |
|-----|-----|------|------|-----|-------|
| 16 | 126k (73%) | --- | --- | 36.8k (80%) | **36.3k** (75%) |
| 32 | --- | --- | --- | **91.8k** (80%) | 115k (73.5%) |
| 64 | --- | --- | --- | **217k** (90.5%) | 364k (7%) |

### Summary

| Algorithm | Easy | Terrain | Periodic | Discrete |
|-----------|------|---------|----------|----------|
| ANS | Reliable | Reliable | Degrades | Poor |
| ANSR | Reliable | Reliable | **Best** | Poor |
| DPNM | Slow | Fails 256D | Fails 64D | Poor |
| DE | Reliable | Reliable | Good | **Best** |
| SHADE | Fast* | Fails hilly >= 128D | Good | Degrades |
| ZG | Scales poorly | Fails | Fails | Fails |

\*Fast but fails at 1024D. No single algorithm dominates all problem types.

## Tuned parameters by problem type

Grid search over 600 (ANS/ANSR), 600 (DPNM), and 600 (DE/SHADE) configurations per group, 10 seeds each. The tuned values reveal distinct parameter regimes across problem classes.

### ANS / ANSR

tau = 10^-8 for ANSR in all cases (no per-problem tuning needed).

| Group | Dim | Alg | sigma | p_self |
|-------|-----|-----|-------|--------|
| Easy | 64--256 | ANS/ANSR | 0.12 | 0.00 |
| Easy | 512 | ANS/ANSR | 0.12 | 0.04 |
| Easy | 1024 | ANS/ANSR | 0.16 | 0.16 |
| Terrain | 64 | ANS/ANSR | 0.32 | 0.92 |
| Terrain | 128 | ANS | 0.36 | 0.92 |
| | | ANSR | 0.28 | 0.92 |
| Terrain | 256 | ANS | 0.32 | 0.96 |
| | | ANSR | 0.36 | 0.96 |
| Periodic | 16 | ANS | 0.04 | 0.24 |
| | | ANSR | 0.04 | 0.00 |
| Periodic | 32 | ANS | 0.04 | 0.40 |
| | | ANSR | 0.04 | 0.04 |
| Periodic | 64 | ANS | 0.04 | 0.56 |
| | | ANSR | 0.04 | 0.00 |
| Discrete | 16 | ANS | 0.20 | 0.80 |
| | | ANSR | 0.04 | 0.00 |
| Discrete | 32--64 | ANS/ANSR | 0.04 | 0.00 |

### DE / SHADE

| Group | Dim | DE F | DE CR | SHADE H | SHADE p_best |
|-------|-----|------|-------|---------|--------------|
| Easy | 64 | 0.12 | 0.60 | 1 | 0.28 |
| Easy | 128 | 0.12 | 0.60 | 1 | 0.72 |
| Easy | 256 | 0.12 | 0.52 | 9 | 0.52 |
| Easy | 512 | 0.12 | 0.44 | 24 | 0.76 |
| Easy | 1024 | 0.12 | 0.32 | 1 | 0.04 |
| Terrain | 64 | 0.20 | 0.12 | 24 | 0.08 |
| Terrain | 128 | 0.32 | 0.08 | 1 | 0.04 |
| Terrain | 256 | 0.24 | 0.04 | 1 | 0.04 |
| Periodic | 16--64 | 0.04 | 0.00 | 2--14 | 0.08--0.12 |
| Discrete | 16 | 0.56 | 0.40 | 16 | 0.20 |
| Discrete | 32 | 0.52 | 0.40 | 22 | 0.32 |
| Discrete | 64 | 0.64 | 0.16 | 1 | 0.04 |

### ANSR DPNM

p = 2.0 (restart decay power) in all cases.

| Group | Dim | sigma | p_self | m_n |
|-------|-----|-------|--------|-----|
| Easy | 64 | 0.20 | 0.40 | 0.50 |
| Easy | 128--256 | 0.20 | 0.60 | 0.50 |
| Easy | 512 | 0.20 | 0.80 | 0.50 |
| Easy | 1024 | 0.20 | 0.60 | 0.50 |
| Terrain | 64 | 0.20 | 0.80 | 0.75 |
| Terrain | 128 | 0.20 | 0.80 | 0.50 |
| Terrain | 256 | 0.20 | 0.00 | 0.50 |
| Periodic | 16 | 0.20 | 0.20 | 0.50 |
| Periodic | 32 | 0.20 | 0.60 | 0.50 |
| Periodic | 64 | 0.20 | 0.00 | 0.50 |
| Discrete | 16--64 | 0.20 | 0.00 | 0.50 |

### Parameter regime patterns

**Easy (unimodal):** ANS/ANSR use low sigma (0.12) with p_self ~ 0 (pure social learning) --- on a single basin, every neighbour's best is informative. DE uses low F (0.12) with high CR that decreases with dimension (0.60 at 64D to 0.32 at 1024D).

**Terrain (multimodal, smooth):** ANS/ANSR switch to high sigma (0.28--0.36) with p_self ~ 0.95 (almost pure individual learning) --- neighbours are likely in different basins, so following them is destructive. DE uses low CR (0.04--0.12), perturbing very few dimensions per trial.

**Periodic (Shubert):** All algorithms use minimal perturbation (sigma = 0.04, F = 0.04, CR = 0.00). Basins are narrow and separated by steep ridges --- large steps waste evaluations. ANSR can afford p_self ~ 0 because restarts maintain diversity; ANS compensates with higher p_self (0.24--0.56) but still degrades.

**Discrete (megacity):** DE uses large mutations (F = 0.52--0.64) to jump across flat plateaus. ANS-family perturbations scaled by particle distances are ineffective on step landscapes where gradient signal is zero everywhere.

## CLI tools

| Binary | Purpose | Command |
|--------|---------|---------|
| `benchmark` | Run all benchmarks (200 seeds) | `cargo run --bin benchmark -r` |
| `tune` | Grid search for optimal parameters (10 seeds) | `cargo run --bin tune -r` |
| `plot` | Visualize a single function | `cargo run --bin plot -r` |

## References

- G. Wu, "Across neighbourhood search for numerical optimization," *Information Sciences*, vol. 329, pp. 597--618, 2016. [doi:10.1016/j.ins.2015.09.051](https://doi.org/10.1016/j.ins.2015.09.051)
- R. Storn and K. Price, "Differential Evolution -- A Simple and Efficient Heuristic for global Optimization over Continuous Spaces," *Journal of Global Optimization*, vol. 11, no. 4, pp. 341--359, 1997.
- R. Tanabe and A. Fukunaga, "Success-History Based Parameter Adaptation for Differential Evolution," in *IEEE Congress on Evolutionary Computation*, 2013, pp. 71--78.
