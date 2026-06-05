#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

pnpm --dir "$ROOT_DIR/app" run test -- src/tests/ui_flow.test.ts
