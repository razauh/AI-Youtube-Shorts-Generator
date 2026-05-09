# Project Licensing Implementation Checklist

> **Document Classification: PROJECT-SPECIFIC**
> This is a project-specific tracking checklist for the desktop app (Tauri) licensing integration.
> For language-agnostic specifications, see: `licensing-worker-language-agnostic-guide.md`.

This checklist covers only the desktop app (project-side) work. Cloudflare Worker implementation is tracked separately.
Contract reference: `docs/licensing-api-contract-matrix.md` for endpoint/auth/reason-code/offline behavior assumptions.

## 1) Core App Gating
- [x] Add global license state store with `loading`, `locked`, `active` states.
- [x] Add startup license initialization before app content is shown.
- [x] Gate all routes until license is active.
- [x] Show dedicated activation UI when locked.
- [ ] Add explicit route guard for deep-link edge cases (optional hardening).

## 2) Activation and Validation Flow
- [x] Add API client methods for `/v1/license/activate`.
- [x] Add API client methods for `/v1/license/validate`.
- [x] Build deterministic device fingerprint + persistent install ID.
- [x] Handle activation success/failure with reason codes.
- [x] Allow offline fallback after at least one successful activation, bounded by policy (`warn at 21 days`, `hard limit at 30 days` since last successful validate).
- [ ] Add retry/backoff policy for transient network errors.

## 3) Secure Local Storage
- [x] Add Tauri commands to save/load/clear license activation token in OS keychain.
- [x] Persist non-secret licensing metadata in UI state (`install_id`, `grace_until`, `last_check_at`).
- [ ] Encrypt non-secret metadata at rest (optional).

## 4) UX and Operational Controls
- [x] Provide activation form + enter key submit.
- [x] Display backend denial reasons in activation view.
- [ ] Add “Deactivate this device” action in settings screen.
- [ ] Add license status badge/details in configuration screen.
- [ ] Add support link/help text for reset requests.

## 5) Config and Environment
- [x] Add `VITE_LICENSE_API_BASE_URL` wiring in frontend API client.
- [x] Add `.env.example` entry for license API URL.
- [x] Add README section: license setup and expected backend endpoints.

## 6) Tests
- [ ] Unit-test license state transitions in store.
- [ ] Integration-test startup lock/unlock behavior.
- [ ] Integration-test bounded offline launch path after prior activation.
- [ ] Integration-test denial reasons from backend response.

## 7) Evidence Links (required for every checked item)
- PR link(s):
- CI workflow run URL:
- Frontend unit test command + output artifact:
- Frontend integration/e2e test command + output artifact:
- Manual QA notes (activation, offline, revoke/reset):
