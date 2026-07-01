pub type Section = [f64; 6];

pub fn default_padlen(sos: &[Section]) -> usize {
    let n_sections = sos.len();
    let b2_zeros = sos.iter().filter(|s| s[2] == 0.0).count();
    let a2_zeros = sos.iter().filter(|s| s[5] == 0.0).count();
    let ntaps = 2 * n_sections + 1 - b2_zeros.min(a2_zeros);
    3 * ntaps
}

pub fn sosfilt_zi(sos: &[Section]) -> Vec<[f64; 2]> {
    let mut zi = Vec::with_capacity(sos.len());
    let mut scale = 1.0f64;
    for s in sos {
        let (b0, b1, b2) = (s[0], s[1], s[2]);
        let (a1, a2) = (s[4], s[5]);

        let m00 = 1.0 + a1;
        let m01 = -1.0;
        let m10 = a2;
        let m11 = 1.0;
        let rhs0 = b1 - a1 * b0;
        let rhs1 = b2 - a2 * b0;
        let det = m00 * m11 - m01 * m10;
        let z0 = (rhs0 * m11 - m01 * rhs1) / det;
        let z1 = (m00 * rhs1 - m10 * rhs0) / det;

        zi.push([scale * z0, scale * z1]);

        let bsum = b0 + b1 + b2;
        let asum = 1.0 + a1 + a2;
        scale *= bsum / asum;
    }
    zi
}

pub fn sosfilt_into(sos: &[Section], x: &[f64], zi: &[[f64; 2]], out: &mut Vec<f64>) {
    out.clear();
    out.reserve(x.len());
    let mut state: Vec<[f64; 2]> = zi.to_vec();
    for &x0 in x {
        let mut xn = x0;
        for (sec, st) in sos.iter().zip(state.iter_mut()) {
            let (b0, b1, b2) = (sec[0], sec[1], sec[2]);
            let (a1, a2) = (sec[4], sec[5]);
            let yn = b0 * xn + st[0];
            st[0] = b1 * xn - a1 * yn + st[1];
            st[1] = b2 * xn - a2 * yn;
            xn = yn;
        }
        out.push(xn);
    }
}

fn odd_ext(x: &[f64], n: usize) -> Vec<f64> {
    let len = x.len();
    let mut out = Vec::with_capacity(len + 2 * n);
    let first = x[0];
    let last = x[len - 1];
    for i in (1..=n).rev() {
        out.push(2.0 * first - x[i]);
    }
    out.extend_from_slice(x);
    for i in 1..=n {
        out.push(2.0 * last - x[len - 1 - i]);
    }
    out
}

pub fn sosfiltfilt(sos: &[Section], x: &[f64], padlen: usize) -> Vec<f64> {
    let edge = padlen;
    let ext = odd_ext(x, edge);
    let zi = sosfilt_zi(sos);

    let x0 = ext[0];
    let zi_fwd: Vec<[f64; 2]> = zi.iter().map(|z| [z[0] * x0, z[1] * x0]).collect();
    let mut forward = Vec::new();
    sosfilt_into(sos, &ext, &zi_fwd, &mut forward);

    forward.reverse();
    let y0 = forward[0];
    let zi_bwd: Vec<[f64; 2]> = zi.iter().map(|z| [z[0] * y0, z[1] * y0]).collect();
    let mut backward = Vec::new();
    sosfilt_into(sos, &forward, &zi_bwd, &mut backward);

    backward.reverse();
    backward[edge..backward.len() - edge].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lowpass_sos() -> Vec<Section> {
        vec![[
            0.020_083_365_564_211_2,
            0.040_166_731_128_422_4,
            0.020_083_365_564_211_2,
            1.0,
            -1.561_018_075_800_718,
            0.641_351_538_057_563,
        ]]
    }

    #[test]
    fn sosfilt_zi_gives_unit_dc_steady_state() {
        let sos = lowpass_sos();
        let zi = sosfilt_zi(&sos);
        let x = vec![1.0f64; 200];
        let zi_scaled: Vec<[f64; 2]> = zi.iter().map(|z| [z[0] * x[0], z[1] * x[0]]).collect();
        let mut y = Vec::new();
        sosfilt_into(&sos, &x, &zi_scaled, &mut y);
        for v in &y {
            assert!((v - 1.0).abs() < 1e-9, "steady state not held: {v}");
        }
    }

    #[test]
    fn sosfiltfilt_is_zero_phase_on_symmetric_input() {
        let sos = lowpass_sos();
        let n = 201;
        let x: Vec<f64> = (0..n)
            .map(|i| {
                let d = i as f64 - (n as f64 - 1.0) / 2.0;
                (-d * d / 200.0).exp()
            })
            .collect();
        let y = sosfiltfilt(&sos, &x, default_padlen(&sos));
        assert_eq!(y.len(), x.len());
        for i in 0..n / 2 {
            assert!(
                (y[i] - y[n - 1 - i]).abs() < 1e-9,
                "not symmetric at {i}: {} vs {}",
                y[i],
                y[n - 1 - i]
            );
        }
    }

    #[test]
    fn sosfiltfilt_preserves_dc_level() {
        let sos = lowpass_sos();
        let x = vec![3.5f64; 500];
        let y = sosfiltfilt(&sos, &x, default_padlen(&sos));
        for v in &y {
            assert!((v - 3.5).abs() < 1e-6, "dc not preserved: {v}");
        }
    }
}
