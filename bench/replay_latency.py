import argparse
import json
import threading
import time

import numpy as np
import psutil
import segovia
from scipy import signal


def design_sos(order, fmin, fmax, fs):
    return np.ascontiguousarray(
        signal.butter(order, [fmin, fmax], btype="band", fs=fs, output="sos"),
        dtype=np.float64,
    )


def open_reader(args):
    if args.kind == "synthetic":
        return segovia.SyntheticEphysReader(
            n_channels=args.n_channels,
            duration_s=args.duration_s,
            sample_rate=args.sample_rate,
            n_units=args.n_units,
            firing_rate=args.firing_rate,
            noise_uv=args.noise_uv,
            seed=args.seed,
        )
    if args.kind == "cbin":
        return segovia.CbinReader(args.cbin, args.ch)
    if args.kind == "spikeglx":
        return segovia.SpikeGlxReader(args.bin, args.meta)
    raise ValueError(args.kind)


def start_rss_sampler(stop_event, peak):
    proc = psutil.Process()

    def sample():
        while not stop_event.is_set():
            try:
                peak["rss"] = max(peak["rss"], proc.memory_info().rss)
            except psutil.Error:
                break
            time.sleep(0.02)

    th = threading.Thread(target=sample)
    th.start()
    return th


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--kind", default="synthetic",
                   choices=["synthetic", "cbin", "spikeglx"])
    p.add_argument("--cbin")
    p.add_argument("--ch")
    p.add_argument("--bin")
    p.add_argument("--meta")
    p.add_argument("--n-channels", type=int, default=384)
    p.add_argument("--duration-s", type=float, default=60.0)
    p.add_argument("--sample-rate", type=float, default=30000.0)
    p.add_argument("--n-units", type=int, default=20)
    p.add_argument("--firing-rate", type=float, default=5.0)
    p.add_argument("--noise-uv", type=float, default=10.0)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--chunk-samples", type=int, default=30000)
    p.add_argument("--margin", type=int, default=1500)
    p.add_argument("--calib-samples", type=int, default=60000)
    p.add_argument("--order", type=int, default=5)
    p.add_argument("--fmin", type=float, default=300.0)
    p.add_argument("--fmax", type=float, default=6000.0)
    p.add_argument("--eps", type=float, default=1e-6)
    p.add_argument("--batch", type=int, default=1)
    p.add_argument("--no-whiten", action="store_true")
    p.add_argument("--warmup", type=int, default=3)
    p.add_argument("--limit-samples", type=int, default=0)
    p.add_argument("--json-only", action="store_true")
    return p.parse_args()


def main():
    args = parse_args()
    reader = open_reader(args)
    fs = reader.sample_rate
    sos = design_sos(args.order, args.fmin, args.fmax, fs)

    period_ms = args.chunk_samples / fs * 1000.0
    lookahead_ms = args.margin / fs * 1000.0

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

    peak = {"rss": 0}
    stop_event = threading.Event()
    sampler = start_rss_sampler(stop_event, peak)

    latencies_ms = []
    nrows = 0
    acc = 0.0
    idx = 0
    it = iter(pre)
    wall_start = time.perf_counter()
    while True:
        t_start = time.perf_counter_ns()
        try:
            chunk = next(it)
        except StopIteration:
            break
        t_end = time.perf_counter_ns()
        lat_ms = (t_end - t_start) / 1e6
        if idx >= args.warmup:
            latencies_ms.append(lat_ms)
        acc += float(chunk.sum())
        nrows += chunk.shape[0]
        idx += 1
        if args.limit_samples and nrows >= args.limit_samples:
            break
    wall_s = time.perf_counter() - wall_start

    stop_event.set()
    sampler.join()

    lat = np.asarray(latencies_ms, dtype=np.float64)
    data_mb = nrows * reader.n_channels * 2 / 1e6
    throughput_mbps = data_mb / wall_s if wall_s > 0 else 0.0
    deadline_ok = float(np.mean(lat <= period_ms)) if lat.size else 0.0

    payload = {
        "engine": "segovia",
        "kind": args.kind,
        "n_samples": nrows,
        "n_channels": reader.n_channels,
        "fs": fs,
        "chunk_samples": args.chunk_samples,
        "batch": args.batch,
        "period_ms": period_ms,
        "lookahead_ms": lookahead_ms,
        "n_chunks_measured": int(lat.size),
        "latency_ms": {
            "mean": float(lat.mean()) if lat.size else None,
            "sd": float(lat.std(ddof=1)) if lat.size > 1 else None,
            "median": float(np.median(lat)) if lat.size else None,
            "p95": float(np.percentile(lat, 95)) if lat.size else None,
            "p99": float(np.percentile(lat, 99)) if lat.size else None,
            "max": float(lat.max()) if lat.size else None,
        },
        "jitter_ms": float(lat.std(ddof=1)) if lat.size > 1 else None,
        "throughput_mbps": throughput_mbps,
        "peak_rss_gb": peak["rss"] / 1e9,
        "deadline_adherence": deadline_ok,
        "wall_s": wall_s,
        "checksum": acc,
    }
    print(json.dumps(payload))

    if args.json_only:
        return

    lm = payload["latency_ms"]
    print("\n=== replay-at-acquisition-rate (Segovia) ===", flush=True)
    print(f"source              {args.kind}")
    print(f"n_samples           {nrows}  ({nrows / fs:.1f} s @ {fs:.0f} Hz)")
    print(f"n_channels          {reader.n_channels}")
    print(f"chunk_samples       {args.chunk_samples}  batch={args.batch}")
    print(f"period (deadline)   {period_ms:.2f} ms / chunk")
    print(f"filter look-ahead   {lookahead_ms:.2f} ms (zero-phase sosfiltfilt margin)")
    print(f"chunks measured     {lat.size} (warmup {args.warmup} discarded)")
    print("latency ms          "
          f"mean={lm['mean']:.2f} sd={lm['sd']:.2f} median={lm['median']:.2f} "
          f"p95={lm['p95']:.2f} p99={lm['p99']:.2f} max={lm['max']:.2f}")
    print(f"throughput          {throughput_mbps:.1f} MB/s")
    print(f"peak RSS            {payload['peak_rss_gb']:.3f} GB")
    print(f"deadline adherence  {deadline_ok * 100:.1f}% of chunks <= period")


if __name__ == "__main__":
    main()
