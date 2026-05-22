import { writable } from 'svelte/store';
import type { AuthCommandError, AuthStateView, DeviceResetInput } from '../authContracts';
import {
  activateLicense,
  clearLocalSession,
  getAuthState,
  getDeviceResetStatus,
  requestDeviceReset,
  validateSession,
} from '../api/authClient';

export type AuthLifecycle =
  | 'checking'
  | 'unauthenticated'
  | 'activating'
  | 'licensed'
  | 'licensed_offline_grace'
  | 'reauth_required'
  | 'device_bound_elsewhere'
  | 'reset_pending'
  | 'reset_approved_unbound'
  | 'reset_rejected'
  | 'reset_expired'
  | 'error';

export type DeviceResetLifecycle =
  | 'idle'
  | 'pending'
  | 'approved'
  | 'rejected'
  | 'expired'
  | 'not_found'
  | 'error';

export interface AuthStateShape {
  lifecycle: AuthLifecycle;
  authState: AuthStateView | null;
  resetRequestId: string | null;
  resetStatus: DeviceResetLifecycle;
  resetStatusMessage: string | null;
  resetError: AuthCommandError | null;
  error: AuthCommandError | null;
}

interface RequestResetOptions {
  preserveLicensedSession?: boolean;
}

const initialState: AuthStateShape = {
  lifecycle: 'checking',
  authState: null,
  resetRequestId: null,
  resetStatus: 'idle',
  resetStatusMessage: null,
  resetError: null,
  error: null,
};

const RESET_CACHE_KEY = 'auth.device_reset_cache.v1';

function toResetLifecycle(status: string): DeviceResetLifecycle {
  switch (status) {
    case 'pending':
      return 'pending';
    case 'approved':
      return 'approved';
    case 'rejected':
      return 'rejected';
    case 'expired':
      return 'expired';
    case 'not_found':
      return 'not_found';
    default:
      return 'error';
  }
}

function statusMessageFor(status: DeviceResetLifecycle): string | null {
  switch (status) {
    case 'pending':
      return 'Device reset request pending. We will keep checking for an admin decision.';
    case 'approved':
      return 'Device reset approved. You can now activate again on this device, or activate from another device.';
    case 'rejected':
      return 'Device reset request rejected. You can retry or contact support if this looks wrong.';
    case 'expired':
      return 'Device reset request expired. Please submit a new request if you still need to move devices.';
    case 'not_found':
      return 'Device reset request not found. Please verify the request id and try again.';
    case 'idle':
    case 'error':
    default:
      return null;
  }
}

function loadResetCache(): { requestId: string; status: DeviceResetLifecycle; message?: string | null } | null {
  try {
    const raw = localStorage.getItem(RESET_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as { requestId?: unknown; status?: unknown; message?: unknown };
    if (typeof parsed.requestId !== 'string' || typeof parsed.status !== 'string') return null;
    const status = toResetLifecycle(parsed.status);
    return {
      requestId: parsed.requestId,
      status,
      message: typeof parsed.message === 'string' ? parsed.message : null,
    };
  } catch (_e) {
    return null;
  }
}

function saveResetCache(requestId: string, status: DeviceResetLifecycle, message: string | null) {
  try {
    localStorage.setItem(
      RESET_CACHE_KEY,
      JSON.stringify({ requestId, status, message }),
    );
  } catch (_e) {
    // ignore storage errors
  }
}

function clearResetCache() {
  try {
    localStorage.removeItem(RESET_CACHE_KEY);
  } catch (_e) {
    // ignore storage errors
  }
}

function safeAuthMessage(code: string): string {
  switch (code) {
    case 'invalid_license_key':
      return 'Invalid license key. Please check and try again.';
    case 'device_already_bound':
      return 'This license is already in use on another device.';
    case 'reauth_required':
      return 'Session expired. Re-enter your license key to continue.';
    case 'worker_unreachable':
      return 'Unable to reach the license service right now. Please try again shortly.';
    case 'invalid_purchase_email':
      return 'Invalid purchaser email. Please verify and try again.';
    case 'invalid_reset_request':
      return 'Unable to request a device reset. This requires your license key on this device—activate again with your license key, then retry.';
    case 'reset_request_not_found':
      return 'Reset request not found. Check the request ID and try again.';
    default:
      return 'Authentication failed. Please try again.';
  }
}

function commandError(error: unknown): AuthCommandError {
  function parseJsonAuthError(raw: string): AuthCommandError | null {
    const trimmed = raw.trim();
    const candidate =
      trimmed.startsWith('{') && trimmed.endsWith('}')
        ? trimmed
        : (() => {
            const start = trimmed.indexOf('{');
            const end = trimmed.lastIndexOf('}');
            if (start === -1 || end === -1 || end <= start) return null;
            return trimmed.slice(start, end + 1);
          })();
    if (!candidate) return null;
    try {
      const parsed = JSON.parse(candidate) as { code?: unknown; message?: unknown };
      if (typeof parsed?.code === 'string') {
        return { code: parsed.code, message: safeAuthMessage(parsed.code) };
      }
    } catch (_e) {
      // ignore parse failures
    }
    return null;
  }

  function extract(err: unknown): AuthCommandError | null {
    if (!err) return null;
    if (typeof err === 'string') {
      return parseJsonAuthError(err);
    }
    if (err instanceof Error) {
      return parseJsonAuthError(err.message);
    }
    if (typeof err === 'object') {
      if (
        'code' in err &&
        'message' in err &&
        typeof (err as { code: unknown }).code === 'string' &&
        typeof (err as { message: unknown }).message === 'string'
      ) {
        const authErr = err as AuthCommandError;
        return { code: authErr.code, message: safeAuthMessage(authErr.code) };
      }
      if ('error' in err) {
        return extract((err as { error?: unknown }).error);
      }
      if ('message' in err && typeof (err as { message?: unknown }).message === 'string') {
        return parseJsonAuthError((err as { message: string }).message);
      }
    }
    return null;
  }

  return extract(error) ?? { code: 'unknown', message: safeAuthMessage('unknown') };
}

function lifecycleFromAuthState(authState: AuthStateView): AuthLifecycle {
  switch (authState.status) {
    case 'licensed':
      return 'licensed';
    case 'licensed_offline_grace':
      return 'licensed_offline_grace';
    case 'reauth_required':
      return 'reauth_required';
    case 'reset_pending':
      return 'reset_pending';
    case 'reset_approved_unbound':
      return 'reset_approved_unbound';
    case 'reset_rejected':
      return 'reset_rejected';
    case 'reset_expired':
      return 'reset_expired';
    case 'unauthenticated':
    default:
      return 'unauthenticated';
  }
}

function resetIdFrom(authState: AuthStateView): string | null {
  return 'request_id' in authState ? authState.request_id : null;
}

export function createAuthState() {
  const { subscribe, set, update } = writable<AuthStateShape>(initialState);
  let resetPollInterval: number | null = null;

  function stopResetPolling() {
    if (resetPollInterval !== null) {
      window.clearInterval(resetPollInterval);
      resetPollInterval = null;
    }
  }

  function maybeStartResetPolling(requestId: string) {
    if (resetPollInterval !== null) return;
    resetPollInterval = window.setInterval(async () => {
      try {
        await api.pollResetStatus(requestId);
      } catch (_e) {
        // polling errors should not kick the user out of the app
      }
    }, 15_000);
  }

  const api = {
    pollResetStatus: async (requestId: string) => {
      const view = await getDeviceResetStatus(requestId);
      const resetStatus = toResetLifecycle(view.status);
      const resetMessage = view.message ?? statusMessageFor(resetStatus);

      set({
        lifecycle: lifecycleFromAuthState(view.auth_state),
        authState: view.auth_state,
        resetRequestId: view.request_id,
        resetStatus,
        resetStatusMessage: resetMessage,
        resetError: null,
        error: null,
      });

      if (resetStatus === 'pending') {
        saveResetCache(view.request_id, resetStatus, resetMessage);
        maybeStartResetPolling(view.request_id);
        return;
      }

      // terminal states
      saveResetCache(view.request_id, resetStatus, resetMessage);
      stopResetPolling();

      if (resetStatus === 'approved') {
        // reset_approved_unbound should now drive the activation flow
        return;
      }
    },
  };

  function applyAuthState(authState: AuthStateView) {
    const resetStatus =
      authState.status === 'reset_pending'
        ? 'pending'
        : authState.status === 'reset_approved_unbound'
          ? 'approved'
          : authState.status === 'reset_rejected'
            ? 'rejected'
            : authState.status === 'reset_expired'
              ? 'expired'
              : 'idle';
    const resetRequestId = resetIdFrom(authState);
    const resetStatusMessage =
      resetStatus === 'approved' && 'message' in authState ? authState.message : statusMessageFor(resetStatus);
    if (resetRequestId && resetStatus !== 'idle') {
      saveResetCache(resetRequestId, resetStatus, resetStatusMessage);
      if (resetStatus === 'pending') {
        maybeStartResetPolling(resetRequestId);
      } else {
        stopResetPolling();
      }
    }

    set({
      lifecycle: lifecycleFromAuthState(authState),
      authState,
      resetRequestId,
      resetStatus,
      resetStatusMessage,
      resetError: null,
      error: null,
    });
  }

  return {
    subscribe,
    reset: () => set(initialState),
    bootstrap: async () => {
      update((state) => ({ ...state, lifecycle: 'checking', error: null }));
      try {
        applyAuthState(await getAuthState());
        const cached = loadResetCache();
        if (cached && cached.requestId && cached.status !== 'idle') {
          update((state) => ({
            ...state,
            resetRequestId: cached.requestId,
            resetStatus: cached.status,
            resetStatusMessage: cached.message ?? statusMessageFor(cached.status),
            resetError: null,
          }));
          if (cached.status === 'pending') {
            maybeStartResetPolling(cached.requestId);
          }
        }
      } catch (error) {
        set({
          lifecycle: 'error',
          authState: null,
          resetRequestId: null,
          resetStatus: 'idle',
          resetStatusMessage: null,
          resetError: null,
          error: commandError(error),
        });
      }
    },
    validate: async () => {
      update((state) => ({ ...state, lifecycle: 'checking', error: null }));
      try {
        const view = await validateSession();
        applyAuthState(view.auth_state);
      } catch (error) {
        set({
          lifecycle: 'error',
          authState: null,
          resetRequestId: null,
          resetStatus: 'idle',
          resetStatusMessage: null,
          resetError: null,
          error: commandError(error),
        });
      }
    },
    activate: async (licenseKey: string) => {
      update((state) => ({ ...state, lifecycle: 'activating', error: null }));
      try {
        const view = await activateLicense(licenseKey);
        applyAuthState(view.auth_state);
      } catch (error) {
        const err = commandError(error);
        set({
          lifecycle: err.code === 'device_already_bound' ? 'device_bound_elsewhere' : 'error',
          authState: null,
          resetRequestId: null,
          resetStatus: 'idle',
          resetStatusMessage: null,
          resetError: null,
          error: err,
        });
      }
    },
    requestReset: async (input: DeviceResetInput, options: RequestResetOptions = {}) => {
      try {
        const view = await requestDeviceReset(input);
        const resetStatus = toResetLifecycle(view.status);
        const resetMessage = view.message ?? statusMessageFor(resetStatus);
        update((state) => {
          const shouldPreserve =
            options.preserveLicensedSession &&
            (state.lifecycle === 'licensed' || state.lifecycle === 'licensed_offline_grace');
          if (shouldPreserve) {
            saveResetCache(view.request_id, resetStatus, resetMessage);
            if (resetStatus === 'pending') {
              maybeStartResetPolling(view.request_id);
            }
            return {
              ...state,
              resetRequestId: view.request_id,
              resetStatus,
              resetStatusMessage: resetMessage,
              resetError: null,
              error: null,
            };
          }
          saveResetCache(view.request_id, resetStatus, resetMessage);
          if (resetStatus === 'pending') {
            maybeStartResetPolling(view.request_id);
          }
          return {
            lifecycle: lifecycleFromAuthState(view.auth_state),
            authState: view.auth_state,
            resetRequestId: view.request_id,
            resetStatus,
            resetStatusMessage: resetMessage,
            resetError: null,
            error: null,
          };
        });
      } catch (error) {
        const err = commandError(error);
        // For non-licensed flows (e.g. device-bound-elsewhere screen), keep existing UX that
        // surfaces errors via the primary auth error field.
        update((state) => {
          const shouldPreserve =
            options.preserveLicensedSession &&
            (state.lifecycle === 'licensed' || state.lifecycle === 'licensed_offline_grace');
          if (!shouldPreserve) {
            return { ...state, lifecycle: 'error', error: err };
          }
          return state;
        });
        update((state) => ({
          ...state,
          resetStatus: 'error',
          resetStatusMessage: null,
          resetError: err,
        }));
        throw err;
      }
    },
    pollResetStatus: async (requestId: string) => {
      await api.pollResetStatus(requestId);
    },
    clearSession: async () => {
      await clearLocalSession();
      stopResetPolling();
      clearResetCache();
      set({
        lifecycle: 'unauthenticated',
        authState: { status: 'unauthenticated' },
        resetRequestId: null,
        resetStatus: 'idle',
        resetStatusMessage: null,
        resetError: null,
        error: null,
      });
    },
  };
}

export const authState = createAuthState();
