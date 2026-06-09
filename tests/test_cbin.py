from pathlib import Path

import numpy as np
import pytest
import segovia

mtscomp = pytest.importorskip("mtscomp")

FIXTURES = Path(__file__).parent / "fixtures"
REAL_BIN = Path(__file__).parent / "data" / "Noise4Sam_g0_t0.imec0.ap.bin"
REAL_META = Path(__file__).parent / "data" / "Noise4Sam_g0_t0.imec0.ap.meta"


def test_mini_fixture_shape_and_values():
    reader = segovia.CbinReader(str(FIXTURES / "mini_int16.cbin"))
    assert reader.n_channels == 4
    assert reader.n_samples == 10
    assert reader.sample_rate == 30000.0

    chunks = list(reader.chunks(3))
    assert [c.shape[0] for c in chunks] == [3, 3, 3, 1]
    assert all(c.shape[1] == 4 for c in chunks)
    assert all(c.dtype == np.int16 for c in chunks)

    reconstructed = np.concatenate(chunks, axis=0)
    expected = np.array(
        [[s * 10 + c for c in range(4)] for s in range(10)], dtype=np.int16
    )
    assert np.array_equal(reconstructed, expected)


def test_default_ch_path():
    reader = segovia.CbinReader(str(FIXTURES / "mini_int16.cbin"))
    assert reader.n_samples == 10


def test_zero_chunk_size_rejected():
    reader = segovia.CbinReader(str(FIXTURES / "mini_int16.cbin"))
    with pytest.raises(ValueError):
        reader.chunks(0)


@pytest.mark.skipif(
    not REAL_BIN.exists(),
    reason="real Noise4Sam_g0 data not present under tests/data/",
)
def test_cbin_matches_spikeglx_on_real_data(tmp_path):
    spikeglx = segovia.SpikeGlxReader(str(REAL_BIN), str(REAL_META))
    spikeglx_full = np.concatenate(list(spikeglx.chunks(256)), axis=0)
    assert spikeglx_full.shape == (spikeglx.n_samples, spikeglx.n_channels)

    raw = tmp_path / "real.bin"
    cbin = tmp_path / "real.cbin"
    ch = tmp_path / "real.ch"
    spikeglx_full.tofile(raw)
    mtscomp.compress(
        str(raw),
        str(cbin),
        str(ch),
        sample_rate=spikeglx.sample_rate,
        n_channels=spikeglx.n_channels,
        dtype=np.int16,
    )

    reader = segovia.CbinReader(str(cbin), str(ch))
    assert reader.n_channels == spikeglx.n_channels
    assert reader.n_samples == spikeglx.n_samples
    assert reader.sample_rate == spikeglx.sample_rate

    cbin_full = np.concatenate(list(reader.chunks(256)), axis=0)
    assert np.array_equal(cbin_full, spikeglx_full)
