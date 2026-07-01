import argparse
import json
import time

import numpy as np
import segovia
from scipy import signal


def design_sos(order, fmin, fmax, fs):
    return np.ascontiguousarray(
        signal.butter(order, [fmin, fmax], btype="band", fs=fs, output="sos"),
        dtype=np.float64,
    )


def open_reader(args):
    if args.kind == "cbin":
        return segovia.CbinReader(args.cbin, args.ch)
    if args.kind == "spikeglx":
        return segovia.SpikeGlxReader(args.bin, args.meta)
    raise ValueError(args.kind)


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--kind", required=True, choices=["cbin", "spikeglx"])
    p.add_argument("--cbin")
    p.add_argument("--ch")
    p.add_argument("--bin")
    p.add_argument("--meta")
    p.add_argument("--chunk-samples", type=int, default=30000)
    p.add_argument("--margin", type=int, default=1500)
    p.add_argument("--calib-samples", type=int, default=60000)
    p.add_argument("--order", type=int, default=5)
    p.add_argument("--fmin", type=float, default=300.0)
    p.add_argument("--fmax", type=float, default=6000.0)
    p.add_argument("--eps", type=float, default=1e-6)
    p.add_argument("--batch", type=int, default=4)
    p.add_argument("--no-whiten", action="store_true")
    p.add_argument("--limit-samples", type=int, default=0)
    return p.parse_args()


def main():
    args = parse_args()
    reader = open_reader(args)
    fs = reader.sample_rate
    sos = design_sos(args.order, args.fmin, args.fmax, fs)

    t0 = time.perf_counter()
    pre = reader.preprocess(
        sos,
        chunk_samples=args.chunk_samples,
        margin=args.margin,
        calib_samples=args.calib_samples,
        eps=args.eps,
        apply_mean=True,
        batch=args.batch,
        whiten=not args.no_whiten,
    )
    acc = 0.0
    nrows = 0
    for chunk in pre:
        acc += float(chunk.sum())
        nrows += chunk.shape[0]
        if args.limit_samples and nrows >= args.limit_samples:
            break
    t1 = time.perf_counter()

    print(
        json.dumps(
            {
                "engine": "segovia",
                "wall_s": t1 - t0,
                "n_samples": nrows,
                "n_channels": reader.n_channels,
                "fs": fs,
                "checksum": acc,
            }
        )
    )


if __name__ == "__main__":
    main()
