import numpy as np
import pytest
import segovia

scipy_signal = pytest.importorskip("scipy.signal")


def _reader(**overrides):
    params = dict(
        n_channels=2,
        duration_s=1.0,
        sample_rate=100000.0,
        n_populations=3,
        event_rate=200.0,
        noise_level=0.01,
        lsb=1.0e-4,
        seed=123,
    )
    params.update(overrides)
    return segovia.SyntheticIfcReader(**params)


def test_shape_and_metadata():
    r = _reader()
    assert r.n_channels == 2
    assert r.n_samples == 100000
    assert r.sample_rate == 100000.0


def test_chunks_reconstruct_full_length():
    r = _reader()
    chunks = list(r.chunks(4096))
    assert sum(c.shape[0] for c in chunks) == r.n_samples
    assert all(c.shape[1] == 2 for c in chunks)
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
    samples, populations, amplitudes = r.ground_truth()
    assert samples.dtype == np.int64
    assert populations.dtype == np.int32
    assert amplitudes.dtype == np.int32
    assert samples.size == populations.size == amplitudes.size
    assert np.all(np.diff(samples) >= 0)
    assert samples.max() < r.n_samples
    assert populations.min() >= 0 and populations.max() < 3
    assert amplitudes.min() > 0


def test_pulse_is_bipolar_when_noiseless():
    r = _reader(n_populations=1, event_rate=40.0, noise_level=0.0)
    signal = np.concatenate(list(r.chunks(4096)), axis=0)
    assert signal[:, 0].max() > 1000
    assert signal[:, 0].min() < -1000


def test_zero_chunk_size_rejected():
    with pytest.raises(ValueError):
        _reader().chunks(0)


def test_invalid_config_rejected():
    with pytest.raises(ValueError):
        segovia.SyntheticIfcReader(n_channels=0)
    with pytest.raises(ValueError):
        segovia.SyntheticIfcReader(duration_s=0.0)
    with pytest.raises(ValueError):
        segovia.SyntheticIfcReader(n_populations=0)


def test_plugs_into_preprocess_chain():
    r = _reader()
    sos = scipy_signal.butter(
        5, [300, 6000], btype="band", fs=r.sample_rate, output="sos"
    )
    out = list(
        r.preprocess(
            sos=sos, chunk_samples=4096, margin=256, calib_samples=20000, whiten=True
        )
    )
    assert sum(c.shape[0] for c in out) == r.n_samples
    assert all(c.shape[1] == 2 for c in out)
    assert all(c.dtype == np.float32 for c in out)
