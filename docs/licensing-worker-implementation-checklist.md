# Licensing Worker Checklist

> **Document Classification: PROJECT-SPECIFIC**
> This is a project-specific tracking checklist for the Cloudflare Worker licensing backend.
> For language-agnostic specifications, see: `licensing-worker-language-agnostic-guide.md`.

Guide reference: `docs/licensing-worker-implementation-guide.md`
Contract reference: `docs/licensing-api-contract-matrix.md`

Rule: every item below must be implemented in compliance with the guide.

## Phase 1 - Foundation
- [x] Create canonical implementation guide.
- [x] Create dedicated `licensing-worker/` repo folder scaffold.
- [x] Add Worker config (`wrangler.toml`, `package.json`, `tsconfig.json`).
- [x] Add D1 initial migration for licensing tables.

## Phase 2 - Core Backend Components
- [x] Define environment bindings/types.
- [x] Add hashing utility for key/device hashes.
- [x] Add token signing/verification helper.
- [x] Add DB repository helpers for licenses/activations/audit events.
- [x] Add provider interface and provider switch.
- [x] Implement full Gumroad verify integration logic.

## Phase 3 - API Endpoints
- [x] Add `POST /v1/license/activate` route skeleton + core logic.
- [x] Add `POST /v1/license/validate` route skeleton + core logic.
- [x] Add `POST /v1/license/deactivate` route skeleton.
- [x] Add `POST /v1/admin/license/reset-device` route skeleton.
- [x] Add `GET /v1/license/status` route skeleton.
- [x] Add robust request validation/error envelopes.
- [x] Add rate limiting per IP and key hash.

## Phase 4 - Hardening + Ops
- [x] Add admin auth hardening.
- [x] Add structured redacted logging.
- [x] Add replay protections/nonce handling.
- [x] Add key rotation support (`kid`).
- [x] Add CI pipeline and deployment workflow.

## Phase 5 - Tests
- [x] Unit tests for hashing/token/provider mapping.
- [x] Integration tests for activate/validate decision matrix.
- [x] Concurrency/race tests for 1-device enforcement.
- [x] Admin reset/revoke tests.

## Evidence Links (required for every checked item)
- CI workflow run URL:
- Unit test command + output artifact:
- Integration test command + output artifact:
- Concurrency test report artifact:
- Migration apply log (staging):
- Staging deployment revision/hash:
