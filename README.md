# Optimizers

Run the benchmark
```bash
cargo run --bin benchmark -r
```

Generate the plot
```bash
cargo run --bin plot -r
```

The table contains algorithms sorted by the average number of calls to a function of dimension 8 to achieve 99% infinium
| №   | algorithm | params                                                       | mean     | shifted sphere | shifted weierstrass | hilly    | forest | megacity |
| --- | --------- | ------------------------------------------------------------ | -------- | -------------- | ------------------- | -------- | ------ | -------- |
| 1   | ansr      | popsize:16, tol:0.001, sigma:0.1, self_instead_neighbour:0.3 | 18888.27 | 271.68         | 16687.44            | 14200.56 | 4606.8 | 58674.88 |

References:
- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)