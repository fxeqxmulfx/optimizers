use std::collections::BTreeMap;

use simd_vector::Vec8;
use simd_vector::fast::FastMath;

pub fn clamp_to_unit_cube(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

/// Pre-computed SIMD bounds for fast repeated `fit_in_bounds_simd` calls.
/// Pre-computes min and range (max-min) as Vec8 pairs per group of 16 values,
/// avoiding redundant loads and subtraction on every call.
pub struct BoundsSimd {
    /// [mins_lo_g0, mins_hi_g0, mins_lo_g1, mins_hi_g1, ...]
    mins: Vec<Vec8>,
    /// [ranges_lo_g0, ranges_hi_g0, ranges_lo_g1, ranges_hi_g1, ...]
    ranges: Vec<Vec8>,
    groups: usize,
}

impl BoundsSimd {
    pub fn new(range_min: &[f32], range_max: &[f32]) -> Self {
        let len = range_min.len();
        let groups = len / 16;
        let mut mins = Vec::with_capacity(groups * 2);
        let mut ranges = Vec::with_capacity(groups * 2);
        for g in 0..groups {
            let base = g * 16;
            let mins_lo = Vec8::from([
                range_min[base],
                range_min[base + 1],
                range_min[base + 2],
                range_min[base + 3],
                range_min[base + 4],
                range_min[base + 5],
                range_min[base + 6],
                range_min[base + 7],
            ]);
            let mins_hi = Vec8::from([
                range_min[base + 8],
                range_min[base + 9],
                range_min[base + 10],
                range_min[base + 11],
                range_min[base + 12],
                range_min[base + 13],
                range_min[base + 14],
                range_min[base + 15],
            ]);
            let maxs_lo = Vec8::from([
                range_max[base],
                range_max[base + 1],
                range_max[base + 2],
                range_max[base + 3],
                range_max[base + 4],
                range_max[base + 5],
                range_max[base + 6],
                range_max[base + 7],
            ]);
            let maxs_hi = Vec8::from([
                range_max[base + 8],
                range_max[base + 9],
                range_max[base + 10],
                range_max[base + 11],
                range_max[base + 12],
                range_max[base + 13],
                range_max[base + 14],
                range_max[base + 15],
            ]);
            mins.push(mins_lo);
            mins.push(mins_hi);
            ranges.push(maxs_lo - mins_lo);
            ranges.push(maxs_hi - mins_hi);
        }
        Self {
            mins,
            ranges,
            groups,
        }
    }

    #[inline]
    pub fn output_len(&self) -> usize {
        self.groups * 2
    }

    /// Transform values into SIMD bounds, writing into `out`.
    /// `out` must have length >= `self.output_len()`.
    #[inline]
    pub fn transform_into(&self, values: &[f32], out: &mut [Vec8]) {
        let ranges = &self.ranges[..self.groups * 2];
        let mins = &self.mins[..self.groups * 2];
        let out = &mut out[..self.groups * 2];
        for g in 0..self.groups {
            let v = &values[g * 16..g * 16 + 16];
            let pair = g * 2;
            let vals_lo = Vec8::from([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
            let vals_hi = Vec8::from([v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15]]);
            let f_lo = vals_lo.mul_add(ranges[pair], mins[pair]);
            let f_hi = vals_hi.mul_add(ranges[pair + 1], mins[pair + 1]);
            out[pair] = Vec8([
                f_lo[0], f_lo[2], f_lo[4], f_lo[6], f_hi[0], f_hi[2], f_hi[4], f_hi[6],
            ]);
            out[pair + 1] = Vec8([
                f_lo[1], f_lo[3], f_lo[5], f_lo[7], f_hi[1], f_hi[3], f_hi[5], f_hi[7],
            ]);
        }
    }
}

pub fn fit_in_bounds_simd(values: &[f32], range_min: &[f32], range_max: &[f32]) -> Vec<Vec8> {
    let len = values.len();
    let groups = len / 16;
    let mut out = Vec::with_capacity(groups * 2);
    for g in 0..groups {
        let base = g * 16;
        let vals_lo = Vec8::from([
            values[base],
            values[base + 1],
            values[base + 2],
            values[base + 3],
            values[base + 4],
            values[base + 5],
            values[base + 6],
            values[base + 7],
        ]);
        let vals_hi = Vec8::from([
            values[base + 8],
            values[base + 9],
            values[base + 10],
            values[base + 11],
            values[base + 12],
            values[base + 13],
            values[base + 14],
            values[base + 15],
        ]);
        let mins_lo = Vec8::from([
            range_min[base],
            range_min[base + 1],
            range_min[base + 2],
            range_min[base + 3],
            range_min[base + 4],
            range_min[base + 5],
            range_min[base + 6],
            range_min[base + 7],
        ]);
        let mins_hi = Vec8::from([
            range_min[base + 8],
            range_min[base + 9],
            range_min[base + 10],
            range_min[base + 11],
            range_min[base + 12],
            range_min[base + 13],
            range_min[base + 14],
            range_min[base + 15],
        ]);
        let maxs_lo = Vec8::from([
            range_max[base],
            range_max[base + 1],
            range_max[base + 2],
            range_max[base + 3],
            range_max[base + 4],
            range_max[base + 5],
            range_max[base + 6],
            range_max[base + 7],
        ]);
        let maxs_hi = Vec8::from([
            range_max[base + 8],
            range_max[base + 9],
            range_max[base + 10],
            range_max[base + 11],
            range_max[base + 12],
            range_max[base + 13],
            range_max[base + 14],
            range_max[base + 15],
        ]);
        let f_lo = mins_lo + vals_lo * (maxs_lo - mins_lo);
        let f_hi = mins_hi + vals_hi * (maxs_hi - mins_hi);
        let evens = Vec8([
            f_lo[0], f_lo[2], f_lo[4], f_lo[6], f_hi[0], f_hi[2], f_hi[4], f_hi[6],
        ]);
        let odds = Vec8([
            f_lo[1], f_lo[3], f_lo[5], f_lo[7], f_hi[1], f_hi[3], f_hi[5], f_hi[7],
        ]);
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
        x.chunks_exact(2)
            .map(|pair| func(pair[0], pair[1]))
            .sum::<f32>()
            * 2.0
            / x.len() as f32
    }
}

pub fn broadcast_simd<F>(func: F) -> impl Fn(&[Vec8]) -> f32 + Sync
where
    F: Fn(Vec8, Vec8) -> Vec8 + Sync,
{
    move |x: &[Vec8]| -> f32 {
        let inv = 1.0 / (x.len() * 4) as f32;
        x.chunks_exact(2)
            .map(|pair| func(pair[0], pair[1]))
            .sum::<Vec8>()
            .sum()
            * inv
    }
}

pub fn broadcast_scalar<F>(func: F) -> impl Fn(&[f32]) -> f32 + Sync
where
    F: Fn(Vec8, Vec8) -> Vec8 + Sync,
{
    move |x: &[f32]| -> f32 {
        x.chunks_exact(2)
            .map(|pair| func(Vec8::splat(pair[0]), Vec8::splat(pair[1])))
            .sum::<Vec8>()
            .sum()
            / (x.len() * 4) as f32
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

pub fn group_by_key<'a>(
    results: &'a [(i64, BTreeMap<String, f32>, BTreeMap<String, f32>)],
    key: &str,
) -> BTreeMap<i64, Vec<&'a (i64, BTreeMap<String, f32>, BTreeMap<String, f32>)>> {
    let mut grouped: BTreeMap<i64, Vec<&(i64, BTreeMap<String, f32>, BTreeMap<String, f32>)>> =
        BTreeMap::new();
    for entry in results {
        let group_key = f32_to_i64(entry.1[key]);
        grouped.entry(group_key).or_default().push(entry);
    }
    grouped
}

pub fn mean_and_mad(values: &[f32]) -> (f32, f32) {
    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let mad = values.iter().map(|v| (v - mean).abs()).sum::<f32>() / n;
    (mean, mad)
}

#[derive(Debug, PartialEq)]
pub struct GroupSummary {
    pub best: f32,
    pub worst: f32,
    pub mean: f32,
    pub mad: f32,
    pub count: usize,
    pub finite_count: usize,
}

pub fn summarize_group(means: &[f32]) -> GroupSummary {
    let finite_vals: Vec<f32> = means.iter().copied().filter(|v| v.is_finite()).collect();
    let finite_count = finite_vals.len();
    if finite_count == 0 {
        return GroupSummary {
            best: f32::INFINITY,
            worst: f32::INFINITY,
            mean: f32::INFINITY,
            mad: 0.0,
            count: means.len(),
            finite_count: 0,
        };
    }
    let (mean, mad) = mean_and_mad(&finite_vals);
    let best = finite_vals.iter().copied().reduce(f32::min).unwrap();
    let worst = finite_vals.iter().copied().reduce(f32::max).unwrap();
    GroupSummary {
        best,
        worst,
        mean,
        mad,
        count: means.len(),
        finite_count,
    }
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

pub trait Vec8Ext {
    fn square(&self) -> Vec8;
    fn cube(&self) -> Vec8;
    fn tesseract(&self) -> Vec8;
}

impl Vec8Ext for Vec8 {
    fn square(&self) -> Vec8 {
        *self * *self
    }
    fn cube(&self) -> Vec8 {
        *self * *self * *self
    }
    fn tesseract(&self) -> Vec8 {
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
        let v1 = Vec8([1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0]);
        let v2 = Vec8([4.0, 3.0, 2.0, 1.0, 4.0, 3.0, 2.0, 1.0]);
        let res = f(&[v1, v2]);
        assert!(almost_equal(res, 5.0, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_subtract() {
        let f = broadcast_simd(|a, b| a - b);
        let v1 = Vec8([1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0]);
        let v2 = Vec8([4.0, 3.0, 2.0, 1.0, 4.0, 3.0, 2.0, 1.0]);
        let res = f(&[v1, v2]);
        assert!(almost_equal(res, 0.0, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_odd_length() {
        let f = broadcast_simd(|a, b| a + b);
        let v1 = Vec8([1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0]);
        let v2 = Vec8([4.0, 3.0, 2.0, 1.0, 4.0, 3.0, 2.0, 1.0]);
        let v3 = Vec8([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let res = f(&[v1, v2, v3]);
        assert!(almost_equal(res, 3.3333333, 1e-6));
    }

    #[test]
    fn test_broadcast_simd_multiple_pairs() {
        let f = broadcast_simd(|a, b| a + b);
        let v1 = Vec8([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        let v2 = Vec8([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        let v3 = Vec8([2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]);
        let v4 = Vec8([2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]);
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
        let values = vec![0.0_f32; 16];
        let mins = (0..16).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..16).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        // evens: indices 0,2,4,6,8,10,12,14
        assert_eq!(out[0], Vec8([0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0]));
        // odds: indices 1,3,5,7,9,11,13,15
        assert_eq!(out[1], Vec8([1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0]));
    }

    #[test]
    fn test_one_values_map_to_maxs() {
        let values = vec![1.0_f32; 16];
        let mins = (0..16).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..16).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        // evens: indices 0,2,4,6,8,10,12,14 → maxs at those indices
        assert_eq!(
            out[0],
            Vec8([10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0])
        );
        // odds: indices 1,3,5,7,9,11,13,15 → maxs at those indices
        assert_eq!(
            out[1],
            Vec8([11.0, 13.0, 15.0, 17.0, 19.0, 21.0, 23.0, 25.0])
        );
    }

    #[test]
    fn test_half_values_map_to_midpoints() {
        let values = vec![0.5_f32; 16];
        let mins = (0..16).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..16).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        // evens: midpoints at indices 0,2,4,6,8,10,12,14
        assert_eq!(out[0], Vec8([5.0, 7.0, 9.0, 11.0, 13.0, 15.0, 17.0, 19.0]));
        // odds: midpoints at indices 1,3,5,7,9,11,13,15
        assert_eq!(out[1], Vec8([6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0]));
    }

    #[test]
    fn test_even_odd_ordering() {
        let values = vec![
            0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5,
        ];
        let mins = vec![0.0_f32; 16];
        let maxs = vec![10.0_f32; 16];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        let evens_expected = Vec8([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let odds_expected = Vec8([5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0]);
        assert_eq!(out[0], evens_expected);
        assert_eq!(out[1], odds_expected);
    }

    #[test]
    fn test_ignores_remainder() {
        let values = vec![1.0_f32; 18];
        let mins = vec![0.0_f32; 18];
        let maxs = vec![2.0_f32; 18];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        assert_eq!(out.len(), 2);
        let expected_vec = Vec8([2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]);
        assert_eq!(out[0], expected_vec);
        assert_eq!(out[1], expected_vec);
    }

    #[test]
    fn test_output_capacity() {
        let len = 48;
        let values = vec![0.0_f32; len];
        let mins = vec![0.0_f32; len];
        let maxs = vec![1.0_f32; len];
        let out = fit_in_bounds_simd(&values, &mins, &maxs);
        assert_eq!(out.capacity(), (len / 16) * 2);
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
    fn test_vec8_ext_methods() {
        let v = Vec8([2.0, 3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0]);
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

    #[test]
    fn test_bounds_simd_new_and_output_len() {
        let mins = vec![0.0_f32; 16];
        let maxs = vec![10.0_f32; 16];
        let bs = BoundsSimd::new(&mins, &maxs);
        assert_eq!(bs.output_len(), 2);
    }

    #[test]
    fn test_bounds_simd_transform_matches_fit_in_bounds_simd() {
        let values = vec![0.5_f32; 16];
        let mins = (0..16).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..16).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let expected = fit_in_bounds_simd(&values, &mins, &maxs);
        let bs = BoundsSimd::new(&mins, &maxs);
        let mut out = vec![Vec8::ZERO; bs.output_len()];
        bs.transform_into(&values, &mut out);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_bounds_simd_transform_zeros() {
        let values = vec![0.0_f32; 16];
        let mins = (0..16).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..16).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let expected = fit_in_bounds_simd(&values, &mins, &maxs);
        let bs = BoundsSimd::new(&mins, &maxs);
        let mut out = vec![Vec8::ZERO; bs.output_len()];
        bs.transform_into(&values, &mut out);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_bounds_simd_transform_ones() {
        let values = vec![1.0_f32; 16];
        let mins = (0..16).map(|i| i as f32).collect::<Vec<f32>>();
        let maxs = (0..16).map(|i| (i as f32) + 10.0).collect::<Vec<f32>>();
        let expected = fit_in_bounds_simd(&values, &mins, &maxs);
        let bs = BoundsSimd::new(&mins, &maxs);
        let mut out = vec![Vec8::ZERO; bs.output_len()];
        bs.transform_into(&values, &mut out);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_bounds_simd_multiple_groups() {
        let values = vec![0.25_f32; 32];
        let mins = vec![0.0_f32; 32];
        let maxs = vec![4.0_f32; 32];
        let expected = fit_in_bounds_simd(&values, &mins, &maxs);
        let bs = BoundsSimd::new(&mins, &maxs);
        assert_eq!(bs.output_len(), 4);
        let mut out = vec![Vec8::ZERO; bs.output_len()];
        bs.transform_into(&values, &mut out);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_mean_and_mad() {
        let (mean, mad) = mean_and_mad(&[1.0, 2.0, 3.0]);
        assert!(almost_equal(mean, 2.0, 1e-6));
        // MAD: |1-2| + |2-2| + |3-2| = 2, / 3 ≈ 0.6667
        assert!(almost_equal(mad, 2.0 / 3.0, 1e-6));
    }

    #[test]
    fn test_mean_and_mad_identical() {
        let (mean, mad) = mean_and_mad(&[5.0, 5.0, 5.0]);
        assert!(almost_equal(mean, 5.0, 1e-6));
        assert!(almost_equal(mad, 0.0, 1e-6));
    }

    #[test]
    fn test_summarize_group_normal() {
        let s = summarize_group(&[1.0, 3.0, 5.0]);
        assert!(almost_equal(s.best, 1.0, 1e-6));
        assert!(almost_equal(s.worst, 5.0, 1e-6));
        assert!(almost_equal(s.mean, 3.0, 1e-6));
        assert_eq!(s.count, 3);
        assert_eq!(s.finite_count, 3);
    }

    #[test]
    fn test_summarize_group_with_inf() {
        let s = summarize_group(&[2.0, f32::INFINITY, 4.0]);
        assert!(almost_equal(s.best, 2.0, 1e-6));
        assert!(almost_equal(s.worst, 4.0, 1e-6));
        assert!(almost_equal(s.mean, 3.0, 1e-6));
        assert_eq!(s.count, 3);
        assert_eq!(s.finite_count, 2);
    }

    #[test]
    fn test_summarize_group_all_inf() {
        let s = summarize_group(&[f32::INFINITY, f32::INFINITY]);
        assert_eq!(s.best, f32::INFINITY);
        assert_eq!(s.worst, f32::INFINITY);
        assert_eq!(s.mean, f32::INFINITY);
        assert!(almost_equal(s.mad, 0.0, 1e-6));
        assert_eq!(s.count, 2);
        assert_eq!(s.finite_count, 0);
    }

    #[test]
    fn test_group_by_key_basic() {
        let mut p1 = BTreeMap::new();
        p1.insert("k".to_string(), 1.0_f32);
        let mut p2 = BTreeMap::new();
        p2.insert("k".to_string(), 2.0_f32);
        let mut p3 = BTreeMap::new();
        p3.insert("k".to_string(), 1.0_f32);
        let results = vec![
            (0_i64, p1, BTreeMap::new()),
            (1, p2, BTreeMap::new()),
            (2, p3, BTreeMap::new()),
        ];
        let grouped = group_by_key(&results, "k");
        // key 1.0 → i64 representation, key 2.0 → i64 representation
        assert_eq!(grouped.len(), 2);
        // group with key=1.0 should have 2 entries
        let key_1 = f32_to_i64(1.0);
        assert_eq!(grouped[&key_1].len(), 2);
        let key_2 = f32_to_i64(2.0);
        assert_eq!(grouped[&key_2].len(), 1);
    }
}
