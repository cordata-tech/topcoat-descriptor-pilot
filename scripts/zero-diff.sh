#!/usr/bin/env bash
# The zero-diff test, mechanised.
#
#   scripts/zero-diff.sh capture   # before the refactor — save baselines
#   scripts/zero-diff.sh check     # after  the refactor — diff against them
#
# `one-skeleton-many-screens` argued that the proof you drew the abstraction
# boundary in the right place is that refactoring a screen you already trust
# through it produces zero diff. In React that meant eyeballing a rendered
# page. Topcoat renders on the server, so the claim is a text comparison:
# diff either returns nothing or the abstraction is wrong.
#
# That is the one part of this pilot worth automating. Everything else about
# Topcoat wants to be experienced by hand — the notes in NOTES.md are only
# worth anything if they came from doing it.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE="$HERE/baseline"
PORT="${PORT:-3000}"

# Add a route here when a screen is added. Names become filenames.
ROUTES=(
  "users:/"
  "invoices:/invoices"
)

usage() { echo "usage: $0 {capture|check}" >&2; exit 2; }
[ $# -eq 1 ] || usage

cd "$HERE"

# Force cargo to see the sources as current before building.
#
# Not paranoia — this harness has already produced a confident wrong answer
# without it. Restoring a file with `mv` (or `cp -p`, or anything that
# preserves mtime) leaves the source OLDER than the compiled binary, so cargo
# skips the rebuild and the server serves the previous build. The check then
# reports CHANGED against code that is already correct, and nothing anywhere
# says a rebuild was skipped.
#
# A full rebuild of this crate is ~1.4s. That is a cheap price for a
# correctness harness never lying about which code it tested.
find src -name '*.rs' -exec touch {} +
cargo build --quiet

cargo run --quiet >/tmp/zero-diff-server.log 2>&1 &
SERVER=$!
trap 'kill "$SERVER" 2>/dev/null || true' EXIT

# Poll rather than sleep: a fixed sleep is either flaky or slow, and on a
# cold build it is both.
for _ in $(seq 1 60); do
  if curl -fsS -o /dev/null "http://localhost:$PORT/" 2>/dev/null; then break; fi
  sleep 0.5
done
curl -fsS -o /dev/null "http://localhost:$PORT/" || {
  echo "server never came up on :$PORT — see /tmp/zero-diff-server.log" >&2
  exit 1
}

fetch() {  # fetch <route> <outfile>
  curl -fsS "http://localhost:$PORT$1" > "$2"
}

case "$1" in
  capture)
    mkdir -p "$BASELINE"
    for entry in "${ROUTES[@]}"; do
      name="${entry%%:*}"; route="${entry#*:}"
      fetch "$route" "$BASELINE/$name.html"
      echo "captured  $name  ($route)"
    done
    echo
    echo "Baselines saved. Commit them — they are the 'before' half of the"
    echo "test, and a baseline that only exists on one machine proves nothing."
    ;;
  check)
    [ -d "$BASELINE" ] || { echo "no baselines — run '$0 capture' first" >&2; exit 1; }
    tmp="$(mktemp -d)"; failed=0
    for entry in "${ROUTES[@]}"; do
      name="${entry%%:*}"; route="${entry#*:}"
      if [ ! -f "$BASELINE/$name.html" ]; then
        echo "SKIP  $name — no baseline (new screen, nothing to compare)"
        continue
      fi
      fetch "$route" "$tmp/$name.html"
      if diff -u "$BASELINE/$name.html" "$tmp/$name.html" > "$tmp/$name.diff"; then
        echo "ZERO DIFF  $name  ($route)"
      else
        echo "CHANGED    $name  ($route)"
        sed -n '1,40p' "$tmp/$name.diff"
        failed=1
      fi
    done
    echo
    if [ "$failed" -eq 0 ]; then
      echo "All screens byte-identical. The boundary held."
    else
      echo "A screen changed. That is the result, not a bug to work around —"
      echo "write down what moved and why in NOTES.md before fixing it."
    fi
    exit "$failed"
    ;;
  *) usage ;;
esac
