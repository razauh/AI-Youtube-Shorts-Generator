# Admin Console Module (Language-Agnostic Guide)

## Purpose
The Admin Console is a privileged operations UI used by support/internal teams to inspect licenses, manage activations, and execute lifecycle actions safely.
Canonical contract reference: `docs/licensing-api-contract-matrix.md` for admin endpoint and auth expectations.

## System Boundary
- This module is a standalone frontend/admin client.
- It does not bypass backend authorization.
- It communicates with licensing admin APIs over HTTPS.

## Core Components
1. Runtime credential input
- Admin key identifier input.
- Admin signing secret input.
- Session-scoped credential persistence for convenience.
- Lock/clear action to remove credentials and reset state.

2. Request signing engine
- Canonical request builder:
  - HTTP method (normalized).
  - Canonical path + sorted query.
  - Unix timestamp.
  - Cryptographic nonce.
  - Request body hash.
  - Key identifier.
- HMAC signature generation over canonical payload.
- Custom auth headers:
  - `x-admin-timestamp`
  - `x-admin-nonce`
  - `x-admin-signature`
  - `x-admin-key-id`

3. Admin API client
- Shared request executor with:
  - Signed headers.
  - JSON body handling.
  - Envelope validation (`ok`, `data`, `error`).
  - Centralized error mapping.
- Exposed operations:
  - Fetch metrics.
  - List licenses (pagination + status filter).
  - List activations (pagination + state filter).
  - Lookup license by raw key.
  - Fetch license status + activation history by ID.
  - Create manual license.
  - Revoke license.
  - Reset device binding.
  - Perform admin license actions.
  - Fetch admin action history.

4. Dashboard state manager
- Metrics state.
- License table state.
- Activation table state.
- Paging state for both lists.
- Request versioning/anti-race guard for concurrent async calls.
- Global loading state and inline message state.
- Explicit utility behaviors:
  - Separate stale-request counters for license-list and activation-list fetches.
  - Only latest request result is applied; older in-flight responses are ignored.
  - Pagination bounds normalization (`offset >= 0`, `1 <= limit <= 100`).
  - Environment-driven default admin key ID with session override.

5. License operations UX
- One-time reveal modal for newly generated license keys.
- Raw key revoke flow.
- Raw key -> license lookup -> reset binding flow.
- Per-row action execution flow.
- Action confirmation prompts for destructive operations.

6. Safe clipboard utilities
- User confirmation before copying sensitive values.
- Success/failure feedback messaging.

## Domain Types Managed by the Console
- License status values:
  - `active`, `past_due`, `cancelled`, `refunded`, `revoked`, `suspended`, `expired`, `network_error`
- License tiers (as surfaced by backend): typically `free`, `pro`, and optionally higher tiers.
- Admin actions:
  - `reset_device`
  - `revoke_license`
  - `delete_license`
  - `force_unbind_delete`
  - `reissue_key`
  - `suspend_license`
  - `unsuspend_license`
  - `add_note`

## API Surface Required by This Module
Canonical source: `docs/licensing-api-contract-matrix.md` (Endpoint Matrix section).

- `GET /v1/admin/metrics`
- `GET /v1/admin/licenses`
- `GET /v1/admin/activations`
- `GET /v1/admin/licenses/{licenseKey}`
- `GET /v1/admin/license/status?license_id=...`
- `POST /v1/admin/licenses`
- `POST /v1/admin/licenses/revoke`
- `POST /v1/admin/license/reset-device`
- `POST /v1/admin/license/actions`
- `GET /v1/admin/license/actions/history?license_id=...`

Contract note:
- `GET /v1/license/status` is a separate public self-status route and is not an alias for admin status routes.
- Admin console must use only `/v1/admin/*` endpoints.

## Security Model
- No static embedded secret in shipped client.
- Admin secret entered at runtime by authorized operator.
- Every request is signed (integrity + authenticity).
- Timestamp + nonce prevent replay attacks.
- Backend remains final authority for authorization decisions.

## Operational Requirements
- Configurable backend base URL per environment.
- Configurable admin key ID for key rotation/multi-key setups.
- Structured error feedback for support teams.
- Auditable actions through backend action-history endpoints.

## Non-Functional Concerns
- Pagination and filtering for large datasets.
- Deterministic request canonicalization across clients.
- Race-safe UI updates under slow/parallel requests.
- Explicit destructive-action confirmations.
- Stale response protection for rapidly changing filters/pages.

## Suggested Portability Checklist
When reimplementing in another stack, keep these invariants:
1. Same canonical signing format and header schema.
2. Same endpoint contracts and response envelope behavior.
3. Same action confirmations and one-time key reveal handling.
4. Same pagination/filter semantics.
5. Same runtime credential flow (never hardcode admin secrets).

---

## Session Management

### Session Lifecycle
1. **Start**: Admin enters key ID + signing secret → credentials stored in session-scoped memory.
2. **Active**: All API requests are signed with session credentials. No idle timeout by default.
3. **Lock**: Admin clicks "Lock" → credentials cleared from memory, UI resets to credential input.
4. **Auto-lock** (recommended): If no admin action for a configurable idle period (default: **15 minutes**), automatically clear credentials and show lock screen.

### Session Scope
- Session is scoped to the **current application window/tab instance**.
- Closing the window/tab destroys the session.
- Credentials must never be persisted to disk, local storage, or cookies.
- Multiple simultaneous sessions (multiple windows) are allowed — each manages its own credentials independently.

### Session State Model
- `locked`: No credentials; only credential input UI is shown.
- `active`: Credentials present; full admin dashboard is accessible.
- Transition: `locked` → (credential entry) → `active` → (lock/timeout/close) → `locked`.

---

## Error Handling UX

### Error Display Patterns

| Error Type | Display Method | Behavior |
|---|---|---|
| **Network error** (timeout, DNS, connection refused) | Inline banner at top of current view | Auto-dismiss after 10 seconds; include "Retry" button |
| **Authentication error** (invalid signature, expired timestamp) | Modal dialog | Require acknowledgment; suggest re-entering credentials |
| **Validation error** (invalid input, missing fields) | Inline field-level error messages | Highlight the offending field; clear on correction |
| **Server error** (5xx) | Inline banner | Show error code and message; include "Retry" button |
| **Not found** (404) | Inline message in the relevant panel | "License not found" or equivalent |
| **Rate limited** (429) | Inline banner | Show retry-after duration; disable action button until cooldown expires |
| **Destructive action failure** | Modal dialog | Show full error details; offer retry or cancel |

### Error Message Rules
- Always show the `reason_code` from the server response when available.
- Never expose raw stack traces, internal error details, or server implementation details.
- Provide actionable guidance: "Check your admin key ID and secret" rather than "HMAC verification failed."
- Log the full error details (including server response) at `ERROR` level for diagnostics.

### Loading States
- **Initial load**: Show a skeleton/placeholder UI for dashboard panels while data loads.
- **Refresh/action**: Show a spinner overlay on the affected panel only, not the entire page.
- **Pagination**: Show inline loading indicator in the table footer during page transitions.
- Disable action buttons while their associated request is in flight to prevent duplicate submissions.

---

## UI Workflows

### Workflow 1: Dashboard Load
```
[Lock Screen] → Enter credentials → [Dashboard]
  ├── Fetch metrics → Display metrics cards
  ├── Fetch licenses (page 1) → Display license table
  └── Fetch activations (page 1) → Display activation table
```

### Workflow 2: License Lookup by Key
```
[Dashboard] → Enter raw key in search → Call GET /v1/admin/licenses/{licenseKey}
  ├── Found → Display license detail panel with activation history
  └── Not found → Show "License not found" inline message
```

### Workflow 3: Revoke License
```
[License Table] → Click "Revoke" on row → Show confirmation modal
  "Are you sure you want to revoke license [ID]? This action is permanent."
  ├── Confirm → Call POST /v1/admin/licenses/revoke → Refresh license table
  └── Cancel → Close modal, no action
```

### Workflow 4: Reset Device Binding
```
[Dashboard] → Enter raw key → Lookup license → Click "Reset Device"
  → Show confirmation modal → Confirm
  → Call POST /v1/admin/license/reset-device → Show success/failure message
```

### Workflow 5: Create Manual License
```
[Dashboard] → Click "Create License" → Show creation form
  → Enter plan_id, tier, optional expiry → Submit
  → Call POST /v1/admin/licenses → Show one-time key reveal modal
  → Admin copies key → Close modal (key never shown again)
```

### Workflow 6: View Action History
```
[License Detail] → Click "Action History" tab
  → Call GET /v1/admin/license/actions/history?license_id=...
  → Display action log table (actor, action, timestamp, details)
```

---

## Accessibility Requirements

### Minimum Requirements
1. **Keyboard navigation**: All interactive elements must be reachable and operable via keyboard alone (Tab, Enter, Escape, Arrow keys).
2. **Focus management**: After modal open/close, focus must move to/from the modal correctly. After destructive action completion, focus must return to a logical element.
3. **Color contrast**: All text must meet WCAG 2.1 AA contrast ratio (4.5:1 for normal text, 3:1 for large text).
4. **Screen reader labels**: All buttons, inputs, and interactive elements must have accessible labels (via `aria-label`, `aria-labelledby`, or visible label association).
5. **Status announcements**: Success/error messages after actions must be announced to screen readers (via `aria-live` regions or equivalent).

### Recommended Enhancements
- Table rows should be navigable with arrow keys.
- Confirmation modals should trap focus while open.
- Loading states should be announced to screen readers.
- Destructive action buttons should use visually distinct styling (e.g., red) and include the action name in the button label (e.g., "Revoke License" not just "Revoke").

---

## Logging Events

All admin console operations must emit structured log events per the Enhanced Logging Event Catalog (`docs/enhanced-logging-event-catalog.md`). At minimum:

| Event | Level | When |
|---|---|---|
| `admin.session.started` | INFO | Credentials entered |
| `admin.session.cleared` | INFO | Credentials cleared/locked |
| `admin.request.sent` | DEBUG | API request dispatched |
| `admin.request.success` | DEBUG | API response received |
| `admin.request.error` | ERROR | API request failed |
| `admin.action.confirmed` | INFO | Destructive action confirmed |
| `admin.action.completed` | INFO | Admin action completed |
| `admin.action.failed` | ERROR | Admin action failed |
| `admin.clipboard.copy` | DEBUG | Sensitive value copied |
| `admin.stale_response.discarded` | DEBUG | Stale response discarded |

---

## Test Strategy

### Unit Tests
- **Canonical request builder**: Verify that the same inputs always produce the same canonical string. Test with: different HTTP methods, query parameters in different orders, empty body, body with special characters.
- **HMAC signature generation**: Verify against known test vectors. Test with different key IDs and secrets.
- **Pagination normalization**: Verify bounds clamping (`offset >= 0`, `1 <= limit <= 100`). Test with negative offset, zero limit, limit > 100.
- **Stale-request guard**: Verify that older responses are discarded when a newer request has been dispatched.
- **Error mapping**: Verify that all server error codes are mapped to user-friendly messages.

### Integration Tests
- **Credential flow**: Enter credentials → verify signed requests succeed → lock → verify credentials cleared → re-enter → verify restored.
- **Dashboard load**: Verify metrics, license table, and activation table all load correctly after credential entry.
- **License CRUD**: Create license → verify one-time reveal → list licenses → verify new license appears → revoke → verify status change.
- **Device reset**: Lookup license → reset device → verify activation table reflects the change.
- **Action history**: Perform actions → verify action history shows correct entries.
- **Pagination**: Navigate pages → verify data changes correctly → verify stale-response guard under rapid pagination.
- **Error scenarios**: Invalid credentials → verify authentication error display. Server unavailable → verify network error display. Invalid input → verify field-level errors.

### Security Tests
- Verify credentials are not persisted to disk after session lock.
- Verify no admin secrets appear in any log output.
- Verify destructive actions require confirmation and cannot be triggered programmatically without user interaction.
- Verify stale-request protection prevents data inconsistency under concurrent requests.

### Accessibility Tests
- Keyboard-only navigation through all workflows.
- Screen reader announcement of action outcomes.
- Color contrast validation for all UI elements.
