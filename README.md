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
| №   | algorithm     | params                                                                     | mean_16D  | shifted_sphere_16D | shifted_weierstrass_16D | hilly_16D | forest_16D | megacity_16D |
| --- | ------------- | -------------------------------------------------------------------------- | --------- | ------------------ | ----------------------- | --------- | ---------- | ------------ |
| 1   | ansr          | popsize:4, restart_tolerance:0.01, sigma:0.05, self_instead_neighbour:0.85 | 51910.645 | 547.98             | 24534.64                | 48610.7   | 27254.4    | 158605.5     |
| inf | zero_gradient | init_jump:0.1                                                              | inf       | 384.19             | inf                     | inf       | inf        | inf          |

References:
- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)
- [ieee.org: Next Generation Metaheuristic: Jaguar Algorithm](https://ieeexplore.ieee.org/document/8267218)