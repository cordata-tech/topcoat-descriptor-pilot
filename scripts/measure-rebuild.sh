#!/usr/bin/env bash
# Q3 — does the fast-rebuild loop hold up, and does view! expansion dominate?
#
#   scripts/measure-rebuild.sh [runs]
#
# Touches one source file and rebuilds, N times, reporting each run and the
# median. A single number proves nothing: the first build after a cargo clean
# is dominated by dependencies, and one warm run is noise.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$HERE"
RUNS="${1:-7}"
TARGET="${TARGET:-src/table.rs}"

cargo build --quiet 2>/dev/null   # warm

times=()
for i in $(seq 1 "$RUNS"); do
  touch "$TARGET"
  start=$(python3 -c 'import time; print(time.time())')
  cargo build --quiet 2>/dev/null
  end=$(python3 -c 'import time; print(time.time())')
  t=$(python3 -c "print(f'{$end - $start:.2f}')")
  times+=("$t")
  printf '  run %d: %ss\n' "$i" "$t"
done

printf '%s\n' "${times[@]}" | sort -n | awk '
  { a[NR]=$1 }
  END { printf "  median: %.2fs   min: %.2fs   max: %.2fs\n", a[int((NR+1)/2)], a[1], a[NR] }'
