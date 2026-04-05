## Optimizers

### CLI tools

| Binary      | Purpose                                                                   | Command                        |
| ----------- | ------------------------------------------------------------------------- | ------------------------------ |
| `benchmark` | Runs all performance tests                                                | `cargo run --bin benchmark -r` |
| `plot`      | Runs a single benchmark and draws the result for a selected function only | `cargo run --bin plot -r`      |
| `tune`      | Searches for the optimal set of algorithm parameters                      | `cargo run --bin tune -r`      |

---

### Algorithms

| Algorithm   | Type | Parameters | Description |
| ----------- | ---- | ---------- | ----------- |
| ANS         | Population | popsize, sigma, p_self | Original Across Neighbourhood Search (Wu 2015) |
| ANS Sort    | Population | popsize, sigma, p_self | ANS with sorted population archive (2*popsize best solutions kept sorted) |
| ANSR        | Population | popsize, sigma, p_self, tau | ANS with pairwise restart detection |
| ANSR DPNM   | Population | popsize, sigma, p_self, tau, decay_power, neighbour_mult | Adaptive ANSR with decay and neighbour scaling |
| DE          | Population | popsize, F, CR | Differential Evolution (DE/rand/1/bin) |
| SHADE       | Population | popsize, H, p_best | Success-History Adaptive DE |
| Zero-Gradient | Single-point | init_jump | Coordinate-wise line search with bisection |

---

### Benchmark Results

200 independent runs per configuration. Success threshold: f(x) <= 0.01. Median nfev among successful runs.

#### Easy Functions (64--1024D, maxiter=50k)

| Dim | ANS | ANS Sort | ANSR | DPNM | DE | SHADE | ZG |
| --- | --- | -------- | ---- | ---- | -- | ----- | -- |
| 64   | 1856  | 1472  | 1856  | 3072  | 2528  | 1856  | **1380** |
| 128  | 2832  | 2272  | 2848  | 5632  | 3968  | **2176**  | 2669 |
| 256  | 4352  | 3456  | 4352  | 10112 | 6272  | **3264**  | 5340 |
| 512  | 6976  | **5312**  | 6960  | 16864 | 10048 | 5376  | 10689 |
| 1024 | 12288 | **9024**  | 12288 | 23776 | 16832 | 10240 (83.3%) | 21350 |

All algorithms achieve 100% success rate except SHADE at 1024D (fails on shifted_sphere, 0% success on that function). SHADE is fastest at 128--256D but its failure at 1024D limits reliability. ANS Sort is 20--27% faster than ANS/ANSR at every dimension. ANS and ANSR are nearly identical on easy functions, confirming the restart mechanism adds no overhead.

#### Medium Terrain (64--256D, maxiter=500k)

| Dim | Func | ANS | ANS Sort | ANSR | DPNM | DE | SHADE |
| --- | ---- | --- | -------- | ---- | ---- | -- | ----- |
| 64 | hilly  | 70.7k (97.5%) | 402k (84.5%) | 70.7k (97.0%) | 61.8k | **42.4k (72.5%)** | 286k (96.5%) |
| 64 | forest | 47.3k | 335k | 47.3k | 47.1k | **29.5k (96.0%)** | 86.0k (92.0%) |
| 128 | hilly  | 159k (96.0%) | --- | 111k (83.5%) | **101k (97.5%)** | 137k (99.0%) | --- |
| 128 | forest | 101k | --- | 73.1k | **69.6k** | 85.6k | 196k |
| 256 | hilly  | 274k (94.5%) | --- | 334k (98.0%) | --- | **202k (95.0%)** | --- |
| 256 | forest | 175k | --- | 209k | --- | **128k** | 450k (94.0%) |

DPNM fails at 256D on both terrain functions (0% success). At lower dimensions DPNM is competitive, but its adaptive sigma schedule reduces perturbation too aggressively at higher dimensions. ANS Sort fails at 128D and above; at 64D it converges but is much slower (335k vs 47k on forest). DE is fastest on forest at 64D and on both functions at 256D. SHADE fails on hilly at 128D and 256D.

#### Medium Periodic: Shubert (16--64D, maxiter=500k)

| Dim | ANS | ANS Sort | ANSR | DPNM | DE | SHADE |
| --- | --- | -------- | ---- | ---- | -- | ----- |
| 16 | 13.8k (98.0%) | **6.6k (88.0%)** | 9.1k | 120k | 16.6k | 41.1k |
| 32 | 44.7k (87.0%) | 32.7k (91.0%) | **28.4k** | 402k | 37.6k | 103k |
| 64 | 187k (77.5%) | --- | **60.3k** | --- | 81.8k | 245k |

ANSR maintains 100% success rate at all dimensions and is the fastest algorithm, requiring only 60k evaluations at 64D. ANS degrades as dimensionality increases: 98% -> 87% -> 77.5% success rate. ANS Sort is fastest at 16D but has lower success rate (88%); at 64D it fails entirely, similar to DPNM.

#### Hard Discrete: Megacity (16--64D, maxiter=500k)

| Dim | ANS | ANS Sort | ANSR | DPNM | DE | SHADE | ZG |
| --- | --- | -------- | ---- | ---- | -- | ----- | -- |
| 16 | 126k (73.0%) | --- | --- | --- | 36.8k (80.0%) | **36.3k (75.0%)** | --- |
| 32 | --- | --- | --- | --- | **91.8k (80.0%)** | 114k (73.5%) | --- |
| 64 | --- | --- | --- | --- | **217k (90.5%)** | 364k (7.0%) | --- |

DE is the only reliable solver with consistently high success rates. ANS Sort, ANSR, DPNM, and ZG achieve 0% at all dimensions. ANS achieves 73% at 16D only. SHADE is competitive at 16D but degrades to 7% at 64D.

---

### Tuned Parameters

Best parameters found by grid search (10 seeds per configuration). Popsize = 64 for all population-based algorithms.

**ANS** — 600 combinations per dimension.

| Group | Dim | sigma | p_self |
| ----- | --- | ----- | ------ |
| Easy | 64--256 | 0.12 | 0.00 |
| Easy | 512 | 0.12 | 0.04 |
| Easy | 1024 | 0.16 | 0.16 |
| Terrain | 64 | 0.32 | 0.92 |
| Terrain | 128 | 0.36 | 0.92 |
| Terrain | 256 | 0.32 | 0.96 |
| Periodic | 16 | 0.04 | 0.24 |
| Periodic | 32 | 0.04 | 0.40 |
| Periodic | 64 | 0.04 | 0.56 |
| Discrete | 16 | 0.20 | 0.80 |
| Discrete | 32--64 | 0.04 | 0.00 |

**ANS Sort** — 600 combinations per dimension.

| Group | Dim | sigma | p_self |
| ----- | --- | ----- | ------ |
| Easy | 64 | 0.12 | 0.00 |
| Easy | 128 | 0.12 | 0.04 |
| Easy | 256 | 0.12 | 0.16 |
| Easy | 512 | 0.12 | 0.08 |
| Easy | 1024 | 0.16 | 0.00 |
| Terrain | 64 | 0.64 | 0.08 |
| Terrain | 128--256 | --- | --- |
| Periodic | 16 | 0.04 | 0.16 |
| Periodic | 32 | 0.12 | 0.32 |
| Periodic | 64 | --- | --- |
| Discrete | 16--64 | --- | --- |

**ANSR** — 600 combinations per dimension (tau = 1e-8 fixed).

| Group | Dim | sigma | p_self |
| ----- | --- | ----- | ------ |
| Easy | 64--256 | 0.12 | 0.00 |
| Easy | 512 | 0.12 | 0.04 |
| Easy | 1024 | 0.16 | 0.16 |
| Terrain | 64 | 0.32 | 0.92 |
| Terrain | 128 | 0.28 | 0.92 |
| Terrain | 256 | 0.36 | 0.96 |
| Periodic | 16 | 0.04 | 0.00 |
| Periodic | 32 | 0.04 | 0.04 |
| Periodic | 64 | 0.04 | 0.00 |
| Discrete | 16 | 0.04 | 0.00 |
| Discrete | 32--64 | 0.04 | 0.00 |

**ANSR DPNM** — 12,000 combinations per dimension (tau = 1e-8, decay_power = 2.0 fixed).

| Group | Dim | sigma | p_self | m_n |
| ----- | --- | ----- | ------ | --- |
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

**DE** — 600 combinations per dimension.

| Group | Dim | F | CR |
| ----- | --- | - | -- |
| Easy | 64--128 | 0.12 | 0.60 |
| Easy | 256 | 0.12 | 0.52 |
| Easy | 512 | 0.12 | 0.44 |
| Easy | 1024 | 0.12 | 0.32 |
| Terrain | 64 | 0.20 | 0.12 |
| Terrain | 128 | 0.32 | 0.08 |
| Terrain | 256 | 0.24 | 0.04 |
| Periodic | 16--64 | 0.04 | 0.00 |
| Discrete | 16 | 0.56 | 0.40 |
| Discrete | 32 | 0.52 | 0.40 |
| Discrete | 64 | 0.64 | 0.16 |

**SHADE** — 600 combinations per dimension.

| Group | Dim | H | p_best |
| ----- | --- | - | ------ |
| Easy | 64 | 1 | 0.28 |
| Easy | 128 | 1 | 0.72 |
| Easy | 256 | 9 | 0.52 |
| Easy | 512 | 24 | 0.76 |
| Easy | 1024 | 1 | 0.04 |
| Terrain | 64 | 24 | 0.08 |
| Terrain | 128 | 1 | 0.04 |
| Terrain | 256 | 1 | 0.04 |
| Periodic | 16 | 2 | 0.12 |
| Periodic | 32 | 12 | 0.12 |
| Periodic | 64 | 14 | 0.08 |
| Discrete | 16 | 16 | 0.20 |
| Discrete | 32 | 22 | 0.32 |
| Discrete | 64 | 1 | 0.04 |

**Zero-Gradient** — single parameter.

| Group | Dim | init_jump |
| ----- | --- | --------- |
| Easy | 64 | 0.20 |
| Easy | 128--1024 | 0.25 |
| Others | all | --- (fails) |

---

### Tune params grid

Parameter search space used by `cargo run --bin tune -r`. Popsize fixed at 64.

**ANS / ANS Sort** — 24 x 25 = **600 combinations**

| Parameter                | Values                                       |
| ------------------------ | -------------------------------------------- |
| `popsize`                | `64`                                         |
| `sigma`                  | `frange(0.04, 0.04, 0.96)` (24 values)      |
| `self_instead_neighbour` | `frange(0.0, 0.04, 0.96)` (25 values)       |

**ANSR** — 24 x 25 = **600 combinations** (tau fixed at 1e-8)

| Parameter                | Values                                       |
| ------------------------ | -------------------------------------------- |
| `popsize`                | `64`                                         |
| `restart_tolerance`      | `1e-8`                                       |
| `sigma`                  | `frange(0.04, 0.04, 0.96)` (24 values)      |
| `self_instead_neighbour` | `frange(0.0, 0.04, 0.96)` (25 values)       |

**ANSR DPNM** — 5 x 6 x 4 x 5 = **600 combinations** (tau fixed at 1e-8)

| Parameter                | Values                                       |
| ------------------------ | -------------------------------------------- |
| `popsize`                | `64`                                         |
| `restart_tolerance`      | `1e-8`                                       |
| `sigma`                  | `frange(0.2, 0.2, 1.0)` (5 values)          |
| `self_instead_neighbour` | `frange(0.0, 0.2, 1.0)` (6 values)          |
| `restart_decay_power`    | `2.0, 4.0, 6.0, 8.0`                        |
| `neighbour_multiplier`   | `0.5, 0.75, 1.0, 1.125, 1.5`                |

**DE** — 24 x 25 = **600 combinations**

| Parameter                | Values                                       |
| ------------------------ | -------------------------------------------- |
| `popsize`                | `64`                                         |
| `f`                      | `frange(0.04, 0.04, 0.96)` (24 values)      |
| `cr`                     | `frange(0.0, 0.04, 0.96)` (25 values)       |

**SHADE** — 24 x 25 = **600 combinations**

| Parameter                | Values                                       |
| ------------------------ | -------------------------------------------- |
| `popsize`                | `64`                                         |
| `h`                      | `frange(1.0, 1.0, 24.0)` (24 values)        |
| `p_best_rate`            | `frange(0.04, 0.04, 1.0)` (25 values)       |

**Zero-Gradient** — **15 combinations**

| Parameter                | Values                                       |
| ------------------------ | -------------------------------------------- |
| `init_jump`              | `log10_range(-8, -1)` + `frange(0.15, 0.05, 1.0)` (15 values) |

---

### References

- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)
- [ieee.org: Next Generation Metaheuristic: Jaguar Algorithm](https://ieeexplore.ieee.org/document/8267218)
