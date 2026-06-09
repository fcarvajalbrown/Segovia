use std::path::Path;
use std::sync::Arc;

use numpy::ndarray::Array2;
use thiserror::Error;
use zarrs::array::data_type::Int16DataType;
use zarrs::array::{Array, ArraySubset};
use zarrs::filesystem::FilesystemStore;

use crate::core::ChunkSource;

#[derive(Debug, Error)]
pub enum ZarrError {
    #[error("failed to open zarr store at {path}: {message}")]
    Store { path: String, message: String },
    #[error("failed to open zarr array '{array_path}': {message}")]
    Array { array_path: String, message: String },
    #[error("zarr array '{array_path}' must be 2-D (samples, channels), found {ndim} dimensions")]
    NotTwoDimensional { array_path: String, ndim: usize },
    #[error("zarr array '{array_path}' must be int16, found {found}")]
    NotInt16 { array_path: String, found: String },
}

pub struct ZarrReader {
    array: Arc<Array<FilesystemStore>>,
    n_samples: usize,
    n_channels: usize,
    sample_rate: f64,
}

impl ZarrReader {
    pub fn open(store_path: &Path, array_path: &str) -> Result<Self, ZarrError> {
        let store = FilesystemStore::new(store_path).map_err(|e| ZarrError::Store {
            path: store_path.display().to_string(),
            message: e.to_string(),
        })?;
        let array = Array::open(Arc::new(store), array_path).map_err(|e| ZarrError::Array {
            array_path: array_path.to_string(),
            message: e.to_string(),
        })?;

        let shape = array.shape();
        if shape.len() != 2 {
            return Err(ZarrError::NotTwoDimensional {
                array_path: array_path.to_string(),
                ndim: shape.len(),
            });
        }
        if !array.data_type().is::<Int16DataType>() {
            return Err(ZarrError::NotInt16 {
                array_path: array_path.to_string(),
                found: format!("{:?}", array.data_type()),
            });
        }

        let n_samples = shape[0] as usize;
        let n_channels = shape[1] as usize;
        let sample_rate = array
            .attributes()
            .get("sampling_frequency")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);

        Ok(Self {
            array: Arc::new(array),
            n_samples,
            n_channels,
            sample_rate,
        })
    }
}

impl ChunkSource for ZarrReader {
    type Chunks = ZarrChunkIter;

    fn n_channels(&self) -> usize {
        self.n_channels
    }

    fn n_samples(&self) -> usize {
        self.n_samples
    }

    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn chunks(&self, chunk_samples: usize) -> ZarrChunkIter {
        ZarrChunkIter {
            array: Arc::clone(&self.array),
            n_samples: self.n_samples,
            n_channels: self.n_channels,
            chunk_samples,
            pos: 0,
        }
    }
}

pub struct ZarrChunkIter {
    array: Arc<Array<FilesystemStore>>,
    n_samples: usize,
    n_channels: usize,
    chunk_samples: usize,
    pos: usize,
}

impl Iterator for ZarrChunkIter {
    type Item = Array2<i16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.n_samples {
            return None;
        }
        let rows = self.chunk_samples.min(self.n_samples - self.pos);
        let subset = ArraySubset::new_with_ranges(&[
            self.pos as u64..(self.pos + rows) as u64,
            0..self.n_channels as u64,
        ]);
        let data: Vec<i16> = self
            .array
            .retrieve_array_subset::<Vec<i16>>(&subset)
            .expect("retrieve zarr subset");
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
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn reads_shape_and_sample_rate() {
        let reader = ZarrReader::open(&fixture("mini_int16.zarr"), "/traces").unwrap();
        assert_eq!(reader.n_channels(), 4);
        assert_eq!(reader.n_samples(), 10);
        assert_eq!(reader.sample_rate(), 30000.0);
    }

    #[test]
    fn chunks_reconstruct_the_recording() {
        let reader = ZarrReader::open(&fixture("mini_int16.zarr"), "/traces").unwrap();

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
    fn rejects_non_int16() {
        let result = ZarrReader::open(&fixture("reject_float32.zarr"), "/traces");
        assert!(matches!(result, Err(ZarrError::NotInt16 { .. })));
    }
}
