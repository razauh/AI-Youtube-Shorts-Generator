# Shared State Machines (Language-Agnostic Guide)

This document provides formal state machine definitions for all modules. Each module guide references its state machine from here. Implementations must enforce these transition rules.

---

## 1. License Lifecycle State Machine

### States

| State | Description | Terminal? |
|---|---|---|
| `active` | License is valid and entitlement is current | No |
| `past_due` | Payment overdue, grace period active | No |
| `suspended` | Temporarily disabled by admin | No |
| `cancelled` | Subscription cancelled | No |
| `expired` | Time-limited license expired | No |
| `revoked` | Permanently revoked | Yes |
| `refunded` | Payment refunded | Yes |

### Transitions

| From | To | Trigger | Actor |
|---|---|---|---|
| `active` | `suspended` | `suspend_license` | Admin |
| `active` | `cancelled` | Subscription cancelled | Provider / Admin |
| `active` | `expired` | `expires_at` reached | System |
| `active` | `revoked` | `revoke_license` | Admin |
| `active` | `refunded` | Refund processed | Provider |
| `past_due` | `active` | Payment received | Provider |
| `past_due` | `cancelled` | Grace period expired | Provider |
| `past_due` | `revoked` | `revoke_license` | Admin |
| `suspended` | `active` | `unsuspend_license` | Admin |
| `suspended` | `revoked` | `revoke_license` | Admin |
| `cancelled` | `revoked` | `revoke_license` | Admin |
| `expired` | `active` | Renewal / reactivation | Provider |
| `expired` | `revoked` | `revoke_license` | Admin |

### Validation Rules
- Only `active` and `past_due` allow successful `/validate`.
- `suspended` denies with `REVOKED` reason code.
- All terminal states deny all operations permanently.

---

## 2. Activation Binding State Machine

### States

| State | Description |
|---|---|
| `unbound` | License has no active device binding |
| `bound` | License is bound to exactly one device |
| `revoked_binding` | Previous binding was revoked (by deactivation or admin reset) |

### Transitions

| From | To | Trigger |
|---|---|---|
| `unbound` | `bound` | Successful activation |
| `bound` | `bound` | Re-activation from same device (idempotent) |
| `bound` | `revoked_binding` → `unbound` | Deactivation or admin reset-device |
| `unbound` | `bound` | New activation after reset |

### Rules
- `bound` + activation from different device → denied (`KEY_BOUND_TO_OTHER_DEVICE`)
- Binding revocation sets `revoked_at` on the activation record
- After revocation, the license transitions to `unbound` and accepts new activation

---

## 3. Updater Lifecycle State Machine

### States

| State | Description | Terminal? |
|---|---|---|
| `IDLE` | No update activity | No |
| `CHECKING` | Update check in progress | No |
| `UP_TO_DATE` | No update available | No |
| `UPDATE_AVAILABLE` | Verified candidate cached | No |
| `DOWNLOADING` | Artifact download in progress | No |
| `DOWNLOAD_FAILED` | Download failed after retries | No |
| `VERIFYING` | Hash + signature verification | No |
| `VERIFY_FAILED` | Verification failed | No |
| `INSTALLING` | Update being applied | No |
| `INSTALL_FAILED` | Installation failed | No |
| `RESTART_REQUIRED` | Install succeeded, restart needed | No |
| `ROLLED_BACK` | Rollback completed | No |

### Transitions

| From | To | Trigger |
|---|---|---|
| `IDLE` | `CHECKING` | `check()` called |
| `CHECKING` | `UP_TO_DATE` | No update found |
| `CHECKING` | `UPDATE_AVAILABLE` | Update found |
| `CHECKING` | `IDLE` | Check failed (error) |
| `UP_TO_DATE` | `IDLE` | Timeout / user action |
| `UP_TO_DATE` | `CHECKING` | `check()` called (after cooldown) |
| `UPDATE_AVAILABLE` | `DOWNLOADING` | `install()` called |
| `UPDATE_AVAILABLE` | `IDLE` | User declines |
| `DOWNLOADING` | `VERIFYING` | Download complete |
| `DOWNLOADING` | `DOWNLOAD_FAILED` | Download error after retries |
| `DOWNLOAD_FAILED` | `IDLE` | Reset / user dismisses |
| `VERIFYING` | `INSTALLING` | Verification passed |
| `VERIFYING` | `VERIFY_FAILED` | Verification failed |
| `VERIFY_FAILED` | `IDLE` | Reset (requires fresh download) |
| `INSTALLING` | `RESTART_REQUIRED` | Install succeeded |
| `INSTALLING` | `INSTALL_FAILED` | Install error |
| `INSTALL_FAILED` | `ROLLED_BACK` | Rollback succeeded |
| `INSTALL_FAILED` | `IDLE` | Rollback failed (FATAL logged) |
| `ROLLED_BACK` | `IDLE` | Acknowledged |
| `RESTART_REQUIRED` | (app exits) | User triggers restart |

### Guard Conditions
- `check()` is only valid from `IDLE` or `UP_TO_DATE`
- `install()` is only valid from `UPDATE_AVAILABLE`
- `VERIFY_FAILED` → `CHECKING` requires going through `IDLE` first (no direct retry of failed verification without fresh download)

---

## 4. Admin Console Session State Machine

### States

| State | Description |
|---|---|
| `locked` | No credentials; only credential input visible |
| `authenticating` | Credentials entered, first request in flight |
| `active` | Credentials valid, dashboard accessible |
| `error` | Authentication failed, showing error |

### Transitions

| From | To | Trigger |
|---|---|---|
| `locked` | `authenticating` | Credentials submitted |
| `authenticating` | `active` | First request succeeds |
| `authenticating` | `error` | First request fails (auth error) |
| `error` | `locked` | User acknowledges error |
| `active` | `locked` | User clicks lock / auto-lock timeout / window close |

---

## 5. Offline Grace State Machine (Client-Side Licensing)

### States

| State | Description |
|---|---|
| `online_valid` | Last validation succeeded, app is online |
| `offline_ok` | Offline, within grace period (< 21 days) |
| `offline_warning` | Offline, grace warning threshold (21–30 days) |
| `offline_locked` | Offline, grace expired (> 30 days) |
| `revoked_locked` | Online validation returned revocation |

### Transitions

| From | To | Trigger |
|---|---|---|
| `online_valid` | `offline_ok` | Network unavailable |
| `offline_ok` | `offline_warning` | 21 days since last validation |
| `offline_warning` | `offline_locked` | 30 days since last validation |
| `offline_ok` | `online_valid` | Successful online validation |
| `offline_warning` | `online_valid` | Successful online validation |
| `offline_locked` | `online_valid` | Successful online validation |
| `online_valid` | `revoked_locked` | Validation returns revocation/inactive |
| `offline_ok` | `revoked_locked` | Comes online, validation returns revocation |
| `offline_warning` | `revoked_locked` | Comes online, validation returns revocation |

### Rules
- `revoked_locked` is terminal until a new activation succeeds.
- `offline_locked` is recoverable if online validation succeeds.
- Grace period timers use monotonic clock + server timestamp anchoring (see Licensing Worker guide, Clock Manipulation Defenses).

---

## Implementation Rules

1. **Every state machine must be implemented as an explicit state type** — not implicit boolean flags.
2. **Invalid transitions must be rejected** — attempting an invalid transition must log a warning and be a no-op.
3. **All transitions must be logged** at `INFO` or `DEBUG` level with the old state, new state, and trigger.
4. **Terminal states must be enforced** — no code path should allow exiting a terminal state.
5. **State machines must be testable in isolation** — provide a way to inject the current state for testing.
