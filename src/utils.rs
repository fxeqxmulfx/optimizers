use std::collections::HashMap;

pub fn clamp_to_unit_cube(value: f32) -> f32 {
    if value > 1.0 {
        return 1.0;
    }
    if value < 0.0 {
        return 0.0;
    }
    return value;
}

pub fn fit_in_bounds(values: &Vec<f32>, range_min: &Vec<f32>, range_max: &Vec<f32>) -> Vec<f32> {
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
    return move |x: &[f32]| -> f32 {
        let mut i = 1;
        let x_len = x.len();
        if x_len == 0 {
            return f32::NAN;
        }
        let mut result = 0.0;
        while i < x_len {
            result += func(x[i - 1], x[i]);
            i += 2;
        }
        return result / x_len as f32 * 2.0;
    };
}

pub fn all_combinations(data: &HashMap<String, Vec<f64>>) -> Vec<HashMap<String, f64>> {
    let keys: Vec<String> = data.keys().cloned().collect();
    let values: Vec<Vec<f64>> = data.values().cloned().collect();
    let mut result = Vec::new();
    fn rec(
        keys: &[String],
        values: &[Vec<f64>],
        depth: usize,
        current: &mut Vec<f64>,
        out: &mut Vec<HashMap<String, f64>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec_map(single: &HashMap<String, f64>) -> HashMap<String, Vec<f64>> {
        single.iter().map(|(k, v)| (k.clone(), vec![*v])).collect()
    }

    fn map_from_vecs(pairs: &[(&str, &[f64])]) -> HashMap<String, Vec<f64>> {
        let mut m = HashMap::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), (*v).to_vec());
        }
        m
    }

    fn map_from_pairs(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        let mut m = HashMap::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), *v);
        }
        m
    }

    #[test]
    fn test_two_keys_two_values() {
        let input = map_from_vecs(&[("a", &[1.0, 2.0]), ("b", &[1.0, 2.0])]);
        let actual = all_combinations(&input);

        let expected = vec![
            map_from_pairs(&[("a", 1.0), ("b", 1.0)]),
            map_from_pairs(&[("a", 1.0), ("b", 2.0)]),
            map_from_pairs(&[("a", 2.0), ("b", 1.0)]),
            map_from_pairs(&[("a", 2.0), ("b", 2.0)]),
        ];

        assert_eq!(actual.len(), expected.len());
        for exp in expected {
            assert!(actual.contains(&exp), "Missing combo: {:?}", exp);
        }
    }

    #[test]
    fn test_single_key() {
        let input = map_from_vecs(&[("x", &[5.5])]);
        let actual = all_combinations(&input);
        let expected = vec![map_from_pairs(&[("x", 5.5)])];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_no_keys() {
        let input: HashMap<String, Vec<f64>> = HashMap::new();
        let actual = all_combinations(&input);
        assert_eq!(actual.len(), 1);
        assert!(actual[0].is_empty());
    }

    #[test]
    fn test_empty_value_vector() {
        let input = map_from_vecs(&[("a", &[])]);
        let actual = all_combinations(&input);
        assert!(
            actual.is_empty(),
            "Expected 0 combinations, got {:#?}",
            actual
        );
    }

    #[test]
    fn test_duplicate_values() {
        let input = map_from_vecs(&[("a", &[1.0, 1.0]), ("b", &[2.0])]);
        let actual = all_combinations(&input);
        let expected = vec![
            map_from_pairs(&[("a", 1.0), ("b", 2.0)]),
            map_from_pairs(&[("a", 1.0), ("b", 2.0)]),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_to_vec_map_conversion() {
        let single: HashMap<String, f64> = map_from_pairs(&[("k1", 42.0), ("k2", 3.14)]);
        let vec_map = to_vec_map(&single);
        let actual = all_combinations(&vec_map);
        let expected = vec![map_from_pairs(&[("k1", 42.0), ("k2", 3.14)])];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_original_immutability() {
        let input = map_from_vecs(&[("a", &[1.0, 2.0]), ("b", &[3.0])]);
        let original_clone = input.clone();
        let _ = all_combinations(&input);
        assert_eq!(input, original_clone, "Input was mutated");
    }

    #[test]
    fn test_clamp_to_unit_cube() {
        assert_eq!(clamp_to_unit_cube(0.5), 0.5);
        assert_eq!(clamp_to_unit_cube(-0.2), 0.0);
        assert_eq!(clamp_to_unit_cube(1.2), 1.0);
        assert_eq!(clamp_to_unit_cube(0.0), 0.0);
        assert_eq!(clamp_to_unit_cube(1.0), 1.0);
    }

    #[test]
    fn test_fit_in_bounds_basic() {
        let values = vec![0.0, 0.5, 1.0];
        let min = vec![10.0, 20.0, 30.0];
        let max = vec![20.0, 30.0, 40.0];
        let expected = vec![10.0, 25.0, 40.0];
        assert_eq!(fit_in_bounds(&values, &min, &max), expected);
    }

    #[test]
    fn test_fit_in_bounds_zero_range() {
        let values = vec![0.0, 0.3, 0.7];
        let min = vec![5.0, 5.0, 5.0];
        let max = vec![5.0, 5.0, 5.0];
        let expected = vec![5.0; 3];
        assert_eq!(fit_in_bounds(&values, &min, &max), expected);
    }

    #[test]
    fn test_format_x_history_single_run() {
        let values = vec![vec![vec![0.0, 1.0]]];
        let bounds = vec![[0.0, 1.0], [1.0, 2.0]];
        let result = format_x_history(&values, &bounds);
        let expected = vec![vec![[0.0, 2.0]]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_x_history_multiple_runs() {
        let values = vec![vec![vec![0.0, 0.5, 1.0], vec![0.2, 0.8, 0.6]]];
        let bounds = vec![[0.0, 1.0], [1.0, 2.0], [2.0, 3.0]];
        let result = format_x_history(&values, &bounds);
        let expected = vec![vec![[0.0, 1.5], [1.5, 3.0], [0.2, 1.8], [1.8, 2.6]]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_best_f_x_history_basic() {
        let values = vec![vec![1.0, 2.0, 3.0], vec![0.5, 1.5]];
        let result = format_best_f_x_history(&values);
        assert_eq!(result, vec![1.0, 0.5]);
    }

    #[test]
    fn test_format_best_f_x_history_empty_inner() {
        let values = vec![vec![], vec![2.0, -1.0]];
        let result = format_best_f_x_history(&values);
        assert_eq!(result, vec![-1.0]);
    }

    #[test]
    fn test_broadcast_empty_slice() {
        let f = broadcast(|_a, _b| 1.0);
        let result = f(&[]);
        assert!(f32::is_nan(result));
    }

    #[test]
    fn test_broadcast_single_element() {
        let f = broadcast(|a, b| a + b);
        let result = f(&[3.0]);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_broadcast_even_length() {
        let f = broadcast(|a, b| a + b);
        let result = f(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_broadcast_odd_length() {
        let f = broadcast(|a, b| a + b);
        let result = f(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_broadcast_custom_func() {
        let f = broadcast(|a, b| (b - a).abs());
        let result = f(&[0.0, 3.0, 3.0, 0.0]);
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_all_combinations_product() {
        let input = map_from_vecs(&[("x", &[1.0, 2.0, 3.0]), ("y", &[4.0, 5.0]), ("z", &[6.0])]);
        let combos = all_combinations(&input);
        assert_eq!(combos.len(), 6);
    }
}
