use std::sync::Arc;

use numpy::ndarray::Array2;
use thiserror::Error;

use crate::core::ChunkSource;
use crate::sim::{mix, Rng};

#[derive(Debug, Error)]
pub enum IfcError {
    #[error("n_channels must be greater than zero")]
    NoChannels,
    #[error("duration_s must be greater than zero")]
    NoDuration,
    #[error("sample_rate must be greater than zero")]
    BadSampleRate,
    #[error("n_populations must be greater than zero")]
    NoPopulations,
    #[error("event_rate must not be negative")]
    BadEventRate,
    #[error("noise_level must not be negative")]
    BadNoise,
    #[error("lsb must be greater than zero")]
    BadLsb,
}

#[derive(Clone, Copy)]
pub struct IfcConfig {
    pub n_channels: usize,
    pub duration_s: f64,
    pub sample_rate: f64,
    pub n_populations: usize,
    pub event_rate: f64,
    pub noise_level: f64,
    pub lsb: f64,
    pub seed: u64,
}

struct Population {
    amplitude: f64,
    gains: Vec<f32>,
    template: Vec<f64>,
}

struct Event {
    sample: i64,
    population: u32,
    amp_scale: f32,
}

struct IfcData {
    n_channels: usize,
    n_samples: usize,
    sample_rate: f64,
    noise_level: f32,
    lsb: f64,
    noise_seed: u64,
    template_len: usize,
    center: usize,
    populations: Vec<Population>,
    events: Vec<Event>,
}

pub struct SyntheticIfcReader {
    data: Arc<IfcData>,
}

impl SyntheticIfcReader {
    pub fn new(config: IfcConfig) -> Result<Self, IfcError> {
        if config.n_channels == 0 {
            return Err(IfcError::NoChannels);
        }
        if config.duration_s <= 0.0 || config.duration_s.is_nan() {
            return Err(IfcError::NoDuration);
        }
        if config.sample_rate <= 0.0 || config.sample_rate.is_nan() {
            return Err(IfcError::BadSampleRate);
        }
        if config.n_populations == 0 {
            return Err(IfcError::NoPopulations);
        }
        if config.event_rate < 0.0 {
            return Err(IfcError::BadEventRate);
        }
        if config.noise_level < 0.0 {
            return Err(IfcError::BadNoise);
        }
        if config.lsb <= 0.0 || config.lsb.is_nan() {
            return Err(IfcError::BadLsb);
        }

        let n_samples = (config.duration_s * config.sample_rate).round() as usize;
        let template_len = ((0.0015 * config.sample_rate).round() as usize).max(3);
        let center = template_len / 2;

        let mut populations = Vec::with_capacity(config.n_populations);
        for p in 0..config.n_populations {
            let mut rng = Rng::seed(mix(config.seed, mix(0x1FC0_0001, p as u64)));
            let amplitude = 0.2 + rng.next_f64() * 1.3;
            let sigma_us = 30.0 + rng.next_f64() * 50.0;
            let separation_us = 150.0 + rng.next_f64() * 200.0;
            let sigma = (sigma_us * 1e-6 * config.sample_rate).max(1.0);
            let separation = separation_us * 1e-6 * config.sample_rate;

            let mut gains = Vec::with_capacity(config.n_channels);
            for c in 0..config.n_channels {
                if c == 0 {
                    gains.push(1.0);
                } else {
                    gains.push((0.6 + rng.next_f64() * 0.4) as f32);
                }
            }

            let c1 = center as f64 - separation * 0.5;
            let c2 = center as f64 + separation * 0.5;
            let template = (0..template_len)
                .map(|k| {
                    let t = k as f64;
                    let a = (-0.5 * ((t - c1) / sigma).powi(2)).exp();
                    let b = (-0.5 * ((t - c2) / sigma).powi(2)).exp();
                    a - b
                })
                .collect();

            populations.push(Population {
                amplitude,
                gains,
                template,
            });
        }

        let mut events = Vec::new();
        if config.event_rate > 0.0 {
            let mut arrivals = Rng::seed(mix(config.seed, 0x1FC0_A551));
            let mut t = 0.0f64;
            let mut idx = 0u64;
            loop {
                let p = arrivals.next_f64().max(f64::MIN_POSITIVE);
                t += -p.ln() / config.event_rate;
                let sample = (t * config.sample_rate).round() as i64;
                if sample < 0 {
                    idx += 1;
                    continue;
                }
                if sample as usize >= n_samples {
                    break;
                }
                let mut ev = Rng::seed(mix(config.seed, mix(0x1FC0_E7E7, idx)));
                let population = (ev.next_f64() * config.n_populations as f64) as usize;
                let population = population.min(config.n_populations - 1);
                let amp_scale = (1.0 + 0.1 * ev.next_gaussian()).max(0.05) as f32;
                events.push(Event {
                    sample,
                    population: population as u32,
                    amp_scale,
                });
                idx += 1;
            }
        }

        events.sort_by_key(|e| e.sample);

        let data = IfcData {
            n_channels: config.n_channels,
            n_samples,
            sample_rate: config.sample_rate,
            noise_level: config.noise_level as f32,
            lsb: config.lsb,
            noise_seed: mix(config.seed, 0x1FC0_0757_D0D0_5EED),
            template_len,
            center,
            populations,
            events,
        };

        Ok(Self {
            data: Arc::new(data),
        })
    }

    pub fn ground_truth(&self) -> (Vec<i64>, Vec<i32>, Vec<i32>) {
        let n = self.data.events.len();
        let mut samples = Vec::with_capacity(n);
        let mut population_ids = Vec::with_capacity(n);
        let mut amplitudes = Vec::with_capacity(n);
        let scale = 1.0 / self.data.lsb;
        for e in &self.data.events {
            samples.push(e.sample);
            population_ids.push(e.population as i32);
            let peak =
                self.data.populations[e.population as usize].amplitude * e.amp_scale as f64 * scale;
            amplitudes.push(peak.round() as i32);
        }
        (samples, population_ids, amplitudes)
    }
}

impl IfcData {
    fn synth_chunk(&self, start: usize, rows: usize) -> Array2<i16> {
        let mut buf = Array2::<f32>::zeros((rows, self.n_channels));

        if self.noise_level > 0.0 {
            for r in 0..rows {
                let s_abs = (start + r) as u64;
                let mut rng = Rng::seed(mix(self.noise_seed, s_abs));
                for c in 0..self.n_channels {
                    buf[[r, c]] = self.noise_level * rng.next_gaussian() as f32;
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
            let pop = &self.populations[event.population as usize];
            let t0 = event.sample - center;
            for k in 0..self.template_len {
                let s = t0 + k as i64;
                if !(start_i..end_i).contains(&s) {
                    continue;
                }
                let r = (s - start_i) as usize;
                let shape = (pop.template[k] * pop.amplitude) as f32 * event.amp_scale;
                for c in 0..self.n_channels {
                    buf[[r, c]] += shape * pop.gains[c];
                }
            }
        }

        let scale = 1.0 / self.lsb;
        buf.mapv(|v| {
            let q = (v as f64 * scale).round();
            q.clamp(i16::MIN as f64, i16::MAX as f64) as i16
        })
    }
}

impl ChunkSource for SyntheticIfcReader {
    type Chunks = IfcChunkIter;

    fn n_channels(&self) -> usize {
        self.data.n_channels
    }

    fn n_samples(&self) -> usize {
        self.data.n_samples
    }

    fn sample_rate(&self) -> f64 {
        self.data.sample_rate
    }

    fn chunks(&self, chunk_samples: usize) -> IfcChunkIter {
        IfcChunkIter {
            data: Arc::clone(&self.data),
            chunk_samples,
            pos: 0,
        }
    }
}

pub struct IfcChunkIter {
    data: Arc<IfcData>,
    chunk_samples: usize,
    pos: usize,
}

impl Iterator for IfcChunkIter {
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

    fn config() -> IfcConfig {
        IfcConfig {
            n_channels: 2,
            duration_s: 0.5,
            sample_rate: 100000.0,
            n_populations: 3,
            event_rate: 200.0,
            noise_level: 0.01,
            lsb: 1.0e-4,
            seed: 42,
        }
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

    #[test]
    fn shape_matches_config() {
        let reader = SyntheticIfcReader::new(config()).unwrap();
        assert_eq!(reader.n_channels(), 2);
        assert_eq!(reader.n_samples(), 50000);
        assert_eq!(reader.sample_rate(), 100000.0);
    }

    #[test]
    fn chunks_reconstruct_full_length() {
        let reader = SyntheticIfcReader::new(config()).unwrap();
        let total: usize = reader.chunks(4096).map(|c| c.nrows()).sum();
        assert_eq!(total, 50000);
        assert!(reader.chunks(4096).all(|c| c.ncols() == 2));
    }

    #[test]
    fn output_is_chunk_size_independent() {
        let reader = SyntheticIfcReader::new(config()).unwrap();
        let big = stack(reader.chunks(50000));
        let small = stack(reader.chunks(1000));
        assert_eq!(big, small);
    }

    #[test]
    fn same_seed_is_deterministic() {
        let a = stack(SyntheticIfcReader::new(config()).unwrap().chunks(4096));
        let b = stack(SyntheticIfcReader::new(config()).unwrap().chunks(4096));
        assert_eq!(a, b);
    }

    #[test]
    fn different_seed_differs() {
        let mut other = config();
        other.seed = 7;
        let a = stack(SyntheticIfcReader::new(config()).unwrap().chunks(4096));
        let b = stack(SyntheticIfcReader::new(other).unwrap().chunks(4096));
        assert_ne!(a, b);
    }

    #[test]
    fn ground_truth_is_sorted_and_in_range() {
        let reader = SyntheticIfcReader::new(config()).unwrap();
        let (samples, pops, amps) = reader.ground_truth();
        assert!(!samples.is_empty());
        assert_eq!(samples.len(), pops.len());
        assert_eq!(samples.len(), amps.len());
        assert!(samples.windows(2).all(|w| w[0] <= w[1]));
        assert!(samples.iter().all(|&s| s >= 0 && (s as usize) < 50000));
        assert!(pops.iter().all(|&p| (0..3).contains(&p)));
    }

    #[test]
    fn pulse_is_bipolar() {
        let cfg = IfcConfig {
            n_populations: 1,
            event_rate: 20.0,
            noise_level: 0.0,
            ..config()
        };
        let reader = SyntheticIfcReader::new(cfg).unwrap();
        let signal = stack(reader.chunks(4096));
        let mut has_pos = false;
        let mut has_neg = false;
        for r in 0..signal.nrows() {
            let v = signal[[r, 0]];
            if v > 1000 {
                has_pos = true;
            }
            if v < -1000 {
                has_neg = true;
            }
        }
        assert!(has_pos && has_neg);
    }
}
