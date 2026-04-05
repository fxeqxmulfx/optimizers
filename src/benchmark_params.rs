use crate::algorithms::{ans::ANS, ans_sort::AnsSorted, ansr::ANSR, ansr_dpnm::AnsrDpnm, de::DE, shade::SHADE, zero_gradient::ZeroGradient};

pub struct BenchmarkParams {
    pub ans: ANS,
    pub ans_sort: AnsSorted,
    pub ansr: ANSR,
    pub ansr_dpnm: AnsrDpnm,
    pub de: DE,
    pub shade: SHADE,
    pub zero_gradient: ZeroGradient,
}

/// Default params — used when no tuned params exist for a test set + dimension.
fn default_params() -> BenchmarkParams {
    BenchmarkParams {
        ans: ANS { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.0 },
        ans_sort: AnsSorted { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.0 },
        ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.12, self_instead_neighbour: 0.0 },
        ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.6, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
        de: DE { popsize: 64, f: 0.12, cr: 0.6 },
        shade: SHADE { popsize: 64, h: 1, p_best_rate: 0.28 },
        zero_gradient: ZeroGradient { init_jump: 0.2 },
    }
}

fn easy_params(dim: usize) -> BenchmarkParams {
    match dim {
        64 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.0 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.0 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.12, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.4, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.12, cr: 0.6 },
            shade: SHADE { popsize: 64, h: 1, p_best_rate: 0.28 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        128 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.0 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.04 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.12, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.6, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.12, cr: 0.6 },
            shade: SHADE { popsize: 64, h: 1, p_best_rate: 0.72 },
            zero_gradient: ZeroGradient { init_jump: 0.25 },
        },
        256 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.0 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.16 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.12, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.6, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.12, cr: 0.52 },
            shade: SHADE { popsize: 64, h: 9, p_best_rate: 0.52 },
            zero_gradient: ZeroGradient { init_jump: 0.25 },
        },
        512 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.04 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.08 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.12, self_instead_neighbour: 0.04 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.8, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.12, cr: 0.44 },
            shade: SHADE { popsize: 64, h: 24, p_best_rate: 0.76 },
            zero_gradient: ZeroGradient { init_jump: 0.25 },
        },
        1024 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.16, self_instead_neighbour: 0.16 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.16, self_instead_neighbour: 0.0 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.16, self_instead_neighbour: 0.16 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.6, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.12, cr: 0.32 },
            // SHADE inf at 1024D — use best available params
            shade: SHADE { popsize: 64, h: 24, p_best_rate: 0.76 },
            zero_gradient: ZeroGradient { init_jump: 0.25 },
        },
        _ => default_params(),
    }
}

fn medium_terrain_params(dim: usize) -> BenchmarkParams {
    match dim {
        64 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.32, self_instead_neighbour: 0.92 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.64, self_instead_neighbour: 0.08 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.32, self_instead_neighbour: 0.92 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.8, restart_decay_power: 2.0, neighbour_multiplier: 0.75 },
            de: DE { popsize: 64, f: 0.2, cr: 0.12 },
            shade: SHADE { popsize: 64, h: 24, p_best_rate: 0.08 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        128 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.36, self_instead_neighbour: 0.92 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.64, self_instead_neighbour: 0.08 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.28, self_instead_neighbour: 0.92 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.8, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.32, cr: 0.08 },
            shade: SHADE { popsize: 64, h: 1, p_best_rate: 0.04 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        256 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.32, self_instead_neighbour: 0.96 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.64, self_instead_neighbour: 0.08 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.36, self_instead_neighbour: 0.96 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.0, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.24, cr: 0.04 },
            shade: SHADE { popsize: 64, h: 1, p_best_rate: 0.04 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        _ => default_params(),
    }
}

fn hard_discrete_params(dim: usize) -> BenchmarkParams {
    match dim {
        16 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.2, self_instead_neighbour: 0.8 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.2, self_instead_neighbour: 0.8 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.0, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.56, cr: 0.4 },
            shade: SHADE { popsize: 64, h: 16, p_best_rate: 0.2 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        32 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.0 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.0, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.52, cr: 0.4 },
            shade: SHADE { popsize: 64, h: 22, p_best_rate: 0.32 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        64 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.0 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.0, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.64, cr: 0.16 },
            shade: SHADE { popsize: 64, h: 1, p_best_rate: 0.04 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        _ => default_params(),
    }
}

fn medium_periodic_params(dim: usize) -> BenchmarkParams {
    match dim {
        16 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.24 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.16 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.2, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.04, cr: 0.0 },
            shade: SHADE { popsize: 64, h: 2, p_best_rate: 0.12 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        32 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.4 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.12, self_instead_neighbour: 0.32 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.04, self_instead_neighbour: 0.04 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.6, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.04, cr: 0.0 },
            shade: SHADE { popsize: 64, h: 12, p_best_rate: 0.12 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        64 => BenchmarkParams {
            ans: ANS { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.56 },
            ans_sort: AnsSorted { popsize: 64, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr: ANSR { popsize: 64, restart_tolerance: 1e-8, sigma: 0.04, self_instead_neighbour: 0.0 },
            ansr_dpnm: AnsrDpnm { popsize: 64, restart_tolerance: 1e-8, sigma: 0.2, self_instead_neighbour: 0.0, restart_decay_power: 2.0, neighbour_multiplier: 0.5 },
            de: DE { popsize: 64, f: 0.04, cr: 0.0 },
            shade: SHADE { popsize: 64, h: 14, p_best_rate: 0.08 },
            zero_gradient: ZeroGradient { init_jump: 0.2 },
        },
        _ => default_params(),
    }
}

/// Returns tuned params for a given test set and dimension.
/// Falls back to default_params() if no specific tune exists.
pub fn get_params(test_set: &str, dim: usize) -> BenchmarkParams {
    match test_set {
        "easy" => easy_params(dim),
        "medium_terrain" => medium_terrain_params(dim),
        "hard_discrete" => hard_discrete_params(dim),
        "medium_periodic" => medium_periodic_params(dim),
        "main" => default_params(),
        "mini" => default_params(),
        "lmmaes" => default_params(),
        "hard" => default_params(),
        "medium_weierstrass" => default_params(),
        _ => default_params(),
    }
}
