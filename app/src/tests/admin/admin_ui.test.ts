import '@testing-library/jest-dom/vitest';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';

const loadAdminConfig = vi.fn();
const listResetRequests = vi.fn();
const approveResetRequest = vi.fn();
const rejectResetRequest = vi.fn();
const saveAdminConfig = vi.fn();
const clearAdminConfig = vi.fn();
const testAdminConnection = vi.fn();
const loadOverview = vi.fn();
const listLicenses = vi.fn();
const listDeviceBindings = vi.fn();
const listAuditEvents = vi.fn();
const listIdempotencyRecords = vi.fn();

vi.mock('../../admin/lib/adminClient', () => ({
  loadAdminConfig: (...args: unknown[]) => loadAdminConfig(...args),
  listResetRequests: (...args: unknown[]) => listResetRequests(...args),
  approveResetRequest: (...args: unknown[]) => approveResetRequest(...args),
  rejectResetRequest: (...args: unknown[]) => rejectResetRequest(...args),
  saveAdminConfig: (...args: unknown[]) => saveAdminConfig(...args),
  clearAdminConfig: (...args: unknown[]) => clearAdminConfig(...args),
  testAdminConnection: (...args: unknown[]) => testAdminConnection(...args),
  loadOverview: (...args: unknown[]) => loadOverview(...args),
  listLicenses: (...args: unknown[]) => listLicenses(...args),
  listDeviceBindings: (...args: unknown[]) => listDeviceBindings(...args),
  listAuditEvents: (...args: unknown[]) => listAuditEvents(...args),
  listIdempotencyRecords: (...args: unknown[]) => listIdempotencyRecords(...args)
}));

const pendingRequest = {
  reset_request_id: 'reset-pending',
  status: 'pending',
  license_state: 'BOUND_ACTIVE',
  message: 'pending',
  masked_license_key: '••••-1234',
  has_license_hash: true,
  purchaser_email: 'b***@example.com',
  created_at_ms: 1,
  updated_at_ms: 2
};

const approvedRequest = {
  ...pendingRequest,
  reset_request_id: 'reset-approved',
  status: 'approved',
  license_state: 'UNBOUND'
};

describe('AdminApp', () => {
  let AdminApp: Awaited<typeof import('../../admin/AdminApp.svelte')>['default'];

  beforeEach(async () => {
    vi.resetModules();
    loadAdminConfig.mockReset();
    listResetRequests.mockReset();
    approveResetRequest.mockReset();
    rejectResetRequest.mockReset();
    saveAdminConfig.mockReset();
    clearAdminConfig.mockReset();
    testAdminConnection.mockReset();
    loadOverview.mockReset();
    listLicenses.mockReset();
    listDeviceBindings.mockReset();
    listAuditEvents.mockReset();
    listIdempotencyRecords.mockReset();
    AdminApp = (await import('../../admin/AdminApp.svelte')).default;
  });

  afterEach(() => cleanup());

  it('shows setup screen until base URL and token are configured', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: null, tokenConfigured: false, tokenRedacted: null });

    render(AdminApp);

    expect(await screen.findByText('Initial Setup')).toBeInTheDocument();
    expect(screen.getByLabelText('Worker API base URL')).toBeInTheDocument();
    expect(screen.getByLabelText('Admin API token')).toBeInTheDocument();
  });

  it('shows approve and reject only for pending requests', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, recent_audit_events_24h: 0 });
    listResetRequests.mockResolvedValue({ requests: [pendingRequest, approvedRequest] });

    render(AdminApp);

    await fireEvent.click(await screen.findByRole('button', { name: 'reset requests' }));
    expect(await screen.findByText('reset-pending')).toBeInTheDocument();
    expect(screen.getByText('reset-approved')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Approve' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Reject' })).toBeInTheDocument();
    expect(screen.getByText('Decision already final')).toBeInTheDocument();
  });

  it('confirms approve actions and sends the optional reason', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, recent_audit_events_24h: 0 });
    listResetRequests.mockResolvedValue({ requests: [pendingRequest] });
    approveResetRequest.mockResolvedValue({ reset_request_id: 'reset-pending', status: 'approved', license_state: 'UNBOUND' });

    render(AdminApp);

    await fireEvent.click(await screen.findByRole('button', { name: 'reset requests' }));
    await fireEvent.click(await screen.findByRole('button', { name: 'Approve' }));
    await fireEvent.input(screen.getByLabelText('Optional reason'), { target: { value: 'verified by support' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Confirm approve' }));

    await waitFor(() => expect(approveResetRequest).toHaveBeenCalledWith('reset-pending', 'verified by support'));
  });

  it('loads overview by default and allows section switching', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 8, entitlement_counts: {}, device_binding_counts: { active: 2 }, reset_request_counts: { pending: 1 }, recent_audit_events_24h: 3 });
    listLicenses.mockResolvedValue({ licenses: [] });

    render(AdminApp);

    expect(await screen.findByText('Total licenses')).toBeInTheDocument();
    expect(screen.getByText('8')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'licenses' }));
    await waitFor(() => expect(listLicenses).toHaveBeenCalled());
  });
});
