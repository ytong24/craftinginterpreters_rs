#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INTERPRETER="$SCRIPT_DIR/target/release/craftinginterpreters_rs"

# Ordered list of jlox chapter suites (each is cumulative, but we run all
# up to the selected chapter to catch regressions with stricter earlier suites).
SUITES=(
  "chap04_scanning"
  "chap06_parsing"
  "chap07_evaluating"
  "chap08_statements"
  "chap09_control"
  "chap10_functions"
  "chap11_resolving"
  "chap12_classes"
  "chap13_inheritance"
  "jlox"
)

LABELS=(
  "Ch 4  — Scanning"
  "Ch 6  — Parsing"
  "Ch 7  — Evaluating"
  "Ch 8  — Statements and State"
  "Ch 9  — Control Flow"
  "Ch 10 — Functions"
  "Ch 11 — Resolving and Binding"
  "Ch 12 — Classes"
  "Ch 13 — Inheritance"
  "Full  — All jlox tests"
)

echo "Select the chapter to test:"
echo ""
for i in "${!SUITES[@]}"; do
  printf "  %2d) %s\n" $((i + 1)) "${LABELS[$i]}"
done
echo ""
read -rp "Enter number [1-${#SUITES[@]}]: " choice

if ! [[ "$choice" =~ ^[0-9]+$ ]] || (( choice < 1 || choice > ${#SUITES[@]} )); then
  echo "Invalid choice."
  exit 1
fi

selected=$((choice - 1))

echo ""
echo "Building interpreter..."
cargo build --release

cd "$SCRIPT_DIR/tests"

any_failed=0

for i in $(seq 0 "$selected"); do
  suite="${SUITES[$i]}"
  label="${LABELS[$i]}"
  echo ""
  echo "========================================"
  echo "  Running: $label ($suite)"
  echo "========================================"
  if ! dart tool/bin/test.dart --interpreter "$INTERPRETER" "$suite"; then
    any_failed=1
  fi
done

echo ""
if (( any_failed )); then
  echo "SOME SUITES HAD FAILURES."
  exit 1
else
  echo "All suites passed."
fi
