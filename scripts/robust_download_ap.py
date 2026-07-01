import os
import shutil
import time

import requests

URL = (
    "https://ibl-brain-wide-map-public.s3.amazonaws.com/data/mainenlab/"
    "Subjects/ZFM-02368/2021-06-01/001/raw_ephys_data/probe00/"
    "_spikeglx_ephysData_g0_t0.imec0.ap.036e9614-d0c6-4ee8-a005-4d4b45c8ea00.cbin"
)
EXPECTED = 29413957992
CACHE = (
    r"C:\Users\Beetlejuice\Downloads\ONE\openalyx.internationalbrainlab.org"
    r"\_spikeglx_ephysData_g0_t0.imec0.ap.036e9614-d0c6-4ee8-a005-4d4b45c8ea00.cbin"
)
DEST = r"C:\Projects\Segovia\tests\data\_spikeglx_ephysData_g0_t0.imec0.ap.cbin"
GB = 1024 ** 3


def download():
    pos = os.path.getsize(CACHE) if os.path.exists(CACHE) else 0
    if pos >= EXPECTED:
        return pos
    headers = {"Range": f"bytes={pos}-"} if pos else {}
    with requests.get(URL, headers=headers, stream=True, timeout=120) as r:
        if pos and r.status_code != 206:
            pos = 0
            mode = "wb"
        else:
            mode = "ab" if pos else "wb"
        r.raise_for_status()
        done = pos
        mark = done
        with open(CACHE, mode) as f:
            for chunk in r.iter_content(chunk_size=8 * 1024 * 1024):
                if not chunk:
                    continue
                f.write(chunk)
                done += len(chunk)
                if done - mark >= GB:
                    mark = done
                    print(f"{done / 1e9:.1f} / {EXPECTED / 1e9:.1f} GB", flush=True)
    return os.path.getsize(CACHE)


def main():
    tries = 0
    while tries < 500:
        size = os.path.getsize(CACHE) if os.path.exists(CACHE) else 0
        if size >= EXPECTED:
            break
        tries += 1
        try:
            download()
        except Exception as e:
            here = os.path.getsize(CACHE) if os.path.exists(CACHE) else 0
            print(
                f"attempt {tries}: {type(e).__name__} at {here / 1e9:.2f} GB, resuming",
                flush=True,
            )
            time.sleep(3)
    final = os.path.getsize(CACHE) if os.path.exists(CACHE) else 0
    print(f"final {final} expected {EXPECTED}", flush=True)
    if final == EXPECTED:
        shutil.copy2(CACHE, DEST)
        print(f"copied complete file to {DEST}", flush=True)
    else:
        print("DOWNLOAD INCOMPLETE", flush=True)


if __name__ == "__main__":
    main()
