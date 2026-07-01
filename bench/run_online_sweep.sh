#!/usr/bin/env bash
set -u
ROOT="C:/Projects/Segovia"
SEG="$ROOT/.venv/Scripts/python.exe"
SI="$ROOT/.venv-si/Scripts/python.exe"
OUT="$ROOT/bench/_tmp/results.jsonl"
SYN_BIN="$ROOT/bench/_tmp/synthetic_replay.bin"
SYN_META="$ROOT/bench/_tmp/synthetic_replay.meta"
REAL_CBIN="$ROOT/tests/data/_spikeglx_ephysData_g0_t0.imec0.ap.cbin"
REAL_CH="$ROOT/tests/data/_spikeglx_ephysData_g0_t0.imec0.ap.ch"
LIMIT=1800000
: > "$OUT"

for CS in 3000 9000 30000; do
  echo "[segovia synthetic cs=$CS]"
  "$SEG" "$ROOT/bench/replay_latency.py" --kind spikeglx --bin "$SYN_BIN" --meta "$SYN_META" \
    --chunk-samples $CS --json-only | head -1 \
    | sed "s/^/{\"source\":\"synthetic\",\"engine_tag\":\"segovia\",\"cs\":$CS,\"payload\":/;s/$/}/" >> "$OUT"

  echo "[si synthetic cs=$CS]"
  "$SI" "$ROOT/bench/replay_latency_si.py" --kind binary --bin "$SYN_BIN" \
    --n-channels 384 --sample-rate 30000 --chunk-samples $CS --json-only | head -1 \
    | sed "s/^/{\"source\":\"synthetic\",\"engine_tag\":\"si\",\"cs\":$CS,\"payload\":/;s/$/}/" >> "$OUT"

  echo "[segovia real cs=$CS]"
  "$SEG" "$ROOT/bench/replay_latency.py" --kind cbin --cbin "$REAL_CBIN" --ch "$REAL_CH" \
    --chunk-samples $CS --limit-samples $LIMIT --json-only | head -1 \
    | sed "s/^/{\"source\":\"real\",\"engine_tag\":\"segovia\",\"cs\":$CS,\"payload\":/;s/$/}/" >> "$OUT"

  echo "[si real cs=$CS]"
  "$SI" "$ROOT/bench/replay_latency_si.py" --kind cbin --cbin "$REAL_CBIN" --load-sync \
    --chunk-samples $CS --limit-samples $LIMIT --json-only | head -1 \
    | sed "s/^/{\"source\":\"real\",\"engine_tag\":\"si\",\"cs\":$CS,\"payload\":/;s/$/}/" >> "$OUT"
done
echo "DONE"
