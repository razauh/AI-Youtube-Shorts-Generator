import { beforeEach, describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: <T>(command: string, args?: Record<string, unknown>) => invoke(command, args) as Promise<T>,
}));

describe('authClient', () => {
  beforeEach(() => {
    vi.resetModules();
    invoke.mockReset();
  });

  it('calls activate_license with camelCase command argument mapped by Tauri', async () => {
    invoke.mockResolvedValue({
      auth_state: { status: 'licensed', masked_license_key: '****-1234', device_id: 'dev', token_expires_at_ms: 1 },
      masked_license_key: '****-1234',
      entitlement: 'active',
    });

    const { activateLicense } = await import('../lib/api/authClient');
    await activateLicense('LICENSE-1234');

    expect(invoke).toHaveBeenCalledWith('activate_license', { licenseKey: 'LICENSE-1234' });
  });

  it('calls reset and status commands with expected argument shapes', async () => {
    invoke.mockResolvedValue({ request_id: 'reset-1', status: 'pending', auth_state: { status: 'reset_pending', request_id: 'reset-1' } });

    const { requestDeviceReset, getDeviceResetStatus } = await import('../lib/api/authClient');
    await requestDeviceReset({});
    await getDeviceResetStatus('reset-1');

    expect(invoke).toHaveBeenNthCalledWith(1, 'request_device_reset', {
      input: {},
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'get_device_reset_status', { requestId: 'reset-1' });
  });

  it('calls session commands without persisting license material', async () => {
    invoke.mockResolvedValue({ status: 'unauthenticated' });
    localStorage.clear();

    const { getAuthState, clearLocalSession } = await import('../lib/api/authClient');
    await getAuthState();
    await clearLocalSession();

    expect(invoke).toHaveBeenNthCalledWith(1, 'get_auth_state', undefined);
    expect(invoke).toHaveBeenNthCalledWith(2, 'clear_local_session', undefined);
    expect(JSON.stringify(localStorage)).not.toContain('LICENSE-1234');
  });
});
