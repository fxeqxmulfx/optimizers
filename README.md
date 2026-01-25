# Optimizers

Run the benchmark
```bash
cargo run --bin benchmark -r
```

Generate the plot
```bash
cargo run --bin plot -r
```

The table lists algorithms sorted by the average number of calls to 16-dimensional functions required to achieve the 99% infimum
| №   | algorithm | params                                                                    | mean      | shifted_sphere | shifted_weierstrass | hilly    | forest   | megacity  |
| --- | --------- | ------------------------------------------------------------------------- | --------- | -------------- | ------------------- | -------- | -------- | --------- |
| 1   | ansr      | popsize:8, restart_tolerance:0.01, sigma:0.05, self_instead_neighbour:0.9 | 51300.832 | 646.68         | 23145.36            | 42157.08 | 11416.08 | 179138.95 |

References:
- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)