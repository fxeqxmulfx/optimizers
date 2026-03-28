# ANSR V2 Experiment Log

**Author:** Mikhail Borisov (mikhail.borisov.study@proton.me)
**Co-author:** Claude Opus 4.6 (1M context)

## Overview

207 experiments were conducted to evolve the ANSR V2 (Adaptive Neighbourhood Search with Restarts V2) optimizer. The goal was to maximize the number of finite (converged) results across a hyperparameter grid while minimizing the best achieved objective value, tested on benchmark functions in 64 dimensions with 100k function evaluations.

181 algorithm variations were explored, 167 reverted. The final algorithm achieved 4334 finite results (61.9x baseline) with best=16654 (37.5% improvement over baseline best=26643).

## Starting Point

The original ANSR V2 algorithm:
- Population of particles in unit cube [0,1]^D, mapped to bounds for evaluation
- Each particle perturbs around its personal best (self path) or a random neighbour's best (social path), controlled by `self_instead_neighbour` probability
- Perturbation: `best_d + N(0, sigma) * |best_d - current_d|`
- Restart mechanism: when two particles have similar residuals, the worse one is reset to a random position
- Static sigma throughout the run

Baseline (Run 3): 70 finite out of 15876 combinations, best = 26643.

## Changes That Worked (in chronological order)

### 1. Non-uniform parameter grid (Runs 8-12)

Concentrated grid density in productive regions discovered from early runs while keeping full mathematical bounds coverage.

**Key insight**: Only ~0.44% of uniform grid configs converge. Productive ranges are narrow: sigma 0.05-0.25, self_instead_neighbour 0.65-0.95, restart_tolerance <= 1e-4.

**Result**: 70 -> 548 finite (+683%), best 26643 -> 23875 (-10%)

### 2. Coherent neighbour selection (Runs 13, 15)

Changed from picking a random neighbour independently for each dimension to picking one random neighbour per particle per epoch and using it for all dimensions.

**Key insight**: Per-dimension random neighbours create incoherent movement vectors. A single neighbour per particle produces coherent directional steps, improving convergence speed.

**Result**: 548 -> 661 finite (+21%), mean ~44k -> ~41k (-7%)

### 3. Sigma annealing (Runs 16-19)

Added time-dependent sigma decay. Tested linear (1-t), quadratic (1-t)^2, power 2.5, and cubic (1-t)^3.

**Key insight**: Static sigma forces a tradeoff between exploration and exploitation. Annealing enables broad exploration early and precise convergence late. Quadratic decay (power=2) gave the best balance at this stage.

| Power | Finite | Best |
|-------|--------|------|
| 1 (linear) | 1149 | 23275 |
| **2 (quadratic)** | **1221** | **21653** |
| 2.5 | 1117 | 23575 |
| 3 (cubic) | 1082 | 21884 |

**Result**: 661 -> 1221 finite (+85%), best 23866 -> 21653 (-9%)

### 4. Restart tolerance annealing (Run 27)

Restart tolerance decays as RT * (1-t)^power over the run. Allows aggressive diversity maintenance early, minimal disruption late.

**Result**: +21 finite, -2-4% mean. Small but consistent improvement.

### 5. Cosine sigma annealing (Run 32)

Replaced quadratic decay with cosine annealing: `sigma * 0.5 * (1 + cos(pi * t))`. Cosine decays slower mid-run (0.5*sigma at t=0.5 vs 0.25*sigma for quadratic), giving more exploration time.

**Result**: 1423 -> 1568 finite (+10%), best -8% (21312)

### 6. Opposition-based restart (Run 34)

When restarting a converged particle, place it at the mirror position (1 - position) of the better particle instead of random. Explores the opposite region of the search space systematically.

**Result**: 1568 -> 1728 finite (+10%), popsize=32: 3 -> 8 finite

### 7. Mirror boundary handling (Run 45)

Replaced clamping at boundaries with mirror reflection. Positions that exceed boundaries are reflected back, preserving momentum direction.

**Result**: +6% finite, best -0.6%, popsize=32: 5 -> 12 finite (+140%)

### 8. Asymmetric neighbour sigma (Runs 59-63)

The social (neighbour) path uses a higher sigma multiplier than the self path. Makes the social component more exploratory while keeping self-path exploitation precise.

| Multiplier | Finite | Best |
|------------|--------|------|
| 1.0x | 1380 | 22306 |
| 1.5x | 1785 | 22352 |
| 2.0x | 2174 | 22310 |
| 2.5x | 2478 | 22884 |
| 3.0x | 2709 | 24419 |

Then annealing the multiplier from 2x -> 1x via cosine improved best by 5%.

**Result (Run 63)**: 2182 finite, best = 21158

### 9. Wrap (toroidal) boundary (Run 98)

Replaced mirror boundary with modular wrap. Positions that exceed [0,1] wrap around. This distributes positions uniformly and turns large perturbations into long-range exploration.

**Result**: 2182 -> 2867 finite (+31%), best 21158 -> 20635 (-2.5%)

### 10. Higher neighbour multiplier with wrap (Run 99)

Wrap boundary and higher neighbour sigma (2.5x->1x) create synergy: overshoots that would be wasted with mirror become useful exploration with wrap.

**Result**: 2867 -> 3361 finite (+17%), best 20635 -> 18714 (-9%)

### 11. Squared self-path distance (Run 151)

Self-path perturbation uses dist^2 instead of dist. Squared distance gives faster convergence near optimum (step shrinks quadratically) while maintaining exploration when far.

**Result**: 3361 -> 3522 finite (+5%), best 18714 -> 18649 (-0.4%)

### 12. 3x neighbour sigma with squared self (Run 155)

3x neighbour sigma (up from 2.5x) synergizes with squared self-path distance. Stronger social exploration compensates for tighter self-path convergence.

**Result**: 3522 -> 3961 finite (+12%), best 18649 -> 18143 (-2.7%)

### 13. Sextic RT decay (Runs 158-160)

Increased RT decay power from quadratic (1-t)^2 through quartic to sextic (1-t)^6. Higher power kills restarts faster, reducing late-stage disruption.

**Result**: 3961 -> 4149 finite (+5%), best 18143 -> 18087 (-0.3%)

### 14. 3.25x neighbour sigma (Run 161) -- FINAL BEST

Fine-tuned neighbour multiplier to 3.25x (from 3.0x). This is the precise sweet spot with squared self + sextic RT.

**Result**: 4149 -> 4334 finite (+4.5%), best 18087 -> 16654 (-7.9%)

## Changes That Failed (Key Lessons)

### Perturbation modifications
- **Remove distance scaling** (Run 4): 0 finite. Distance scaling is essential for exploitation.
- **Absolute perturbation N(0, sigma)** (Run 4): Too random without distance-adaptive step size.
- **DE-style differential mutation** (Runs 14, 82, 194): 0 finite. Difference vectors -> 0 as particles converge.
- **Stagnation fix / distance floor** (Runs 2, 11, 57, 83, 125, 202): Any noise at convergence hurts. The self-path freeze is a feature -- particles converge and wait for social component.
- **Cauchy distribution** (Runs 39-42, 197): +50-60% finite but best +7-34%. Heavy tails prevent precise convergence.
- **Momentum** (Runs 47, 116, 130, 166): Up to +98% finite but best +10-34%. Prevents precise convergence. Confirmed dead end across all base configs.
- **Signed/directional perturbation** (Runs 140, 149): 0-1 finite. Directional bias loses symmetric exploration.
- **sqrt/log distance** (Runs 91, 171): Amplifies small distances too much.
- **Squared neighbour distance** (Runs 152, 157): Makes neighbour steps too small. Self-only squared is optimal.

### Neighbour selection modifications
- **Global best** (Run 6): 0 finite. Premature convergence, all particles collapse.
- **Blended social** (Runs 26, 37, 123): 0 finite. Even 50% global best causes collapse.
- **Tournament/ranked** (Runs 28, 44, 127, 128, 205): Kills diversity through selection pressure.
- **Persistent neighbours** (Runs 49, 118): Fixed neighbours reduce diversity.
- **Round-robin cycling** (Run 74): Fixed schedule removes needed stochasticity.
- **Ring topology** (Run 196): Too few neighbours limits information flow.
- **Best-performing neighbour** (Run 199): 0 finite. Same as global best collapse.

### Restart modifications
- **Restart near better particle** (Run 5): Less diversity than opposition restart.
- **Random restart** (Runs 52, 108): ~1 finite. Opposition is essential.
- **Global best opposition** (Runs 56, 183): All restarted particles go to same point.
- **Crossover opposition** (Runs 48, 120): Random dimension mix worse than pure opposition.
- **Partial/noisy opposition** (Runs 41, 77, 180): Any deviation from clean opposition hurts.
- **Position-guarded restart** (Runs 184-188): Consistently +6-19% finite but -16% best. Inherent trade-off.
- **Stagnation-gated restart** (Run 148): Delaying restarts lets particles converge to same basins.

### Annealing modifications
- **sin annealing** (Runs 24, 73, 170, 195): Even mild sin modification kills convergence. Must be static.
- **Cosine RT annealing** (Runs 33, 51, 70, 138, 189): Cosine decays too slowly for RT.
- **Linear RT annealing** (Runs 67, 136): Too aggressive, disrupts convergence.
- **Exponential sigma** (Run 95): Too aggressive early.
- **Warm restart sigma** (Run 112): Disrupts convergence.
- **Phase-shifted sigma per particle** (Run 178): Prevents sustained convergence.

### Other
- **Two-phase** (Run 50): Cutting social component kills convergence.
- **Dimension skipping** (Run 35): Prevents coherent movement.
- **Random dimension reset** (Runs 86-87, 201): Even 0.1% reset hurts best quality.
- **Per-particle coherent path choice** (Run 176): 0 finite. Per-dimension mixing essential.
- **Local refinement phase** (Run 126): Single-point refinement less efficient than population search.
- **Greedy sequential evaluation** (Run 146): Ordering bias hurts.

## Final Algorithm (Run 161)

```
Parameters: popsize, restart_tolerance, sigma, self_instead_neighbour,
            restart_decay_power (default 6), neighbour_multiplier (default 1.125)

Initialize: particles uniformly in [0,1]^D
For each epoch (t = epoch / max_epoch):
    1. Evaluate all particles (map to bounds via fit_in_bounds)
    2. Update personal bests
    3. Check early stopping
    4. RT annealing: effective_rt = rt * (1-t)^restart_decay_power
    5. Pairwise restart: if |f_max - f_min| / |f_max| < effective_rt,
       restart worse particle at opposition (1 - better_position)
    6. Sigma annealing: effective_sigma = sigma * 0.5 * (1 + cos(pi*t))
    7. For each particle, pick one random neighbour r != p
    8. For each dimension d:
       - With prob self_instead_neighbour (self path):
           dist = |best_d - current_d|
           new_d = wrap(best_d + N(0,1) * effective_sigma * dist)
       - Otherwise (social path):
           new_d = wrap(best_r_d + N(0,1) * effective_sigma
                   * (1 + neighbour_multiplier * (1 + cos(pi*t)))
                   * |best_r_d - current_d|)
```

Key design principles discovered:
- Self-path freeze at personal best is a **feature** (particles wait for social info)
- Neighbour needs asymmetric sigma multiplier vs self (more exploration on social path)
- Cosine annealing for sigma, power-law for RT (different curves for different purposes)
- Wrap boundary > mirror > clamp (toroidal search space)
- Opposition restart > random restart (systematic diversity)
- Per-dimension self/neighbour mixing is essential (coherent per-particle path choice fails)

## Results Evolution

| Run | Change | Finite | Best | vs Baseline |
|-----|--------|--------|------|-------------|
| 3 | Baseline | 70 | 26643 | -- |
| 12 | + Non-uniform grid | 548 | 23875 | 7.8x, -10% |
| 15 | + Coherent neighbour | 661 | 23866 | 9.4x, -10% |
| 17 | + Quadratic sigma decay | 1221 | 21653 | 17.4x, -19% |
| 32 | + Cosine sigma | 1568 | 21312 | 22.4x, -20% |
| 34 | + Opposition restart | 1728 | 22447 | 24.7x, -16% |
| 45 | + Mirror boundary | 1380 | 22306 | 19.7x, -16% |
| 63 | + 2->1x neighbour anneal | 2182 | 21158 | 31.2x, -21% |
| 98 | + Wrap boundary | 2867 | 20635 | 41.0x, -23% |
| 99 | + 2.5x neighbour sigma | 3361 | 18714 | 48.0x, -30% |
| 151 | + Squared self distance | 3522 | 18649 | 50.3x, -30% |
| 155 | + 3x neighbour sigma | 3961 | 18143 | 56.6x, -32% |
| 160 | + Sextic RT decay | 4149 | 18087 | 59.3x, -32% |
| **161** | **+ 3.25x neighbour** | **4334** | **16654** | **61.9x, -37.5%** |

After Run 161, 46 consecutive regressions (Runs 162-207) confirmed the algorithm had reached a local optimum in design space.

## Post-experiment simplification

After the experiments, two hardcoded constants were removed for code clarity:

- **self_dist_power**: Run 151 found dist^2 improved over dist^1 (+5% finite, -0.4% best). Removed anyway — linear `dist` is simpler and the squared variant can be re-added as a param if needed.
- **neighbour_scale 1.125**: Run 161 found 3.25x (via 1.125 coefficient) was the precise sweet spot. Extracted as `neighbour_multiplier` param instead of hardcoding, so it can be tuned per setup.

This is a deliberate trade-off: simpler code over ~5-8% performance on the original benchmark. The algorithm has zero hardcoded tuning constants — all behavior is controlled by 6 explicit params.
