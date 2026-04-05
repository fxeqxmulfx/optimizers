# Tune Progress

## Tune main (16D, MAIN_TEST_FUNCTIONS, 10 seeds, 1M maxiter)

| Algorithm | Best mean | Best params |
| --------- | --------- | ----------- |
| SHADE     | 38830.40  | popsize: 56, h: 20, p_best_rate: 0.3 |
| ANSR      | 44003.84  | popsize: 16, restart_tolerance: 0.01, self_instead_neighbour: 0.9, sigma: 0.01 |
| DE        | 110531.84 | cr: 0.1, f: 0.6, popsize: 64 |
| ANSR DPNM   | 153115.03 | popsize: 8, restart_tolerance: 0.01, self_instead_neighbour: 0.85, sigma: 0.3, restart_decay_power: 2.0, neighbour_multiplier: 0.5 |
| ANS       | inf       | — |
| ZG        | inf       | — |

## Tune mini (64D, MINI_TEST_FUNCTIONS, 10 seeds, 1M maxiter)

| Algorithm | Best mean | Best params |
| --------- | --------- | ----------- |
| ANSR DPNM   | 20918.40  | popsize: 32, sigma: 0.1, self_instead_neighbour: 0.9, restart_tolerance: 1e-4, restart_decay_power: 4.0, neighbour_multiplier: 1.0 |
| ANSR      | 32629.33  | popsize: 64, restart_tolerance: 1e-6, self_instead_neighbour: 0.85, sigma: 0.3 |
| ANS       | 35212.80  | popsize: 192, sigma: 0.1, self_instead_neighbour: 0.7 |
| DE        | 41361.07  | cr: 0.1, f: 0.6, popsize: 32 |
| SHADE     | 224855.45 | h: 20, p_best_rate: 0.05, popsize: 64 |
| ZG        | inf       | — |

## Tune medium (64D, MEDIUM_TEST_FUNCTIONS, 10 seeds, 300k maxiter) — INCOMPLETE

| Algorithm | Best mean | Best params |
| --------- | --------- | ----------- |
| ANS       | inf       | — |
| ANSR      | inf (500/1536) | stopped early |
| Others    | not started | — |

## Tune lmmaes — not started

## Settings changed
- main: 500k maxiter (tune and benchmark)
- mini, medium, lmmaes: 300k maxiter (tune and benchmark)
- benchmark: 200 seeds

## Current defaults (from main tune)
- ANS: popsize: 64, sigma: 0.15, self_instead_neighbour: 0.7
- ANSR: popsize: 16, restart_tolerance: 0.01, sigma: 0.01, self_instead_neighbour: 0.9
- ANSR DPNM: popsize: 8, restart_tolerance: 0.01, sigma: 0.3, self_instead_neighbour: 0.85, restart_decay_power: 2.0, neighbour_multiplier: 0.5
- DE: popsize: 64, f: 0.6, cr: 0.1
- SHADE: popsize: 56, h: 20, p_best_rate: 0.3
- ZG: init_jump: 0.1
