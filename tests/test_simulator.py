import numpy as np
import pytest
import segovia

scipy_signal = pytest.importorskip("scipy.signal")


def _reader(**overrides):
    params = dict(
        n_channels=32,
        duration_s=1.0,
        sample_rate=30000.0,
        n_units=8,
        firing_rate=15.0,
        noise_uv=8.0,
        seed=123,
    )
    params.update(overrides)
    return segovia.SyntheticEphysReader(**params)


def test_shape_and_metadata():
    r = _reader()
    assert r.n_channels == 32
    assert r.n_samples == 30000
    assert r.sample_rate == 30000.0


def test_chunks_reconstruct_full_length():
    r = _reader()
    chunks = list(r.chunks(4096))
    assert sum(c.shape[0] for c in chunks) == r.n_samples
    assert all(c.shape[1] == 32 for c in chunks)
    assert all(c.dtype == np.int16 for c in chunks)


def test_output_is_chunk_size_independent():
    r = _reader()
    big = np.concatenate(list(r.chunks(r.n_samples)), axis=0)
    small = np.concatenate(list(r.chunks(777)), axis=0)
    assert np.array_equal(big, small)


def test_same_seed_is_deterministic():
    a = np.concatenate(list(_reader().chunks(4096)), axis=0)
    b = np.concatenate(list(_reader().chunks(4096)), axis=0)
    assert np.array_equal(a, b)


def test_different_seed_differs():
    a = np.concatenate(list(_reader(seed=1).chunks(4096)), axis=0)
    b = np.concatenate(list(_reader(seed=2).chunks(4096)), axis=0)
    assert not np.array_equal(a, b)


def test_ground_truth_is_sorted_and_in_range():
    r = _reader()
    samples, units, peaks = r.ground_truth()
    assert samples.dtype == np.int64
    assert units.dtype == np.int32
    assert peaks.dtype == np.int32
    assert samples.size == units.size == peaks.size
    assert np.all(np.diff(samples) >= 0)
    assert samples.max() < r.n_samples
    assert peaks.min() >= 0 and peaks.max() < r.n_channels


def test_noiseless_energy_peaks_on_ground_truth_channel():
    r = _reader(n_units=1, firing_rate=50.0, noise_uv=0.0)
    _, _, peaks = r.ground_truth()
    signal = np.concatenate(list(r.chunks(4096)), axis=0).astype(np.float64)
    energy = (signal**2).sum(axis=0)
    assert int(energy.argmax()) == int(peaks[0])


def test_zero_chunk_size_rejected():
    with pytest.raises(ValueError):
        _reader().chunks(0)


def test_invalid_config_rejected():
    with pytest.raises(ValueError):
        segovia.SyntheticEphysReader(n_channels=0, duration_s=1.0)
    with pytest.raises(ValueError):
        segovia.SyntheticEphysReader(n_channels=8, duration_s=0.0)


def test_plugs_into_preprocess_chain():
    r = _reader()
    sos = scipy_signal.butter(3, [300, 6000], btype="band", fs=30000.0, output="sos")
    out = list(
        r.preprocess(
            sos=sos, chunk_samples=4096, margin=128, calib_samples=15000, whiten=True
        )
    )
    assert sum(c.shape[0] for c in out) == r.n_samples
    assert all(c.shape[1] == 32 for c in out)
    assert all(c.dtype == np.float32 for c in out)
