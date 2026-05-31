# Incident Handbook (Solo Dev / Support / On‑Call)

This handbook is a step‑by‑step runbook for crashes and production incidents in **AI YouTube Shorts Generator** (desktop Tauri app + Cloudflare Worker/D1 + local runtime/model pipeline).

**Rules**

- Preserve evidence first (logs/state) before “fixing”.
- Don’t paste secrets into tickets/chats/logs: license keys, tokens, emails. Use `[redacted]`.
- Prefer reversible actions: rename/quarantine instead of delete; deploy forward instead of “downgrade”.
- Always record: timestamp window, app version, OS, mode (`api`/`local`), last progress stage, and `run_id` (if present).

---

## 0) One‑Minute Triage (always do first)

### 0.1 Identify the incident class

- App won’t launch / crashes on startup
- License/auth issue (license required, validate fails, worker unreachable)
- Generation stuck / extremely slow
- Local mode broken (runtime‑pack or model)
- Worker/D1 outage
- Updater broken
- Privacy deletion failed/stuck

### 0.2 Capture Worker liveness/readiness (no changes)

```bash
curl -sS https://license-worker.demandscout.workers.dev/health
curl -sS https://license-worker.demandscout.workers.dev/readyz
curl -sS "https://license-worker.demandscout.workers.dev/readyz?deep=1"
```

### 0.3 Capture local repo state (if you’re about to change code)

```bash
cd /home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator
git status --short
git diff
```

---

## 1) Local Validation (safe to run, no installs)

Run this when you suspect regressions in readiness/runtime/generation wiring.

```bash
cd /home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator
bash .scripts/run-availability-validation.sh
```

If you need the parts individually:

```bash
cd /home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator
node --test worker/test/contract.test.js
cargo test --locked --manifest-path app/src-tauri/Cargo.toml
pnpm --dir app run test
```

---

## 2) Desktop App Crashes / Won’t Launch

### 2.1 Preserve evidence

1) If UI opens:
   - Settings → Diagnostics → note `logPath` and copy `logs/app.log`.
2) If UI does not open:
   - Locate the app data directory for your OS and copy:
     - `logs/app.log`
     - crash log (if present)
     - `config/` folder (especially auth/session/runtime‑pack state)

### 2.2 Minimal recovery attempts (reversible)

1) Make a backup copy of the entire app data directory.
2) Quarantine (rename, don’t delete) suspicious JSON files in app‑data `config/`:
   - session/auth state
   - runtime‑pack state
   - local model profile state
3) Relaunch the app and re-check logs.

Stop and diagnose before doing anything destructive.

---

## 3) License/Auth Incidents

### 3.1 Check Worker endpoints

```bash
curl -sS https://license-worker.demandscout.workers.dev/health
curl -sS https://license-worker.demandscout.workers.dev/readyz
```

If you want external reachability checks too:

```bash
curl -sS "https://license-worker.demandscout.workers.dev/readyz?deep=1"
```

### 3.2 If `/readyz` is not ready

1) Fix bindings/secrets in Cloudflare (do not copy values anywhere).
2) Re-check:

```bash
curl -sS https://license-worker.demandscout.workers.dev/readyz
```

### 3.3 If `/readyz` is ready but clients fail

- Treat as high severity: possible secret rotation mismatch or contract drift.
- Collect desktop logs + Worker logs; avoid guessing.

---

## 4) Generation Stuck / Slow

### 4.1 Immediate recovery

1) Click **Cancel Run** in the UI.
2) If it does not stop, copy logs, then restart the app.

### 4.2 Retry strategy

- Reduce `num_clips`.
- Reduce resolution.
- Switch mode (`api` ↔ `local`) to isolate the fault domain.

### 4.3 What to capture for escalation

- `run_id` (if shown in progress/events)
- last stage (e.g. `download:start`, `transcribe:end`, `clip:start`)
- timestamp window + app version + OS
- desktop `logs/app.log`

---

## 5) Local Mode Broken (runtime‑pack)

### 5.1 Check runtime‑pack manifest route

```bash
curl -sS https://license-worker.demandscout.workers.dev/runtime-pack/manifest.json | head -n 40
```

### 5.2 Recovery steps

1) In app Settings → Diagnostics:
   - note runtime‑pack status + `errorCode` + `debugRef` (if present)
2) Retry runtime‑pack prepare/repair from UI.
3) If you maintain the manifest:
   - verify the asset URL, size, sha256, platform/arch mapping.

---

## 6) Local Model Download Fails

### 6.1 Common fixes (in order)

1) Disk space: free space, then retry.
2) Missing deps/native import failures: fix runtime‑pack first.
3) Network/TLS issues: confirm connectivity and proxy/TLS interception.

### 6.2 Preserve evidence

- Keep model cache and logs until you understand the failure.

---

## 7) Worker/D1 Incidents (Production)

### 7.1 Verify readiness

```bash
curl -sS https://license-worker.demandscout.workers.dev/readyz
curl -sS "https://license-worker.demandscout.workers.dev/readyz?deep=1"
```

### 7.2 Apply migrations (only when needed)

```bash
cd /home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator/worker
wrangler d1 migrations apply license_worker_db --remote
```

### 7.3 Deploy Worker code changes

```bash
cd /home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator/worker
pnpm run deploy
```

### 7.4 Post‑deploy smoke

```bash
curl -sS https://license-worker.demandscout.workers.dev/readyz
curl -sS "https://license-worker.demandscout.workers.dev/readyz?deep=1"
curl -sS https://license-worker.demandscout.workers.dev/runtime-pack/manifest.json | head -n 40
```

---

## 8) Updater Incidents

### 8.1 Verify Worker + updater endpoints

```bash
curl -sS "https://license-worker.demandscout.workers.dev/readyz?deep=1"
curl -i https://license-worker.demandscout.workers.dev/updates/windows/x86_64/0.0.0
curl -i https://license-worker.demandscout.workers.dev/updates/linux/x86_64/0.0.0
curl -i https://license-worker.demandscout.workers.dev/updates/darwin/aarch64/0.0.0
```

### 8.2 Recovery principle

- Fix forward by publishing a new manifest/release; don’t depend on downgrade behavior.

---

## 9) Privacy Deletion Incidents (Admin)

### 9.1 If a deletion request is `failed`

1) Verify Worker readiness:

```bash
curl -sS https://license-worker.demandscout.workers.dev/readyz
```

2) Retry admin approve with a **new** `X-Idempotency-Key` (never reuse keys across different payloads).

### 9.2 If repeated failures occur

- Treat as storage/readiness/schema issue first; fix `/readyz` failures before retrying again.
- Preserve audit trail; do not “manually” complete deletions without a documented plan.

---

## 10) After‑Action Checklist

1) Write an incident note:
   - start/end time, impact, root cause, fix, and how to detect earlier next time.
2) Attach evidence:
   - `/readyz` output (internal)
   - `logs/app.log` (redacted if shared externally)
3) Confirm repo/worktree state:

```bash
cd /home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator
git status --short
```

