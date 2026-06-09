use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use memmap2::Mmap;
use numpy::ndarray::Array2;
use thiserror::Error;

use crate::core::ChunkSource;
use crate::ephys::meta::{MetaError, SpikeGlxMeta};

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error(transparent)]
    Meta(#[from] MetaError),
    #[error("failed to open bin file: {0}")]
    Io(#[from] std::io::Error),
    #[error("bin file of {actual} bytes is not a whole number of frames ({n_channels} channels x 2 bytes)")]
    UnalignedFile { actual: u64, n_channels: usize },
}

pub struct SpikeGlxReader {
    meta: SpikeGlxMeta,
    n_samples: usize,
    mmap: Arc<Mmap>,
}

impl SpikeGlxReader {
    pub fn open(bin_path: &Path, meta_path: &Path) -> Result<Self, ReaderError> {
        let meta = SpikeGlxMeta::from_path(meta_path)?;
        Self::open_with_meta(bin_path, meta)
    }

    pub fn open_with_meta(bin_path: &Path, meta: SpikeGlxMeta) -> Result<Self, ReaderError> {
        let file = File::open(bin_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let actual = mmap.len() as u64;

        let frame_bytes = meta.n_channels.checked_mul(2).unwrap_or(0);
        if frame_bytes == 0 || mmap.len() % frame_bytes != 0 {
            return Err(ReaderError::UnalignedFile {
                actual,
                n_channels: meta.n_channels,
            });
        }
        let n_samples = mmap.len() / frame_bytes;

        Ok(Self {
            meta,
            n_samples,
            mmap: Arc::new(mmap),
        })
    }

    pub fn meta(&self) -> &SpikeGlxMeta {
        &self.meta
    }
}

impl ChunkSource for SpikeGlxReader {
    type Chunks = ChunkIter;

    fn n_channels(&self) -> usize {
        self.meta.n_channels
    }

    fn sample_rate(&self) -> f64 {
        self.meta.sample_rate
    }

    fn n_samples(&self) -> usize {
        self.n_samples
    }

    fn chunks(&self, chunk_samples: usize) -> ChunkIter {
        ChunkIter {
            mmap: Arc::clone(&self.mmap),
            n_channels: self.meta.n_channels,
            n_samples: self.n_samples,
            chunk_samples,
            pos: 0,
        }
    }
}

pub struct ChunkIter {
    mmap: Arc<Mmap>,
    n_channels: usize,
    n_samples: usize,
    chunk_samples: usize,
    pos: usize,
}

impl Iterator for ChunkIter {
    type Item = Array2<i16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.n_samples {
            return None;
        }
        let rows = self.chunk_samples.min(self.n_samples - self.pos);
        let start = self.pos * self.n_channels;
        let end = start + rows * self.n_channels;
        let bytes: &[u8] = &self.mmap;
        let samples: &[i16] = bytemuck::cast_slice(bytes);
        let data = samples[start..end].to_vec();
        self.pos += rows;
        Some(
            Array2::from_shape_vec((rows, self.n_channels), data)
                .expect("chunk length matches shape"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_fixture(
        dir: &Path,
        n_channels: usize,
        n_samples: usize,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let bin_path = dir.join("run_g0_t0.imec0.ap.bin");
        let meta_path = dir.join("run_g0_t0.imec0.ap.meta");

        let mut samples = Vec::with_capacity(n_channels * n_samples);
        for s in 0..n_samples {
            for c in 0..n_channels {
                samples.push((s * 10 + c) as i16);
            }
        }
        let bytes: Vec<u8> = samples.iter().flat_map(|v| v.to_le_bytes()).collect();
        File::create(&bin_path).unwrap().write_all(&bytes).unwrap();

        let meta = format!(
            "nSavedChans={}\nimSampRate=30000\ntypeThis=imec\nfileSizeBytes={}\n",
            n_channels,
            bytes.len()
        );
        File::create(&meta_path)
            .unwrap()
            .write_all(meta.as_bytes())
            .unwrap();

        (bin_path, meta_path)
    }

    #[test]
    fn reads_metadata_from_actual_file() {
        let dir = tempdir().unwrap();
        let (bin, meta) = write_fixture(dir.path(), 4, 10);
        let reader = SpikeGlxReader::open(&bin, &meta).unwrap();
        assert_eq!(reader.n_channels(), 4);
        assert_eq!(reader.n_samples(), 10);
        assert_eq!(reader.sample_rate(), 30000.0);
    }

    #[test]
    fn chunks_cover_every_sample_exactly_once() {
        let dir = tempdir().unwrap();
        let (bin, meta) = write_fixture(dir.path(), 4, 10);
        let reader = SpikeGlxReader::open(&bin, &meta).unwrap();

        let chunks: Vec<_> = reader.chunks(3).collect();
        let rows: Vec<usize> = chunks.iter().map(|c| c.nrows()).collect();
        assert_eq!(rows, vec![3, 3, 3, 1]);
        assert!(chunks.iter().all(|c| c.ncols() == 4));

        assert_eq!(chunks[0][[0, 0]], 0);
        assert_eq!(chunks[0][[0, 1]], 1);
        assert_eq!(chunks[0][[1, 0]], 10);
        assert_eq!(chunks[3][[0, 0]], 90);
        assert_eq!(chunks[3][[0, 3]], 93);
    }

    #[test]
    fn reads_actual_bytes_when_declared_size_is_stale() {
        let dir = tempdir().unwrap();
        let (bin, _) = write_fixture(dir.path(), 4, 10);
        let stale_meta = dir.path().join("stale.meta");
        File::create(&stale_meta)
            .unwrap()
            .write_all(b"nSavedChans=4\nimSampRate=30000\nfileSizeBytes=999999\n")
            .unwrap();
        let reader = SpikeGlxReader::open(&bin, &stale_meta).unwrap();
        assert_eq!(reader.n_samples(), 10);
        assert_eq!(reader.meta().declared_file_size_bytes, Some(999999));
    }

    #[test]
    fn detects_unaligned_file() {
        let dir = tempdir().unwrap();
        let (bin, _) = write_fixture(dir.path(), 4, 10);
        let bad_meta = dir.path().join("bad.meta");
        File::create(&bad_meta)
            .unwrap()
            .write_all(b"nSavedChans=3\nimSampRate=30000\n")
            .unwrap();
        let result = SpikeGlxReader::open(&bin, &bad_meta);
        assert!(matches!(result, Err(ReaderError::UnalignedFile { .. })));
    }
}
