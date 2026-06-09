import numpy as np
import pytest
import segovia


def write_fixture(tmp_path, n_channels, n_samples):
    data = np.empty((n_samples, n_channels), dtype=np.int16)
    for s in range(n_samples):
        for c in range(n_channels):
            data[s, c] = s * 10 + c

    bin_path = tmp_path / "run_g0_t0.imec0.ap.bin"
    meta_path = tmp_path / "run_g0_t0.imec0.ap.meta"
    data.tofile(bin_path)
    meta_path.write_text(
        f"nSavedChans={n_channels}\n"
        f"imSampRate=30000\n"
        f"typeThis=imec\n"
        f"fileSizeBytes={bin_path.stat().st_size}\n"
    )
    return bin_path, meta_path, data


def test_metadata_matches_fixture(tmp_path):
    bin_path, meta_path, _ = write_fixture(tmp_path, 4, 10)
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))
    assert reader.n_channels == 4
    assert reader.n_samples == 10
    assert reader.sample_rate == 30000.0
    assert reader.stream_type == "imec"


def test_raw_fields_exposed(tmp_path):
    bin_path, meta_path, _ = write_fixture(tmp_path, 4, 10)
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))
    assert reader.fields["nSavedChans"] == "4"
    assert reader.fields["typeThis"] == "imec"


def test_meta_path_inferred_from_bin(tmp_path):
    bin_path, _, _ = write_fixture(tmp_path, 4, 10)
    reader = segovia.SpikeGlxReader(str(bin_path))
    assert reader.n_channels == 4


def test_chunks_reconstruct_full_recording(tmp_path):
    bin_path, meta_path, data = write_fixture(tmp_path, 4, 10)
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))

    chunks = list(reader.chunks(3))
    assert [c.shape[0] for c in chunks] == [3, 3, 3, 1]
    assert all(c.shape[1] == 4 for c in chunks)
    assert all(c.dtype == np.int16 for c in chunks)

    reconstructed = np.concatenate(chunks, axis=0)
    assert np.array_equal(reconstructed, data)


def test_chunk_larger_than_recording_yields_one_chunk(tmp_path):
    bin_path, meta_path, data = write_fixture(tmp_path, 4, 10)
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))
    chunks = list(reader.chunks(1000))
    assert len(chunks) == 1
    assert np.array_equal(chunks[0], data)


def test_zero_chunk_size_rejected(tmp_path):
    bin_path, meta_path, _ = write_fixture(tmp_path, 4, 10)
    reader = segovia.SpikeGlxReader(str(bin_path), str(meta_path))
    with pytest.raises(ValueError):
        reader.chunks(0)


def test_stale_declared_size_is_tolerated(tmp_path):
    bin_path, meta_path, data = write_fixture(tmp_path, 4, 10)
    stale_meta = tmp_path / "stale.meta"
    stale_meta.write_text("nSavedChans=4\nimSampRate=30000\nfileSizeBytes=999999\n")
    reader = segovia.SpikeGlxReader(str(bin_path), str(stale_meta))
    assert reader.n_samples == 10
    assert reader.declared_file_size_bytes == 999999
    assert np.array_equal(np.concatenate(list(reader.chunks(4)), axis=0), data)
