use std::{collections::BTreeMap, f32::consts::PI};

use once_cell::sync::Lazy;
use simd_vector::Vec8;
use simd_vector::fast::FastMath;

use crate::utils::Vec8Ext;

#[derive(Clone)]
pub struct TestFunction {
    pub func: fn(Vec8, Vec8) -> Vec8,
    pub bounds: [[f32; 2]; 2],
}

fn scale(v: Vec8, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> Vec8 {
    let in_range = in_max - in_min;
    let out_range = out_max - out_min;
    (v - in_min) / in_range * out_range + out_min
}

pub const SHIFTED_SPHERE_BOUNDS: [[f32; 2]; 2] = [[-10.0, 10.0], [-10.0, 10.0]];

pub fn shifted_sphere(x: Vec8, y: Vec8) -> Vec8 {
    let x = x + PI;
    let y = y + PI;
    let result = x * x + y * y;
    scale(result, 0.0, 345.402914946, 0.0, 1.0)
}

const WEIERSTRASS_AK: [Vec8; 27] = [
    Vec8::splat(1.0),
    Vec8::splat(0.5),
    Vec8::splat(0.25),
    Vec8::splat(0.125),
    Vec8::splat(0.0625),
    Vec8::splat(0.03125),
    Vec8::splat(0.015625),
    Vec8::splat(0.0078125),
    Vec8::splat(0.00390625),
    Vec8::splat(0.001953125),
    Vec8::splat(0.0009765625),
    Vec8::splat(0.00048828125),
    Vec8::splat(0.00024414063),
    Vec8::splat(0.00012207031),
    Vec8::splat(6.1035156e-5),
    Vec8::splat(3.0517578e-5),
    Vec8::splat(1.5258789e-5),
    Vec8::splat(7.6293945e-6),
    Vec8::splat(3.8146973e-6),
    Vec8::splat(1.9073486e-6),
    Vec8::splat(9.536743e-7),
    Vec8::splat(4.7683716e-7),
    Vec8::splat(2.3841858e-7),
    Vec8::splat(1.1920929e-7),
    Vec8::splat(5.9604645e-8),
    Vec8::splat(2.9802322e-8),
    Vec8::splat(1.4901161e-8),
];

const WEIERSTRASS_BK_PI: [f32; 27] = [
    3.141592653589793,
    21.991148575128552,
    153.93804002589985,
    1077.5662801812991,
    7542.9639612690935,
    52800.74772888365,
    369605.2341021856,
    2587236.638715299,
    18110656.471007094,
    126774604.72182761,
    887422201.6368667,
    6211955034.4669485,
    43483685555.42791,
    304385802029.588,
    2130700708454.8955,
    14914904330865.738,
    104404328745263.84,
    730830301216846.9,
    5115812202765708.0,
    3.5810684791041424e+16,
    2.5067477782932672e+17,
    1.754723570468993e+18,
    1.2283064930451098e+19,
    8.598145451315769e+19,
    6.018701847336965e+20,
    4.213091575879214e+21,
    2.9491639460358173e+22,
];

pub fn weierstrass(x: Vec8) -> Vec8 {
    let mut total = Vec8::ZERO;
    for k in 0..=26 {
        total = WEIERSTRASS_AK[k].mul_add((WEIERSTRASS_BK_PI[k] * x).cos(), total);
    }
    total
}

pub const SHIFTED_WEIERSTRASS_BOUNDS: [[f32; 2]; 2] = [[-10.0, 10.0], [-10.0, 10.0]];

pub fn shifted_weierstrass(x: Vec8, y: Vec8) -> Vec8 {
    let x = x + PI;
    let y = y + PI;
    let result = (weierstrass(x) + weierstrass(y)) / 2.0;
    scale(result, -2.0, 2.0, 0.0, 1.0)
}

pub const HILLY_BOUNDS: [[f32; 2]; 2] = [[-3.0, 3.0], [-3.0, 3.0]];

pub fn hilly(x: Vec8, y: Vec8) -> Vec8 {
    let result = 20.0 + x.square() + y.square()
        - 10.0 * (2.0 * PI * x).cos()
        - 10.0 * (2.0 * PI * y).cos()
        - 30.0 * (-((x - 1.0).square() + y.square()) / 0.1).exp()
        + 200.0 * (-((x + PI * 0.47).square() + (y - PI * 0.2).square()) / 0.1).exp()
        + 100.0 * (-((x - 0.5).square() + (y + 0.5).square()) / 0.01).exp()
        - 60.0 * (-((x - 1.33).square() + (y - 2.0).square()) / 0.02).exp()
        - 40.0 * (-((x + 1.3).square() + (y + 0.2).square()) / 0.5).exp()
        + 60.0 * (-((x - 1.5).square() + (y + 1.5).square()) / 0.1).exp();
    let result = -result;
    scale(result, -229.91931214214105, 39.701816104859866, 0.0, 1.0)
}

pub const FOREST_BOUNDS: [[f32; 2]; 2] = [[-43.50, -39.0], [-47.35, -40.0]];

pub fn forest(x: Vec8, y: Vec8) -> Vec8 {
    let a = ((x - 1.13).abs() + (y - 2.0).abs()).sqrt().sin();
    let b = (x.sin().abs().sqrt() + ((y - 2.0).sin().abs().sqrt())).cos();
    let f = a
        + b
        + 1.01 * (-(((x + 42.0).square() + (y + 43.5).square()) / 0.9)).exp()
        + 1.0 * (-(((x + 40.2).square() + (y + 46.0).square()) / 0.3)).exp();
    let mut result =
        f.tesseract() - 0.3 * (-(((x + 42.3).square() + (y + 46.0).square()) / 0.02)).exp();
    result = -result;
    scale(result, -1.8779867959790217, 0.26489289358875895, 0.0, 1.0)
}

pub const MEGACITY_BOUNDS: [[f32; 2]; 2] = [[-10.0, -2.0], [-10.5, 10.0]];

pub fn megacity(x: Vec8, y: Vec8) -> Vec8 {
    let a = ((x - 1.13).abs() + (y - 2.0).abs()).sqrt().sin();
    let b = (x.sin().abs().sqrt() + (y - 2.0).sin().abs().sqrt()).cos();
    let f = a + b;
    let term1 = f.tesseract().floor();
    let exp_arg = -(((x + 9.5).square() + (y + 7.5).square()) / 0.4);
    let term2 = (2.0 * exp_arg.exp()).floor();
    let result = term1 - term2;
    let result = -result;
    scale(result, -12.0_f32, 2.0_f32, 0.0_f32, 1.0_f32)
}

pub const SPHERE_BOUNDS: [[f32; 2]; 2] = [[-5.0, 5.0], [-5.0, 5.0]];

pub fn sphere(x: Vec8, y: Vec8) -> Vec8 {
    let result = x.square() + y.square();
    scale(result, 0.0, 50.0, 0.0, 1.0)
}

pub const ELLIPSOID_BOUNDS: [[f32; 2]; 2] = [[-5.0, 5.0], [-5.0, 5.0]];

pub fn ellipsoid(x: Vec8, y: Vec8) -> Vec8 {
    let result = x.square() + 1_000_000.0 * y.square();
    scale(result, 0.0, 25_000_025.0, 0.0, 1.0)
}

pub const ROSENBROCK_BOUNDS: [[f32; 2]; 2] = [[-5.0, 5.0], [-5.0, 5.0]];

pub fn rosenbrock(x: Vec8, y: Vec8) -> Vec8 {
    let result = 100.0 * (x.square() - y).square() + (x - 1.0).square();
    scale(result, 0.0, 90_036.0, 0.0, 1.0)
}

pub const DISCUS_BOUNDS: [[f32; 2]; 2] = [[-5.0, 5.0], [-5.0, 5.0]];

pub fn discus(x: Vec8, y: Vec8) -> Vec8 {
    let result = 1_000_000.0 * x.square() + y.square();
    scale(result, 0.0, 25_000_025.0, 0.0, 1.0)
}

pub const DIFFERENT_POWERS_BOUNDS: [[f32; 2]; 2] = [[-5.0, 5.0], [-5.0, 5.0]];

pub fn different_powers(x: Vec8, y: Vec8) -> Vec8 {
    let y_sq = y.square();
    let result = x.square() + (y_sq * y_sq * y_sq);
    scale(result, 0.0, 15_650.0, 0.0, 1.0)
}

pub static MINI_TEST_FUNCTIONS: Lazy<BTreeMap<String, TestFunction>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert(
        "shifted_sphere".to_string(),
        TestFunction {
            func: shifted_sphere,
            bounds: SHIFTED_SPHERE_BOUNDS,
        },
    );
    m.insert(
        "hilly".to_string(),
        TestFunction {
            func: hilly,
            bounds: HILLY_BOUNDS,
        },
    );
    m.insert(
        "forest".to_string(),
        TestFunction {
            func: forest,
            bounds: FOREST_BOUNDS,
        },
    );
    m
});

pub static MAIN_TEST_FUNCTIONS: Lazy<BTreeMap<String, TestFunction>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert(
        "shifted_sphere".to_string(),
        TestFunction {
            func: shifted_sphere,
            bounds: SHIFTED_SPHERE_BOUNDS,
        },
    );
    m.insert(
        "shifted_weierstrass".to_string(),
        TestFunction {
            func: shifted_weierstrass,
            bounds: SHIFTED_WEIERSTRASS_BOUNDS,
        },
    );
    m.insert(
        "hilly".to_string(),
        TestFunction {
            func: hilly,
            bounds: HILLY_BOUNDS,
        },
    );
    m.insert(
        "forest".to_string(),
        TestFunction {
            func: forest,
            bounds: FOREST_BOUNDS,
        },
    );
    m.insert(
        "megacity".to_string(),
        TestFunction {
            func: megacity,
            bounds: MEGACITY_BOUNDS,
        },
    );
    m
});

pub static LMMAES_TEST_FUNCTIONS: Lazy<BTreeMap<String, TestFunction>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert(
        "sphere".to_string(),
        TestFunction {
            func: sphere,
            bounds: SPHERE_BOUNDS,
        },
    );
    m.insert(
        "ellipsoid".to_string(),
        TestFunction {
            func: ellipsoid,
            bounds: ELLIPSOID_BOUNDS,
        },
    );
    m.insert(
        "rosenbrock".to_string(),
        TestFunction {
            func: rosenbrock,
            bounds: ROSENBROCK_BOUNDS,
        },
    );
    m.insert(
        "discus".to_string(),
        TestFunction {
            func: discus,
            bounds: DISCUS_BOUNDS,
        },
    );
    m.insert(
        "different_powers".to_string(),
        TestFunction {
            func: different_powers,
            bounds: DIFFERENT_POWERS_BOUNDS,
        },
    );
    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_extrema() {
        let s_min = shifted_sphere(Vec8::splat(-PI), Vec8::splat(-PI))[0];
        assert!(s_min.abs() < 1e-3, "Shifted sphere min not 0");
        let s_max = shifted_sphere(Vec8::splat(10.0), Vec8::splat(10.0))[0];
        assert!((s_max - 1.0).abs() < 1e-3, "Shifted sphere max not 1");
        let w_min = shifted_weierstrass(Vec8::splat(1.0 - PI), Vec8::splat(1.0 - PI))[0];
        assert!(w_min.abs() < 1e-2, "Shifted weierstrass min not 0");
        let w_max = shifted_weierstrass(Vec8::splat(-PI), Vec8::splat(-PI))[0];
        assert!((w_max - 1.0).abs() < 1e-3, "Shifted weierstrass max not 1");
        let h_min = hilly(
            Vec8::splat(-1.4809053654574758),
            Vec8::splat(0.6254111843389699),
        )[0];
        assert!(h_min.abs() < 1e-3, "Hilly min not 0");
        let h_max = hilly(
            Vec8::splat(1.3200361419666748),
            Vec8::splat(1.9993728393766546),
        )[0];
        assert!((h_max - 1.0).abs() < 1e-3, "Hilly max not 1");
        let f_min = forest(
            Vec8::splat(-40.840704496667314),
            Vec8::splat(-41.982297150257104),
        )[0];
        assert!(f_min.abs() < 1e-3, "Forest min not 0");
        let f_max = forest(
            Vec8::splat(-42.2988573690385010),
            Vec8::splat(-45.9956119113080675),
        )[0];
        assert!((f_max - 1.0).abs() < 1e-3, "Forest max not 1");
        let m_min = megacity(
            Vec8::splat(-3.1357545740179393),
            Vec8::splat(2.006136371058429),
        )[0];
        assert!(m_min.abs() < 1e-3, "Megacity min not 0");
        let m_max = megacity(Vec8::splat(-9.5), Vec8::splat(-7.5))[0];
        assert!((m_max - 1.0).abs() < 1e-3, "Megacity max not 1");
        let s_min = sphere(Vec8::splat(0.0), Vec8::splat(0.0))[0];
        assert!(s_min.abs() < 1e-3, "Sphere min not 0");
        let s_max = sphere(Vec8::splat(5.0), Vec8::splat(5.0))[0];
        assert!((s_max - 1.0).abs() < 1e-3, "Sphere max not 1");
        let e_min = ellipsoid(Vec8::splat(0.0), Vec8::splat(0.0))[0];
        assert!(e_min.abs() < 1e-3, "Ellipsoid min not 0");
        let e_max = ellipsoid(Vec8::splat(5.0), Vec8::splat(5.0))[0];
        assert!((e_max - 1.0).abs() < 1e-3, "Ellipsoid max not 1");
        let r_min = rosenbrock(Vec8::splat(1.0), Vec8::splat(1.0))[0];
        assert!(r_min.abs() < 1e-3, "Rosenbrock min not 0");
        let r_max = rosenbrock(Vec8::splat(-5.0), Vec8::splat(-5.0))[0];
        assert!((r_max - 1.0).abs() < 1e-3, "Rosenbrock max not 1");
        let d_min = discus(Vec8::splat(0.0), Vec8::splat(0.0))[0];
        assert!(d_min.abs() < 1e-3, "Discus min not 0");
        let d_max = discus(Vec8::splat(5.0), Vec8::splat(5.0))[0];
        assert!((d_max - 1.0).abs() < 1e-3, "Discus max not 1");
        let dp_min = different_powers(Vec8::splat(0.0), Vec8::splat(0.0))[0];
        assert!(dp_min.abs() < 1e-3, "Different Powers min not 0");
        let dp_max = different_powers(Vec8::splat(5.0), Vec8::splat(5.0))[0];
        assert!((dp_max - 1.0).abs() < 1e-3, "Different Powers max not 1");
    }
}
