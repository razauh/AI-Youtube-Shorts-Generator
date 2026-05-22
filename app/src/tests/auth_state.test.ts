import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const getAuthState = vi.fn();
const validateSession = vi.fn();
const activateLicense = vi.fn();
const requestDeviceReset = vi.fn();
const getDeviceResetStatus = vi.fn();
const clearLocalSession = vi.fn();

vi.mock('../lib/api/authClient', () => ({
  getAuthState: () => getAuthState(),
  validateSession: () => validateSession(),
  activateLicense: (licenseKey: string) => activateLicense(licenseKey),
  requestDeviceReset: (input: unknown) => requestDeviceReset(input),
  getDeviceResetStatus: (requestId: string) => getDeviceResetStatus(requestId),
  clearLocalSession: () => clearLocalSession(),
}));

describe('authState store', () => {
  beforeEach(() => {
    vi.resetModules();
    getAuthState.mockReset();
    validateSession.mockReset();
    activateLicense.mockReset();
    requestDeviceReset.mockReset();
    getDeviceResetStatus.mockReset();
    clearLocalSession.mockReset();
    localStorage.clear();
  });

  it('bootstraps unauthenticated state', async () => {
    getAuthState.mockResolvedValue({ status: 'unauthenticated' });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.bootstrap();

    expect(get(state).lifecycle).toBe('unauthenticated');
  });

  it('maps activation success to licensed without retaining plaintext license key', async () => {
    activateLicense.mockResolvedValue({
      auth_state: {
        status: 'licensed',
        masked_license_key: '****-1234',
        device_id: 'dev',
        token_expires_at_ms: 1,
        last_validated_at_ms: 1,
        next_validation_due_ms: 2,
      },
      masked_license_key: '****-1234',
      entitlement: 'active',
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.activate('LICENSE-1234');

    expect(activateLicense).toHaveBeenCalledWith('LICENSE-1234');
    expect(get(state).lifecycle).toBe('licensed');
    expect(JSON.stringify(get(state))).not.toContain('LICENSE-1234');
    expect(JSON.stringify(localStorage)).not.toContain('LICENSE-1234');
  });

  it('maps device bound errors to reset path', async () => {
    activateLicense.mockRejectedValue({
      code: 'device_already_bound',
      message: 'license is already bound to another device',
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.activate('LICENSE-1234');

    expect(get(state).lifecycle).toBe('device_bound_elsewhere');
    expect(get(state).error?.code).toBe('device_already_bound');
    expect(get(state).error?.message).toBe('This license is already in use on another device.');
    expect(JSON.stringify(get(state))).not.toContain('LICENSE-1234');
  });

  it('does not surface raw backend auth error text to login UI state', async () => {
    activateLicense.mockRejectedValue({
      code: 'invalid_license_key',
      message: 'backend validation failed: malformed payload id=abc123',
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.activate('LICENSE-1234');

    expect(get(state).lifecycle).toBe('error');
    expect(get(state).error?.code).toBe('invalid_license_key');
    expect(get(state).error?.message).toBe('Invalid license key. Please check and try again.');
    expect(get(state).error?.message).not.toContain('malformed payload');
  });

  it('tracks reset request and terminal polling state', async () => {
    requestDeviceReset.mockResolvedValue({
      request_id: 'reset-1',
      status: 'pending',
      auth_state: { status: 'reset_pending', request_id: 'reset-1', masked_license_key: '****-1234' },
    });
    getDeviceResetStatus.mockResolvedValue({
      request_id: 'reset-1',
      status: 'approved',
      auth_state: {
        status: 'reset_approved_unbound',
        request_id: 'reset-1',
        masked_license_key: '****-1234',
        message: 'Device reset approved. You can now use this license key to activate a device.',
      },
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.requestReset({ purchaser_email: 'buyer@example.com', receipt_reference: null });
    expect(get(state).lifecycle).toBe('reset_pending');
    expect(get(state).resetRequestId).toBe('reset-1');

    await state.pollResetStatus('reset-1');
    expect(get(state).lifecycle).toBe('reset_approved_unbound');
  });
});
