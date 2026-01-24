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
| №   | algorithm | params                                                      | mean     | shifted_sphere | shifted_weierstrass | hilly   | forest   | megacity |
| --- | --------- | ----------------------------------------------------------- | -------- | -------------- | ------------------- | ------- | -------- | -------- |
| 1   | ansr      | popsize:8, tol:0.01, sigma:0.05, self_instead_neighbour:0.8 | 75656.93 | 554.8          | 27693.92            | 51263.2 | 32939.04 | 265833.7 |

References:
- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)