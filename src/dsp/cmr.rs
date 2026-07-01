use numpy::ndarray::Array2;

fn median_of(buf: &mut [f32]) -> f32 {
    let n = buf.len();
    let mid = n / 2;
    buf.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap());
    if n % 2 == 1 {
        buf[mid]
    } else {
        let upper = buf[mid];
        let lower = buf[..mid].iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (lower + upper) / 2.0
    }
}

pub fn common_median_reference(chunk: &mut Array2<f32>, scratch: &mut Vec<f32>) {
    for mut row in chunk.rows_mut() {
        scratch.clear();
        scratch.extend(row.iter().copied());
        let med = median_of(scratch);
        for v in row.iter_mut() {
            *v -= med;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::ndarray::array;

    #[test]
    fn subtracts_per_sample_median_odd_channels() {
        let mut chunk = array![[1.0f32, 2.0, 3.0], [10.0, 40.0, 70.0]];
        let mut scratch = Vec::new();
        common_median_reference(&mut chunk, &mut scratch);
        assert_eq!(chunk, array![[-1.0f32, 0.0, 1.0], [-30.0, 0.0, 30.0]]);
    }

    #[test]
    fn even_channels_average_two_middles() {
        let mut chunk = array![[1.0f32, 2.0, 3.0, 4.0]];
        let mut scratch = Vec::new();
        common_median_reference(&mut chunk, &mut scratch);
        assert_eq!(chunk, array![[-1.5f32, -0.5, 0.5, 1.5]]);
    }
}
