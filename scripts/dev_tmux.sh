#!/usr/bin/env bash
set -euo pipefail

SESSION="rust-fintech"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is not installed. Install it first, then rerun this script."
  exit 1
fi

if tmux has-session -t "$SESSION" 2>/dev/null; then
  tmux attach -t "$SESSION"
  exit 0
fi

if cargo watch --version >/dev/null 2>&1; then
  API_CMD="cargo watch -x run"
else
  API_CMD="echo 'cargo-watch is not installed; running cargo run without auto-restart.' && echo 'Install it with: cargo install cargo-watch' && cargo run"
fi

tmux new-session -d -s "$SESSION" -n api -c "$ROOT_DIR/crates/api"
tmux send-keys -t "$SESSION:api" "$API_CMD" C-m

tmux new-window -t "$SESSION" -n web -c "$ROOT_DIR/apps/web"
tmux send-keys -t "$SESSION:web" "pnpm dev" C-m

tmux select-window -t "$SESSION:api"
tmux attach -t "$SESSION"
