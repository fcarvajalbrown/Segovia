import argparse
import json
import time

import numpy as np
import spikeinterface.preprocessing as sp
from spikeinterface.core.job_tools import ChunkRecordingExecutor
from spikeinterface.extractors import read_cbin_ibl


def chain_init(rec):
    return {"rec": rec}


def chain_func(segment_index, start_frame, end_frame, worker_ctx):
    rec = worker_ctx["rec"]
    tr = rec.get_traces(
        segment_index=segment_index, start_frame=start_frame, end_frame=end_frame
    )
    return float(np.asarray(tr).sum())


def open_recording(args):
    if args.kind == "cbin":
        return read_cbin_ibl(
            cbin_file_path=args.cbin,
            stream_name=args.stream,
            load_sync_channel=args.load_sync,
        )
    raise ValueError(args.kind)


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--kind", required=True, choices=["cbin"])
    p.add_argument("--cbin")
    p.add_argument("--stream", default="ap")
    p.add_argument("--load-sync", action="store_true")
    p.add_argument("--chunk-samples", type=int, default=30000)
    p.add_argument("--order", type=int, default=5)
    p.add_argument("--fmin", type=float, default=300.0)
    p.add_argument("--fmax", type=float, default=6000.0)
    p.add_argument("--n-jobs", type=int, default=8)
    p.add_argument("--pool-engine", default="thread", choices=["thread", "process"])
    p.add_argument("--no-whiten", action="store_true")
    p.add_argument("--limit-samples", type=int, default=0)
    return p.parse_args()


def main():
    args = parse_args()
    rec = open_recording(args)
    if args.limit_samples:
        rec = rec.frame_slice(start_frame=0, end_frame=args.limit_samples)

    t0 = time.perf_counter()
    rec_f = sp.bandpass_filter(
        rec,
        freq_min=args.fmin,
        freq_max=args.fmax,
        filter_order=args.order,
        ftype="butter",
    )
    rec_c = sp.common_reference(rec_f, operator="median")
    rec_w = rec_c if args.no_whiten else sp.whiten(
        rec_c, mode="global", apply_mean=True, dtype="float32"
    )

    mp_context = "spawn" if args.pool_engine == "process" else None
    executor = ChunkRecordingExecutor(
        rec_w,
        chain_func,
        chain_init,
        (rec_w,),
        handle_returns=True,
        n_jobs=args.n_jobs,
        pool_engine=args.pool_engine,
        mp_context=mp_context,
        chunk_size=args.chunk_samples,
        progress_bar=False,
        job_name="bench",
    )
    results = executor.run()
    t1 = time.perf_counter()

    acc = float(np.sum([r for r in results]))
    print(
        json.dumps(
            {
                "engine": f"spikeinterface-{args.pool_engine}",
                "wall_s": t1 - t0,
                "n_samples": int(rec_w.get_num_frames()),
                "n_channels": int(rec_w.get_num_channels()),
                "fs": float(rec_w.get_sampling_frequency()),
                "n_jobs": args.n_jobs,
                "checksum": acc,
            }
        )
    )


if __name__ == "__main__":
    main()
