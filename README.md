## Optimizers

### CLI tools

| Binary      | Purpose                                                                   | Command                        |
| ----------- | ------------------------------------------------------------------------- | ------------------------------ |
| `benchmark` | Runs all performance tests                                                | `cargo run --bin benchmark -r` |
| `plot`      | Runs a single benchmark and draws the result for a selected function only | `cargo run --bin plot -r`      |
| `tune`      | Searches for the optimal set of algorithm parameters                      | `cargo run --bin tune -r`      |

---

### Results

The table below lists the algorithms sorted by the *average* number of function evaluations on 16‑dimensional test problems.  
Only runs that achieved at least **99 % of the infimum** are shown.

| №   | algorithm     | params                                                                         | mean_16D | shifted_sphere_16D | shifted_weierstrass_16D | hilly_16D | forest_16D | megacity_16D |
| --- | ------------- | ------------------------------------------------------------------------------ | -------- | ------------------ | ----------------------- | --------- | ---------- | ------------ |
| 1   | ansr          | popsize: 4, restart_tolerance: 0.01, sigma: 0.05, self_instead_neighbour: 0.85 | 51902.48 | 547.98             | 24493.82                | 48610.7   | 27254.4    | 158605.5     |
| inf | zero_gradient | init_jump: 0.1                                                                 | inf      | 384.195            | inf                     | inf       | inf        | inf          |

> **NOTE** – The “inf” rows indicate that the algorithm never reached the 99 % threshold for any of the tested functions.

---

### References

- [arxiv.org: Across neighbourhood search for numerical optimization](https://arxiv.org/abs/1401.3376)
- [github.com: Population-optimization-algorithms-MQL5](https://github.com/JQSakaJoo/Population-optimization-algorithms-MQL5)
- [ieee.org: Next Generation Metaheuristic: Jaguar Algorithm](https://ieeexplore.ieee.org/document/8267218)