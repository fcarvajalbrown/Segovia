use std::sync::Arc;

use numpy::ndarray::Array2;
use thiserror::Error;

use crate::core::ChunkSource;
use crate::sim::{mix, Rng};

#[derive(Debug, Error)]
pub enum SimError {
    #[error("n_channels must be greater than zero")]
    NoChannels,
    #[error("duration_s must be greater than zero")]
    NoDuration,
    #[error("sample_rate must be greater than zero")]
    BadSampleRate,
    #[error("pitch must be greater than zero")]
    BadPitch,
    #[error("lsb_uv must be greater than zero")]
    BadLsb,
    #[error("firing_rate must not be negative")]
    BadFiringRate,
    #[error("noise_uv must not be negative")]
    BadNoise,
}

#[derive(Clone, Copy)]
pub struct SimConfig {
    pub n_channels: usize,
    pub duration_s: f64,
    pub sample_rate: f64,
    pub n_units: usize,
    pub firing_rate: f64,
    pub pitch: f64,
    pub noise_uv: f64,
    pub lsb_uv: f64,
    pub seed: u64,
}

struct Unit {
    amp: Vec<f32>,
    template: Vec<f64>,
    peak_channel: u32,
}

struct Event {
    sample: i64,
    unit: u32,
}

struct SimData {
    n_channels: usize,
    n_samples: usize,
    sample_rate: f64,
    noise_uv: f32,
    lsb_uv: f64,
    noise_seed: u64,
    template_len: usize,
    center: usize,
    units: Vec<Unit>,
    events: Vec<Event>,
}

pub struct SyntheticEphysReader {
    data: Arc<SimData>,
}

impl SyntheticEphysReader {
    pub fn new(config: SimConfig) -> Result<Self, SimError> {
        if config.n_channels == 0 {
            return Err(SimError::NoChannels);
        }
        if config.duration_s <= 0.0 || config.duration_s.is_nan() {
            return Err(SimError::NoDuration);
        }
        if config.sample_rate <= 0.0 || config.sample_rate.is_nan() {
            return Err(SimError::BadSampleRate);
        }
        if config.pitch <= 0.0 || config.pitch.is_nan() {
            return Err(SimError::BadPitch);
        }
        if config.lsb_uv <= 0.0 || config.lsb_uv.is_nan() {
            return Err(SimError::BadLsb);
        }
        if config.firing_rate < 0.0 {
            return Err(SimError::BadFiringRate);
        }
        if config.noise_uv < 0.0 {
            return Err(SimError::BadNoise);
        }

        let n_samples = (config.duration_s * config.sample_rate).round() as usize;
        let template_len = ((0.004 * config.sample_rate).round() as usize).max(3);
        let center = template_len / 2;
        let span = (config.n_channels.saturating_sub(1)) as f64 * config.pitch;

        let mut units = Vec::with_capacity(config.n_units);
        let mut events = Vec::new();

        for u in 0..config.n_units {
            let mut params = Rng::seed(mix(config.seed, (2 * u as u64) + 1));
            let soma_pos = params.next_f64() * span;
            let d_perp = 10.0 + params.next_f64() * 40.0;
            let amplitude = 50.0 + params.next_f64() * 450.0;
            let sigma_ms = 0.2 + params.next_f64() * 0.2;
            let sigma = sigma_ms * 1e-3 * config.sample_rate;

            let mut amp = Vec::with_capacity(config.n_channels);
            let mut peak_channel = 0u32;
            let mut peak_amp = f32::MIN;
            for c in 0..config.n_channels {
                let ch_pos = c as f64 * config.pitch;
                let dist = ch_pos - soma_pos;
                let r = (dist * dist + d_perp * d_perp).sqrt();
                let value = (amplitude * d_perp / r) as f32;
                if value > peak_amp {
                    peak_amp = value;
                    peak_channel = c as u32;
                }
                amp.push(value);
            }

            let template = (0..template_len)
                .map(|k| {
                    let tau = k as f64 - center as f64;
                    let x = tau / sigma;
                    -(1.0 - x * x) * (-0.5 * x * x).exp()
                })
                .collect();

            units.push(Unit {
                amp,
                template,
                peak_channel,
            });

            if config.firing_rate > 0.0 {
                let mut train = Rng::seed(mix(config.seed, 2 * u as u64));
                let mut t = 0.0f64;
                loop {
                    let p = train.next_f64().max(f64::MIN_POSITIVE);
                    t += -p.ln() / config.firing_rate;
                    let sample = (t * config.sample_rate).round() as i64;
                    if sample < 0 {
                        continue;
                    }
                    if sample as usize >= n_samples {
                        break;
                    }
                    events.push(Event {
                        sample,
                        unit: u as u32,
                    });
                }
            }
        }

        events.sort_by_key(|e| e.sample);

        let data = SimData {
            n_channels: config.n_channels,
            n_samples,
            sample_rate: config.sample_rate,
            noise_uv: config.noise_uv as f32,
            lsb_uv: config.lsb_uv,
            noise_seed: mix(config.seed, 0xA5A5_5A5A_F00D_BEEF),
            template_len,
            center,
            units,
            events,
        };

        Ok(Self {
            data: Arc::new(data),
        })
    }

    pub fn ground_truth(&self) -> (Vec<i64>, Vec<i32>, Vec<i32>) {
        let n = self.data.events.len();
        let mut samples = Vec::with_capacity(n);
        let mut unit_ids = Vec::with_capacity(n);
        let mut peak_channels = Vec::with_capacity(n);
        for e in &self.data.events {
            samples.push(e.sample);
            unit_ids.push(e.unit as i32);
            peak_channels.push(self.data.units[e.unit as usize].peak_channel as i32);
        }
        (samples, unit_ids, peak_channels)
    }
}

impl SimData {
    fn synth_chunk(&self, start: usize, rows: usize) -> Array2<i16> {
        let mut buf = Array2::<f32>::zeros((rows, self.n_channels));

        if self.noise_uv > 0.0 {
            for r in 0..rows {
                let s_abs = (start + r) as u64;
                let mut rng = Rng::seed(mix(self.noise_seed, s_abs));
                for c in 0..self.n_channels {
                    buf[[r, c]] = self.noise_uv * rng.next_gaussian() as f32;
                }
            }
        }

        let start_i = start as i64;
        let end_i = (start + rows) as i64;
        let tlen = self.template_len as i64;
        let center = self.center as i64;
        let lo = self.events.partition_point(|e| e.sample < start_i - tlen);
        let hi = self.events.partition_point(|e| e.sample < end_i + tlen);

        for event in &self.events[lo..hi] {
            let unit = &self.units[event.unit as usize];
            let t0 = event.sample - center;
            for k in 0..self.template_len {
                let s = t0 + k as i64;
                if !(start_i..end_i).contains(&s) {
                    continue;
                }
                let r = (s - start_i) as usize;
                let shape = unit.template[k] as f32;
                for c in 0..self.n_channels {
                    buf[[r, c]] += shape * unit.amp[c];
                }
            }
        }

        let scale = 1.0 / self.lsb_uv;
        buf.mapv(|v| {
            let q = (v as f64 * scale).round();
            q.clamp(i16::MIN as f64, i16::MAX as f64) as i16
        })
    }
}

impl ChunkSource for SyntheticEphysReader {
    type Chunks = SimChunkIter;

    fn n_channels(&self) -> usize {
        self.data.n_channels
    }

    fn n_samples(&self) -> usize {
        self.data.n_samples
    }

    fn sample_rate(&self) -> f64 {
        self.data.sample_rate
    }

    fn chunks(&self, chunk_samples: usize) -> SimChunkIter {
        SimChunkIter {
            data: Arc::clone(&self.data),
            chunk_samples,
            pos: 0,
        }
    }
}

pub struct SimChunkIter {
    data: Arc<SimData>,
    chunk_samples: usize,
    pos: usize,
}

impl Iterator for SimChunkIter {
    type Item = Array2<i16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.n_samples {
            return None;
        }
        let rows = self.chunk_samples.min(self.data.n_samples - self.pos);
        let chunk = self.data.synth_chunk(self.pos, rows);
        self.pos += rows;
        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> SimConfig {
        SimConfig {
            n_channels: 16,
            duration_s: 0.5,
            sample_rate: 30000.0,
            n_units: 4,
            firing_rate: 20.0,
            pitch: 20.0,
            noise_uv: 5.0,
            lsb_uv: 2.34,
            seed: 42,
        }
    }

    #[test]
    fn shape_matches_config() {
        let reader = SyntheticEphysReader::new(config()).unwrap();
        assert_eq!(reader.n_channels(), 16);
        assert_eq!(reader.n_samples(), 15000);
        assert_eq!(reader.sample_rate(), 30000.0);
    }

    #[test]
    fn chunks_reconstruct_full_length() {
        let reader = SyntheticEphysReader::new(config()).unwrap();
        let total: usize = reader.chunks(4096).map(|c| c.nrows()).sum();
        assert_eq!(total, 15000);
        assert!(reader.chunks(4096).all(|c| c.ncols() == 16));
    }

    #[test]
    fn output_is_chunk_size_independent() {
        let reader = SyntheticEphysReader::new(config()).unwrap();
        let big = stack(reader.chunks(15000));
        let small = stack(reader.chunks(1000));
        assert_eq!(big, small);
    }

    #[test]
    fn same_seed_is_deterministic() {
        let a = stack(SyntheticEphysReader::new(config()).unwrap().chunks(4096));
        let b = stack(SyntheticEphysReader::new(config()).unwrap().chunks(4096));
        assert_eq!(a, b);
    }

    #[test]
    fn different_seed_differs() {
        let mut other = config();
        other.seed = 7;
        let a = stack(SyntheticEphysReader::new(config()).unwrap().chunks(4096));
        let b = stack(SyntheticEphysReader::new(other).unwrap().chunks(4096));
        assert_ne!(a, b);
    }

    #[test]
    fn ground_truth_samples_are_sorted_and_in_range() {
        let reader = SyntheticEphysReader::new(config()).unwrap();
        let (samples, units, peaks) = reader.ground_truth();
        assert!(!samples.is_empty());
        assert_eq!(samples.len(), units.len());
        assert_eq!(samples.len(), peaks.len());
        assert!(samples.windows(2).all(|w| w[0] <= w[1]));
        assert!(samples.iter().all(|&s| s >= 0 && (s as usize) < 15000));
        assert!(peaks.iter().all(|&p| (0..16).contains(&p)));
    }

    #[test]
    fn noiseless_signal_is_strongest_on_peak_channel() {
        let cfg = SimConfig {
            n_units: 1,
            firing_rate: 50.0,
            noise_uv: 0.0,
            ..config()
        };
        let reader = SyntheticEphysReader::new(cfg).unwrap();
        let (_, _, peaks) = reader.ground_truth();
        let peak = peaks[0] as usize;
        let signal = stack(reader.chunks(4096));
        let mut energy = [0.0f64; 16];
        for r in 0..signal.nrows() {
            for c in 0..16 {
                let v = signal[[r, c]] as f64;
                energy[c] += v * v;
            }
        }
        let argmax = energy
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(argmax, peak);
    }

    fn stack(chunks: impl Iterator<Item = Array2<i16>>) -> Array2<i16> {
        let collected: Vec<Array2<i16>> = chunks.collect();
        let n_cols = collected[0].ncols();
        let n_rows: usize = collected.iter().map(|c| c.nrows()).sum();
        let mut out = Array2::<i16>::zeros((n_rows, n_cols));
        let mut row = 0;
        for chunk in collected {
            for r in 0..chunk.nrows() {
                for c in 0..n_cols {
                    out[[row + r, c]] = chunk[[r, c]];
                }
            }
            row += chunk.nrows();
        }
        out
    }
}
