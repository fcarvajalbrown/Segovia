import argparse
from pathlib import Path

import numpy as np
import segovia


def write_meta(meta_path, n_channels, sample_rate, file_size_bytes, n_samples):
    lines = [
        "typeThis=imec",
        f"nSavedChans={n_channels}",
        f"imSampRate={sample_rate:.1f}",
        f"fileSizeBytes={file_size_bytes}",
        f"fileTimeSecs={n_samples / sample_rate:.4f}",
    ]
    meta_path.write_text("\n".join(lines) + "\n")


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--out-dir", default=r"C:\Projects\Segovia\bench\_tmp")
    p.add_argument("--stem", default="synthetic_replay")
    p.add_argument("--n-channels", type=int, default=384)
    p.add_argument("--duration-s", type=float, default=60.0)
    p.add_argument("--sample-rate", type=float, default=30000.0)
    p.add_argument("--n-units", type=int, default=20)
    p.add_argument("--firing-rate", type=float, default=5.0)
    p.add_argument("--noise-uv", type=float, default=10.0)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--chunk-samples", type=int, default=30000)
    return p.parse_args()


def main():
    args = parse_args()
    reader = segovia.SyntheticEphysReader(
        n_channels=args.n_channels,
        duration_s=args.duration_s,
        sample_rate=args.sample_rate,
        n_units=args.n_units,
        firing_rate=args.firing_rate,
        noise_uv=args.noise_uv,
        seed=args.seed,
    )

    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    bin_path = out / f"{args.stem}.bin"
    meta_path = out / f"{args.stem}.meta"

    written = 0
    with open(bin_path, "wb") as f:
        for chunk in reader.chunks(args.chunk_samples):
            f.write(np.ascontiguousarray(chunk, dtype=np.int16).tobytes())
            written += chunk.shape[0]

    size = bin_path.stat().st_size
    write_meta(meta_path, reader.n_channels, reader.sample_rate, size, written)
    print(
        f"wrote {bin_path} ({size / 1e6:.1f} MB), "
        f"{written} samples x {reader.n_channels} ch @ {reader.sample_rate:.0f} Hz"
    )


if __name__ == "__main__":
    main()
