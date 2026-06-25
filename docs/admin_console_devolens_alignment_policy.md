# Admin Console Devolens Alignment Policy

This document defines the alignment, retention, and migration decisions for each section and action in the Desktop Admin Console, ensuring Devolens/Cryptolens remains the single source of truth for licensing while protecting user privacy.

---

## 1. Screen & Command Inventory Decisions

| Screen Section | Command / Action | Current Behavior | Alignment Decision | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **overview** | `loadOverview` | Reads D1 license, binding, reset, deletion, and audit metrics. | **Retain** | Useful dashboard for system state; does not duplicate authority. |
| **reset_requests** | `approveResetRequest`, `rejectResetRequest` | Mutates D1 `reset_requests` and `device_bindings` to unbound/inactive. | **Replace with Devolens Flow** | A reset approval must trigger Devolens `/api/key/Deactivate` or `/api/key/BlockKey` on the backend to clear the binding on Cryptolens. |
| **delete_requests** | `approveDeletionRequest`, `rejectDeletionRequest` | Deletes/anonymizes local D1 client records. | **Retain & Sync** | GDPR deletion of local logs is worker-specific, but the corresponding key must be blocked on Devolens. |
| **licenses** | `listLicenses`, `disableLicense` | Lists license rows and performs custom disable mutation on D1. | **Replace with Devolens Link** | D1 is not the license source of truth. Direct license blocking/disabling should be done on the Devolens management portal. |
| **device_bindings** | `listDeviceBindings` | Lists active and inactive device fingerprint mappings from D1. | **Replace with Devolens Flow** | Device binding queries and deactivations should call Devolens device list APIs. |
| **audit_events** | `listAuditEvents` | Lists local admin activity audit events from D1. | **Retain** | Local compliance audit log of admin action history. |
| **idempotency** | `listIdempotencyRecords` | Lists webhook/API idempotency status from D1. | **Retain** | Worker-specific request replay protection; not a Devolens concern. |

---

## 2. Security & Privacy Safeguards

1. **Email and License Hash Redaction**:
   - Masked license keys (`••••-1234`) and hashed license values are displayed instead of full plaintext keys.
   - Emails must be masked (e.g. `b***@example.com`) in overview tables. Full emails are only revealed in detail views with `super_admin` credentials.
2. **Access Control**:
   - High-privileged actions (GDPR deletion approvals, database index cleanups) require `super_admin` API tokens.
   - Regular reset approvals require `admin_token` verification.
3. **No Direct D1 Authority**:
   - The UI must make it clear that disabling a license in D1 is a local cache invalidation. Cryptolens/Devolens must remain the authority for enforcement.

---

## 3. Transition Strategy & Feature Flags

- **Phased Rollout**: Hide the legacy direct-mutate screens behind UI feature flags (`ENABLE_LEGACY_D1_ADMIN=false`) during the migration to Devolens console links.
- **Verification**: The alignment contracts are tested under `app/src/tests/admin/admin_inventory.test.ts`.
