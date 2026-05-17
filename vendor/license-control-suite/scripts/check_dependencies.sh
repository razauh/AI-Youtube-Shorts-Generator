#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

echo "[deps] cargo check"
cargo check

echo "[deps] cargo tree -d"
cargo tree -d

echo "[deps] cargo metadata --format-version 1"
cargo metadata --format-version 1 > docs/baseline/cargo_metadata/license-control-suite.metadata.json

echo "[deps] ensure no external git shared-contracts"
if cargo tree | rg -q "shared-contracts.*git\\+"; then
  echo "external git shared-contracts detected in dependency tree" >&2
  exit 1
fi

echo "[deps] dependency checks passed"
