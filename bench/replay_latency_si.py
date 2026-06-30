import argparse
import json
import threading
import time

import numpy as np
import psutil
import spikeinterface.preprocessing as sp
from spikeinterface.extractors import BinaryRecordingExtractor, read_cbin_ibl


def open_recording(args):
    if args.kind == "cbin":
        return read_cbin_ibl(
            cbin_file_path=args.cbin,
            stream_name=args.stream,
            load_sync_channel=args.load_sync,
        )
    if args.kind == "binary":
        return BinaryRecordingExtractor(
            file_paths=args.bin,
            sampling_frequency=args.sample_rate,
            num_channels=args.n_channels,
            dtype="int16",
            time_axis=0,
        )
    raise ValueError(args.kind)


def build_chain(rec, args):
    rec_f = sp.bandpass_filter(
        rec,
        freq_min=args.fmin,
        freq_max=args.fmax,
        filter_order=args.order,
        ftype="butter",
        margin_ms=args.margin_ms,
    )
    rec_c = sp.common_reference(rec_f, operator="median")
    if args.no_whiten:
        return rec_c
    return sp.whiten(rec_c, mode="global", apply_mean=True, dtype="float32")


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
    p.add_argument("--kind", required=True, choices=["cbin", "binary"])
    p.add_argument("--cbin")
    p.add_argument("--stream", default="ap")
    p.add_argument("--load-sync", action="store_true")
    p.add_argument("--bin")
    p.add_argument("--n-channels", type=int, default=384)
    p.add_argument("--sample-rate", type=float, default=30000.0)
    p.add_argument("--chunk-samples", type=int, default=30000)
    p.add_argument("--margin-ms", type=float, default=50.0)
    p.add_argument("--order", type=int, default=5)
    p.add_argument("--fmin", type=float, default=300.0)
    p.add_argument("--fmax", type=float, default=6000.0)
    p.add_argument("--no-whiten", action="store_true")
    p.add_argument("--warmup", type=int, default=3)
    p.add_argument("--limit-samples", type=int, default=0)
    p.add_argument("--json-only", action="store_true")
    return p.parse_args()


def main():
    args = parse_args()
    rec = open_recording(args)
    rec_w = build_chain(rec, args)

    fs = float(rec_w.get_sampling_frequency())
    n_channels = int(rec_w.get_num_channels())
    total = int(rec_w.get_num_frames())
    if args.limit_samples:
        total = min(total, args.limit_samples)

    period_ms = args.chunk_samples / fs * 1000.0
    lookahead_ms = args.margin_ms

    peak = {"rss": 0}
    stop_event = threading.Event()
    sampler = start_rss_sampler(stop_event, peak)

    latencies_ms = []
    nrows = 0
    acc = 0.0
    idx = 0
    wall_start = time.perf_counter()
    s0 = 0
    while s0 < total:
        s1 = min(s0 + args.chunk_samples, total)
        t_start = time.perf_counter_ns()
        tr = rec_w.get_traces(segment_index=0, start_frame=s0, end_frame=s1)
        t_end = time.perf_counter_ns()
        lat_ms = (t_end - t_start) / 1e6
        if idx >= args.warmup:
            latencies_ms.append(lat_ms)
        acc += float(np.asarray(tr).sum())
        nrows += tr.shape[0]
        idx += 1
        s0 = s1
    wall_s = time.perf_counter() - wall_start

    stop_event.set()
    sampler.join()

    lat = np.asarray(latencies_ms, dtype=np.float64)
    data_mb = nrows * n_channels * 2 / 1e6
    throughput_mbps = data_mb / wall_s if wall_s > 0 else 0.0
    deadline_ok = float(np.mean(lat <= period_ms)) if lat.size else 0.0

    payload = {
        "engine": "spikeinterface-online",
        "kind": args.kind,
        "n_samples": nrows,
        "n_channels": n_channels,
        "fs": fs,
        "chunk_samples": args.chunk_samples,
        "batch": 1,
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
    print("\n=== replay-at-acquisition-rate (SpikeInterface online) ===", flush=True)
    print(f"source              {args.kind}")
    print(f"n_samples           {nrows}  ({nrows / fs:.1f} s @ {fs:.0f} Hz)")
    print(f"n_channels          {n_channels}")
    print(f"chunk_samples       {args.chunk_samples}  batch=1")
    print(f"period (deadline)   {period_ms:.2f} ms / chunk")
    print(f"filter look-ahead   {lookahead_ms:.2f} ms (bandpass margin_ms)")
    print(f"chunks measured     {lat.size} (warmup {args.warmup} discarded)")
    print("latency ms          "
          f"mean={lm['mean']:.2f} sd={lm['sd']:.2f} median={lm['median']:.2f} "
          f"p95={lm['p95']:.2f} p99={lm['p99']:.2f} max={lm['max']:.2f}")
    print(f"throughput          {throughput_mbps:.1f} MB/s")
    print(f"peak RSS            {payload['peak_rss_gb']:.3f} GB")
    print(f"deadline adherence  {deadline_ok * 100:.1f}% of chunks <= period")


if __name__ == "__main__":
    main()
