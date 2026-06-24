# Devolens Device Reset/Unbind Design & Mapping

This document describes how the machine limit error and device reset lifecycle map from Devolens to the custom Tauri application logic.

---

## 1. Machine Limit Detection & Error Mapping

When a client attempts to activate a license key on a new device, Devolens validates the number of currently active machine registrations. If the maximum allowed limit is reached, Devolens responds with a `1` result and the error message `"Machine limit reached."`.

### Mapping Flow
1. **Response Parser**: `parse_devolens_activation_response` scans the error message case-insensitively for the keyword `"machine"`.
2. **Error Translation**: The response is mapped directly to `AuthError::DeviceAlreadyBound`.
3. **Tauri IPC Command**: `activate_license` catches `AuthError::DeviceAlreadyBound` and serializes it to `AuthCommandError` with:
   - `code`: `"device_already_bound"`
   - `message`: `"license is already bound to another device"`
4. **Frontend State**: The frontend receives `"device_already_bound"` and transitions the auth state lifecycle to `'device_bound_elsewhere'`.

---

## 2. Reset / Unbind Lifecycle Mapping

Under the custom Worker flow, resets were pending admin approval. Under Devolens, reset operations deactivate machine activations.

### State Transitions
- **Unauthenticated**: The initial state.
- **Machine Limit Reached**: Activation fails with `device_already_bound`.
- **Reset Initiated**: The user clicks "Request Reset" which calls `request_device_reset`.
- **Pending/Approved**:
  - Reset request transitions to pending or approved based on Devolens reset capability/admin policy.
  - Once approved, Devolens deactivates the existing machine registrations, freeing up activation slots.
- **Successful Re-activation**: The user triggers activation again, which now succeeds because the machine registration limit is no longer exceeded.
