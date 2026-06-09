import shutil
from pathlib import Path

import numpy as np
import zarr
from mtscomp import compress

HERE = Path(__file__).parent


def reset(path):
    if path.exists():
        shutil.rmtree(path)


def make_mini_cbin():
    raw = HERE / "mini_int16.bin"
    cbin = HERE / "mini_int16.cbin"
    ch = HERE / "mini_int16.ch"
    for p in (raw, cbin, ch):
        if p.exists():
            p.unlink()

    n_samples, n_channels = 10, 4
    data = np.empty((n_samples, n_channels), dtype=np.int16)
    for s in range(n_samples):
        for c in range(n_channels):
            data[s, c] = s * 10 + c
    data.tofile(raw)

    compress(
        str(raw),
        str(cbin),
        str(ch),
        sample_rate=30000.0,
        n_channels=n_channels,
        dtype=np.int16,
        chunk_duration=0.0001,
    )
    raw.unlink()


def make_mini_int16():
    path = HERE / "mini_int16.zarr"
    reset(path)
    n_samples, n_channels = 10, 4
    data = np.empty((n_samples, n_channels), dtype=np.int16)
    for s in range(n_samples):
        for c in range(n_channels):
            data[s, c] = s * 10 + c

    group = zarr.open_group(store=str(path), mode="w", zarr_format=3)
    arr = group.create_array(
        name="traces",
        shape=(n_samples, n_channels),
        chunks=(3, n_channels),
        dtype="int16",
        compressors=[zarr.codecs.GzipCodec(level=5)],
    )
    arr[:] = data
    arr.attrs["sampling_frequency"] = 30000.0


def make_reject_float32():
    path = HERE / "reject_float32.zarr"
    reset(path)
    group = zarr.open_group(store=str(path), mode="w", zarr_format=3)
    group.create_array(
        name="traces",
        shape=(4, 2),
        chunks=(2, 2),
        dtype="float32",
    )


if __name__ == "__main__":
    make_mini_int16()
    make_reject_float32()
    make_mini_cbin()
    print("fixtures written to", HERE)
