use std::{collections::BTreeMap, f32::consts::PI};

use glam::Vec4;
use once_cell::sync::Lazy;

use crate::utils::Vec4Ext;

pub struct TestFunction {
    pub func: fn(Vec4, Vec4) -> Vec4,
    pub bounds: [[f32; 2]; 2],
}

fn scale(v: Vec4, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> Vec4 {
    let in_range = in_max - in_min;
    let out_range = out_max - out_min;
    (v - in_min) / in_range * out_range + out_min
}

pub const SHIFTED_SPHERE_BOUNDS: [[f32; 2]; 2] = [[-10.0, 10.0], [-10.0, 10.0]];

pub fn shifted_sphere(x: Vec4, y: Vec4) -> Vec4 {
    let x = x + PI;
    let y = y + PI;
    let result = x * x + y * y;
    scale(result, 0.0, 345.402914946, 0.0, 1.0)
}

const WEIERSTRASS_AK: [f32; 27] = [
    1.0,
    0.5,
    0.25,
    0.125,
    0.0625,
    0.03125,
    0.015625,
    0.0078125,
    0.00390625,
    0.001953125,
    0.0009765625,
    0.00048828125,
    0.00024414063,
    0.00012207031,
    6.1035156e-5,
    3.0517578e-5,
    1.5258789e-5,
    7.6293945e-6,
    3.8146973e-6,
    1.9073486e-6,
    9.536743e-7,
    4.7683716e-7,
    2.3841858e-7,
    1.1920929e-7,
    5.9604645e-8,
    2.9802322e-8,
    1.4901161e-8,
];

const WEIERSTRASS_BK: [f32; 27] = [
    1.0,
    7.0,
    49.0,
    343.0,
    2401.0,
    16807.0,
    117649.0,
    823543.0,
    5764801.0,
    40353610.0,
    282475260.0,
    1977326700.0,
    13841287000.0,
    96889010000.0,
    678223100000.0,
    4747561500000.0,
    33232930000000.0,
    232630510000000.0,
    1628413600000000.0,
    1.1398895e16,
    7.979226e16,
    5.5854586e17,
    3.909821e18,
    2.7368747e19,
    1.9158123e20,
    1.3410687e21,
    9.3874804e21,
];

pub fn weierstrass(x: Vec4) -> Vec4 {
    let t0 = WEIERSTRASS_AK[0] * ((WEIERSTRASS_BK[0] * PI * x).cos());
    let t1 = WEIERSTRASS_AK[1] * ((WEIERSTRASS_BK[1] * PI * x).cos());
    let t2 = WEIERSTRASS_AK[2] * ((WEIERSTRASS_BK[2] * PI * x).cos());
    let t3 = WEIERSTRASS_AK[3] * ((WEIERSTRASS_BK[3] * PI * x).cos());
    let t4 = WEIERSTRASS_AK[4] * ((WEIERSTRASS_BK[4] * PI * x).cos());
    let t5 = WEIERSTRASS_AK[5] * ((WEIERSTRASS_BK[5] * PI * x).cos());
    let t6 = WEIERSTRASS_AK[6] * ((WEIERSTRASS_BK[6] * PI * x).cos());
    let t7 = WEIERSTRASS_AK[7] * ((WEIERSTRASS_BK[7] * PI * x).cos());
    let t8 = WEIERSTRASS_AK[8] * ((WEIERSTRASS_BK[8] * PI * x).cos());
    let t9 = WEIERSTRASS_AK[9] * ((WEIERSTRASS_BK[9] * PI * x).cos());
    let t10 = WEIERSTRASS_AK[10] * ((WEIERSTRASS_BK[10] * PI * x).cos());
    let t11 = WEIERSTRASS_AK[11] * ((WEIERSTRASS_BK[11] * PI * x).cos());
    let t12 = WEIERSTRASS_AK[12] * ((WEIERSTRASS_BK[12] * PI * x).cos());
    let t13 = WEIERSTRASS_AK[13] * ((WEIERSTRASS_BK[13] * PI * x).cos());
    let t14 = WEIERSTRASS_AK[14] * ((WEIERSTRASS_BK[14] * PI * x).cos());
    let t15 = WEIERSTRASS_AK[15] * ((WEIERSTRASS_BK[15] * PI * x).cos());
    let t16 = WEIERSTRASS_AK[16] * ((WEIERSTRASS_BK[16] * PI * x).cos());
    let t17 = WEIERSTRASS_AK[17] * ((WEIERSTRASS_BK[17] * PI * x).cos());
    let t18 = WEIERSTRASS_AK[18] * ((WEIERSTRASS_BK[18] * PI * x).cos());
    let t19 = WEIERSTRASS_AK[19] * ((WEIERSTRASS_BK[19] * PI * x).cos());
    let t20 = WEIERSTRASS_AK[20] * ((WEIERSTRASS_BK[20] * PI * x).cos());
    let t21 = WEIERSTRASS_AK[21] * ((WEIERSTRASS_BK[21] * PI * x).cos());
    let t22 = WEIERSTRASS_AK[22] * ((WEIERSTRASS_BK[22] * PI * x).cos());
    let t23 = WEIERSTRASS_AK[23] * ((WEIERSTRASS_BK[23] * PI * x).cos());
    let t24 = WEIERSTRASS_AK[24] * ((WEIERSTRASS_BK[24] * PI * x).cos());
    let t25 = WEIERSTRASS_AK[25] * ((WEIERSTRASS_BK[25] * PI * x).cos());
    let t26 = WEIERSTRASS_AK[26] * ((WEIERSTRASS_BK[26] * PI * x).cos());
    t0 + t1
        + t2
        + t3
        + t4
        + t5
        + t6
        + t7
        + t8
        + t9
        + t10
        + t11
        + t12
        + t13
        + t14
        + t15
        + t16
        + t17
        + t18
        + t19
        + t20
        + t21
        + t22
        + t23
        + t24
        + t25
        + t26
}

pub const SHIFTED_WEIERSTRASS_BOUNDS: [[f32; 2]; 2] = [[-10.0, 10.0], [-10.0, 10.0]];

pub fn shifted_weierstrass(x: Vec4, y: Vec4) -> Vec4 {
    let x = x + PI;
    let y = y + PI;
    let result = (weierstrass(x) + weierstrass(y)) / 2.0;
    scale(result, -2.0, 2.0, 0.0, 1.0)
}

pub const HILLY_BOUNDS: [[f32; 2]; 2] = [[-3.0, 3.0], [-3.0, 3.0]];

pub fn hilly(x: Vec4, y: Vec4) -> Vec4 {
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

pub fn forest(x: Vec4, y: Vec4) -> Vec4 {
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

pub fn megacity(x: Vec4, y: Vec4) -> Vec4 {
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

pub static TEST_FUNCTIONS: Lazy<BTreeMap<String, TestFunction>> = Lazy::new(|| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_extrema() {
        let s_min = shifted_sphere(Vec4::splat(-PI), Vec4::splat(-PI)).x;
        assert!(s_min.abs() < 1e-3, "Shifted sphere min not 0");
        let s_max = shifted_sphere(Vec4::splat(10.0), Vec4::splat(10.0)).x;
        assert!((s_max - 1.0).abs() < 1e-3, "Shifted sphere max not 1");
        let w_min = shifted_weierstrass(Vec4::splat(1.0 - PI), Vec4::splat(1.0 - PI)).x;
        assert!(w_min.abs() < 1e-2, "Shifted weierstrass min not 0");
        let w_max = shifted_weierstrass(Vec4::splat(-PI), Vec4::splat(-PI)).x;
        assert!((w_max - 1.0).abs() < 1e-3, "Shifted weierstrass max not 1");
        let h_min = hilly(
            Vec4::splat(-1.4809053654574758),
            Vec4::splat(0.6254111843389699),
        )
        .x;
        assert!(h_min.abs() < 1e-3, "Hilly min not 0");
        let h_max = hilly(
            Vec4::splat(1.3200361419666748),
            Vec4::splat(1.9993728393766546),
        )
        .x;
        assert!((h_max - 1.0).abs() < 1e-3, "Hilly max not 1");
        let f_min = forest(
            Vec4::splat(-40.840704496667314),
            Vec4::splat(-41.982297150257104),
        )
        .x;
        assert!(f_min.abs() < 1e-3, "Forest min not 0");
        let f_max = forest(
            Vec4::splat(-42.2988573690385010),
            Vec4::splat(-45.9956119113080675),
        )
        .x;
        assert!((f_max - 1.0).abs() < 1e-3, "Forest max not 1");
        let m_min = megacity(
            Vec4::splat(-3.1357545740179393),
            Vec4::splat(2.006136371058429),
        )
        .x;
        assert!(m_min.abs() < 1e-3, "Megacity min not 0");
        let m_max = megacity(Vec4::splat(-9.5), Vec4::splat(-7.5)).x;
        assert!((m_max - 1.0).abs() < 1e-3, "Megacity max not 1");
    }
}
