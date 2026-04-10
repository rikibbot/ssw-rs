#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3000}"
OUTPUT_DIR="${2:-./artifacts/ssw-intake-demo}"
WAIT_MS="${WAIT_MS:-500}"

mkdir -p "$OUTPUT_DIR"

capture() {
  local url="$1"
  local path="$2"

  agent-browser open "$url" >/dev/null
  agent-browser wait "$WAIT_MS" >/dev/null
  agent-browser screenshot "$path" >/dev/null
}

capture "$BASE_URL/" "$OUTPUT_DIR/home.png"
capture "$BASE_URL/style-guide" "$OUTPUT_DIR/style-guide.png"

printf 'Saved screenshots to %s\n' "$OUTPUT_DIR"
