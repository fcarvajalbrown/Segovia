from pathlib import Path

import numpy as np
import pytest
import segovia

zarr = pytest.importorskip("zarr")

FIXTURES = Path(__file__).parent / "fixtures"
REAL_BIN = Path(__file__).parent / "data" / "Noise4Sam_g0_t0.imec0.ap.bin"
REAL_META = Path(__file__).parent / "data" / "Noise4Sam_g0_t0.imec0.ap.meta"


def test_mini_fixture_shape_and_values():
    reader = segovia.ZarrReader(str(FIXTURES / "mini_int16.zarr"))
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


def test_zero_chunk_size_rejected():
    reader = segovia.ZarrReader(str(FIXTURES / "mini_int16.zarr"))
    with pytest.raises(ValueError):
        reader.chunks(0)


def _write_zarr(path, data, sample_rate, compressor):
    group = zarr.open_group(store=str(path), mode="w", zarr_format=3)
    arr = group.create_array(
        name="traces",
        shape=data.shape,
        chunks=(min(20000, data.shape[0]), data.shape[1]),
        dtype="int16",
        compressors=[compressor],
    )
    arr[:] = data
    arr.attrs["sampling_frequency"] = sample_rate


@pytest.mark.skipif(
    not REAL_BIN.exists(),
    reason="real Noise4Sam_g0 data not present under tests/data/",
)
@pytest.mark.parametrize(
    "compressor",
    [
        zarr.codecs.GzipCodec(level=5),
        zarr.codecs.ZstdCodec(),
        zarr.codecs.BloscCodec(),
    ],
)
def test_zarr_matches_spikeglx_on_real_data(tmp_path, compressor):
    spikeglx = segovia.SpikeGlxReader(str(REAL_BIN), str(REAL_META))
    spikeglx_full = np.concatenate(list(spikeglx.chunks(256)), axis=0)
    assert spikeglx_full.shape == (spikeglx.n_samples, spikeglx.n_channels)

    store = tmp_path / "real.zarr"
    _write_zarr(store, spikeglx_full, spikeglx.sample_rate, compressor)

    zarr_reader = segovia.ZarrReader(str(store))
    assert zarr_reader.n_channels == spikeglx.n_channels
    assert zarr_reader.n_samples == spikeglx.n_samples
    assert zarr_reader.sample_rate == spikeglx.sample_rate

    zarr_full = np.concatenate(list(zarr_reader.chunks(256)), axis=0)
    assert np.array_equal(zarr_full, spikeglx_full)
