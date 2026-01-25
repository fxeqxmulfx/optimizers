# Optimizers

Run the benchmark
```bash
cargo run --bin benchmark -r
```

Generate the plot
```bash
cargo run --bin plot -r
```

The table presents algorithms sorted by the average number of function evaluations on 16-dimensional functions, provided that 99% of the infimum is achieved for each
| №   | algorithm     | params                                                                    | mean_16D  | shifted_sphere_16D | shifted_weierstrass_16D | hilly_16D | forest_16D | megacity_16D |
| --- | ------------- | ------------------------------------------------------------------------- | --------- | ------------------ | ----------------------- | --------- | ---------- | ------------ |
| 1   | ansr          | popsize:8, restart_tolerance:0.01, sigma:0.05, self_instead_neighbour:0.9 | 51300.832 | 646.68             | 23145.36                | 42157.08  | 11416.08   | 179138.95    |
| inf | zero_gradient | init_jump:0.25                                                            | inf       | 332.77             | inf                     | inf       | inf        | inf          |

References:
- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)
- [ieee.org: Next Generation Metaheuristic: Jaguar Algorithm](https://ieeexplore.ieee.org/document/8267218)