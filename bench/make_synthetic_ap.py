import argparse
from pathlib import Path

import numpy as np
from mtscomp import compress

FS = 30000
N_NEURAL = 384
N_TOTAL = 385


def imro_table():
    sites = "".join(f"({i} 0 0 500 250 1)" for i in range(N_NEURAL))
    return f"(0,{N_NEURAL}){sites}"


def chan_map():
    sites = "".join(f"(AP{i};{i}:{i})" for i in range(N_NEURAL))
    return f"({N_NEURAL},0,1){sites}(SY0;{N_NEURAL}:{N_NEURAL})"


def write_meta(meta_path, n_samples, file_size_bytes):
    lines = [
        "acqApLfSy=384,384,1",
        "appVersion=20190413",
        "fileName=synthetic_ap.bin",
        f"fileSizeBytes={file_size_bytes}",
        f"fileTimeSecs={n_samples / FS:.4f}",
        "gateMode=Immediate",
        "imAiRangeMax=0.6",
        "imAiRangeMin=-0.6",
        "imDatPrb_type=0",
        f"imSampRate={FS}",
        f"nSavedChans={N_TOTAL}",
        "snsApLfSy=384,0,1",
        "snsSaveChanSubset=0:384",
        "trigMode=Immediate",
        "typeThis=imec",
        f"~imroTbl={imro_table()}",
        f"~snsChanMap={chan_map()}",
    ]
    meta_path.write_text("\n".join(lines) + "\n")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--seconds", type=float, default=10.0)
    p.add_argument("--out-dir", default=r"C:\Projects\Segovia\tests\data")
    args = p.parse_args()

    n_samples = int(args.seconds * FS)
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    stem = "synthetic_ap_g0_t0.imec0.ap"
    raw = out / f"{stem}.bin"
    cbin = out / f"{stem}.cbin"
    ch = out / f"{stem}.ch"
    meta = out / f"{stem}.meta"

    rng = np.random.default_rng(7)
    t = np.arange(n_samples) / FS
    data = np.empty((n_samples, N_TOTAL), dtype=np.int16)
    block = 60000
    for s0 in range(0, n_samples, block):
        s1 = min(s0 + block, n_samples)
        tt = t[s0:s1][:, None]
        neural = (
            300 * np.sin(2 * np.pi * 1000 * tt + np.arange(N_NEURAL) * 0.1)
            + 150 * rng.standard_normal((s1 - s0, N_NEURAL))
        )
        data[s0:s1, :N_NEURAL] = np.clip(neural, -3000, 3000).astype(np.int16)
        data[s0:s1, N_NEURAL] = ((np.arange(s0, s1) // 15000) % 2).astype(np.int16)

    data.tofile(raw)
    write_meta(meta, n_samples, raw.stat().st_size)

    if cbin.exists():
        cbin.unlink()
    if ch.exists():
        ch.unlink()
    compress(
        str(raw),
        str(cbin),
        str(ch),
        sample_rate=FS,
        n_channels=N_TOTAL,
        dtype=np.int16,
    )
    raw.unlink()
    print(f"wrote {cbin} ({cbin.stat().st_size / 1e6:.1f} MB), {n_samples} samples")


if __name__ == "__main__":
    main()
