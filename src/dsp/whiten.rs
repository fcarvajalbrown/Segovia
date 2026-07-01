use nalgebra::{DMatrix, SymmetricEigen};
use numpy::ndarray::{Array1, Array2, Axis};

pub struct Whitener {
    w: Array2<f32>,
    mean: Option<Array1<f32>>,
}

impl Whitener {
    pub fn estimate(calib: &Array2<f32>, eps: f64, apply_mean: bool) -> Self {
        let n = calib.nrows();
        let c = calib.ncols();

        let calib64 = calib.mapv(|v| v as f64);
        let mean64 = if apply_mean {
            Some(calib64.mean_axis(Axis(0)).expect("non-empty calibration"))
        } else {
            None
        };

        let mut centered = calib64.clone();
        if let Some(m) = &mean64 {
            for mut row in centered.rows_mut() {
                row -= m;
            }
        }

        let cov = centered.t().dot(&centered) / (n as f64);

        let mut dm = DMatrix::<f64>::zeros(c, c);
        for i in 0..c {
            for j in 0..c {
                dm[(i, j)] = cov[[i, j]];
            }
        }

        let eig = SymmetricEigen::new(dm);
        let vals = &eig.eigenvalues;
        let vecs = &eig.eigenvectors;

        let mut d = DMatrix::<f64>::zeros(c, c);
        for i in 0..c {
            d[(i, i)] = 1.0 / (vals[i] + eps).sqrt();
        }
        let w_dm = vecs * d * vecs.transpose();

        let mut w = Array2::<f32>::zeros((c, c));
        for i in 0..c {
            for j in 0..c {
                w[[i, j]] = w_dm[(i, j)] as f32;
            }
        }
        let mean = mean64.map(|m| m.mapv(|v| v as f32));

        Self { w, mean }
    }

    pub fn apply(&self, chunk: &Array2<f32>) -> Array2<f32> {
        let s = chunk.nrows();
        let c = chunk.ncols();

        let mut centered = chunk.to_owned();
        if let Some(m) = &self.mean {
            for mut row in centered.rows_mut() {
                row -= m;
            }
        }

        let mut out = Array2::<f32>::zeros((s, c));
        let lhs = centered
            .as_slice()
            .expect("centered is row-major contiguous");
        let rhs = self.w.as_slice().expect("w is row-major contiguous");
        let dst = out.as_slice_mut().expect("out is row-major contiguous");
        let cs = c as isize;
        unsafe {
            gemm::gemm(
                s,
                c,
                c,
                dst.as_mut_ptr(),
                1,
                cs,
                false,
                lhs.as_ptr(),
                1,
                cs,
                rhs.as_ptr(),
                1,
                cs,
                0.0f32,
                1.0f32,
                false,
                false,
                false,
                gemm::Parallelism::None,
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::ndarray::Array2;

    #[test]
    fn whitened_data_has_identity_covariance() {
        let n = 4000;
        let c = 3;
        let mut data = Array2::<f32>::zeros((n, c));
        let mut s: u64 = 1;
        let mut rng = || {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((s >> 33) as f64) / (1u64 << 31) as f64 - 1.0
        };
        for i in 0..n {
            let a = rng();
            let b = rng();
            let d = rng();
            data[[i, 0]] = (2.0 * a) as f32;
            data[[i, 1]] = (a + 0.5 * b) as f32;
            data[[i, 2]] = (a - b + 0.3 * d) as f32;
        }

        let w = Whitener::estimate(&data, 0.0, true);
        let out = w.apply(&data).mapv(|v| v as f64);

        let mean = out.mean_axis(Axis(0)).unwrap();
        let mut centered = out.clone();
        for mut row in centered.rows_mut() {
            row -= &mean;
        }
        let cov = centered.t().dot(&centered) / (n as f64);
        for i in 0..c {
            for j in 0..c {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (cov[[i, j]] - expected).abs() < 1e-4,
                    "cov[{i},{j}] = {} expected {expected}",
                    cov[[i, j]]
                );
            }
        }
    }
}
