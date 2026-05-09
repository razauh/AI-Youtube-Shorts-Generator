# Licensing API Contract Matrix (Canonical)

This document is the single source of truth for Licensing Worker API contracts.
If any other guide conflicts with this file, this file wins.

## Global Contract Rules

- Base path: `/v1`
- Response envelope:
  - success: `{ ok: true, data: ... }`
  - failure: `{ ok: false, error: { code, message, reason_code? } }`
- Content type: JSON for request/response bodies unless noted.
- Sensitive identifiers (license key, device fingerprint) must never be returned in plaintext unless explicitly required by an admin action.

## Authentication Modes

- Public license routes:
  - `activate`: no bearer token required.
  - `validate`, `deactivate`, `license/status`: bearer token required.
- Admin routes:
  - Cloudflare Access gate required.
  - Per-request HMAC signature headers required:
    - `x-admin-timestamp`
    - `x-admin-nonce`
    - `x-admin-signature`
    - `x-admin-key-id`

## Token Standard

- Access tokens are JWTs signed asymmetrically.
- Algorithm policy:
  - preferred: `EdDSA` (`Ed25519`)
  - fallback: `ES256`
- Verifier requirements:
  - validate `alg` and `kid`
  - reject unknown `kid`
  - reject expired tokens
  - reject future-issued tokens beyond allowed clock skew

## Reason Code Taxonomy

- `INVALID_KEY`
- `SUBSCRIPTION_INACTIVE`
- `KEY_BOUND_TO_OTHER_DEVICE`
- `REVOKED`
- `RATE_LIMITED`
- `SERVER_ERROR`

## Offline Policy Contract

- Offline use is allowed after at least one successful activation.
- Client policy:
  - warn after 21 days since last successful `/license/validate`
  - hard lock after 30 days since last successful `/license/validate`
  - lock immediately after any successful online validation returns a revocation/inactive denial

## Endpoint Matrix

| Route | Method | Auth | Purpose | Notes |
|---|---|---|---|---|
| `/v1/license/activate` | POST | none | Activate key on device and issue token | Enforces single active device binding |
| `/v1/license/validate` | POST | bearer token | Revalidate entitlement and device binding; optionally rotate token | Returns deterministic denial reason on failure |
| `/v1/license/deactivate` | POST | bearer token | Release current device binding by policy | May be policy-restricted |
| `/v1/license/status` | GET | bearer token | Public self-status for current token/license context | Not an admin lookup route |
| `/v1/admin/license/reset-device` | POST | admin signed + access | Manual support reset of binding | Audit required |
| `/v1/admin/license/actions` | POST | admin signed + access | Execute admin lifecycle action | Audit required |
| `/v1/admin/license/actions/history` | GET | admin signed + access | Retrieve action history | Supports filters/pagination |
| `/v1/admin/licenses` | GET | admin signed + access | List licenses | Supports filters/pagination |
| `/v1/admin/licenses` | POST | admin signed + access | Create manual/internal license | New key reveal flow handled by console |
| `/v1/admin/licenses` | PUT | admin signed + access | Replace/update license fields | Idempotent update semantics |
| `/v1/admin/licenses` | PATCH | admin signed + access | Partial update license fields | |
| `/v1/admin/licenses/{licenseKey}` | GET | admin signed + access | Lookup license by raw key | `revoke` remains dedicated route |
| `/v1/admin/licenses/revoke` | POST | admin signed + access | Revoke license | Audit required |
| `/v1/admin/activations` | GET | admin signed + access | List activations | Supports filters/pagination |
| `/v1/admin/license/status` | GET | admin signed + access | Admin status lookup by explicit identifier | No aliasing to `/v1/license/status` |
| `/v1/admin/metrics` | GET | admin signed + access | Admin operational metrics | |

## Routing Edge Cases

- `/v1/admin/licenses/{licenseKey}` accepts exactly one non-empty path segment.
- `/v1/admin/licenses/` (trailing slash without key) resolves to `404`.
- `/v1/admin/licenses/revoke` is a dedicated route, not interpreted as `{licenseKey}`.
- Invalid URL-encoding in `{licenseKey}` is rejected as validation error.

## Change Management

- Any API change must update this document in the same pull request.
- Any checklist item marked complete must link to evidence that validates this contract.

---

## Request/Response Schemas

### POST /v1/license/activate

**Request:**
```json
{
  "license_key": "string (required)",
  "device_fingerprint": "string (required, SHA-256 hash)",
  "install_id": "string (optional, UUID)",
  "app_version": "string (required, semver)",
  "platform": "string (required, e.g. 'windows-x64', 'darwin-arm64', 'linux-x64')"
}
```

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "status": "allowed | already_bound_same_device",
    "token": "string (JWT)",
    "expires_at": "ISO 8601 UTC",
    "next_revalidate_at": "ISO 8601 UTC"
  }
}
```

**Failure Response (4xx):**
```json
{
  "ok": false,
  "error": {
    "code": "ACTIVATION_DENIED",
    "message": "Human-readable description",
    "reason_code": "INVALID_KEY | SUBSCRIPTION_INACTIVE | KEY_BOUND_TO_OTHER_DEVICE | REVOKED | RATE_LIMITED"
  }
}
```

### POST /v1/license/validate

**Request:**
- Header: `Authorization: Bearer <token>`
```json
{
  "device_fingerprint": "string (required, SHA-256 hash)"
}
```

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "status": "valid",
    "token": "string (JWT, rotated)",
    "expires_at": "ISO 8601 UTC",
    "next_revalidate_at": "ISO 8601 UTC"
  }
}
```

**Failure Response (4xx):**
```json
{
  "ok": false,
  "error": {
    "code": "VALIDATION_DENIED",
    "message": "Human-readable description",
    "reason_code": "KEY_BOUND_TO_OTHER_DEVICE | REVOKED | SUBSCRIPTION_INACTIVE | RATE_LIMITED"
  }
}
```

### POST /v1/license/deactivate

**Request:**
- Header: `Authorization: Bearer <token>`
```json
{}
```

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "status": "deactivated"
  }
}
```

### GET /v1/license/status

**Request:**
- Header: `Authorization: Bearer <token>`
- No body.

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "license_id": "string",
    "status": "active | past_due | cancelled | refunded | revoked | suspended | expired",
    "tier": "string",
    "expires_at": "ISO 8601 UTC | null",
    "device_bound": true,
    "last_validated_at": "ISO 8601 UTC"
  }
}
```

### POST /v1/admin/licenses

**Request:**
- Headers: admin signed headers
```json
{
  "plan_id": "string (required)",
  "tier": "string (required)",
  "provider": "internal (required for manual creation)",
  "expires_at": "ISO 8601 UTC (optional)",
  "notes": "string (optional)"
}
```

**Success Response (201):**
```json
{
  "ok": true,
  "data": {
    "license_id": "string",
    "license_key": "string (plaintext, ONE-TIME REVEAL ONLY)",
    "status": "active",
    "created_at": "ISO 8601 UTC"
  }
}
```

### POST /v1/admin/licenses/revoke

**Request:**
- Headers: admin signed headers
```json
{
  "license_id": "string (required)",
  "reason": "string (optional)"
}
```

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "license_id": "string",
    "status": "revoked",
    "revoked_at": "ISO 8601 UTC"
  }
}
```

### POST /v1/admin/license/reset-device

**Request:**
- Headers: admin signed headers
```json
{
  "license_id": "string (required)",
  "reason": "string (optional)"
}
```

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "license_id": "string",
    "device_binding_cleared": true
  }
}
```

### GET /v1/admin/licenses

**Request:**
- Headers: admin signed headers
- Query params: `offset` (int, default 0), `limit` (int, default 20, max 100), `status` (optional filter)

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "items": [
      {
        "id": "string",
        "license_key_hash": "string",
        "status": "string",
        "tier": "string",
        "provider": "string",
        "created_at": "ISO 8601 UTC",
        "updated_at": "ISO 8601 UTC",
        "expires_at": "ISO 8601 UTC | null"
      }
    ],
    "total": 0,
    "offset": 0,
    "limit": 20
  }
}
```

### GET /v1/admin/metrics

**Request:**
- Headers: admin signed headers

**Success Response (200):**
```json
{
  "ok": true,
  "data": {
    "total_licenses": 0,
    "active_licenses": 0,
    "total_activations": 0,
    "active_activations": 0,
    "revoked_licenses": 0,
    "licenses_by_status": { "active": 0, "revoked": 0 },
    "licenses_by_tier": { "free": 0, "pro": 0 }
  }
}
```

---

## HTTP Status Code Mapping

| Scenario | Status Code | Notes |
|---|---|---|
| Successful operation | `200 OK` | Standard success |
| Resource created | `201 Created` | License creation |
| Invalid request body / missing fields | `400 Bad Request` | Include field-level details in error message |
| Missing or invalid bearer token | `401 Unauthorized` | Public routes requiring auth |
| Invalid admin signature / expired timestamp / replayed nonce | `401 Unauthorized` | Admin routes |
| Insufficient permissions | `403 Forbidden` | Reserved for future RBAC |
| Resource not found | `404 Not Found` | License lookup miss, unknown routes |
| Method not allowed | `405 Method Not Allowed` | Wrong HTTP method for route |
| Rate limit exceeded | `429 Too Many Requests` | Must include `Retry-After` header |
| Server error | `500 Internal Server Error` | Unexpected failures |
| Upstream provider error | `502 Bad Gateway` | Billing provider failures |
| Service unavailable | `503 Service Unavailable` | Maintenance or overload |

---

## Rate-Limit Response Format

When a rate limit is exceeded, the response must include:

**Response (429):**
```json
{
  "ok": false,
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests. Please retry after N seconds.",
    "reason_code": "RATE_LIMITED"
  }
}
```

**Required Headers:**
| Header | Value | Description |
|---|---|---|
| `Retry-After` | Integer (seconds) | Seconds until the client may retry |
| `X-RateLimit-Limit` | Integer | Maximum requests allowed in window |
| `X-RateLimit-Remaining` | Integer | Requests remaining in current window |
| `X-RateLimit-Reset` | Unix timestamp | When the current window resets |

---

## CORS Specification

### Public License Routes
- `Access-Control-Allow-Origin`: Configurable allowlist of origins. Default: deny all cross-origin requests (desktop apps typically do not need CORS).
- `Access-Control-Allow-Methods`: `POST, GET, OPTIONS`
- `Access-Control-Allow-Headers`: `Content-Type, Authorization`
- `Access-Control-Max-Age`: `86400` (24 hours)

### Admin Routes
- `Access-Control-Allow-Origin`: Configurable allowlist specific to admin console origin(s).
- `Access-Control-Allow-Methods`: `POST, GET, PUT, PATCH, OPTIONS`
- `Access-Control-Allow-Headers`: `Content-Type, x-admin-timestamp, x-admin-nonce, x-admin-signature, x-admin-key-id`
- `Access-Control-Max-Age`: `86400`

### Rules
- Preflight `OPTIONS` requests must be handled for all routes.
- Wildcard (`*`) origin is never allowed.
- CORS configuration must be environment-specific (different origins for staging vs production).
