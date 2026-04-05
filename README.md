# Extending Across Neighbourhood Search with Restarts

This repository contains Rust implementations of six optimization algorithms benchmarked on four problem groups spanning 16--1024 dimensions with 200 independent runs per configuration.

**ANSR** (ANS with Restarts) adds a single mechanism to the original ANS: when two particles converge to similar fitness, the worse one is reset to a random position. This minimal change improves Shubert 64D success rate from 77.5% to 100% while adding zero overhead on unimodal problems.

**ANSR DPNM** (negative result) adds cosine sigma annealing, power-law restart decay, opposition-based reinitialization, and toroidal boundary handling. Despite more adaptive mechanisms, it consistently underperforms plain ANSR --- failing entirely on Shubert at 64D and on terrain functions at 256D.

## Algorithms

| Algorithm | Description | Parameters |
|-----------|-------------|------------|
| **ANS** | Wu's original Across Neighbourhood Search. Each particle perturbs around its own best or a random neighbour's best, controlled by p_self. | m, sigma, p_self |
| **ANSR** | ANS + pairwise convergence detection with random restart. | m, sigma, p_self, tau |
| **ANSR DPNM** | Adaptive ANSR with decay power and neighbour multiplier (negative result). | m, sigma, p_self, tau, p, m_n |
| **DE** | Classic Differential Evolution (DE/rand/1/bin). | m, F, CR |
| **SHADE** | Success-History Adaptive DE with current-to-pbest/1 mutation and external archive. | m, H, p_best |
| **Zero-Gradient** | Coordinate-wise line search using doubling steps and bisection refinement. | init_jump |

All population-based algorithms use a fixed population size of 64.

## Benchmark functions

| Group | Functions | Dims | Maxiter |
|-------|-----------|------|---------|
| Easy | sphere, shifted_sphere, ellipsoid, discus, different_powers, rosenbrock | 64--1024 | 50k |
| Medium terrain | hilly, forest | 64, 128, 256 | 500k |
| Medium periodic | shubert | 16, 32, 64 | 500k |
| Hard discrete | megacity | 16, 32, 64 | 500k |

All functions operate on coordinate pairs and are scaled to [0, 1] output range. A run is successful if residual f(x) <= 0.01. Results report success rate and median nfev over 200 independent runs.

## Results

### Easy functions (64--1024D)

Median of per-function median nfev across 6 functions. All algorithms achieve 100% success rate except SHADE (fails shifted_sphere at 1024D) and ZG (fails discus at 128D).

| Dim | ANS | ANSR | DPNM | DE | SHADE | ZG |
|-----|-----|------|------|----|-------|----|
| 64 | 1856 | 1856 | 3072 | 2528 | **1856** | 1380 |
| 128 | 2832 | 2848 | 5632 | 3968 | **2176** | 2669 |
| 256 | 4352 | 4352 | 10112 | 6272 | **3264** | 5340 |
| 512 | 6976 | 6960 | 16864 | 10048 | **5376** | 10689 |
| 1024 | **12288** | **12288** | 23776 | 16832 | ---* | 21350 |

\*SHADE fails on shifted_sphere at 1024D (0% success rate).

ANS and ANSR are identical on easy functions --- the restart mechanism never triggers. DPNM is 2--3x slower at every dimension. SHADE is fastest at 128--512D but fails at 1024D.

### Medium terrain (64--256D)

Median nfev (success rate if < 100%). ZG fails entirely and is omitted.

| Dim | Func | ANS | ANSR | DPNM | DE | SHADE |
|-----|------|-----|------|------|----|-------|
| 64 | hilly | 70.7k (97.5%) | 70.7k (97%) | 61.8k (99.5%) | **42.4k** (72.5%) | 286k (96.5%) |
| 64 | forest | 47.3k | 47.3k | 47.1k | **29.5k** (96%) | 86.0k (92%) |
| 128 | hilly | 159k (96%) | 111k (83.5%) | **101k** (97.5%) | 137k (99%) | --- |
| 128 | forest | 101k | 73.1k | **69.6k** | 85.6k | 196k (99.5%) |
| 256 | hilly | 274k (94.5%) | 334k (98%) | --- | **202k** (95%) | --- |
| 256 | forest | 175k | 209k | --- | **128k** | 450k (94%) |

DPNM fails at 256D on both functions (0% success). SHADE fails on hilly at 128D and 256D.

### Medium periodic: Shubert (16--64D)

The critical test case for the restart mechanism.

| Dim | ANS | **ANSR** | DPNM | DE | SHADE |
|-----|-----|----------|------|----|-------|
| 16 | 13.8k (98%) | **9.1k** (100%) | 120k (100%) | 16.6k (100%) | 41.1k (100%) |
| 32 | 44.7k (87%) | **28.4k** (100%) | 403k (100%) | 37.6k (100%) | 103k (100%) |
| 64 | 187k (77.5%) | **60.3k** (100%) | --- (0%) | 81.8k (100%) | 245k (100%) |

ANSR maintains 100% success rate at all dimensions and is the fastest algorithm. ANS degrades from 98% to 77.5% as dimensionality increases. DPNM fails entirely at 64D. ANSR is 3.1x faster than ANS at 64D (60k vs 187k) and 1.4x faster than DE (60k vs 82k). All differences are statistically significant (Mann--Whitney p < 10^-30, large effect sizes).

### Hard discrete: Megacity (16--64D)

| Dim | ANS | ANSR | DPNM | **DE** | SHADE |
|-----|-----|------|------|--------|-------|
| 16 | 126k (73%) | --- | --- | 36.8k (80%) | **36.3k** (75%) |
| 32 | --- | --- | --- | **91.8k** (80%) | 115k (73.5%) |
| 64 | --- | --- | --- | **217k** (90.5%) | 364k (7%) |

DE is the clear winner on discrete landscapes. The ANS family fails because Gaussian perturbation scaled by particle distances is ineffective on step landscapes. ANSR's restart mechanism is actively harmful here --- it discards progress toward the integer grid.

### Summary

| Algorithm | Easy | Terrain | Periodic | Discrete |
|-----------|------|---------|----------|----------|
| ANS | Reliable | Reliable | Degrades | Poor |
| **ANSR** | **Reliable** | **Reliable** | **Best** | Poor |
| DPNM | Slow | Fails 256D | Fails 64D | Poor |
| DE | Reliable | Reliable | Good | **Best** |
| SHADE | Fast* | Fails hilly >= 128D | Good | Degrades |
| ZG | Scales poorly | Fails | Fails | Fails |

\*Fast but fails at 1024D.

## Key findings

1. **Restarts are the single most impactful modification to ANS.** On Shubert 64D, only 3 of 600 ANS parameter configurations converge on all seeds; ANSR finds 31. The best ANS nfev (197k) is 3.4x worse than the best ANSR (58k). No parameter setting compensates for the absence of restarts.

2. **Additional adaptive complexity does not help.** DPNM's cosine annealing drives sigma to zero too early, and decaying restart tolerance turns off restarts when they are most needed. It is 2--3x slower on easy functions and fails entirely at high dimensions on terrain and periodic problems.

3. **p_self acts as a switch between two search modes.** On unimodal problems, p_self ~ 0 (social learning) is optimal --- all neighbours point toward the same basin. On multimodal problems, p_self ~ 1 (individual learning) is optimal --- neighbours are in different basins. Restarts enable pure social search on periodic problems by maintaining diversity externally.

4. **tau = 10^-8 requires no per-problem tuning**, making ANSR effectively a three-parameter algorithm.

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
