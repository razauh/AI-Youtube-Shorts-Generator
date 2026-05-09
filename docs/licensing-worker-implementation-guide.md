# Demand Scout Licensing Worker - Implementation Guide

> **Document Classification: PROJECT-SPECIFIC**
> This is the project-specific implementation guide for the Cloudflare Workers licensing backend.
> For language-agnostic specifications, see: `licensing-worker-language-agnostic-guide.md`, `licensing-api-contract-matrix.md`.

This document is the canonical implementation guide for the separate licensing backend repository.
API/auth/reason-code contracts are canonically defined in `docs/licensing-api-contract-matrix.md`.
If this guide conflicts with that matrix for API behavior, the matrix wins.

## 1. Goals
- Enforce `1 key = 1 active device` server-side.
- Support two entitlement modes:
  - `gumroad`: verify purchase/subscription status via Gumroad.
  - `internal`: manage entitlement entirely in our own DB.
- Provide a stable `/v1` API consumed by the Tauri app.
- Provide admin operations for reset/revoke and support audits.

## 2. Runtime and Data
- Runtime: Cloudflare Workers.
- Primary storage: D1.
- Optional cache: KV (read-through cache only).
- Source of truth: D1 (never KV-only for activation binding).

## 3. Non-Negotiable Policy
- Maximum active devices per license: `1`.
- Offline fallback at app side is allowed after a successful activation, but with bounded risk controls:
  - normal offline allowance: up to `30 days` since last successful `/validate`
  - soft warning starts at `21 days` offline
  - hard lock after `30 days` until online validation succeeds
  - immediate lock once a revocation is observed on any successful online validation
- Device transfer: manual support reset only.
- Server must return deterministic denial reason codes.

## 4. API Surface
Canonical source: `docs/licensing-api-contract-matrix.md` (Endpoint Matrix section).

- `POST /v1/license/activate`
- `POST /v1/license/validate`
- `POST /v1/license/deactivate`
- `GET /v1/license/status` (public, token-authenticated self status only)
- `POST /v1/admin/license/reset-device`
- `POST /v1/admin/license/actions`
- `GET /v1/admin/license/actions/history`
- `GET /v1/admin/licenses`
- `GET /v1/admin/activations`
- `GET /v1/admin/licenses/{licenseKey}`
- `POST /v1/admin/licenses`
- `PUT /v1/admin/licenses`
- `PATCH /v1/admin/licenses`
- `POST /v1/admin/licenses/revoke`
- `GET /v1/admin/metrics`
- `GET /v1/admin/license/status` (admin lookup by license id; no aliasing to public route)

Reason codes:
Canonical source: `docs/licensing-api-contract-matrix.md` (Reason Code Taxonomy section).

- `INVALID_KEY`
- `SUBSCRIPTION_INACTIVE`
- `KEY_BOUND_TO_OTHER_DEVICE`
- `REVOKED`
- `RATE_LIMITED`
- `SERVER_ERROR`

## 5. Data Model
- `licenses`
  - hashed key, provider, provider reference, plan, status.
- `activations`
  - license id, hashed device fingerprint, install id, first/last seen, revoked timestamp.
  - one active activation per license must be enforced.
- `activation_events`
  - audit trail for activation/validation denials/success.
- `admin_actions`
  - support operations and reset/revoke history.

## 6. Security
- Token format and signing are mandatory and explicit:
  - use JWT with asymmetric signing (`EdDSA`/`Ed25519` preferred; `ES256` acceptable fallback)
  - private key lives only in Worker secrets
  - public verification key is distributed to trusted verifiers/clients
  - include and validate `kid` header for key rotation
  - reject unknown `alg`, unknown `kid`, expired tokens, and future-issued tokens beyond small clock skew
- Do not store plaintext license keys or raw device fingerprints.
- Rate-limit activation/validation by IP and key hash.
- Protect admin endpoints with both:
  - Cloudflare Access (network/auth perimeter), and
  - per-request HMAC admin signing headers (`x-admin-timestamp`, `x-admin-nonce`, `x-admin-signature`, `x-admin-key-id`)
- Enforce replay protection (`admin_nonces`) and timestamp freshness checks.
- Emit structured redacted logs only.

## 7. Provider Abstraction
- Implement `BillingProvider` contract:
  - `verifyEntitlement(licenseKey)`.
- Implementations:
  - `GumroadProvider` (calls Gumroad verify API).
  - `InternalProvider` (DB-only entitlement).
- Switch with `BILLING_PROVIDER` env.

## 8. Implementation Phases
1. Scaffold repo + migrations + config.
2. Build core hashing/token helpers.
3. Build DB access and service layer.
4. Build `/activate` and `/validate`.
5. Add admin routes and audit logging.
6. Add provider implementations and switch logic.
7. Add tests and deployment pipeline.

## 9. Acceptance Criteria
- First activation binds key to one device.
- Same device can re-activate.
- Different device is denied until admin reset.
- Revoked/inactive entitlements are denied on validate.
- Admin reset re-allows next activation.
- No plaintext secrets in DB/logs.

## 10. Operational Telemetry
Worker emits structured metric counter logs under event `metric.counter`.

Recommended alerts:
- `licensing.provider.gumroad.http_non_200`
- `licensing.provider.gumroad.exception`
- `licensing.activate.denied`
- `licensing.validate.denied`
- `licensing.validate.revoked_hit`
- `licensing.validate.device_conflict_hit`
- `licensing.rate_limit.hit`

Detailed setup guidance: `docs/licensing-observability-runbook.md`.
