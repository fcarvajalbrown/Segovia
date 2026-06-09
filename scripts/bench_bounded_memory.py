import argparse
import glob
import threading
import time
from pathlib import Path

import psutil

import segovia

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "tests" / "data"
PEAK_LIMIT_MB = 2048.0


def find_cbin(explicit):
    if explicit:
        return Path(explicit)
    matches = sorted(glob.glob(str(DATA / "*.lf.cbin"))) or sorted(
        glob.glob(str(DATA / "*.cbin"))
    )
    if not matches:
        raise SystemExit(
            f"no .cbin found under {DATA}; pass one explicitly or run download_ibl_lf.py"
        )
    return Path(matches[0])


class PeakSampler(threading.Thread):
    def __init__(self, interval=0.02):
        super().__init__(daemon=True)
        self.interval = interval
        self.process = psutil.Process()
        self.peak_bytes = 0
        self._stop = threading.Event()

    def run(self):
        while not self._stop.is_set():
            rss = self.process.memory_info().rss
            if rss > self.peak_bytes:
                self.peak_bytes = rss
            time.sleep(self.interval)

    def stop(self):
        rss = self.process.memory_info().rss
        if rss > self.peak_bytes:
            self.peak_bytes = rss
        self._stop.set()
        self.join()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("cbin", nargs="?", default=None)
    parser.add_argument("--ch", default=None)
    parser.add_argument("--chunk-samples", type=int, default=65536)
    args = parser.parse_args()

    cbin = find_cbin(args.cbin)
    ch = Path(args.ch) if args.ch else cbin.with_suffix(".ch")

    reader = segovia.CbinReader(str(cbin), str(ch))
    n_samples = reader.n_samples
    n_channels = reader.n_channels
    sample_rate = reader.sample_rate
    duration_s = n_samples / sample_rate if sample_rate else 0.0
    decompressed_bytes = n_samples * n_channels * 2

    print(f"file:         {cbin.name}")
    print(f"channels:     {n_channels}")
    print(f"samples:      {n_samples:,}  ({duration_s / 60:.1f} min @ {sample_rate:g} Hz)")
    print(f"decompressed: {decompressed_bytes / 1e9:.2f} GB")
    print(f"chunk_samples:{args.chunk_samples:,}")

    sampler = PeakSampler()
    sampler.start()
    start = time.perf_counter()

    seen = 0
    for chunk in reader.chunks(args.chunk_samples):
        seen += chunk.shape[0]

    elapsed = time.perf_counter() - start
    sampler.stop()

    peak_mb = sampler.peak_bytes / 1e6
    throughput = decompressed_bytes / 1e6 / elapsed if elapsed else 0.0

    print("-" * 48)
    print(f"samples streamed: {seen:,}  (expected {n_samples:,})")
    print(f"wall time:        {elapsed:.2f} s")
    print(f"throughput:       {throughput:.1f} MB/s (decompressed)")
    print(f"peak RSS:         {peak_mb:.1f} MB")
    print("-" * 48)

    if seen != n_samples:
        raise SystemExit("FAIL: streamed sample count does not match n_samples")
    if peak_mb >= PEAK_LIMIT_MB:
        raise SystemExit(f"FAIL: peak RSS {peak_mb:.1f} MB >= {PEAK_LIMIT_MB:.0f} MB")
    print(f"PASS: streamed {duration_s / 60:.1f} min in {peak_mb:.1f} MB (< {PEAK_LIMIT_MB:.0f} MB)")


if __name__ == "__main__":
    main()
