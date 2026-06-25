#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/gumroad-webhook-validation-${STAMP}.log"

mkdir -p "${LOG_DIR}"

run_step() {
  local name="$1"
  shift
  {
    echo "== ${name} =="
    echo "command: $*"
    "$@"
    echo
  } 2>&1 | tee -a "${LOG_FILE}"
}

cd "${ROOT_DIR}/worker"

# We use the local node environment to run node:test on the new failure contract file
run_step "node-test-worker-contract" node --test test/contract.test.js
run_step "node-test-gumroad-webhook-failures" node --test test/gumroad_webhook_failures.test.js

cd "${ROOT_DIR}"
run_step "docs-policy-duplicate-mismatch" grep -F "Duplicate Mismatched Payload" docs/gumroad_webhook_state_policy.md
run_step "docs-policy-refund-blockkey-failure" grep -F "Refund BlockKey Fails" docs/gumroad_webhook_state_policy.md
run_step "docs-policy-missing-license-key" grep -F "Verified Sale Missing License Key" docs/gumroad_webhook_state_policy.md
run_step "docs-policy-createkey-network-failure" grep -F "Devolens CreateKey HTTP/Network Failure" docs/gumroad_webhook_state_policy.md

echo "validation logs: ${LOG_FILE}"
