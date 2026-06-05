#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/.logs"
mkdir -p "$LOG_DIR"

timestamp="$(date +%Y%m%d-%H%M%S)"
log_file="$LOG_DIR/policy-muapi-liability-validation-$timestamp.log"

exec > >(tee "$log_file") 2>&1

policy_file="$ROOT_DIR/app/src/lib/legal/policiesContent.ts"
test_file="$ROOT_DIR/app/src/tests/ui_flow.test.ts"

fail() {
  echo "FAIL: $1"
  exit 1
}

echo "Checking MuAPI dependency and liability Terms wording..."

grep -q "supported generation workflow depends on MuAPI availability" "$policy_file" \
  || fail "Terms do not explicitly state that generation depends on MuAPI availability"

grep -q "that outage or service change is outside our control and is not our responsibility" "$policy_file" \
  || fail "Terms do not explicitly disclaim responsibility for MuAPI outage/service changes"

grep -q "supported generation workflow depends on MuAPI availability" "$test_file" \
  || fail "UI flow policy test does not cover MuAPI availability wording"

grep -q "outside our control and is not our responsibility" "$test_file" \
  || fail "UI flow policy test does not cover MuAPI responsibility wording"

echo "Static validation passed."
echo "Manual test command still required:"
echo "pnpm --dir app run test -- src/tests/ui_flow.test.ts"
