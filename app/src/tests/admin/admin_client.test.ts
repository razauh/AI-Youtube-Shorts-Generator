import { beforeEach, describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: <T>(command: string, args?: Record<string, unknown>) => invoke(command, args) as Promise<T>,
}));

describe('adminClient', () => {
  beforeEach(() => {
    vi.resetModules();
    invoke.mockReset();
  });

  it('saves admin config without persisting token in browser storage', async () => {
    invoke.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    localStorage.clear();

    const { saveAdminConfig } = await import('../../admin/lib/adminClient');
    await saveAdminConfig('https://worker.example.test/', 'admin-test-token-1234');

    expect(invoke).toHaveBeenCalledWith('admin_config_save', {
      baseUrl: 'https://worker.example.test/',
      token: 'admin-test-token-1234'
    });
    expect(JSON.stringify(localStorage)).not.toContain('admin-test-token-1234');
  });

  it('sends status filter through the typed Tauri command', async () => {
    invoke.mockResolvedValue({ requests: [] });

    const { listResetRequests } = await import('../../admin/lib/adminClient');
    await listResetRequests('approved');

    expect(invoke).toHaveBeenCalledWith('admin_list_reset_requests', { status: 'approved' });
  });

  it('uses approve and reject commands with optional reason input', async () => {
    invoke.mockResolvedValue({ reset_request_id: 'reset-1', status: 'approved', license_state: 'UNBOUND' });

    const { approveResetRequest, rejectResetRequest } = await import('../../admin/lib/adminClient');
    await approveResetRequest('reset-1', ' verified ');
    await rejectResetRequest('reset-2', '');

    expect(invoke).toHaveBeenNthCalledWith(1, 'admin_approve_reset_request', {
      requestId: 'reset-1',
      reason: 'verified'
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'admin_reject_reset_request', {
      requestId: 'reset-2',
      reason: null
    });
  });
});
