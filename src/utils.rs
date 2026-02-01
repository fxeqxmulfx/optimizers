use std::collections::BTreeMap;

use glam::Vec4;

pub fn clamp_to_unit_cube(value: f32) -> f32 {
    if value > 1.0 {
        return 1.0;
    }
    if value < 0.0 {
        return 0.0;
    }
    return value;
}

pub fn fit_in_bounds_simd(values: &[f32], range_min: &[f32], range_max: &[f32]) -> Vec<Vec4> {
    let len = values.len();
    let groups = len / 8;
    let mut out = Vec::with_capacity(groups * 2);
    for g in 0..groups {
        let base = g * 8;
        let vals_lo = Vec4::from_array([
            values[base],
            values[base + 1],
            values[base + 2],
            values[base + 3],
        ]);
        let vals_hi = Vec4::from_array([
            values[base + 4],
            values[base + 5],
            values[base + 6],
            values[base + 7],
        ]);
        let mins_lo = Vec4::from_array([
            range_min[base],
            range_min[base + 1],
            range_min[base + 2],
            range_min[base + 3],
        ]);
        let mins_hi = Vec4::from_array([
            range_min[base + 4],
            range_min[base + 5],
            range_min[base + 6],
            range_min[base + 7],
        ]);
        let maxs_lo = Vec4::from_array([
            range_max[base],
            range_max[base + 1],
            range_max[base + 2],
            range_max[base + 3],
        ]);
        let maxs_hi = Vec4::from_array([
            range_max[base + 4],
            range_max[base + 5],
            range_max[base + 6],
            range_max[base + 7],
        ]);
        let f_lo = mins_lo + vals_lo * (maxs_lo - mins_lo);
        let f_hi = mins_hi + vals_hi * (maxs_hi - mins_hi);
        let evens = Vec4::new(f_lo.x, f_lo.z, f_hi.x, f_hi.z);
        let odds = Vec4::new(f_lo.y, f_lo.w, f_hi.y, f_hi.w);
        out.push(evens);
        out.push(odds);
    }
    out
}

pub fn fit_in_bounds(values: &[f32], range_min: &[f32], range_max: &[f32]) -> Vec<f32> {
    let values_len = values.len();
    let mut result = vec![0.0; values_len];
    for i in 0..values_len {
        result[i] = range_min[i] + values[i] * (range_max[i] - range_min[i]);
    }
    return result;
}

pub fn format_x_history(values: &Vec<Vec<Vec<f32>>>, bounds: &Vec<[f32; 2]>) -> Vec<Vec<[f32; 2]>> {
    let mut result: Vec<Vec<[f32; 2]>> = Vec::with_capacity(values.len());
    for i in 0..values.len() {
        let inner_len = values[i].len() * (values[i][0].len() - 1);
        let mut temp_result = Vec::with_capacity(inner_len);
        for j in 0..values[i].len() {
            for x in 1..values[i][j].len() {
                let lhs =
                    bounds[x - 1][0] + values[i][j][x - 1] * (bounds[x - 1][1] - bounds[x - 1][0]);
                let rhs = bounds[x][0] + values[i][j][x] * (bounds[x][1] - bounds[x][0]);
                temp_result.push([lhs, rhs]);
            }
        }
        result.push(temp_result);
    }
    return result;
}

pub fn format_best_f_x_history(values: &Vec<Vec<f32>>) -> Vec<f32> {
    return values
        .iter()
        .filter_map(|inner_vec| {
            inner_vec
                .iter()
                .min_by(|a, b| a.total_cmp(b))
                .map(|max_val_ref| *max_val_ref)
        })
        .collect();
}

pub fn broadcast<F>(func: F) -> impl Fn(&[f32]) -> f32 + Sync
where
    F: Fn(f32, f32) -> f32 + Sync,
{
    move |x: &[f32]| -> f32 {
        let x_len = x.len();
        let sum: f32 = x.chunks_exact(2).map(|pair| func(pair[0], pair[1])).sum();
        (sum / x_len as f32) * 2.0
    }
}

pub fn broadcast_simd<F>(func: F) -> impl Fn(&[Vec4]) -> f32 + Sync
where
    F: Fn(Vec4, Vec4) -> Vec4 + Sync,
{
    move |x: &[Vec4]| {
        let x_len = x.len();
        let sum: f32 = x
            .chunks_exact(2)
            .map(|pair| func(pair[0], pair[1]).to_array().iter().sum::<f32>())
            .sum();
        (sum / (x_len * 4) as f32) * 2.0
    }
}

pub fn broadcast_scalar<F>(func: F) -> impl Fn(&[f32]) -> f32 + Sync
where
    F: Fn(Vec4, Vec4) -> Vec4 + Sync,
{
    move |x: &[f32]| -> f32 {
        let x_len = x.len();
        let sum: f32 = x
            .chunks_exact(2)
            .map(|pair| {
                func(Vec4::splat(pair[0]), Vec4::splat(pair[1]))
                    .to_array()
                    .iter()
                    .sum::<f32>()
            })
            .sum();
        (sum / (x_len * 4) as f32) * 2.0
    }
}

pub fn all_combinations(data: &BTreeMap<String, Vec<f32>>) -> Vec<BTreeMap<String, f32>> {
    let keys: Vec<String> = data.keys().cloned().collect();
    let values: Vec<Vec<f32>> = data.values().cloned().collect();
    let mut result = Vec::new();
    fn rec(
        keys: &[String],
        values: &[Vec<f32>],
        depth: usize,
        current: &mut Vec<f32>,
        out: &mut Vec<BTreeMap<String, f32>>,
    ) {
        if depth == keys.len() {
            let map = keys
                .iter()
                .zip(current.iter())
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            out.push(map);
            return;
        }
        for &val in &values[depth] {
            current[depth] = val;
            rec(keys, values, depth + 1, current, out);
        }
    }
    let mut current = vec![0.0; keys.len()];
    rec(&keys, &values, 0, &mut current, &mut result);
    result
}

pub fn f32_to_i64(x: f32) -> i64 {
    if x.is_infinite() {
        if x.is_sign_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    } else if x.is_nan() {
        0
    } else {
        x as i64
    }
}

pub trait Vec4Ext {
    fn square(&self) -> Vec4;
    fn cube(&self) -> Vec4;
    fn tesseract(&self) -> Vec4;
}

impl Vec4Ext for Vec4 {
    fn square(&self) -> Vec4 {
        *self * *self
    }
    fn cube(&self) -> Vec4 {
        *self * *self * *self
    }
    fn tesseract(&self) -> Vec4 {
        *self * *self * *self * *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[inline]
    fn almost_equal(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_broadcast_add() {
        let f = broadcast(|a, b| a + b);
        let res = f(&[1.0, 2.0, 3.0, 4.0]);
        assert!(almost_equal(res, 5.0, 1e-6));
    }

    #[test]
    fn test_broadcast_subtract() {
        let f = broadcast(|a, b| a - b);
        let res = f(&[5.0, 3.0, 2.0, 1.0]);
        assert!(almost_equal(res, 1.5, 1e-6));
    }

    #[test]
    fn test_broadcast_odd_length() {
        let f = broadcast(|a, b| a + b);
        let res = f(&[1.0, 2.0, 3.0]);
        assert!(almost_equal(res, 2.0, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_add() {
        let f = broadcast_simd(|a, b| a + b);
        let v1 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vec4::new(4.0, 3.0, 2.0, 1.0);
        let res = f(&[v1, v2]);
        assert!(almost_equal(res, 5.0, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_subtract() {
        let f = broadcast_simd(|a, b| a - b);
        let v1 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vec4::new(4.0, 3.0, 2.0, 1.0);
        let res = f(&[v1, v2]);
        assert!(almost_equal(res, 0.0, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_odd_length() {
        let f = broadcast_simd(|a, b| a + b);
        let v1 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vec4::new(4.0, 3.0, 2.0, 1.0);
        let v3 = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let res = f(&[v1, v2, v3]);
        assert!(almost_equal(res, 3.3333333, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_multiple_pairs() {
        let f = broadcast_simd(|a, b| a + b);
        let v1 = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let v2 = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let v3 = Vec4::new(2.0, 2.0, 2.0, 2.0);
        let v4 = Vec4::new(2.0, 2.0, 2.0, 2.0);
        let res = f(&[v1, v2, v3, v4]);
        assert!(almost_equal(res, 3.0, 1e-6));
    }

    #[test]
    fn test_sync_traits() {
        let f = broadcast(|a, b| a + b);
        let _: &dyn Sync = &f;
        let f2 = broadcast_simd(|a, b| a + b);
        let _: &dyn Sync = &f2;
    }

    #[test]
    fn test_zero_values_map_to_mins() {
        let values = vec![0.0_f32; 8];
        let mins = (0..8).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..8).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        let expected_lo = Vec4::from_array([0.0, 1.0, 2.0, 3.0]);
        let expected_hi = Vec4::from_array([4.0, 5.0, 6.0, 7.0]);
        assert_eq!(
            out[0],
            Vec4::new(expected_lo.x, expected_lo.z, expected_hi.x, expected_hi.z)
        );
        assert_eq!(
            out[1],
            Vec4::new(expected_lo.y, expected_lo.w, expected_hi.y, expected_hi.w)
        );
    }

    #[test]
    fn test_one_values_map_to_maxs() {
        let values = vec![1.0_f32; 8];
        let mins = (0..8).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..8).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        let expected_lo = Vec4::from_array([10.0, 11.0, 12.0, 13.0]);
        let expected_hi = Vec4::from_array([14.0, 15.0, 16.0, 17.0]);
        assert_eq!(
            out[0],
            Vec4::new(expected_lo.x, expected_lo.z, expected_hi.x, expected_hi.z)
        );
        assert_eq!(
            out[1],
            Vec4::new(expected_lo.y, expected_lo.w, expected_hi.y, expected_hi.w)
        );
    }

    #[test]
    fn test_half_values_map_to_midpoints() {
        let values = vec![0.5_f32; 8];
        let mins = (0..8).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..8).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        let expected_lo = Vec4::from_array([5.0, 6.0, 7.0, 8.0]);
        let expected_hi = Vec4::from_array([9.0, 10.0, 11.0, 12.0]);
        assert_eq!(
            out[0],
            Vec4::new(expected_lo.x, expected_lo.z, expected_hi.x, expected_hi.z)
        );
        assert_eq!(
            out[1],
            Vec4::new(expected_lo.y, expected_lo.w, expected_hi.y, expected_hi.w)
        );
    }

    #[test]
    fn test_even_odd_ordering() {
        let values = vec![0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5];
        let mins = vec![0.0_f32; 8];
        let maxs = vec![10.0_f32; 8];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        let evens_expected = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let odds_expected = Vec4::new(5.0, 5.0, 5.0, 5.0);
        assert_eq!(out[0], evens_expected);
        assert_eq!(out[1], odds_expected);
    }

    #[test]
    fn test_ignores_remainder() {
        let values = vec![1.0_f32; 10];
        let mins = vec![0.0_f32; 10];
        let maxs = vec![2.0_f32; 10];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        assert_eq!(out.len(), 2);
        let expected_vec = Vec4::new(2.0, 2.0, 2.0, 2.0);
        assert_eq!(out[0], expected_vec);
        assert_eq!(out[1], expected_vec);
    }

    #[test]
    fn test_output_capacity() {
        let len = 24;
        let values = vec![0.0_f32; len];
        let mins = vec![0.0_f32; len];
        let maxs = vec![1.0_f32; len];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        assert_eq!(out.capacity(), (len / 8) * 2);
        assert_eq!(out.len(), out.capacity());
    }

    #[test]
    fn test_clamp_to_unit_cube() {
        assert!(almost_equal(clamp_to_unit_cube(-0.5), 0.0, 1e-6));
        assert!(almost_equal(clamp_to_unit_cube(0.0), 0.0, 1e-6));
        assert!(almost_equal(clamp_to_unit_cube(0.5), 0.5, 1e-6));
        assert!(almost_equal(clamp_to_unit_cube(1.0), 1.0, 1e-6));
        assert!(almost_equal(clamp_to_unit_cube(1.5), 1.0, 1e-6));
    }

    #[test]
    fn test_fit_in_bounds_scalar() {
        let values = vec![0.0, 0.5, 1.0];
        let mins = vec![0.0, 10.0, 20.0];
        let maxs = vec![10.0, 20.0, 30.0];
        let out = fit_in_bounds(&values, &mins, &maxs);
        assert_eq!(out, vec![0.0, 15.0, 30.0]);
    }

    #[test]
    fn test_fit_in_bounds_empty() {
        let values: Vec<f32> = vec![];
        let mins: Vec<f32> = vec![];
        let maxs: Vec<f32> = vec![];
        let out = fit_in_bounds(&values, &mins, &maxs);
        assert!(out.is_empty());
    }

    #[test]
    fn test_broadcast_scalar_add() {
        let f = broadcast_scalar(|a, b| a + b);
        let res = f(&[1.0, 2.0, 3.0, 4.0]);
        assert!(almost_equal(res, 5.0, 1e-6));
    }

    #[test]
    fn test_broadcast_scalar_mul() {
        let f = broadcast_scalar(|a, b| a * b);
        let res = f(&[1.0, 2.0, 3.0, 4.0]);
        assert!(almost_equal(res, 7.0, 1e-6));
    }

    #[test]
    fn test_broadcast_scalar_odd_length() {
        let f = broadcast_scalar(|a, b| a + b);
        let res = f(&[1.0, 2.0, 3.0]);
        assert!(almost_equal(res, 2.0, 1e-6));
    }

    #[test]
    fn test_all_combinations_simple() {
        let mut map: BTreeMap<String, Vec<f32>> = BTreeMap::new();
        map.insert("a".to_string(), vec![1.0, 2.0]);
        map.insert("b".to_string(), vec![3.0, 4.0]);
        let combos = all_combinations(&map);
        assert_eq!(combos.len(), 4);
        let mut expected = Vec::new();
        for &a in &[1.0, 2.0] {
            for &b in &[3.0, 4.0] {
                let mut m = BTreeMap::new();
                m.insert("a".to_string(), a);
                m.insert("b".to_string(), b);
                expected.push(m);
            }
        }
        assert_eq!(combos, expected);
    }

    #[test]
    fn test_format_x_history_simple() {
        let values = vec![vec![vec![0.0, 1.0], vec![2.0, 3.0]]];
        let bounds = vec![[0.0, 1.0], [1.0, 2.0]];
        let out = format_x_history(&values, &bounds);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], vec![[0.0, 2.0], [2.0, 4.0]]);
    }

    #[test]
    fn test_format_best_f_x_history_min() {
        let values = vec![vec![5.0, 3.0, 7.0], vec![4.0, 6.0], vec![2.0]];
        let out = format_best_f_x_history(&values);
        assert_eq!(out, vec![3.0, 4.0, 2.0]);
    }

    #[test]
    fn test_f32_to_i64() {
        assert_eq!(f32_to_i64(42.7), 42);
        assert_eq!(f32_to_i64(-0.1), 0);
        assert_eq!(f32_to_i64(f32::INFINITY), i64::MAX);
        assert_eq!(f32_to_i64(f32::NEG_INFINITY), i64::MIN);
        assert_eq!(f32_to_i64(f32::NAN), 0);
    }

    #[test]
    fn test_vec4_ext_methods() {
        let v = Vec4::new(2.0, 3.0, 4.0, 5.0);
        assert_eq!(v.square(), v * v);
        assert_eq!(v.cube(), v * v * v);
        assert_eq!(v.tesseract(), v * v * v * v);
    }

    #[test]
    fn test_fit_in_bounds_simd_empty() {
        let values: Vec<f32> = vec![];
        let mins: Vec<f32> = vec![];
        let maxs: Vec<f32> = vec![];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        assert!(out.is_empty());
    }
}
