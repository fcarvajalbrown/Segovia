use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use flate2::read::ZlibDecoder;
use numpy::ndarray::Array2;
use serde::Deserialize;
use thiserror::Error;

use crate::core::ChunkSource;

#[derive(Debug, Error)]
pub enum CbinError {
    #[error("failed to read cbin/ch file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse ch header: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported compression algorithm '{algorithm}', only 'zlib' is supported")]
    UnsupportedAlgorithm { algorithm: String },
    #[error("cbin dtype must be int16, found '{dtype}'")]
    NotInt16 { dtype: String },
    #[error("spatial differencing (do_spatial_diff) is not supported")]
    UnsupportedSpatialDiff,
    #[error("malformed ch header: {message}")]
    BadHeader { message: String },
}

#[derive(Debug, Deserialize)]
struct CbinHeader {
    algorithm: String,
    dtype: String,
    n_channels: usize,
    sample_rate: f64,
    chunk_bounds: Vec<u64>,
    chunk_offsets: Vec<u64>,
    #[serde(default)]
    do_time_diff: bool,
    #[serde(default)]
    do_spatial_diff: bool,
    #[serde(default = "default_chunk_order")]
    chunk_order: String,
}

fn default_chunk_order() -> String {
    "C".to_string()
}

pub struct CbinReader {
    cbin: Arc<File>,
    chunk_offsets: Arc<Vec<u64>>,
    chunk_bounds: Arc<Vec<u64>>,
    n_channels: usize,
    n_samples: usize,
    sample_rate: f64,
    is_fortran: bool,
    do_time_diff: bool,
}

impl CbinReader {
    pub fn open(cbin_path: &Path, ch_path: &Path) -> Result<Self, CbinError> {
        let header_text = std::fs::read_to_string(ch_path)?;
        let header: CbinHeader = serde_json::from_str(&header_text)?;

        if header.algorithm != "zlib" {
            return Err(CbinError::UnsupportedAlgorithm {
                algorithm: header.algorithm,
            });
        }
        if header.dtype != "int16" {
            return Err(CbinError::NotInt16 {
                dtype: header.dtype,
            });
        }
        if header.do_spatial_diff {
            return Err(CbinError::UnsupportedSpatialDiff);
        }
        if header.chunk_bounds.len() != header.chunk_offsets.len() {
            return Err(CbinError::BadHeader {
                message: format!(
                    "chunk_bounds has {} entries but chunk_offsets has {}",
                    header.chunk_bounds.len(),
                    header.chunk_offsets.len()
                ),
            });
        }
        if header.chunk_bounds.len() < 2 {
            return Err(CbinError::BadHeader {
                message: "need at least one chunk (two boundary entries)".to_string(),
            });
        }
        let is_fortran = match header.chunk_order.as_str() {
            "F" => true,
            "C" => false,
            other => {
                return Err(CbinError::BadHeader {
                    message: format!("unknown chunk_order '{other}', expected 'C' or 'F'"),
                })
            }
        };

        let cbin = File::open(cbin_path)?;

        let n_samples = *header.chunk_bounds.last().unwrap() as usize;

        Ok(Self {
            cbin: Arc::new(cbin),
            chunk_offsets: Arc::new(header.chunk_offsets),
            chunk_bounds: Arc::new(header.chunk_bounds),
            n_channels: header.n_channels,
            n_samples,
            sample_rate: header.sample_rate,
            is_fortran,
            do_time_diff: header.do_time_diff,
        })
    }
}

impl ChunkSource for CbinReader {
    type Chunks = CbinChunkIter;

    fn n_channels(&self) -> usize {
        self.n_channels
    }

    fn n_samples(&self) -> usize {
        self.n_samples
    }

    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn chunks(&self, chunk_samples: usize) -> CbinChunkIter {
        CbinChunkIter {
            cbin: Arc::clone(&self.cbin),
            chunk_offsets: Arc::clone(&self.chunk_offsets),
            chunk_bounds: Arc::clone(&self.chunk_bounds),
            n_channels: self.n_channels,
            is_fortran: self.is_fortran,
            do_time_diff: self.do_time_diff,
            chunk_samples,
            next_native: 0,
            buffer: Vec::new(),
            buffered_rows: 0,
        }
    }
}

pub struct CbinChunkIter {
    cbin: Arc<File>,
    chunk_offsets: Arc<Vec<u64>>,
    chunk_bounds: Arc<Vec<u64>>,
    n_channels: usize,
    is_fortran: bool,
    do_time_diff: bool,
    chunk_samples: usize,
    next_native: usize,
    buffer: Vec<i16>,
    buffered_rows: usize,
}

impl CbinChunkIter {
    fn decode_native(&self, i: usize) -> Vec<i16> {
        let off0 = self.chunk_offsets[i];
        let off1 = self.chunk_offsets[i + 1];
        let rows = (self.chunk_bounds[i + 1] - self.chunk_bounds[i]) as usize;

        let mut compressed = vec![0u8; (off1 - off0) as usize];
        read_exact_at(&self.cbin, &mut compressed, off0).expect("read cbin chunk bytes");

        let mut decoded = Vec::with_capacity(rows * self.n_channels * 2);
        ZlibDecoder::new(&compressed[..])
            .read_to_end(&mut decoded)
            .expect("zlib decompress cbin chunk");

        let delta: &[i16] = bytemuck::cast_slice(&decoded);
        reconstruct(
            delta,
            rows,
            self.n_channels,
            self.is_fortran,
            self.do_time_diff,
        )
    }
}

impl Iterator for CbinChunkIter {
    type Item = Array2<i16>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffered_rows < self.chunk_samples
            && self.next_native + 1 < self.chunk_bounds.len()
        {
            let rows = (self.chunk_bounds[self.next_native + 1]
                - self.chunk_bounds[self.next_native]) as usize;
            let reconstructed = self.decode_native(self.next_native);
            self.buffer.extend_from_slice(&reconstructed);
            self.buffered_rows += rows;
            self.next_native += 1;
        }

        if self.buffered_rows == 0 {
            return None;
        }

        let rows_out = self.chunk_samples.min(self.buffered_rows);
        let take = rows_out * self.n_channels;
        let emitted: Vec<i16> = self.buffer.drain(0..take).collect();
        self.buffered_rows -= rows_out;

        Some(
            Array2::from_shape_vec((rows_out, self.n_channels), emitted)
                .expect("chunk length matches shape"),
        )
    }
}

fn reconstruct(
    delta: &[i16],
    rows: usize,
    n_channels: usize,
    is_fortran: bool,
    do_time_diff: bool,
) -> Vec<i16> {
    let mut out = vec![0i16; rows * n_channels];
    for c in 0..n_channels {
        let mut acc = 0i16;
        for r in 0..rows {
            let stored = if is_fortran {
                delta[c * rows + r]
            } else {
                delta[r * n_channels + c]
            };
            let value = if do_time_diff {
                acc = acc.wrapping_add(stored);
                acc
            } else {
                stored
            };
            out[r * n_channels + c] = value;
        }
    }
    out
}

#[cfg(unix)]
fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    file.read_exact_at(buf, offset)
}

#[cfg(windows)]
fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    let mut filled = 0;
    while filled < buf.len() {
        match file.seek_read(&mut buf[filled..], offset + filled as u64) {
            Ok(0) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "unexpected end of cbin while reading chunk",
                ))
            }
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn time_diff_encode(rows: usize, n_channels: usize, order_f: bool) -> (Vec<i16>, Vec<u8>) {
        let mut original = vec![0i16; rows * n_channels];
        for r in 0..rows {
            for c in 0..n_channels {
                original[r * n_channels + c] = (r * 10 + c) as i16;
            }
        }
        let mut delta = vec![0i16; rows * n_channels];
        for c in 0..n_channels {
            let mut prev = 0i16;
            for r in 0..rows {
                let v = original[r * n_channels + c];
                let d = v.wrapping_sub(prev);
                prev = v;
                let pos = if order_f {
                    c * rows + r
                } else {
                    r * n_channels + c
                };
                delta[pos] = d;
            }
        }
        let bytes: Vec<u8> = delta.iter().flat_map(|v| v.to_le_bytes()).collect();
        (original, bytes)
    }

    fn write_cbin(
        dir: &Path,
        chunks: &[(usize, Vec<u8>)],
        dtype: &str,
        do_spatial_diff: bool,
        algorithm: &str,
        order: &str,
    ) -> (PathBuf, PathBuf) {
        let cbin_path = dir.join("rec.ap.cbin");
        let ch_path = dir.join("rec.ap.ch");

        let mut compressed = Vec::new();
        let mut offsets = vec![0u64];
        let mut bounds = vec![0u64];
        let mut total_rows = 0u64;
        for (rows, raw) in chunks {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(raw).unwrap();
            let block = encoder.finish().unwrap();
            compressed.extend_from_slice(&block);
            offsets.push(compressed.len() as u64);
            total_rows += *rows as u64;
            bounds.push(total_rows);
        }
        File::create(&cbin_path)
            .unwrap()
            .write_all(&compressed)
            .unwrap();

        let header = serde_json::json!({
            "algorithm": algorithm,
            "dtype": dtype,
            "n_channels": 4,
            "sample_rate": 30000.0,
            "chunk_bounds": bounds,
            "chunk_offsets": offsets,
            "do_time_diff": true,
            "do_spatial_diff": do_spatial_diff,
            "chunk_order": order,
        });
        File::create(&ch_path)
            .unwrap()
            .write_all(header.to_string().as_bytes())
            .unwrap();

        (cbin_path, ch_path)
    }

    #[test]
    fn reads_shape_and_sample_rate() {
        let dir = tempdir().unwrap();
        let (_, bytes) = time_diff_encode(5, 4, false);
        let (cbin, ch) = write_cbin(dir.path(), &[(5, bytes)], "int16", false, "zlib", "C");
        let reader = CbinReader::open(&cbin, &ch).unwrap();
        assert_eq!(reader.n_channels(), 4);
        assert_eq!(reader.n_samples(), 5);
        assert_eq!(reader.sample_rate(), 30000.0);
    }

    #[test]
    fn chunks_reconstruct_across_native_boundaries_c_order() {
        let dir = tempdir().unwrap();
        let (orig0, b0) = time_diff_encode(5, 4, false);
        let (orig1, b1) = time_diff_encode(5, 4, false);
        let (cbin, ch) = write_cbin(dir.path(), &[(5, b0), (5, b1)], "int16", false, "zlib", "C");
        let reader = CbinReader::open(&cbin, &ch).unwrap();

        let chunks: Vec<_> = reader.chunks(3).collect();
        let rows: Vec<usize> = chunks.iter().map(|c| c.nrows()).collect();
        assert_eq!(rows, vec![3, 3, 3, 1]);
        assert!(chunks.iter().all(|c| c.ncols() == 4));

        let flat: Vec<i16> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
        let mut expected = orig0.clone();
        expected.extend_from_slice(&orig1);
        assert_eq!(flat, expected);
    }

    #[test]
    fn reconstructs_fortran_order() {
        let dir = tempdir().unwrap();
        let (orig, bytes) = time_diff_encode(6, 4, true);
        let (cbin, ch) = write_cbin(dir.path(), &[(6, bytes)], "int16", false, "zlib", "F");
        let reader = CbinReader::open(&cbin, &ch).unwrap();

        let chunk = reader.chunks(6).next().unwrap();
        let flat: Vec<i16> = chunk.iter().copied().collect();
        assert_eq!(flat, orig);
    }

    #[test]
    fn rejects_spatial_diff() {
        let dir = tempdir().unwrap();
        let (_, bytes) = time_diff_encode(5, 4, false);
        let (cbin, ch) = write_cbin(dir.path(), &[(5, bytes)], "int16", true, "zlib", "C");
        let result = CbinReader::open(&cbin, &ch);
        assert!(matches!(result, Err(CbinError::UnsupportedSpatialDiff)));
    }

    #[test]
    fn rejects_non_int16() {
        let dir = tempdir().unwrap();
        let (_, bytes) = time_diff_encode(5, 4, false);
        let (cbin, ch) = write_cbin(dir.path(), &[(5, bytes)], "float32", false, "zlib", "C");
        let result = CbinReader::open(&cbin, &ch);
        assert!(matches!(result, Err(CbinError::NotInt16 { .. })));
    }

    #[test]
    fn rejects_unknown_algorithm() {
        let dir = tempdir().unwrap();
        let (_, bytes) = time_diff_encode(5, 4, false);
        let (cbin, ch) = write_cbin(dir.path(), &[(5, bytes)], "int16", false, "lz4", "C");
        let result = CbinReader::open(&cbin, &ch);
        assert!(matches!(
            result,
            Err(CbinError::UnsupportedAlgorithm { .. })
        ));
    }
}
