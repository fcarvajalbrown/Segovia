use std::path::PathBuf;

use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray2};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

mod core;
mod ephys;

use core::ChunkSource;
use ephys::reader::{ChunkIter, ReaderError};
use ephys::zarr::{ZarrChunkIter, ZarrError};

impl From<ReaderError> for PyErr {
    fn from(err: ReaderError) -> PyErr {
        let message = err.to_string();
        match err {
            ReaderError::Io(_) => PyIOError::new_err(message),
            _ => PyValueError::new_err(message),
        }
    }
}

impl From<ZarrError> for PyErr {
    fn from(err: ZarrError) -> PyErr {
        let message = err.to_string();
        match err {
            ZarrError::Store { .. } | ZarrError::Array { .. } => PyIOError::new_err(message),
            _ => PyValueError::new_err(message),
        }
    }
}

#[pyfunction]
fn zeros<'py>(py: Python<'py>, channels: usize, samples: usize) -> Bound<'py, PyArray2<i16>> {
    Array2::<i16>::zeros((channels, samples)).into_pyarray(py)
}

#[pyclass(name = "SpikeGlxReader")]
struct PySpikeGlxReader {
    inner: ephys::reader::SpikeGlxReader,
}

#[pymethods]
impl PySpikeGlxReader {
    #[new]
    #[pyo3(signature = (bin_path, meta_path = None))]
    fn new(bin_path: PathBuf, meta_path: Option<PathBuf>) -> PyResult<Self> {
        let meta_path = meta_path.unwrap_or_else(|| bin_path.with_extension("meta"));
        let inner = ephys::reader::SpikeGlxReader::open(&bin_path, &meta_path)?;
        Ok(Self { inner })
    }

    #[getter]
    fn n_channels(&self) -> usize {
        self.inner.n_channels()
    }

    #[getter]
    fn sample_rate(&self) -> f64 {
        self.inner.sample_rate()
    }

    #[getter]
    fn n_samples(&self) -> usize {
        self.inner.n_samples()
    }

    #[getter]
    fn stream_type(&self) -> String {
        self.inner.meta().stream_type.clone()
    }

    #[getter]
    fn declared_file_size_bytes(&self) -> Option<u64> {
        self.inner.meta().declared_file_size_bytes
    }

    #[getter]
    fn fields(&self) -> std::collections::HashMap<String, String> {
        self.inner.meta().fields.clone()
    }

    #[pyo3(signature = (chunk_samples))]
    fn chunks(&self, chunk_samples: usize) -> PyResult<PySpikeGlxChunks> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        Ok(PySpikeGlxChunks {
            inner: self.inner.chunks(chunk_samples),
        })
    }
}

#[pyclass(name = "SpikeGlxChunks")]
struct PySpikeGlxChunks {
    inner: ChunkIter,
}

#[pymethods]
impl PySpikeGlxChunks {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
    ) -> Option<Bound<'py, PyArray2<i16>>> {
        let inner = &mut slf.inner;
        let next = py.detach(|| inner.next());
        next.map(|array| array.into_pyarray(py))
    }
}

#[pyclass(name = "ZarrReader")]
struct PyZarrReader {
    inner: ephys::zarr::ZarrReader,
}

#[pymethods]
impl PyZarrReader {
    #[new]
    #[pyo3(signature = (store_path, array_path = "/traces".to_string()))]
    fn new(store_path: PathBuf, array_path: String) -> PyResult<Self> {
        let inner = ephys::zarr::ZarrReader::open(&store_path, &array_path)?;
        Ok(Self { inner })
    }

    #[getter]
    fn n_channels(&self) -> usize {
        self.inner.n_channels()
    }

    #[getter]
    fn sample_rate(&self) -> f64 {
        self.inner.sample_rate()
    }

    #[getter]
    fn n_samples(&self) -> usize {
        self.inner.n_samples()
    }

    #[pyo3(signature = (chunk_samples))]
    fn chunks(&self, chunk_samples: usize) -> PyResult<PyZarrChunks> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        Ok(PyZarrChunks {
            inner: self.inner.chunks(chunk_samples),
        })
    }
}

#[pyclass(name = "ZarrChunks")]
struct PyZarrChunks {
    inner: ZarrChunkIter,
}

#[pymethods]
impl PyZarrChunks {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
    ) -> Option<Bound<'py, PyArray2<i16>>> {
        let inner = &mut slf.inner;
        let next = py.detach(|| inner.next());
        next.map(|array| array.into_pyarray(py))
    }
}

#[pymodule]
fn segovia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(zeros, m)?)?;
    m.add_class::<PySpikeGlxReader>()?;
    m.add_class::<PySpikeGlxChunks>()?;
    m.add_class::<PyZarrReader>()?;
    m.add_class::<PyZarrChunks>()?;
    Ok(())
}
