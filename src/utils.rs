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
