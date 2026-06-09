import argparse
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "tests" / "data"

PUBLIC_BASE_URL = "https://openalyx.internationalbrainlab.org"
PUBLIC_PASSWORD = "international"
DEFAULT_EID = "803dd5b6-248a-4811-b36b-d7070bbfa3a1"
DEFAULT_COLLECTION = "raw_ephys_data/probe01"

UUID_IN_NAME = re.compile(
    r"\.[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}(?=\.[^.]+$)"
)


def strip_uuid(name):
    return UUID_IN_NAME.sub("", name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--eid", default=DEFAULT_EID)
    parser.add_argument("--collection", default=DEFAULT_COLLECTION)
    parser.add_argument("--band", default="lf", choices=["lf", "ap"])
    args = parser.parse_args()

    from one.api import ONE

    one = ONE(
        base_url=PUBLIC_BASE_URL,
        password=PUBLIC_PASSWORD,
        silent=True,
        mode="remote",
    )

    records = one.alyx.rest(
        "datasets",
        "list",
        session=args.eid,
        django=f"collection,{args.collection},name__icontains,{args.band}.",
    )
    wanted = {f"{args.band}.cbin", f"{args.band}.ch", f"{args.band}.meta"}
    trio = [d for d in records if any(d["name"].endswith(s) for s in wanted)]
    if not any(d["name"].endswith(f"{args.band}.cbin") for d in trio):
        raise SystemExit(
            f"no {args.band}.cbin in {args.collection} for session {args.eid}"
        )

    DATA.mkdir(parents=True, exist_ok=True)
    copied = []
    for d in trio:
        url = next(
            fr["data_url"]
            for fr in d["file_records"]
            if fr.get("exists") and "flatiron" in (fr.get("data_repository") or "")
        )
        local = one.alyx.download_file(url)
        dest = DATA / strip_uuid(Path(local).name)
        shutil.copy2(local, dest)
        copied.append(dest)

    print(f"session:    {args.eid}")
    print(f"collection: {args.collection}")
    for c in sorted(copied):
        print(f"  -> {c.name}  ({c.stat().st_size / 1e9:.2f} GB)")
    print("\nrun: python scripts/bench_bounded_memory.py")


if __name__ == "__main__":
    main()
