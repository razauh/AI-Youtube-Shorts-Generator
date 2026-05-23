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
    validateSession.mockResolvedValue({ auth_state: { status: 'unauthenticated' } });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.bootstrap();

    expect(get(state).lifecycle).toBe('unauthenticated');
  });

  it('shows startup-specific prompt when bootstrap lands in reauth_required', async () => {
    validateSession.mockResolvedValue({
      auth_state: { status: 'reauth_required', masked_license_key: '****-1234' },
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await state.bootstrap();

    expect(get(state).lifecycle).toBe('reauth_required');
    expect(get(state).reauthMessage).toBe(
      'For security, re-enter your license key to continue on this device.',
    );
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

    await state.requestReset({});
    expect(get(state).lifecycle).toBe('reset_pending');
    expect(get(state).resetRequestId).toBe('reset-1');
    expect(get(state).resetStatus).toBe('pending');

    await state.pollResetStatus('reset-1');
    expect(get(state).lifecycle).toBe('reset_approved_unbound');
    expect(get(state).resetStatus).toBe('approved');
  });

  it('can preserve licensed shell state after settings reset request', async () => {
    requestDeviceReset.mockResolvedValue({
      request_id: 'reset-1',
      status: 'pending',
      auth_state: {
        status: 'licensed',
        masked_license_key: '****-1234',
        device_id: 'dev',
        token_expires_at_ms: 1,
        last_validated_at_ms: 1,
        next_validation_due_ms: 2,
      },
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();
    state.reset();
    validateSession.mockResolvedValue({
      auth_state: {
        status: 'licensed',
        masked_license_key: '****-1234',
        device_id: 'dev',
        token_expires_at_ms: 1,
        last_validated_at_ms: 1,
        next_validation_due_ms: 2,
      },
    });

    await state.bootstrap();
    await state.requestReset(
      {},
      { preserveLicensedSession: true },
    );

    expect(get(state).lifecycle).toBe('licensed');
    expect(get(state).resetRequestId).toBe('reset-1');
    expect(get(state).resetStatus).toBe('pending');
  });

  it('surfaces reset request failures to settings callers without faking success', async () => {
    requestDeviceReset.mockRejectedValue({
      code: 'worker_unreachable',
      message: 'backend unreachable',
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await expect(state.requestReset({}, { preserveLicensedSession: true })).rejects.toMatchObject({
      code: 'worker_unreachable',
      message: 'Unable to reach the license service right now. Please try again shortly.',
    });
    expect(get(state).resetStatus).toBe('error');
    expect(get(state).resetError?.code).toBe('worker_unreachable');
  });

  it('surfaces reset request failures to non-preserved flows via primary auth error', async () => {
    requestDeviceReset.mockRejectedValue({
      code: 'worker_unreachable',
      message: 'backend unreachable',
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await expect(state.requestReset({})).rejects.toMatchObject({
      code: 'worker_unreachable',
    });
    expect(get(state).lifecycle).toBe('error');
    expect(get(state).error?.code).toBe('worker_unreachable');
  });

  it('parses stringified command errors from bridge failures', async () => {
    requestDeviceReset.mockRejectedValue('{"code":"worker_unreachable","message":"ignored"}');
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await expect(state.requestReset({})).rejects.toMatchObject({ code: 'worker_unreachable' });
    expect(get(state).error?.message).toBe(
      'Unable to reach the license service right now. Please try again shortly.',
    );
  });

  it('maps invalid_reset_request to a specific user-facing message', async () => {
    requestDeviceReset.mockRejectedValue({
      code: 'invalid_reset_request',
      message: 'raw backend message should not be shown',
    });
    const { createAuthState } = await import('../lib/stores/authState');
    const state = createAuthState();

    await expect(state.requestReset({}, { preserveLicensedSession: true })).rejects.toMatchObject({
      code: 'invalid_reset_request',
    });
    expect(get(state).resetError?.message).toBe(
      'Unable to request a device reset. This requires your license key on this device—activate again with your license key, then retry.',
    );
  });
});
