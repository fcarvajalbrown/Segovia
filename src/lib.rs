use std::path::PathBuf;

use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray1, PyArray2, PyReadonlyArray2};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

mod core;
mod dsp;
mod ephys;
mod sim;

use core::ChunkSource;
use dsp::pipeline::{ChainParams, Pipeline};
use ephys::cbin::{CbinChunkIter, CbinError};
use ephys::reader::{ChunkIter, ReaderError};
use ephys::zarr::{ZarrChunkIter, ZarrError};
use sim::ephys::{SimChunkIter, SimConfig, SimError, SyntheticEphysReader};
use sim::ifc::{IfcChunkIter, IfcConfig, IfcError, SyntheticIfcReader};

type ChunkStream = Box<dyn Iterator<Item = Array2<i16>> + Send + Sync>;

type GroundTruthArrays<'py> = (
    Bound<'py, PyArray1<i64>>,
    Bound<'py, PyArray1<i32>>,
    Bound<'py, PyArray1<i32>>,
);

#[allow(clippy::too_many_arguments)]
fn build_preprocessor(
    py: Python<'_>,
    calib: ChunkStream,
    stream: ChunkStream,
    sos: PyReadonlyArray2<'_, f64>,
    margin: usize,
    calib_samples: usize,
    eps: f64,
    apply_mean: bool,
    batch: usize,
    whiten: bool,
) -> PyResult<PyPreprocessor> {
    let sos_arr = sos.as_array();
    if sos_arr.ncols() != 6 {
        return Err(PyValueError::new_err("sos must have shape (n_sections, 6)"));
    }
    let sos_vec: Vec<[f64; 6]> = sos_arr
        .outer_iter()
        .map(|r| [r[0], r[1], r[2], r[3], r[4], r[5]])
        .collect();
    if sos_vec.is_empty() {
        return Err(PyValueError::new_err("sos must have at least one section"));
    }
    let padlen = dsp::filter::default_padlen(&sos_vec);
    let batch = if batch == 0 {
        rayon::current_num_threads().max(1)
    } else {
        batch
    };
    let params = ChainParams {
        sos: sos_vec,
        padlen,
        margin,
        batch,
        eps,
        apply_mean,
        calib_samples,
        whiten,
    };
    let inner = py.detach(move || Pipeline::new(calib, stream, params));
    Ok(PyPreprocessor { inner })
}

#[pyclass(name = "Preprocessor")]
struct PyPreprocessor {
    inner: Pipeline,
}

#[pymethods]
impl PyPreprocessor {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
    ) -> Option<Bound<'py, PyArray2<f32>>> {
        let inner = &mut slf.inner;
        let next = py.detach(|| inner.next_chunk());
        next.map(|array| array.into_pyarray(py))
    }
}

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

impl From<CbinError> for PyErr {
    fn from(err: CbinError) -> PyErr {
        let message = err.to_string();
        match err {
            CbinError::Io(_) => PyIOError::new_err(message),
            _ => PyValueError::new_err(message),
        }
    }
}

impl From<SimError> for PyErr {
    fn from(err: SimError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

impl From<IfcError> for PyErr {
    fn from(err: IfcError) -> PyErr {
        PyValueError::new_err(err.to_string())
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

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (sos, chunk_samples, margin, calib_samples, eps = 1e-6, apply_mean = true, batch = 0, whiten = true))]
    fn preprocess(
        &self,
        py: Python<'_>,
        sos: PyReadonlyArray2<'_, f64>,
        chunk_samples: usize,
        margin: usize,
        calib_samples: usize,
        eps: f64,
        apply_mean: bool,
        batch: usize,
        whiten: bool,
    ) -> PyResult<PyPreprocessor> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        let calib: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        let stream: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        build_preprocessor(
            py,
            calib,
            stream,
            sos,
            margin,
            calib_samples,
            eps,
            apply_mean,
            batch,
            whiten,
        )
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

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (sos, chunk_samples, margin, calib_samples, eps = 1e-6, apply_mean = true, batch = 0, whiten = true))]
    fn preprocess(
        &self,
        py: Python<'_>,
        sos: PyReadonlyArray2<'_, f64>,
        chunk_samples: usize,
        margin: usize,
        calib_samples: usize,
        eps: f64,
        apply_mean: bool,
        batch: usize,
        whiten: bool,
    ) -> PyResult<PyPreprocessor> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        let calib: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        let stream: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        build_preprocessor(
            py,
            calib,
            stream,
            sos,
            margin,
            calib_samples,
            eps,
            apply_mean,
            batch,
            whiten,
        )
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

#[pyclass(name = "CbinReader")]
struct PyCbinReader {
    inner: ephys::cbin::CbinReader,
}

#[pymethods]
impl PyCbinReader {
    #[new]
    #[pyo3(signature = (cbin_path, ch_path = None))]
    fn new(cbin_path: PathBuf, ch_path: Option<PathBuf>) -> PyResult<Self> {
        let ch_path = ch_path.unwrap_or_else(|| cbin_path.with_extension("ch"));
        let inner = ephys::cbin::CbinReader::open(&cbin_path, &ch_path)?;
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
    fn chunks(&self, chunk_samples: usize) -> PyResult<PyCbinChunks> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        Ok(PyCbinChunks {
            inner: self.inner.chunks(chunk_samples),
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (sos, chunk_samples, margin, calib_samples, eps = 1e-6, apply_mean = true, batch = 0, whiten = true))]
    fn preprocess(
        &self,
        py: Python<'_>,
        sos: PyReadonlyArray2<'_, f64>,
        chunk_samples: usize,
        margin: usize,
        calib_samples: usize,
        eps: f64,
        apply_mean: bool,
        batch: usize,
        whiten: bool,
    ) -> PyResult<PyPreprocessor> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        let calib: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        let stream: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        build_preprocessor(
            py,
            calib,
            stream,
            sos,
            margin,
            calib_samples,
            eps,
            apply_mean,
            batch,
            whiten,
        )
    }
}

#[pyclass(name = "CbinChunks")]
struct PyCbinChunks {
    inner: CbinChunkIter,
}

#[pymethods]
impl PyCbinChunks {
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

#[pyclass(name = "SyntheticEphysReader")]
struct PySyntheticEphysReader {
    inner: SyntheticEphysReader,
}

#[pymethods]
impl PySyntheticEphysReader {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (n_channels, duration_s, sample_rate = 30000.0, n_units = 20, firing_rate = 5.0, pitch = 20.0, noise_uv = 10.0, lsb_uv = 2.34, seed = 0))]
    fn new(
        n_channels: usize,
        duration_s: f64,
        sample_rate: f64,
        n_units: usize,
        firing_rate: f64,
        pitch: f64,
        noise_uv: f64,
        lsb_uv: f64,
        seed: u64,
    ) -> PyResult<Self> {
        let inner = SyntheticEphysReader::new(SimConfig {
            n_channels,
            duration_s,
            sample_rate,
            n_units,
            firing_rate,
            pitch,
            noise_uv,
            lsb_uv,
            seed,
        })?;
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

    fn ground_truth<'py>(&self, py: Python<'py>) -> GroundTruthArrays<'py> {
        let (samples, units, peaks) = self.inner.ground_truth();
        (
            samples.into_pyarray(py),
            units.into_pyarray(py),
            peaks.into_pyarray(py),
        )
    }

    #[pyo3(signature = (chunk_samples))]
    fn chunks(&self, chunk_samples: usize) -> PyResult<PySyntheticEphysChunks> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        Ok(PySyntheticEphysChunks {
            inner: self.inner.chunks(chunk_samples),
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (sos, chunk_samples, margin, calib_samples, eps = 1e-6, apply_mean = true, batch = 0, whiten = true))]
    fn preprocess(
        &self,
        py: Python<'_>,
        sos: PyReadonlyArray2<'_, f64>,
        chunk_samples: usize,
        margin: usize,
        calib_samples: usize,
        eps: f64,
        apply_mean: bool,
        batch: usize,
        whiten: bool,
    ) -> PyResult<PyPreprocessor> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        let calib: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        let stream: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        build_preprocessor(
            py,
            calib,
            stream,
            sos,
            margin,
            calib_samples,
            eps,
            apply_mean,
            batch,
            whiten,
        )
    }
}

#[pyclass(name = "SyntheticEphysChunks")]
struct PySyntheticEphysChunks {
    inner: SimChunkIter,
}

#[pymethods]
impl PySyntheticEphysChunks {
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

#[pyclass(name = "SyntheticIfcReader")]
struct PySyntheticIfcReader {
    inner: SyntheticIfcReader,
}

#[pymethods]
impl PySyntheticIfcReader {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (n_channels = 2, duration_s = 1.0, sample_rate = 100000.0, n_populations = 3, event_rate = 100.0, noise_level = 0.01, lsb = 1.0e-4, seed = 0))]
    fn new(
        n_channels: usize,
        duration_s: f64,
        sample_rate: f64,
        n_populations: usize,
        event_rate: f64,
        noise_level: f64,
        lsb: f64,
        seed: u64,
    ) -> PyResult<Self> {
        let inner = SyntheticIfcReader::new(IfcConfig {
            n_channels,
            duration_s,
            sample_rate,
            n_populations,
            event_rate,
            noise_level,
            lsb,
            seed,
        })?;
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

    fn ground_truth<'py>(&self, py: Python<'py>) -> GroundTruthArrays<'py> {
        let (samples, populations, amplitudes) = self.inner.ground_truth();
        (
            samples.into_pyarray(py),
            populations.into_pyarray(py),
            amplitudes.into_pyarray(py),
        )
    }

    #[pyo3(signature = (chunk_samples))]
    fn chunks(&self, chunk_samples: usize) -> PyResult<PySyntheticIfcChunks> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        Ok(PySyntheticIfcChunks {
            inner: self.inner.chunks(chunk_samples),
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (sos, chunk_samples, margin, calib_samples, eps = 1e-6, apply_mean = true, batch = 0, whiten = true))]
    fn preprocess(
        &self,
        py: Python<'_>,
        sos: PyReadonlyArray2<'_, f64>,
        chunk_samples: usize,
        margin: usize,
        calib_samples: usize,
        eps: f64,
        apply_mean: bool,
        batch: usize,
        whiten: bool,
    ) -> PyResult<PyPreprocessor> {
        if chunk_samples == 0 {
            return Err(PyValueError::new_err(
                "chunk_samples must be greater than zero",
            ));
        }
        let calib: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        let stream: ChunkStream = Box::new(self.inner.chunks(chunk_samples));
        build_preprocessor(
            py,
            calib,
            stream,
            sos,
            margin,
            calib_samples,
            eps,
            apply_mean,
            batch,
            whiten,
        )
    }
}

#[pyclass(name = "SyntheticIfcChunks")]
struct PySyntheticIfcChunks {
    inner: IfcChunkIter,
}

#[pymethods]
impl PySyntheticIfcChunks {
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
    m.add_class::<PyCbinReader>()?;
    m.add_class::<PyCbinChunks>()?;
    m.add_class::<PySyntheticEphysReader>()?;
    m.add_class::<PySyntheticEphysChunks>()?;
    m.add_class::<PySyntheticIfcReader>()?;
    m.add_class::<PySyntheticIfcChunks>()?;
    m.add_class::<PyPreprocessor>()?;
    Ok(())
}
