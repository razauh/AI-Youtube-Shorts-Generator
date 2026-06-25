#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/.logs"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="$LOG_DIR/ui-flow-node22-${STAMP}.log"

mkdir -p "$LOG_DIR"

if [[ -s "$HOME/.nvm/nvm.sh" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/.nvm/nvm.sh"
else
  echo "nvm was not found at $HOME/.nvm/nvm.sh" >&2
  exit 1
fi

nvm use 22.13.1
hash -r

echo "Node: $(node -v)"
echo "Node path: $(which node)"
echo "pnpm: $(pnpm -v)"

{
  echo "== ui-flow-auth-machine-limit-node22 =="
  echo "command: pnpm --dir $ROOT_DIR/app run test -- src/tests/ui_flow.test.ts src/tests/auth_client.test.ts src/tests/auth_state.test.ts"
  pnpm --dir "$ROOT_DIR/app" run test -- src/tests/ui_flow.test.ts src/tests/auth_client.test.ts src/tests/auth_state.test.ts
} 2>&1 | tee "$LOG_FILE"

echo "validation logs: $LOG_FILE"
