#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3000}"
OUTPUT_DIR="${2:-./artifacts/ssw-intake-demo}"
WAIT_MS="${WAIT_MS:-500}"
VIEWPORT_WIDTH="${VIEWPORT_WIDTH:-1720}"
VIEWPORT_HEIGHT="${VIEWPORT_HEIGHT:-1400}"
FULL_PAGE="${FULL_PAGE:-false}"

mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"

agent-browser set viewport "$VIEWPORT_WIDTH" "$VIEWPORT_HEIGHT" >/dev/null

screenshot() {
  local path="$1"

  if [[ "$FULL_PAGE" == "true" ]]; then
    agent-browser screenshot --full "$path" >/dev/null
  else
    agent-browser screenshot "$path" >/dev/null
  fi
}

capture() {
  local url="$1"
  local path="$2"

  agent-browser open "$url" >/dev/null
  agent-browser wait "$WAIT_MS" >/dev/null
  screenshot "$path"
}

capture "$BASE_URL/" "$OUTPUT_DIR/home.png"
capture "$BASE_URL/style-guide" "$OUTPUT_DIR/style-guide.png"

printf 'Saved screenshots to %s\n' "$OUTPUT_DIR"
