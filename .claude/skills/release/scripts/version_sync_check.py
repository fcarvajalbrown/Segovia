import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[4]
errors = []

cargo = (root / "Cargo.toml").read_text(encoding="utf-8")
match = re.search(r'(?m)^version\s*=\s*"([^"]+)"', cargo)
version = match.group(1) if match else None
if version is None:
    errors.append("Cargo.toml: no [package] version found")

pyproject = (root / "pyproject.toml").read_text(encoding="utf-8")
if 'dynamic = ["version"]' not in pyproject.replace("'", '"'):
    errors.append('pyproject.toml: must declare dynamic = ["version"]')
if re.search(r'(?m)^\s*version\s*=\s*"', pyproject):
    errors.append("pyproject.toml: static version field present; versions must derive from Cargo.toml")

if version is not None:
    changelog = (root / "CHANGELOG.md").read_text(encoding="utf-8")
    if f"[{version}]" not in changelog:
        errors.append(f"CHANGELOG.md: no entry for [{version}]")

raw = ""
try:
    if not sys.stdin.isatty():
        raw = sys.stdin.read()
except Exception:
    raw = ""

if raw.strip():
    try:
        command = json.loads(raw).get("tool_input", {}).get("command", "")
    except Exception:
        command = ""
    if "git tag" in command and version is not None:
        tag = re.search(r"v\d+\.\d+\.\d+[^\s'\"]*", command)
        if tag and tag.group(0) != f"v{version}":
            errors.append(f"git tag {tag.group(0)} does not match Cargo.toml version v{version}")

if errors:
    sys.stderr.write("version-sync check FAILED:\n")
    for item in errors:
        sys.stderr.write(f"  - {item}\n")
    sys.exit(2)

sys.stdout.write(f"version-sync ok: v{version}\n")
sys.exit(0)
