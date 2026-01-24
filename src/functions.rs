use std::f32::consts::PI;

pub struct TestFunction {
    pub name: &'static str,
    pub func: fn(f32, f32) -> f32,
    pub bounds: [[f32; 2]; 2],
}

pub fn scale(result: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    let old_span = old_max - old_min;
    let new_span = new_max - new_min;
    ((result - old_min) / old_span) * new_span + new_min
}

pub const SHIFTED_SPHERE_BOUNDS: [[f32; 2]; 2] = [[-10.0, 10.0], [-10.0, 10.0]];

pub fn shifted_sphere(x: f32, y: f32) -> f32 {
    let x = x + PI;
    let y = y + PI;
    let result = x.powi(2) + y.powi(2);
    scale(result, 0.0, 345.402914946, 0.0, 1.0)
}

fn _weierstrass(x: f32, a: f32, b: f32, k_max: usize) -> f32 {
    let mut total = 0.0_f32;
    for k in 0..k_max {
        total += a.powi(k as i32) * f32::cos(b.powi(k as i32) * PI * x);
    }
    total
}

fn _weierstrass_default(x: f32) -> f32 {
    _weierstrass(x, 0.5, 7.0, 20)
}

pub const SHIFTED_WEIERSTRASS_BOUNDS: [[f32; 2]; 2] = [[-10.0, 10.0], [-10.0, 10.0]];

pub fn shifted_weierstrass(x: f32, y: f32) -> f32 {
    let x = x + PI;
    let y = y + PI;
    let result = (_weierstrass_default(x) + _weierstrass_default(y)) / 2.0;
    scale(result, -2.0, 2.0, 0.0, 1.0)
}

pub const HILLY_BOUNDS: [[f32; 2]; 2] = [[-3.0, 3.0], [-3.0, 3.0]];

pub fn hilly(x: f32, y: f32) -> f32 {
    let result = 20.0 + x.powi(2) + y.powi(2)
        - 10.0 * f32::cos(2.0 * PI * x)
        - 10.0 * f32::cos(2.0 * PI * y)
        - 30.0 * f32::exp(-((x - 1.0).powi(2) + y.powi(2)) / 0.1)
        + 200.0 * f32::exp(-((x + PI * 0.47).powi(2) + (y - PI * 0.2).powi(2)) / 0.1)
        + 100.0 * f32::exp(-((x - 0.5).powi(2) + (y + 0.5).powi(2)) / 0.01)
        - 60.0 * f32::exp(-((x - 1.33).powi(2) + (y - 2.0).powi(2)) / 0.02)
        - 40.0 * f32::exp(-((x + 1.3).powi(2) + (y + 0.2).powi(2)) / 0.5)
        + 60.0 * f32::exp(-((x - 1.5).powi(2) + (y + 1.5).powi(2)) / 0.1);
    let result = -result;
    scale(result, -229.91931214214105, 39.701816104859866, 0.0, 1.0)
}

pub const FOREST_BOUNDS: [[f32; 2]; 2] = [[-43.50, -39.0], [-47.35, -40.0]];

pub fn forest(x: f32, y: f32) -> f32 {
    let a = f32::sin(f32::sqrt(f32::abs(x - 1.13) + f32::abs(y - 2.0)));
    let b = f32::cos(f32::sqrt(f32::abs(f32::sin(x))) + f32::sqrt(f32::abs(f32::sin(y - 2.0))));
    let f = a
        + b
        + 1.01 * f32::exp(-(((x + 42.0).powi(2) + (y + 43.5).powi(2)) / 0.9))
        + 1.0 * f32::exp(-(((x + 40.2).powi(2) + (y + 46.0).powi(2)) / 0.3));

    let result = f.powi(4) - 0.3 * f32::exp(-(((x + 42.3).powi(2) + (y + 46.0).powi(2)) / 0.02));
    let result = -result;
    scale(result, -1.8779867959790217, 0.26489289358875895, 0.0, 1.0)
}

pub const MEGACITY_BOUNDS: [[f32; 2]; 2] = [[-10.0, -2.0], [-10.5, 10.0]];

pub fn megacity(x: f32, y: f32) -> f32 {
    let a = f32::sin(f32::sqrt(f32::abs(x - 1.13) + f32::abs(y - 2.0)));
    let b = f32::cos(f32::sqrt(f32::abs(f32::sin(x))) + f32::sqrt(f32::abs(f32::sin(y - 2.0))));
    let f = a + b;
    let result = f32::floor(f.powi(4))
        - f32::floor(2.0 * f32::exp(-((x + 9.5).powi(2) + (y + 7.5).powi(2)) / 0.4));
    let result = -result;
    scale(result, -12.0_f32, 1.0_f32, 0.0_f32, 1.0_f32)
}

pub const TEST_FUNCTIONS: [TestFunction; 5] = [
    TestFunction {
        name: "shifted_sphere",
        func: shifted_sphere,
        bounds: SHIFTED_SPHERE_BOUNDS,
    },
    TestFunction {
        name: "shifted_weierstrass",
        func: shifted_weierstrass,
        bounds: SHIFTED_WEIERSTRASS_BOUNDS,
    },
    TestFunction {
        name: "hilly",
        func: hilly,
        bounds: HILLY_BOUNDS,
    },
    TestFunction {
        name: "forest",
        func: forest,
        bounds: FOREST_BOUNDS,
    },
    TestFunction {
        name: "megacity",
        func: megacity,
        bounds: MEGACITY_BOUNDS,
    },
];
