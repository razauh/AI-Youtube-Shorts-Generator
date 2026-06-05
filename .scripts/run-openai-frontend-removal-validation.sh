#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/.logs"
mkdir -p "$LOG_DIR"

timestamp="$(date +%Y%m%d-%H%M%S)"
log_file="$LOG_DIR/openai-frontend-removal-validation-$timestamp.log"

exec > >(tee "$log_file") 2>&1

page_file="$ROOT_DIR/app/src/routes/+page.svelte"
policy_file="$ROOT_DIR/app/src/lib/legal/policiesContent.ts"
test_file="$ROOT_DIR/app/src/tests/ui_flow.test.ts"

fail() {
  echo "FAIL: $1"
  exit 1
}

echo "Checking customer frontend OpenAI removal..."

if grep -Eq "OpenAI Access|OpenAI profile name|OpenAI key|Add OpenAI Profile|OpenAI key profiles|OpenAI Configured" "$page_file"; then
  fail "OpenAI Settings UI strings are still present in app/src/routes/+page.svelte"
fi

if grep -q "apiKeyProfiles('openai')" "$page_file"; then
  fail "Customer page still calls apiKeyProfiles('openai')"
fi

if grep -q "OpenAI" "$policy_file"; then
  fail "Customer-visible policy content still mentions OpenAI"
fi

if ! grep -q "queryByRole('button', { name: 'OpenAI Access help' })" "$test_file"; then
  fail "UI flow test does not assert OpenAI access help is absent"
fi

if ! grep -q "not.toHaveBeenCalledWith('openai')" "$test_file"; then
  fail "UI flow test does not assert OpenAI profiles are not loaded"
fi

if ! grep -q "Add MuAPI Profile" "$test_file"; then
  fail "UI flow test no longer covers MuAPI profile add behavior"
fi

echo "Static validation passed."
echo "Manual test command still required:"
echo "pnpm --dir app run test -- src/tests/ui_flow.test.ts"
