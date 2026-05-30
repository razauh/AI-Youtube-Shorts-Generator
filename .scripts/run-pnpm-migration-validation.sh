#!/usr/bin/env bash
set -euo pipefail

LOG_DIR=".logs"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/pnpm-migration-validation-${TS}.log"

mkdir -p "${LOG_DIR}"

run_and_log() {
  local title="$1"
  shift
  echo "===== ${title} =====" | tee -a "${LOG_FILE}"
  "$@" 2>&1 | tee -a "${LOG_FILE}"
}

run_and_log "check pnpm migration config" node -e '
const fs = require("fs");
const rootPkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
const workspace = fs.readFileSync("pnpm-workspace.yaml", "utf8");
const policies = fs.readFileSync("app/src/lib/legal/policiesContent.ts", "utf8");
const releaseWorkflow = fs.readFileSync(".github/workflows/release.yml", "utf8");
const buildWorkflow = fs.readFileSync(".github/workflows/build.yml", "utf8");
const gitignore = fs.readFileSync(".gitignore", "utf8");
const bwrapScript = fs.readFileSync("scripts/secure-pnpm-install-bwrap.sh", "utf8");
if (!fs.existsSync("LICENSE")) {
  throw new Error("root LICENSE file is required because README declares MIT");
}
if (fs.existsSync("package-lock.json") || fs.existsSync("npm-shrinkwrap.json")) {
  throw new Error("npm lockfiles must not be present in this pnpm workspace");
}
if (!String(rootPkg.packageManager || "").startsWith("pnpm@")) {
  throw new Error("packageManager must pin pnpm");
}
if (rootPkg.scripts["deps:sandbox-install"] !== "bash scripts/secure-pnpm-install-bwrap.sh") {
  throw new Error("deps:sandbox-install must point to the local bubblewrap wrapper");
}
for (const expected of [
  "minimumReleaseAge: 43200",
  "minimumReleaseAgeStrict: true",
  "minimumReleaseAgeIgnoreMissingTime: false",
  "ignoreScripts: true",
  "strictDepBuilds: true",
  "dangerouslyAllowAllBuilds: false",
  "engineStrict: true"
]) {
  if (!workspace.includes(expected)) {
    throw new Error(`missing pnpm workspace setting: ${expected}`);
  }
}
if (policies.includes("Node/npm")) {
  throw new Error("in-app policy copy still references Node/npm");
}
if (/bwrap|bubblewrap/.test(releaseWorkflow) || /bwrap|bubblewrap/.test(buildWorkflow)) {
  throw new Error("bubblewrap must remain out of GitHub build/release workflows");
}
if (!gitignore.includes(".pnpm-store/")) {
  throw new Error(".pnpm-store/ must be ignored");
}
if (!bwrapScript.includes(".pnpm-store") || bwrapScript.includes("--store-dir /tmp/pnpm-store")) {
  throw new Error("bubblewrap install must use the persistent repo-local pnpm store");
}
if (!bwrapScript.includes("/opt/node-toolchain") || !bwrapScript.includes("COREPACK_HOME /opt/corepack-cache")) {
  throw new Error("bubblewrap install must mount Node/Corepack explicitly");
}
if (!bwrapScript.includes("NETWORK_ARGS=(--share-net)")) {
  throw new Error("bubblewrap online mode must retain host networking for release-age verification");
}
'

run_and_log "pnpm version" pnpm --version
run_and_log "frontend tests" pnpm --dir app run test
run_and_log "worker tests" pnpm --dir worker run test

echo "pnpm migration validation completed. Log: ${LOG_FILE}" | tee -a "${LOG_FILE}"
