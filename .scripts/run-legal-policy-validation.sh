#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/legal-policy-validation-${TS}.log"

mkdir -p "${LOG_DIR}"
exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] legal policy validation started: ${TS}"

cd "${ROOT_DIR}"

node <<'NODE'
const fs = require('fs');

const files = [
  'app/src/routes/+page.svelte',
  'app/src/lib/legal/policiesContent.ts',
];

const forbidden = [
  /\bVERIFY\b/,
  /Fill in/i,
  /Final legal entity/i,
  /May 23, 2026/,
  /repomix/i,
  /\[(APP NAME|DEVELOPER NAME|LEGAL COMPANY|COMPANY ADDRESS|CONTACT EMAIL|SUPPORT EMAIL|PRIVACY EMAIL|WEBSITE OR SUPPORT URL|JURISDICTION|EFFECTIVE DATE|RETENTION PERIOD|TO BE COMPLETED)/,
  /Signal Forge/,
  /AI Shorts App/,
  /placeholders? that require/i,
  /Node\/npm/,
];

let failed = false;
for (const file of files) {
  const text = fs.readFileSync(file, 'utf8');
  for (const pattern of forbidden) {
    if (pattern.test(text)) {
      console.error(`[fail] ${file} contains forbidden release placeholder or stale product text: ${pattern}`);
      failed = true;
    }
  }
}

const policy = fs.readFileSync('app/src/lib/legal/policiesContent.ts', 'utf8');
for (const expected of [
  'AI YouTube Shorts Generator',
  "export const POLICY_LAST_UPDATED_LABEL = 'May 30, 2026';",
  "export type PolicyTab = 'terms' | 'privacy' | 'compliance' | 'notices' | 'refund';",
  '16. Prohibited Uses',
  '18. Limitation of Liability',
  'Refunded, charged-back, revoked, disabled, or disputed purchases may lose access',
  'MuAPI',
  'OpenAI',
  'Gumroad',
  'Cloudflare Workers and D1',
  'YouTube, Google, and other source platforms',
  'update hosts',
  'FFmpeg',
  'Tauri/Rust desktop app with Svelte UI',
  'Vite',
  'Rust, Tauri, and Native Dependencies',
  'license-control-suite',
  'No general telemetry or analytics SDK was identified during repository inspection.',
  'Crash reports are submitted only when an endpoint is configured and the user submits a draft.',
]) {
  if (!policy.includes(expected)) {
    console.error(`[fail] in-app policy content missing expected text: ${expected}`);
    failed = true;
  }
}

const lastUpdatedDates = policy.match(/Last updated: [A-Za-z]+ \d{1,2}, \d{4}/g) || [];
if (lastUpdatedDates.length < 5 || lastUpdatedDates.some((date) => date !== 'Last updated: May 30, 2026')) {
  console.error('[fail] in-app policy content has missing or inconsistent Last updated dates');
  failed = true;
}

for (const tab of ['terms', 'privacy', 'compliance', 'notices', 'refund']) {
  if (!policy.includes(`  "${tab}": [`)) {
    console.error(`[fail] in-app policy content missing tab: ${tab}`);
    failed = true;
  }
}

if (failed) {
  process.exit(1);
}
console.log('[pass] app-rendered legal and policy surfaces contain no known release placeholders');
NODE

echo "[info] legal policy validation complete: PASS"
echo "[info] checked app-rendered policy sources only"
echo "[info] log written to ${LOG_FILE}"
