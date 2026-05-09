# Updater Module (Language-Agnostic Guide)

## Purpose
The Updater Module delivers secure in-app software updates: detect new releases, verify authenticity, download/install, and coordinate restart behavior.

## High-Level Architecture
1. Frontend/update client layer
- Exposes update-check and update-install calls to UI.
- Handles user-facing statuses/messages.
- Normalizes backend/plugin errors into actionable diagnostics.

2. Backend/update execution layer
- Talks to platform updater runtime/plugin.
- Checks remote update metadata.
- Caches a verified pending update between check and install.
- Streams progress events during download/install.

3. Contract/config validation layer
- Defines updater contract metadata.
- Validates that runtime config and signing key configuration are consistent.

## Core Components
1. Check-for-update command
- Reads current app version.
- Calls updater runtime to detect available release.
- Returns:
  - availability flag
  - current version
  - target version
  - release notes/body (optional)
- Stores verified update candidate in memory/state for later install.

2. Install-update command
- Requires a previously verified pending update.
- Downloads and installs selected update package.
- Emits progress events with:
  - downloaded bytes
  - total bytes (if known)
  - percentage (if derivable)
- Emits completion event when packaging phase finishes.

3. Frontend orchestration service
- `checkForUpdates()`:
  - maps “no update” to up-to-date state.
  - maps availability to update candidate.
  - maps exceptions to stable user-facing error messages.
- `installAvailableUpdate()`:
  - executes install flow.
  - handles expected IPC disconnect/restart scenarios as success-with-restart.
  - reports failure otherwise.

4. Policy utilities
- Semantic version normalization/comparison.
- Check cooldown policy (`UPDATE_CHECK_COOLDOWN_MS`).
- Error-to-message mapping:
  - no update
  - no verified pending update
  - signature verification failure
  - network/timeout failures
  - generic fallback

5. Event channel contracts
- Exact progress event name: `updater-progress`.
- Exact completion event name: `updater-finished`.
- Progress payload includes downloaded bytes, optional total bytes, and optional percent.
- Consumer UI should subscribe/unsubscribe cleanly.

## Security Model
1. Signed updates
- Release artifacts must be signed with a private key (kept only in secure CI/release infra).
- Client embeds/verifies with public key.
- Signature mismatch must hard-fail install.

2. Verified check/install sequence
- Installation should only proceed after a successful check generated a verified candidate.
- Installing without verified candidate must fail safely.

3. Trust boundaries
- Update host is untrusted without valid signatures.
- Signature verification is authoritative control.

## Data & State Components
1. Update check result model
- `available`
- `current_version`
- `version` (target)
- `body` (release notes)

2. Frontend status model
- `up_to_date`
- `update_available`
- `error`

3. Install result model
- Success + restart flag.
- Failure + user-facing message.

4. In-memory pending update state
- Stores a single verified update candidate.
- Consumed and cleared when install begins.

## Configuration Components
1. Updater contract metadata
- Schema version.
- Path to app updater config.
- Environment variable name for updater public key.

2. Runtime config dependencies
- Update endpoint URL(s).
- Embedded public key.
- Artifact generation flags in build pipeline.

3. CI/CD secrets
- Private signing key.
- Key password/passphrase.
- Release publishing credentials.

## Operational Lifecycle
1. Build/release pipeline
- Increment version.
- Build platform artifacts.
- Sign artifacts.
- Publish manifest + binaries to update host.

2. Client lifecycle
- Periodically check based on cooldown policy.
- Surface update availability.
- Install on user trigger (or policy trigger).
- Handle restart transition.

## Failure Modes to Explicitly Handle
1. No update available.
2. Network timeout/server unreachable.
3. Signature verification failure.
4. “Install called without verified candidate.”
5. IPC/channel disconnect during restart (expected).

## Test Components
- Serialization/contract tests for update-check payload.
- Policy tests for:
  - version normalization/comparison
  - cooldown logic
  - error-message mapping
  - expected disconnect detection

## Critical Behavioral Invariants
1. Never install unverified artifacts.
2. Never trust remote metadata without signature verification.
3. Keep user messaging deterministic for common failure classes.
4. Treat restart-triggered IPC teardown as expected behavior.

## Portability Checklist
1. Preserve check/install command separation.
2. Preserve pending verified update state semantics.
3. Preserve progress + completion event contracts.
4. Preserve signature-verification-first security model.
5. Preserve cooldown/version/error policy behavior.

---

## Manifest Format

The update server must publish a manifest file that the client fetches during the check phase.

### Required Manifest Fields

| Field | Type | Description |
|---|---|---|
| `version` | String | Target version (semver) |
| `release_date` | ISO 8601 | When this release was published |
| `platforms` | Object | Per-platform artifact entries (keyed by platform identifier) |
| `notes` | String (nullable) | Release notes / changelog body |
| `minimum_client_version` | String (nullable) | Oldest client version that can apply this update directly |
| `mandatory` | Boolean | If true, user cannot defer this update |

### Per-Platform Artifact Entry

| Field | Type | Description |
|---|---|---|
| `url` | String | Download URL for the artifact |
| `size_bytes` | Number | Expected download size |
| `hash_algorithm` | String | Hash algorithm used (e.g., `sha256`) |
| `hash_value` | String | Hex-encoded hash of the artifact |
| `signature` | String | Detached signature of the artifact |
| `format` | String | Artifact format (e.g., `msi`, `dmg`, `appimage`, `nsis`, `tar.gz`) |

### Example Manifest

```json
{
  "version": "2.3.0",
  "release_date": "2026-05-09T12:00:00Z",
  "notes": "Bug fixes and performance improvements.",
  "minimum_client_version": "2.0.0",
  "mandatory": false,
  "platforms": {
    "windows-x64": {
      "url": "https://updates.example.com/v2.3.0/app-2.3.0-x64.msi",
      "size_bytes": 52428800,
      "hash_algorithm": "sha256",
      "hash_value": "a1b2c3d4e5f6...",
      "signature": "base64-encoded-detached-signature...",
      "format": "msi"
    },
    "darwin-arm64": {
      "url": "https://updates.example.com/v2.3.0/app-2.3.0-arm64.dmg",
      "size_bytes": 48000000,
      "hash_algorithm": "sha256",
      "hash_value": "f6e5d4c3b2a1...",
      "signature": "base64-encoded-detached-signature...",
      "format": "dmg"
    }
  }
}
```

### Manifest Validation Rules
- Reject manifests missing required fields.
- Reject manifests where the target `version` is not newer than the current version.
- Reject manifests where the current version is older than `minimum_client_version` (this requires a full reinstall, not an in-app update).
- Reject manifests for unrecognized platform identifiers.

---

## Download Lifecycle

### Pre-Download Checks
1. **Disk space**: Verify available space is at least `2.5 × artifact size_bytes` (artifact + temporary copy + safety margin).
2. **Temp directory**: Create a dedicated temporary directory for download artifacts. Use a subdirectory within the application's data directory, not the system temp directory.
3. **Existing partial download**: If a partial download from a previous attempt exists, verify its hash prefix before resuming. If invalid, delete and start fresh.

### Download Process
1. Initiate HTTP GET to the artifact `url` from the manifest.
2. Support `Range` headers for resume if the server supports it (check for `Accept-Ranges` in response).
3. Write to a temporary file with a `.partial` extension.
4. Emit `updater-progress` events at a throttled rate (maximum 1 per second) with:
   - `downloaded_bytes`: bytes received so far
   - `total_bytes`: from manifest `size_bytes` or `Content-Length` header
   - `percent`: calculated if total is known
5. On completion, rename `.partial` to `.download` (indicates download complete, pending verification).

### Download Failure Handling
- On network timeout: retry per the retry policy (see below).
- On HTTP 4xx: fail permanently with error message; do not retry.
- On HTTP 5xx: retry per the retry policy.
- On disk write error: fail with a user-facing message about insufficient storage.
- On any failure: leave the `.partial` file for potential resume on next attempt.

---

## Integrity Verification

Verification must follow a strict **hash-then-signature** sequence:

### Step 1: Hash Verification
1. Compute the hash of the downloaded artifact using the algorithm specified in the manifest (`hash_algorithm`).
2. Compare the computed hash with `hash_value` from the manifest.
3. If mismatch: delete the downloaded file, log `updater.verify.failure` with `failure_reason: hash_mismatch`, and fail.

### Step 2: Signature Verification
1. Verify the detached signature (`signature` from manifest) against the artifact using the embedded public key.
2. The public key must be embedded in the client binary at build time — never fetched at runtime.
3. If signature verification fails: delete the downloaded file, log `updater.verify.failure` with `failure_reason: signature_invalid`, and fail.

### Post-Verification
- On success: move the file from `.download` to its final staging location and mark the update as a verified candidate.
- The verified candidate is now eligible for installation.

### Critical Rules
- Never install an artifact that has not passed both hash and signature verification.
- Never skip hash verification even if signature verification passes.
- Log all verification outcomes as audit events.

---

## Rollback Strategy

### Pre-Install Backup
Before applying an update:
1. Create a backup of the current application binaries and critical configuration in a dedicated backup directory.
2. Record the current version in a rollback manifest file.
3. The backup must persist across the install process (not in temporary storage).

### Rollback Trigger Conditions
- Installation process fails mid-way (file copy error, permission error).
- Application fails to start after update (crash within first 30 seconds of launch).
- User explicitly requests rollback (if supported by UX policy).

### Rollback Process
1. Log `updater.rollback.started` with the reason.
2. Restore all files from the pre-install backup.
3. Restore the rollback manifest version as the current version.
4. Delete the failed update artifacts.
5. Log `updater.rollback.complete`.
6. If rollback itself fails: log `updater.rollback.error` as FATAL and surface a manual recovery instruction to the user.

### Backup Retention
- Retain exactly one backup (the version prior to the last successful update).
- Delete the backup only after the updated application has launched successfully at least once.

---

## Offline and Retry Behavior

### Offline Detection
- If the update check fails with a network error (DNS failure, connection refused, timeout):
  - Report "offline / unable to reach update server" to the UI.
  - Do not treat this as an application error — the app should continue functioning.
  - Schedule the next check after the cooldown period.

### Retry Policy
- Maximum retry attempts per operation: **3**.
- Backoff strategy: exponential with jitter.
  - Attempt 1: immediate
  - Attempt 2: 2–4 seconds (randomized)
  - Attempt 3: 8–16 seconds (randomized)
- After all retries exhausted:
  - For **check**: report "unable to check for updates" and reset cooldown.
  - For **download**: preserve the `.partial` file for future resume and report failure.
  - For **install**: trigger rollback if applicable.
- Retry only on transient errors (network timeout, 5xx). Do not retry on 4xx, signature failure, or hash mismatch.

### Cooldown Policy
- Default cooldown: configurable, recommended **1 hour** (`3600000 ms`).
- After a successful check (whether update found or not): enforce cooldown.
- After a failed check: enforce a shorter cooldown (recommended: **5 minutes**).
- Cooldown must persist across the check lifecycle but not across app restarts (an app restart should allow an immediate check).

---

## Silent vs Manual Update Modes

### Manual Mode (default)
- User explicitly triggers "Check for Updates" from the UI.
- Update availability is displayed with version and release notes.
- User explicitly triggers "Install Update."
- Progress is shown in the UI.
- Restart prompt on completion.

### Silent Mode (optional)
If the application supports policy-driven updates:
- Check runs on a periodic schedule (e.g., every 24 hours) in the background.
- If a non-mandatory update is found: notify the user but do not install automatically.
- If a `mandatory` update is found: download and prompt for install. The user may defer, but the prompt recurs on every app launch.
- Never install silently while the user is actively working — always prompt before restart.

### Mode Configuration
- Update mode must be configurable: `manual` | `notify` | `auto-download`.
- `manual`: no background checks; user-initiated only.
- `notify`: background checks with user notification on availability.
- `auto-download`: background checks with automatic download; user-initiated install.

---

## Update Channels

### Supported Channels
- `stable` (default): production-ready releases.
- `beta` (optional): pre-release builds for testing.
- `nightly` (optional): development builds.

### Channel Behavior
- Each channel has its own manifest URL.
- A user on `beta` receives both beta and stable releases (whichever is newer).
- Channel switching is a configuration change — it does not trigger immediate downgrade.
- Downgrade across channels is not supported by default; switching from `beta` to `stable` waits for the next stable release that is newer than the current beta version.

---

## Platform Considerations

> These are optional guidance points, not requirements. Adapt to your target platforms.

| Platform | Consideration |
|---|---|
| **Windows** | UAC elevation may be needed for install; use a manifest that requests elevation. MSI/NSIS installers handle this natively. |
| **macOS** | Gatekeeper and notarization: signed `.dmg`/`.app` bundles must be notarized. Code signing is mandatory for distribution outside the App Store. |
| **Linux** | AppImage: self-contained, replace-and-relaunch. Deb/RPM: may require `sudo`; consider a helper process or polkit integration. |
| **All** | File locking: ensure the running binary is not locked by the OS before attempting in-place replacement. Stage the update and apply on restart if needed. |

---

## Version Comparison Algorithm

### Rules
1. Parse versions as semantic version triples: `MAJOR.MINOR.PATCH`.
2. Compare numerically: `MAJOR` first, then `MINOR`, then `PATCH`.
3. Pre-release suffixes (e.g., `-beta.1`, `-rc.2`) sort before the release version: `2.3.0-beta.1 < 2.3.0`.
4. Build metadata (e.g., `+build.123`) is ignored for comparison purposes.
5. Normalize input: strip leading `v` prefix if present (`v2.3.0` → `2.3.0`).

### Examples
```
1.0.0 < 1.0.1 < 1.1.0 < 2.0.0
2.0.0-alpha.1 < 2.0.0-beta.1 < 2.0.0-rc.1 < 2.0.0
v2.3.0 == 2.3.0
```

### Edge Cases
- Non-semver versions: if a version string cannot be parsed as semver, treat the comparison as "update available" and log a warning.
- Same version: treated as "no update available."

---

## Update Lifecycle State Machine

```
┌──────────┐
│   IDLE   │◄──────────────────────────────────────┐
└────┬─────┘                                       │
     │ check()                                     │
     ▼                                             │
┌──────────────┐   no update    ┌──────────────┐   │
│  CHECKING    │──────────────►│  UP_TO_DATE  │───┘
└──────┬───────┘               └──────────────┘
       │ update found                    ▲
       ▼                                 │
┌──────────────────┐   user declines     │
│ UPDATE_AVAILABLE │─────────────────────┘
└──────┬───────────┘
       │ install()
       ▼
┌──────────────┐
│ DOWNLOADING  │──► error ──► DOWNLOAD_FAILED ──► IDLE (retry later)
└──────┬───────┘
       │ complete
       ▼
┌──────────────┐
│  VERIFYING   │──► failure ──► VERIFY_FAILED ──► IDLE (security alert)
└──────┬───────┘
       │ pass
       ▼
┌──────────────┐
│ INSTALLING   │──► error ──► INSTALL_FAILED ──► ROLLBACK
└──────┬───────┘                                    │
       │ success                                    ▼
       ▼                                     ┌──────────┐
┌────────────────────┐                       │ ROLLED   │
│ RESTART_REQUIRED   │                       │  BACK    │──► IDLE
└────────────────────┘                       └──────────┘
```

### State Definitions

| State | Description |
|---|---|
| `IDLE` | No update activity. Ready for next check. |
| `CHECKING` | Update check in progress. |
| `UP_TO_DATE` | Check completed; no update available. |
| `UPDATE_AVAILABLE` | Check completed; update candidate cached. |
| `DOWNLOADING` | Artifact download in progress. |
| `DOWNLOAD_FAILED` | Download failed after retries. |
| `VERIFYING` | Hash and signature verification in progress. |
| `VERIFY_FAILED` | Verification failed — security concern. |
| `INSTALLING` | Update being applied. |
| `INSTALL_FAILED` | Installation failed. |
| `RESTART_REQUIRED` | Install succeeded; restart needed. |
| `ROLLED_BACK` | Rollback completed after failed install. |

### Transition Rules
- Only `IDLE` and `UP_TO_DATE` allow starting a new check.
- Only `UPDATE_AVAILABLE` allows starting an install.
- `VERIFY_FAILED` must not allow retry without a fresh download.
- `INSTALL_FAILED` must trigger rollback before returning to `IDLE`.

---

## Logging Events

All update operations must emit structured log events per the Enhanced Logging Event Catalog (`docs/enhanced-logging-event-catalog.md`). At minimum:

| Event | Level | When |
|---|---|---|
| `updater.check.started` | INFO | Check initiated |
| `updater.check.up_to_date` | INFO | No update found |
| `updater.check.available` | INFO | Update found |
| `updater.check.error` | ERROR | Check failed |
| `updater.download.started` | INFO | Download initiated |
| `updater.download.progress` | DEBUG | Progress (max 1/sec) |
| `updater.download.complete` | INFO | Download finished |
| `updater.download.error` | ERROR | Download failed |
| `updater.verify.started` | INFO | Verification started |
| `updater.verify.success` | INFO | Verification passed |
| `updater.verify.failure` | ERROR | Verification failed |
| `updater.install.started` | INFO | Install initiated |
| `updater.install.complete` | INFO | Install finished |
| `updater.install.error` | ERROR | Install failed |
| `updater.rollback.started` | WARN | Rollback initiated |
| `updater.rollback.complete` | INFO | Rollback finished |
| `updater.rollback.error` | FATAL | Rollback failed |

---

## Test Strategy

### Unit Tests
- Version comparison: all semver comparisons including pre-release, normalization, and edge cases.
- Manifest parsing: valid manifests, missing fields, unknown platforms, malformed JSON.
- Cooldown logic: enforcement, reset, and boundary conditions.
- Error-to-message mapping: all known error codes produce correct user-facing messages.
- Hash computation: known-input-known-output for the selected hash algorithm.

### Integration Tests
- Check flow: mock server returning a manifest → verify correct parsing and candidate caching.
- Download flow: mock server with throttled response → verify progress events and file integrity.
- Verify flow: valid artifact → passes; tampered artifact → fails; wrong signature → fails.
- Install flow: mock install → verify backup creation, file replacement, and post-install state.
- Rollback flow: simulate install failure → verify backup restoration and state reset.
- Offline behavior: no network → verify graceful failure and retry scheduling.

### Security Tests
- Manifest with valid hash but invalid signature → must reject.
- Manifest with invalid hash but valid signature → must reject.
- MITM simulation (corrupted download) → must fail at hash verification.
- Replay of old manifest with known-vulnerable version → must reject if version is not newer.

### Edge Case Tests
- Check during active download → must be rejected or queued.
- App restart between download and install → partial file handling.
- Disk full during download → graceful failure with user message.
- Concurrent check calls → only one proceeds; others are deduplicated.
