import argparse
import collections
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
    if args.kind == "cbin":
        return segovia.CbinReader(args.cbin, args.ch)
    if args.kind == "spikeglx":
        return segovia.SpikeGlxReader(args.bin, args.meta)
    if args.kind == "synthetic":
        return segovia.SyntheticEphysReader(
            n_channels=args.n_channels,
            duration_s=args.synthetic_seconds,
            sample_rate=args.sample_rate,
            n_units=args.n_units,
            firing_rate=args.firing_rate,
            noise_uv=args.noise_uv,
            seed=args.seed,
        )
    raise ValueError(args.kind)


class Stream:
    def __init__(self, args):
        self.args = args
        self.reader = open_reader(args)
        self.fs = self.reader.sample_rate
        self.n_channels = self.reader.n_channels
        self.period_s = args.chunk_samples / self.fs
        self.shown = np.linspace(
            0, self.n_channels - 1, args.channels_shown, dtype=int
        )
        self.queue = collections.deque()
        self.lock = threading.Lock()
        self.stop_event = threading.Event()
        self.done = False
        self.peak_rss = 0
        self.proc = psutil.Process()

    def run(self):
        sos = design_sos(
            self.args.order, self.args.fmin, self.args.fmax, self.fs
        )
        pre = self.reader.preprocess(
            sos,
            chunk_samples=self.args.chunk_samples,
            margin=self.args.margin,
            calib_samples=self.args.calib_samples,
            batch=1,
            whiten=not self.args.no_whiten,
        )
        limit = int(self.args.limit_seconds * self.fs)
        thr = -abs(self.args.threshold)
        produced = 0
        it = iter(pre)
        while not self.stop_event.is_set():
            t0 = time.perf_counter_ns()
            try:
                chunk = next(it)
            except StopIteration:
                break
            t_ready = time.perf_counter_ns()
            compute_ms = (t_ready - t0) / 1e6
            sub = np.ascontiguousarray(chunk[:, self.shown], dtype=np.float32)
            hits = np.argwhere(sub < thr)
            trigger_ms = (time.perf_counter_ns() - t_ready) / 1e6
            self.peak_rss = max(self.peak_rss, self.proc.memory_info().rss)
            with self.lock:
                self.queue.append(
                    {
                        "data": sub,
                        "start": produced,
                        "hits": hits,
                        "compute_ms": compute_ms,
                        "trigger_ms": trigger_ms,
                    }
                )
            produced += sub.shape[0]
            if limit and produced >= limit:
                break
            slack = self.period_s - (time.perf_counter_ns() - t0) / 1e9
            if slack > 0:
                time.sleep(slack)
        self.done = True


def build_figure(stream, args):
    import matplotlib.pyplot as plt

    fig, (ax_tr, ax_tx) = plt.subplots(
        2, 1, figsize=(11, 7), height_ratios=[5, 1],
        gridspec_kw={"hspace": 0.25},
    )
    fig.suptitle(
        f"Segovia live monitor — {args.kind} — "
        f"{stream.n_channels} ch @ {stream.fs:.0f} Hz — "
        f"{args.chunk_samples/stream.fs*1000:.0f} ms chunks",
        fontsize=11,
    )

    window = int(args.window_seconds * stream.fs)
    n_shown = len(stream.shown)
    offset = args.trace_offset
    ax_tr.set_xlim(0, args.window_seconds)
    ax_tr.set_ylim(-offset, offset * n_shown)
    ax_tr.set_xlabel("time (s, rolling)")
    ax_tr.set_yticks([offset * i for i in range(n_shown)])
    ax_tr.set_yticklabels([f"ch {c}" for c in stream.shown])
    t_axis = np.arange(window) / stream.fs
    lines = [
        ax_tr.plot([], [], lw=0.5, color="#1b3a6b")[0] for _ in range(n_shown)
    ]
    marks = ax_tr.scatter([], [], s=14, color="#CE422B", zorder=3)

    ax_tx.axis("off")
    text = ax_tx.text(
        0.0, 0.5, "", family="monospace", fontsize=10, va="center",
        transform=ax_tx.transAxes,
    )

    buf = np.zeros((window, n_shown), dtype=np.float32)
    filled = {"n": 0}
    lat = collections.deque(maxlen=400)
    trg = collections.deque(maxlen=400)
    total = {"rows": 0, "trig": 0}
    wall0 = {"t": None}

    def drain():
        with stream.lock:
            items = list(stream.queue)
            stream.queue.clear()
        return items

    def update(_frame):
        if wall0["t"] is None:
            wall0["t"] = time.perf_counter()
        det_pts = []
        for it in drain():
            d = it["data"]
            n = d.shape[0]
            if n >= window:
                buf[:] = d[-window:]
            else:
                buf[:-n] = buf[n:]
                buf[-n:] = d
            filled["n"] = min(window, filled["n"] + n)
            lat.append(it["compute_ms"])
            trg.append(it["trigger_ms"])
            total["rows"] += n
            total["trig"] += len(it["hits"])

        for i, ln in enumerate(lines):
            ln.set_data(t_axis, buf[:, i] + offset * i)

        recent = buf[:, :]
        hit_idx = np.argwhere(recent < -abs(args.threshold))
        if hit_idx.size:
            xs = t_axis[hit_idx[:, 0]]
            ys = buf[hit_idx[:, 0], hit_idx[:, 1]] + offset * hit_idx[:, 1]
            marks.set_offsets(np.column_stack([xs, ys]))
        else:
            marks.set_offsets(np.empty((0, 2)))

        period_ms = stream.period_s * 1000.0
        lat_arr = np.asarray(lat) if lat else np.zeros(1)
        elapsed = max(1e-9, time.perf_counter() - wall0["t"])
        mbps = total["rows"] * stream.n_channels * 2 / 1e6 / elapsed
        adher = float(np.mean(lat_arr <= period_ms)) * 100.0
        trg_arr = np.asarray(trg) if trg else np.zeros(1)
        text.set_text(
            f"latency   mean {lat_arr.mean():6.1f} ms   "
            f"p95 {np.percentile(lat_arr,95):6.1f} ms   "
            f"deadline({period_ms:.0f}ms) {adher:5.1f}%\n"
            f"trigger   mean {trg_arr.mean():6.2f} ms   "
            f"detections {total['trig']:6d}   "
            f"rate {total['trig']/elapsed:6.1f}/s\n"
            f"throughput {mbps:5.1f} MB/s   "
            f"peak RSS {stream.peak_rss/1e9:5.3f} GB   "
            f"streamed {total['rows']/stream.fs:5.1f} s"
        )
        return lines + [marks, text]

    return fig, update


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--kind", default="cbin",
                   choices=["cbin", "spikeglx", "synthetic"])
    p.add_argument("--cbin",
                   default=r"C:\Projects\Segovia\tests\data\_spikeglx_ephysData_g0_t0.imec0.ap.cbin")
    p.add_argument("--ch",
                   default=r"C:\Projects\Segovia\tests\data\_spikeglx_ephysData_g0_t0.imec0.ap.ch")
    p.add_argument("--bin")
    p.add_argument("--meta")
    p.add_argument("--n-channels", type=int, default=384)
    p.add_argument("--synthetic-seconds", type=float, default=60.0)
    p.add_argument("--sample-rate", type=float, default=30000.0)
    p.add_argument("--n-units", type=int, default=20)
    p.add_argument("--firing-rate", type=float, default=5.0)
    p.add_argument("--noise-uv", type=float, default=10.0)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--chunk-samples", type=int, default=3000)
    p.add_argument("--margin", type=int, default=1500)
    p.add_argument("--calib-samples", type=int, default=60000)
    p.add_argument("--order", type=int, default=5)
    p.add_argument("--fmin", type=float, default=300.0)
    p.add_argument("--fmax", type=float, default=6000.0)
    p.add_argument("--no-whiten", action="store_true")
    p.add_argument("--threshold", type=float, default=5.0)
    p.add_argument("--channels-shown", type=int, default=8)
    p.add_argument("--window-seconds", type=float, default=2.0)
    p.add_argument("--trace-offset", type=float, default=12.0)
    p.add_argument("--limit-seconds", type=float, default=30.0)
    p.add_argument("--interval-ms", type=int, default=50)
    p.add_argument("--save")
    p.add_argument("--snapshot-seconds", type=float, default=6.0)
    args = p.parse_args()

    import matplotlib
    if args.save:
        matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.animation import FuncAnimation

    stream = Stream(args)
    worker = threading.Thread(target=stream.run, daemon=True)
    worker.start()

    fig, update = build_figure(stream, args)

    if args.save:
        deadline = time.perf_counter() + args.snapshot_seconds
        while time.perf_counter() < deadline and not stream.done:
            update(0)
            time.sleep(args.interval_ms / 1000.0)
        update(0)
        fig.savefig(args.save, dpi=140, bbox_inches="tight")
        stream.stop_event.set()
        print(f"saved {args.save}")
        return

    _anim = FuncAnimation(
        fig, update, interval=args.interval_ms, blit=False, cache_frame_data=False
    )
    plt.show()
    stream.stop_event.set()


if __name__ == "__main__":
    main()
