import numpy as np
import pytest
import segovia
from scipy import signal

FS = 30000.0
BAND = (300.0, 6000.0)
ORDER = 5


def _sos():
    return np.ascontiguousarray(
        signal.butter(ORDER, BAND, btype="band", fs=FS, output="sos"),
        dtype=np.float64,
    )


def write_recording(tmp_path, n_channels, n_samples, seed=0):
    rng = np.random.default_rng(seed)
    t = np.arange(n_samples) / FS
    data = np.zeros((n_samples, n_channels), dtype=np.float64)
    for c in range(n_channels):
        in_band = np.sin(2 * np.pi * (500 + 80 * c) * t)
        low = 0.7 * np.sin(2 * np.pi * 30 * t)
        high = 0.4 * np.sin(2 * np.pi * 11000 * t)
        noise = 0.3 * rng.standard_normal(n_samples)
        offset = 50 * (c - n_channels / 2)
        data[:, c] = 400 * (in_band + low + high) + 200 * noise + offset
    data = np.round(data).astype(np.int16)

    bin_path = tmp_path / "rec_g0_t0.imec0.ap.bin"
    meta_path = tmp_path / "rec_g0_t0.imec0.ap.meta"
    data.tofile(bin_path)
    meta_path.write_text(
        f"nSavedChans={n_channels}\n"
        f"imSampRate={int(FS)}\n"
        f"typeThis=imec\n"
        f"fileSizeBytes={bin_path.stat().st_size}\n"
    )
    return bin_path, meta_path, data


def reference_bandpass(raw, sos):
    return signal.sosfiltfilt(sos, raw, axis=0)


def reference_cmr(filtered):
    return filtered - np.median(filtered, axis=1, keepdims=True)


def reference_whitener(calib_cmr, eps, apply_mean):
    mean = calib_cmr.mean(axis=0) if apply_mean else np.zeros(calib_cmr.shape[1])
    centered = calib_cmr - mean
    cov = centered.T @ centered / centered.shape[0]
    evals, evecs = np.linalg.eigh(cov)
    w = evecs @ np.diag(1.0 / np.sqrt(evals + eps)) @ evecs.T
    return mean, w


def reference_chain(raw, sos, chunk_samples, calib_samples, eps, apply_mean):
    raw = raw.astype(np.float64)
    full = reference_cmr(reference_bandpass(raw, sos))
    calib = raw[:calib_samples]
    calib_cmr = reference_cmr(reference_bandpass(calib, sos))
    mean, w = reference_whitener(calib_cmr, eps, apply_mean)
    return (full - mean) @ w


def test_bandpass_only_matches_scipy(tmp_path):
    bin_path, meta_path, data = write_recording(tmp_path, 8, 30000)
    sos = _sos()
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))

    pre = reader.preprocess(
        sos,
        chunk_samples=6000,
        margin=4000,
        calib_samples=1,
        batch=4,
        whiten=False,
    )
    seg = np.concatenate(list(pre), axis=0)

    ref_bp = reference_bandpass(data.astype(np.float64), sos)
    ref_cmr = reference_cmr(ref_bp)

    scale = np.max(np.abs(ref_cmr))
    rel = np.max(np.abs(seg.astype(np.float64) - ref_cmr)) / scale
    assert rel < 1e-4, f"bandpass+CMR relative error {rel:.2e}"


def test_full_chain_matches_scipy(tmp_path):
    bin_path, meta_path, data = write_recording(tmp_path, 16, 30000)
    sos = _sos()
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))

    chunk_samples = 6000
    calib_samples = 12000
    eps = 1e-6
    margin = 4000

    pre = reader.preprocess(
        sos,
        chunk_samples=chunk_samples,
        margin=margin,
        calib_samples=calib_samples,
        eps=eps,
        apply_mean=True,
        batch=4,
    )
    seg = np.concatenate(list(pre), axis=0).astype(np.float64)

    ref = reference_chain(data, sos, chunk_samples, calib_samples, eps, True)

    assert seg.shape == ref.shape
    scale = np.max(np.abs(ref))
    rel = np.max(np.abs(seg - ref)) / scale
    assert rel < 1e-3, f"full-chain relative error {rel:.2e}"


def test_output_is_float32_and_bounded_chunks(tmp_path):
    bin_path, meta_path, data = write_recording(tmp_path, 8, 20000)
    sos = _sos()
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))
    pre = reader.preprocess(
        sos, chunk_samples=5000, margin=2000, calib_samples=10000
    )
    chunks = list(pre)
    assert all(c.dtype == np.float32 for c in chunks)
    assert all(c.shape[1] == 8 for c in chunks)
    assert sum(c.shape[0] for c in chunks) == data.shape[0]
