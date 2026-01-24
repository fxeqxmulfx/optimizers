# Optimizers

Run the benchmark
```bash
cargo run --bin benchmark -r
```

Generate the plot
```bash
cargo run --bin plot -r
```

The table contains algorithms sorted by the average number of calls to a function of dimension 16 to achieve 99% infinium
| №   | algorithm | params                                                                    | mean     | shifted_sphere | shifted_weierstrass | hilly    | forest   | megacity  |
| --- | --------- | ------------------------------------------------------------------------- | -------- | -------------- | ------------------- | -------- | -------- | --------- |
| 1   | ansr      | popsize:8, restart_tolerance:0.01, sigma:0.05, self_instead_neighbour:0.8 | 69900.17 | 550.48         | 24687.44            | 47874.04 | 22762.96 | 253625.95 |

References:
- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)