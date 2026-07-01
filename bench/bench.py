import argparse
import json
import subprocess
import threading
import time

import psutil

VENV = r"C:\Projects\Segovia\.venv\Scripts\python.exe"
VENV_SI = r"C:\Projects\Segovia\.venv-si\Scripts\python.exe"
SEG_SCRIPT = r"C:\Projects\Segovia\bench\segovia_chain.py"
SI_SCRIPT = r"C:\Projects\Segovia\bench\spikeinterface_chain.py"


def run_monitored(cmd):
    proc = subprocess.Popen(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )
    parent = psutil.Process(proc.pid)
    peak = {"rss": 0}

    def sample():
        while proc.poll() is None:
            try:
                rss = parent.memory_info().rss
                for child in parent.children(recursive=True):
                    try:
                        rss += child.memory_info().rss
                    except psutil.Error:
                        pass
                peak["rss"] = max(peak["rss"], rss)
            except psutil.Error:
                break
            time.sleep(0.05)

    th = threading.Thread(target=sample)
    th.start()
    out, err = proc.communicate()
    th.join()

    payload = None
    for line in out.strip().splitlines():
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            payload = json.loads(line)
    return proc.returncode, payload, err, peak["rss"]


def segovia_cmd(args):
    cmd = [VENV, SEG_SCRIPT, "--kind", args.kind]
    if args.kind == "cbin":
        cmd += ["--cbin", args.cbin, "--ch", args.ch]
    else:
        cmd += ["--bin", args.bin, "--meta", args.meta]
    cmd += [
        "--chunk-samples", str(args.chunk_samples),
        "--margin", str(args.margin),
        "--calib-samples", str(args.calib_samples),
        "--order", str(args.order),
        "--fmin", str(args.fmin),
        "--fmax", str(args.fmax),
        "--batch", str(args.batch),
    ]
    if args.limit_samples:
        cmd += ["--limit-samples", str(args.limit_samples)]
    if args.no_whiten:
        cmd += ["--no-whiten"]
    return cmd


def si_cmd(args, pool_engine):
    cmd = [
        VENV_SI, SI_SCRIPT,
        "--kind", "cbin",
        "--cbin", args.cbin,
        "--stream", args.stream,
        "--chunk-samples", str(args.chunk_samples),
        "--order", str(args.order),
        "--fmin", str(args.fmin),
        "--fmax", str(args.fmax),
        "--n-jobs", str(args.n_jobs),
        "--pool-engine", pool_engine,
        "--load-sync",
    ]
    if args.limit_samples:
        cmd += ["--limit-samples", str(args.limit_samples)]
    if args.no_whiten:
        cmd += ["--no-whiten"]
    return cmd


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--kind", default="cbin", choices=["cbin", "spikeglx"])
    p.add_argument("--cbin")
    p.add_argument("--ch")
    p.add_argument("--bin")
    p.add_argument("--meta")
    p.add_argument("--stream", default="ap")
    p.add_argument("--chunk-samples", type=int, default=30000)
    p.add_argument("--margin", type=int, default=1500)
    p.add_argument("--calib-samples", type=int, default=60000)
    p.add_argument("--order", type=int, default=5)
    p.add_argument("--fmin", type=float, default=300.0)
    p.add_argument("--fmax", type=float, default=6000.0)
    p.add_argument("--n-jobs", type=int, default=8)
    p.add_argument("--batch", type=int, default=4)
    p.add_argument("--limit-samples", type=int, default=0)
    p.add_argument("--no-whiten", action="store_true")
    p.add_argument(
        "--engines",
        default="segovia,si-thread,si-process",
    )
    return p.parse_args()


def main():
    args = parse_args()
    engines = args.engines.split(",")
    rows = []

    plans = []
    if "segovia" in engines:
        plans.append(("segovia", segovia_cmd(args)))
    if "si-thread" in engines:
        plans.append(("spikeinterface-thread", si_cmd(args, "thread")))
    if "si-process" in engines:
        plans.append(("spikeinterface-process", si_cmd(args, "process")))

    for name, cmd in plans:
        print(f"running {name} ...", flush=True)
        code, payload, err, peak = run_monitored(cmd)
        if code != 0 or payload is None:
            print(f"  FAILED (exit {code})")
            print(err[-2000:])
            rows.append((name, None, None, None, None))
            continue
        wall = payload["wall_s"]
        n_samp = payload["n_samples"]
        n_ch = payload["n_channels"]
        data_mb = n_samp * n_ch * 2 / 1e6
        mbps = data_mb / wall if wall > 0 else 0.0
        peak_gb = peak / 1e9
        rows.append((name, wall, mbps, peak_gb, payload.get("checksum")))
        print(
            f"  wall={wall:.2f}s  throughput={mbps:.1f} MB/s  "
            f"peak_tree_RSS={peak_gb:.3f} GB  samples={n_samp}"
        )

    print("\n=== SC1 comparison ===")
    print(f"{'engine':<26}{'wall_s':>10}{'MB/s':>10}{'peak_GB':>10}")
    for name, wall, mbps, peak_gb, _ in rows:
        if wall is None:
            print(f"{name:<26}{'FAIL':>10}")
            continue
        print(f"{name:<26}{wall:>10.2f}{mbps:>10.1f}{peak_gb:>10.3f}")

    seg = next((r for r in rows if r[0] == "segovia" and r[1] is not None), None)
    if seg:
        seg_wall, seg_peak = seg[1], seg[3]
        mem_ok = seg_peak < 2.0
        faster = []
        for name, wall, _, _, _ in rows:
            if name.startswith("spikeinterface") and wall is not None:
                faster.append((name, wall / seg_wall))
        print("\nSegovia peak memory < 2 GB:", "PASS" if mem_ok else "FAIL",
              f"({seg_peak:.3f} GB)")
        for name, ratio in faster:
            verdict = "faster" if ratio > 1 else "SLOWER"
            print(f"Segovia vs {name}: {ratio:.2f}x ({verdict})")


if __name__ == "__main__":
    main()
