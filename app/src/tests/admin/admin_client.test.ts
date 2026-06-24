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

    const { listResetRequests, listDeletionRequests } = await import('../../admin/lib/adminClient');
    await listResetRequests('approved');
    await listDeletionRequests('failed');

    expect(invoke).toHaveBeenNthCalledWith(1, 'admin_list_reset_requests', { status: 'approved' });
    expect(invoke).toHaveBeenNthCalledWith(2, 'admin_list_deletion_requests', { status: 'failed' });
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

  it('uses deletion decision commands with required confirmation', async () => {
    invoke.mockResolvedValue({ deletion_request_id: 'del-1', status: 'completed', deletion_summary: null });

    const { approveDeletionRequest, rejectDeletionRequest } = await import('../../admin/lib/adminClient');
    await approveDeletionRequest('del-1', ' DELETE USER DATA ', ' verified ');
    await rejectDeletionRequest('del-2', '');

    expect(invoke).toHaveBeenNthCalledWith(1, 'admin_approve_deletion_request', {
      requestId: 'del-1',
      confirmation: 'DELETE USER DATA',
      reason: 'verified'
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'admin_reject_deletion_request', {
      requestId: 'del-2',
      reason: null
    });
  });

  it('calls new admin listing commands with typed filters', async () => {
    invoke.mockResolvedValue({});
    const {
      loadOverview,
      listLicenses,
      listDeviceBindings,
      listAuditEvents,
      listIdempotencyRecords,
      disableLicense
    } = await import('../../admin/lib/adminClient');

    await loadOverview();
    await listLicenses({ q: ' buyer@example.com ', entitlementStatus: 'active', provider: 'gumroad', limit: 10 });
    await listDeviceBindings({ q: 'dev_', status: 'active', licenseHashPrefix: 'abc123', limit: 15 });
    await listAuditEvents({ eventType: 'gumroad_sale_verified', actor: 'gumroad', limit: 20 });
    await listIdempotencyRecords({ op: 'reset_request', limit: 5 });
    await disableLicense(' hash123 ', ' abuse ', true);

    expect(invoke).toHaveBeenNthCalledWith(1, 'admin_overview', undefined);
    expect(invoke).toHaveBeenNthCalledWith(2, 'admin_list_licenses', {
      q: 'buyer@example.com',
      entitlementStatus: 'active',
      provider: 'gumroad',
      limit: 10
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'admin_list_device_bindings', {
      q: 'dev_',
      status: 'active',
      licenseHashPrefix: 'abc123',
      limit: 15
    });
    expect(invoke).toHaveBeenNthCalledWith(4, 'admin_list_audit_events', {
      eventType: 'gumroad_sale_verified',
      actor: 'gumroad',
      limit: 20
    });
    expect(invoke).toHaveBeenNthCalledWith(5, 'admin_list_idempotency_records', {
      op: 'reset_request',
      limit: 5
    });
    expect(invoke).toHaveBeenNthCalledWith(6, 'admin_disable_license', {
      licenseHashPrefix: 'hash123',
      reason: 'abuse',
      deactivateBindings: true
    });
  });
});
