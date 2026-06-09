import json
import sys
import time
from pathlib import Path

CADENCE_DAYS = 7
PROJECT = "Segovia"
STATE_PATH = Path(__file__).resolve().parents[1] / "state" / "publishing_reminder.json"


def load_last_epoch():
    try:
        return int(json.loads(STATE_PATH.read_text(encoding="utf-8")).get("last_epoch", 0))
    except Exception:
        return 0


def save_epoch(epoch):
    STATE_PATH.parent.mkdir(parents=True, exist_ok=True)
    STATE_PATH.write_text(json.dumps({"last_epoch": epoch}), encoding="utf-8")


def reminder_text():
    return (
        f"[{PROJECT}] Publishing reminder (every {CADENCE_DAYS} days): consider a milestone-only "
        f"post for {PROJECT} and MaskOps - benchmark results, a release, or a tutorial, not every "
        f"commit. LinkedIn: adapt assets/draft_linkedin_es.* (ES), best Tue/Wed 9-11h local, 3-5 "
        f"hashtags. dev.to: a technical write-up mirroring the MaskOps cadence. Keep the honest "
        f"ephys->leukemia arc (aided-by, not made-for)."
    )


def main():
    now = int(time.time())
    last = load_last_epoch()
    if last and (now - last) < CADENCE_DAYS * 86400:
        print(json.dumps({"suppressOutput": True}))
        return 0
    save_epoch(now)
    print(json.dumps({"systemMessage": reminder_text(), "suppressOutput": True}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
