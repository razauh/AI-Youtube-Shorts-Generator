#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="${LOG_DIR}/validate-release-ci-config-${TS}.log"

mkdir -p "${LOG_DIR}"

exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] release CI config validation started: ${TS}"

fail() {
  echo "[fail] $1"
  exit 1
}

pass() {
  echo "[pass] $1"
}

contains() {
  grep -Eq "$1" "$2"
}

RELEASE_WORKFLOW="${ROOT_DIR}/.github/workflows/release.yml"
ROOT_PACKAGE_JSON="${ROOT_DIR}/package.json"
APP_PACKAGE_JSON="${ROOT_DIR}/app/package.json"
CUSTOMER_TAURI_CONFIG="${ROOT_DIR}/app/src-tauri/tauri.conf.json"
RUST_CONFIG="${ROOT_DIR}/app/src-tauri/src/core/config.rs"
RUNTIME_COMMANDS="${ROOT_DIR}/app/src-tauri/src/commands/runtime.rs"
LICENSE_FILE="${ROOT_DIR}/LICENSE"

[ -f "${RELEASE_WORKFLOW}" ] || fail "missing .github/workflows/release.yml"
pass "release workflow exists"

[ -f "${ROOT_PACKAGE_JSON}" ] || fail "missing package.json"
pass "root package.json exists"

[ -f "${APP_PACKAGE_JSON}" ] || fail "missing app/package.json"
pass "app package.json exists"

[ -f "${CUSTOMER_TAURI_CONFIG}" ] || fail "missing app/src-tauri/tauri.conf.json"
pass "customer Tauri config exists"

[ -f "${RUST_CONFIG}" ] || fail "missing app/src-tauri/src/core/config.rs"
pass "Rust production config source exists"

[ -f "${RUNTIME_COMMANDS}" ] || fail "missing app/src-tauri/src/commands/runtime.rs"
pass "runtime command source exists"

[ -f "${LICENSE_FILE}" ] || fail "missing root LICENSE"
pass "root LICENSE exists"

if [ -f "${ROOT_DIR}/package-lock.json" ] || [ -f "${ROOT_DIR}/npm-shrinkwrap.json" ]; then
  fail "npm lockfiles must not be tracked in the pnpm workspace"
else
  pass "no root npm lockfile is present"
fi

if awk '
  /push:/ { in_push = 1 }
  in_push && /tags:/ { in_tags = 1 }
  in_tags && /- '\''v\*'\''/ { found = 1 }
  END { exit found ? 0 : 1 }
' "${RELEASE_WORKFLOW}"; then
  pass "release workflow has v* tag trigger"
else
  fail "release workflow missing expected v* tag trigger"
fi

if contains "workflow_dispatch:" "${RELEASE_WORKFLOW}"; then
  pass "release workflow has workflow_dispatch"
else
  fail "release workflow missing workflow_dispatch"
fi

if contains "bundle:customer" "${ROOT_PACKAGE_JSON}"; then
  pass "bundle:customer script exists"
else
  fail "bundle:customer script missing"
fi

if contains "bundle:admin" "${ROOT_PACKAGE_JSON}"; then
  pass "bundle:admin script exists"
else
  fail "bundle:admin script missing"
fi

if contains "bundle:all" "${ROOT_PACKAGE_JSON}"; then
  pass "bundle:all script exists"
else
  fail "bundle:all script missing"
fi

if contains "tauri\.conf\.json" "${APP_PACKAGE_JSON}"; then
  pass "customer bundle script references tauri.conf.json"
else
  fail "customer bundle script missing tauri.conf.json reference"
fi

if contains "tauri\.admin\.conf\.json" "${APP_PACKAGE_JSON}"; then
  pass "admin bundle script references tauri.admin.conf.json"
else
  fail "admin bundle script missing tauri.admin.conf.json reference"
fi

if contains "Build customer Tauri bundles|bundle:customer" "${RELEASE_WORKFLOW}"; then
  pass "workflow references customer build command"
else
  fail "workflow missing customer build command reference"
fi

if contains "Build admin Tauri bundles|bundle:admin" "${RELEASE_WORKFLOW}"; then
  pass "workflow references admin build command"
else
  fail "workflow missing admin build command reference"
fi

if contains "customer-linux-x64|customer-windows-x64|customer-macos-x64" "${RELEASE_WORKFLOW}"; then
  pass "workflow has customer artifact naming"
else
  fail "workflow missing customer artifact names"
fi

if contains "admin-linux-x64|admin-windows-x64|admin-macos-x64" "${RELEASE_WORKFLOW}"; then
  pass "workflow has admin artifact naming"
else
  fail "workflow missing admin artifact names"
fi

if contains "softprops/action-gh-release" "${RELEASE_WORKFLOW}"; then
  pass "workflow has GitHub Release attachment step"
else
  fail "workflow missing GitHub Release attachment step"
fi

if contains "TAURI_SIGNING_PRIVATE_KEY" "${RELEASE_WORKFLOW}"; then
  pass "workflow passes Tauri updater signing key to customer build"
else
  fail "workflow missing TAURI_SIGNING_PRIVATE_KEY"
fi

if contains "generate-customer-updater-manifest\.mjs" "${RELEASE_WORKFLOW}"; then
  pass "workflow generates customer updater manifest"
else
  fail "workflow missing customer updater manifest generation"
fi

if contains "customer-latest\.json" "${RELEASE_WORKFLOW}"; then
  pass "workflow publishes customer-latest.json"
else
  fail "workflow missing customer-latest.json publication"
fi

if contains "release-upload/\*\*" "${RELEASE_WORKFLOW}"; then
  pass "workflow publishes deterministic flattened customer updater assets"
else
  fail "workflow missing flattened customer updater assets"
fi

if contains "updates\.example\.com|YOUR_CLOUDFLARE_SUBDOMAIN|REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY" "${CUSTOMER_TAURI_CONFIG}"; then
  fail "customer updater config still contains production placeholders"
else
  pass "customer updater config has production endpoint and public key"
fi

if contains "http://127\.0\.0\.1:8787|localhost:8787" "${RUST_CONFIG}"; then
  fail "Rust license worker default still points at a development endpoint"
else
  pass "Rust license worker default does not point at localhost"
fi

if contains "PRODUCTION_LICENSE_WORKER_BASE_URL.*license-worker\.demandscout\.workers\.dev|license-worker\.demandscout\.workers\.dev" "${RUST_CONFIG}"; then
  pass "Rust license worker default names the production Worker origin"
else
  fail "Rust license worker default is missing the production Worker origin"
fi

if contains "DEFAULT_LOCAL_RUNTIME_PACK_MANIFEST_URL.*https://license-worker\.demandscout\.workers\.dev/runtime-pack/manifest\.json|license-worker\.demandscout\.workers\.dev/runtime-pack/manifest\.json" "${RUNTIME_COMMANDS}"; then
  pass "runtime-pack manifest has a production HTTPS default"
else
  fail "runtime-pack manifest default is missing or non-production"
fi

for smoke_script in \
  "${ROOT_DIR}/tests/e2e/installer_smoke/linux-smoke.sh" \
  "${ROOT_DIR}/tests/e2e/installer_smoke/windows-smoke.ps1" \
  "${ROOT_DIR}/tests/e2e/installer_smoke/macos-smoke.sh"
do
  [ -f "${smoke_script}" ] || fail "missing installer smoke script: ${smoke_script#${ROOT_DIR}/}"
done
pass "installer smoke scripts exist for Linux, Windows, and macOS"

echo "[info] validation complete: PASS"
echo "[info] log written to ${LOG_FILE}"
