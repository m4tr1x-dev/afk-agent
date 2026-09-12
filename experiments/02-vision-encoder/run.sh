#!/usr/bin/env bash
# Run the device control, start the model host, run the probe, stop.
#
# The probe cannot answer one of its three questions from inside an HTTP
# request. Whether the runtime actually used the graphics processor is not
# visible there: a runtime that fell back to the processor answers exactly the
# same way, only slower, and "slower" is not a measurement without a baseline.
#
# So the control is taken from outside, by the thing that starts the server,
# and it supplies its own baseline: the same model benchmarked with every layer
# on the graphics processor and with none of them.
#
#   ./run.sh --model PATH --projector PATH [--seed N] [--out FILE]

set -u

RUNTIME=${RUNTIME:-/e/afk-agent/runtime/llama-b10930-vulkan}
PORT=${PORT:-8080}
CONTEXT=${CONTEXT:-8192}
GPU_LAYERS=${GPU_LAYERS:-999}
MODEL=""
PROJECTOR=""
SEED=""
OUT=""

while [ $# -gt 0 ]; do
  case "$1" in
    --model)     MODEL="$2"; shift 2 ;;
    --projector) PROJECTOR="$2"; shift 2 ;;
    --seed)      SEED="$2"; shift 2 ;;
    --out)       OUT="$2"; shift 2 ;;
    --port)      PORT="$2"; shift 2 ;;
    --context)   CONTEXT="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

[ -n "$MODEL" ] && [ -n "$PROJECTOR" ] || { echo "--model and --projector are required" >&2; exit 2; }
for path in "$MODEL" "$PROJECTOR" "$RUNTIME/llama-server.exe"; do
  [ -f "$path" ] || { echo "missing: $path" >&2; exit 1; }
done

model_mib=$(( $(stat -c %s "$MODEL") / 1048576 ))
proj_mib=$(( $(stat -c %s "$PROJECTOR") / 1048576 ))

echo "model      $MODEL  (${model_mib} MiB)"
echo "projector  $PROJECTOR  (${proj_mib} MiB)"
echo "runtime    $RUNTIME"
echo

# The device control: generation throughput with every layer on the graphics
# processor, against the same model with none of them.
#
# The first version of this control read free graphics memory before and after
# the load. It does not work: this runtime's device enumeration reports a
# static figure - 23749 MiB free on this card whether a model is resident or
# not - so the reading never moved and the control reported a fallback that had
# not happened. A control that fires when nothing is wrong is worth no more than
# one that stays silent when something is.
#
# Throughput discriminates because the two cases are not close. A silent
# fallback is not a few percent slower; it is several times slower, and the
# ratio is the assertion.
echo "--- device control: generation throughput, all layers against none ---"
bench=$("$RUNTIME/llama-bench.exe" -m "$MODEL" -n 32 -p 0 -r 1 -ngl 0,999 -o csv 2>/dev/null)
read -r cpu_ts gpu_ts <<EOF
$(printf '%s\n' "$bench" | python -c '
import csv, sys
rows = {r["n_gpu_layers"]: float(r["avg_ts"]) for r in csv.DictReader(sys.stdin)}
print(rows.get("0", 0.0), rows.get("999", 0.0))
')
EOF

echo "  no layers on the graphics processor:  ${cpu_ts} tokens per second"
echo "  every layer on it:                    ${gpu_ts} tokens per second"
ratio=$(python -c "print(f'{${gpu_ts} / max(${cpu_ts}, 1e-9):.2f}')")
echo "  ratio: ${ratio}x"
if python -c "import sys; sys.exit(0 if ${ratio} >= 3.0 else 1)"; then
  echo "  control: the graphics processor was used."
else
  echo "  CONTROL FAILED: throughput is within 3x of processor-only. It fell back."
fi
echo

log=$(mktemp)
"$RUNTIME/llama-server.exe" \
  --model "$MODEL" \
  --mmproj "$PROJECTOR" \
  --n-gpu-layers "$GPU_LAYERS" \
  --ctx-size "$CONTEXT" \
  --host 127.0.0.1 --port "$PORT" \
  --parallel 1 > "$log" 2>&1 &
server=$!

cleanup() {
  if kill -0 "$server" 2>/dev/null; then
    kill "$server" 2>/dev/null
    sleep 2
    kill -9 "$server" 2>/dev/null
    echo "server stopped (pid $server)"
  fi
  echo "server log: $log"
}
trap cleanup EXIT

# Ask, rather than sleeping for a guessed interval. A cold load takes as long
# as it takes.
ready=0
for _ in $(seq 1 600); do
  if ! kill -0 "$server" 2>/dev/null; then
    echo "the server exited; last lines of its log:" >&2
    tail -20 "$log" >&2
    exit 1
  fi
  if curl -fsS --max-time 3 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
[ "$ready" -eq 1 ] || { echo "the server did not become ready within ten minutes" >&2; exit 1; }

echo "server ready"
echo

args=(--host "127.0.0.1:$PORT")
[ -n "$SEED" ] && args+=(--seed "$SEED")

report=$(./target/x86_64-pc-windows-msvc/release/vision-encoder-probe.exe "${args[@]}" 2>&1)
status=$?
echo "$report"

if [ -n "$OUT" ]; then
  {
    echo "model      $MODEL (${model_mib} MiB)"
    echo "projector  $PROJECTOR (${proj_mib} MiB)"
    echo "runtime    $RUNTIME"
    echo "device control: ${cpu_ts} tokens per second with no layers offloaded, ${gpu_ts} with all of them, ratio ${ratio}x"
    echo
    echo "$report"
  } > "$OUT"
  echo "transcript written to $OUT"
fi

exit "$status"
