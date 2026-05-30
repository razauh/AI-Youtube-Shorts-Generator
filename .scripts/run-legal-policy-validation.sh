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
  'app/index.html',
  'app/src/routes/+page.svelte',
  'app/src/lib/legal/policiesContent.ts',
];

const forbidden = [
  /VERIFY/i,
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
  '16. Prohibited Uses',
  '18. Limitation of Liability',
]) {
  if (!policy.includes(expected)) {
    console.error(`[fail] in-app policy content missing expected text: ${expected}`);
    failed = true;
  }
}

if (failed) {
  process.exit(1);
}
console.log('[pass] legal and policy surfaces contain no known release placeholders');
NODE

echo "[info] legal policy validation complete: PASS"
echo "[info] checked app-rendered policy sources only"
echo "[info] log written to ${LOG_FILE}"
