import json
import sys
from pathlib import Path

PROJECT = "Segovia"
STATE_PATH = Path(__file__).resolve().parents[1] / "state" / "gigabyte_reminder.json"


def is_dismissed():
    try:
        return bool(json.loads(STATE_PATH.read_text(encoding="utf-8")).get("dismissed", False))
    except Exception:
        return False


def reminder_text():
    return (
        f"[{PROJECT}] GigaByte submission - two items still open: "
        f"(1) check for the editors' reply on the fee-waiver + software data-deposit question "
        f"(the editorial@gigabytejournal.com thread; draft in docs/papers/GigaByte/editorial-email.md). "
        f"(2) register Segovia at SciCrunch.org for an RRID, then have Claude drop it into "
        f"docs/papers/GigaByte/template/paper.tex + paper.md and rebuild the PDF. "
        f"To silence: set {{\"dismissed\": true}} in .claude/state/gigabyte_reminder.json, "
        f"or remove the SessionStart block from .claude/settings.json."
    )


def main():
    if is_dismissed():
        print(json.dumps({"suppressOutput": True}))
        return 0
    print(json.dumps({"systemMessage": reminder_text(), "suppressOutput": True}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
