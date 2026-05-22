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

export interface AuthStateShape {
  lifecycle: AuthLifecycle;
  authState: AuthStateView | null;
  resetRequestId: string | null;
  error: AuthCommandError | null;
}

interface RequestResetOptions {
  preserveLicensedSession?: boolean;
}

const initialState: AuthStateShape = {
  lifecycle: 'checking',
  authState: null,
  resetRequestId: null,
  error: null,
};

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
    case 'reset_request_not_found':
      return 'Reset request not found. Check the request ID and try again.';
    default:
      return 'Authentication failed. Please try again.';
  }
}

function commandError(error: unknown): AuthCommandError {
  if (
    error &&
    typeof error === 'object' &&
    'code' in error &&
    'message' in error &&
    typeof (error as { code: unknown }).code === 'string' &&
    typeof (error as { message: unknown }).message === 'string'
  ) {
    const err = error as AuthCommandError;
    return { code: err.code, message: safeAuthMessage(err.code) };
  }
  return {
    code: 'unknown',
    message: safeAuthMessage('unknown'),
  };
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

  function applyAuthState(authState: AuthStateView) {
    set({
      lifecycle: lifecycleFromAuthState(authState),
      authState,
      resetRequestId: resetIdFrom(authState),
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
      } catch (error) {
        set({
          lifecycle: 'error',
          authState: null,
          resetRequestId: null,
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
          error: err,
        });
      }
    },
    requestReset: async (input: DeviceResetInput, options: RequestResetOptions = {}) => {
      try {
        const view = await requestDeviceReset(input);
        update((state) => {
          if (
            options.preserveLicensedSession &&
            (state.lifecycle === 'licensed' || state.lifecycle === 'licensed_offline_grace')
          ) {
            return {
              ...state,
              resetRequestId: view.request_id,
              error: null,
            };
          }
          return {
            lifecycle: 'reset_pending',
            authState: view.auth_state,
            resetRequestId: view.request_id,
            error: null,
          };
        });
      } catch (error) {
        update((state) => ({ ...state, lifecycle: 'error', error: commandError(error) }));
      }
    },
    pollResetStatus: async (requestId: string) => {
      const view = await getDeviceResetStatus(requestId);
      set({
        lifecycle: lifecycleFromAuthState(view.auth_state),
        authState: view.auth_state,
        resetRequestId: view.request_id,
        error: null,
      });
    },
    clearSession: async () => {
      await clearLocalSession();
      set({
        lifecycle: 'unauthenticated',
        authState: { status: 'unauthenticated' },
        resetRequestId: null,
        error: null,
      });
    },
  };
}

export const authState = createAuthState();
