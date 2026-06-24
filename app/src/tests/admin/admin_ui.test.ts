import '@testing-library/jest-dom/vitest';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';

const loadAdminConfig = vi.fn();
const listResetRequests = vi.fn();
const listDeletionRequests = vi.fn();
const approveResetRequest = vi.fn();
const rejectResetRequest = vi.fn();
const approveDeletionRequest = vi.fn();
const rejectDeletionRequest = vi.fn();
const saveAdminConfig = vi.fn();
const clearAdminConfig = vi.fn();
const testAdminConnection = vi.fn();
const loadOverview = vi.fn();
const listLicenses = vi.fn();
const disableLicense = vi.fn();
const listDeviceBindings = vi.fn();
const listAuditEvents = vi.fn();
const listIdempotencyRecords = vi.fn();

vi.mock('../../admin/lib/adminClient', () => ({
  loadAdminConfig: (...args: unknown[]) => loadAdminConfig(...args),
  listResetRequests: (...args: unknown[]) => listResetRequests(...args),
  listDeletionRequests: (...args: unknown[]) => listDeletionRequests(...args),
  approveResetRequest: (...args: unknown[]) => approveResetRequest(...args),
  rejectResetRequest: (...args: unknown[]) => rejectResetRequest(...args),
  approveDeletionRequest: (...args: unknown[]) => approveDeletionRequest(...args),
  rejectDeletionRequest: (...args: unknown[]) => rejectDeletionRequest(...args),
  saveAdminConfig: (...args: unknown[]) => saveAdminConfig(...args),
  clearAdminConfig: (...args: unknown[]) => clearAdminConfig(...args),
  testAdminConnection: (...args: unknown[]) => testAdminConnection(...args),
  loadOverview: (...args: unknown[]) => loadOverview(...args),
  listLicenses: (...args: unknown[]) => listLicenses(...args),
  disableLicense: (...args: unknown[]) => disableLicense(...args),
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

const pendingDeletionRequest = {
  deletion_request_id: 'del-pending',
  status: 'pending',
  masked_license_key: '••••-1234',
  has_license_hash: true,
  license_hash_prefix: 'hash-1',
  purchaser_email: 'b***@example.com',
  requested_scope: 'backend_licensing_data',
  deletion_preview: { licenses: 1, device_bindings: 1, reset_requests: 1 },
  deletion_summary: null,
  error_code: null,
  error_message_safe: null,
  created_at_ms: 1,
  updated_at_ms: 2,
  decided_at_ms: null,
  completed_at_ms: null
};

describe('AdminApp', () => {
  let AdminApp: Awaited<typeof import('../../admin/AdminApp.svelte')>['default'];

  beforeEach(async () => {
    vi.resetModules();
    loadAdminConfig.mockReset();
    listResetRequests.mockReset();
    listDeletionRequests.mockReset();
    approveResetRequest.mockReset();
    rejectResetRequest.mockReset();
    approveDeletionRequest.mockReset();
    rejectDeletionRequest.mockReset();
    saveAdminConfig.mockReset();
    clearAdminConfig.mockReset();
    testAdminConnection.mockReset();
    loadOverview.mockReset();
    listLicenses.mockReset();
    disableLicense.mockReset();
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
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, deletion_request_counts: {}, recent_audit_events_24h: 0 });
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
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, deletion_request_counts: {}, recent_audit_events_24h: 0 });
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
    loadOverview.mockResolvedValue({ total_licenses: 8, entitlement_counts: {}, device_binding_counts: { active: 2 }, reset_request_counts: { pending: 1 }, deletion_request_counts: { pending: 1 }, recent_audit_events_24h: 3 });
    listLicenses.mockResolvedValue({ licenses: [] });

    render(AdminApp);

    expect(await screen.findByText('Total licenses')).toBeInTheDocument();
    expect(await screen.findByText('8')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'licenses' }));
    await waitFor(() => expect(listLicenses).toHaveBeenCalled());
  });

  it('renders table headers across admin tabs', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, deletion_request_counts: {}, recent_audit_events_24h: 0 });
    listResetRequests.mockResolvedValue({ requests: [pendingRequest] });
    listDeletionRequests.mockResolvedValue({ requests: [pendingDeletionRequest] });
    listLicenses.mockResolvedValue({ licenses: [{ license_hash_prefix: 'hash-1', purchaser_email_masked: 'b***@example.com', entitlement_status: 'active', provider: 'gumroad', provider_sale_id: 'sale-1', updated_at_ms: 1, active_device_count: 1, inactive_device_count: 0 }] });
    listDeviceBindings.mockResolvedValue({ bindings: [{ device_id: 'dev-1', status: 'active', license_hash_prefix: 'hash-1', updated_at_ms: 1, purchaser_email_masked: 'b***@example.com', public_key_prefix: 'abc', fingerprint_summary: { os_name: 'linux', platform_family: 'linux', arch: 'x64', app_version: '1.0.0' } }] });
    listAuditEvents.mockResolvedValue({ events: [{ event_type: 'license_disabled', actor: 'admin', created_at_ms: 1, metadata_summary: {} }] });
    listIdempotencyRecords.mockResolvedValue({ records: [{ op: 'admin_license_disable', idempotency_key_prefix: 'idk', payload_hash_prefix: 'ph', response_status: 200, response_body_size: 10, created_at_ms: 1 }] });

    render(AdminApp);
    await fireEvent.click(await screen.findByRole('button', { name: 'reset requests' }));
    expect(await screen.findByText('Request ID')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'delete requests' }));
    expect(await screen.findByText('del-pending')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'licenses' }));
    expect(await screen.findByText('Entitlement Status')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'device bindings' }));
    expect(await screen.findByText('Fingerprint Summary')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'audit events' }));
    expect(await screen.findByText('Event Type')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'idempotency' }));
    expect(await screen.findByText('Idempotency Key')).toBeInTheDocument();
  });

  it('opens disable modal, requires reason, and sends toggle value', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, deletion_request_counts: {}, recent_audit_events_24h: 0 });
    listLicenses.mockResolvedValue({ licenses: [{ license_hash_prefix: 'hash-1', purchaser_email_masked: 'b***@example.com', entitlement_status: 'active', provider: 'gumroad', provider_sale_id: 'sale-1', updated_at_ms: 1, active_device_count: 1, inactive_device_count: 0 }] });
    disableLicense.mockResolvedValue({ license_hash_prefix: 'hash-1', entitlement_status: 'disabled', deactivate_bindings: false });

    render(AdminApp);
    await fireEvent.click(await screen.findByRole('button', { name: 'licenses' }));
    await fireEvent.click(await screen.findByRole('button', { name: 'Disable License' }));

    expect(screen.getByText('Disable License?')).toBeInTheDocument();
    await fireEvent.input(screen.getByLabelText('Reason for disabling'), { target: { value: 'fraud' } });
    await fireEvent.click(screen.getByLabelText('Deactivate active device bindings now'));
    const disableButtons = screen.getAllByRole('button', { name: 'Disable License' });
    await fireEvent.click(disableButtons[disableButtons.length - 1]);

    await waitFor(() => expect(disableLicense).toHaveBeenCalledWith('hash-1', 'fraud', false));
  });

  it('requires typed confirmation before approving deletion requests', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, deletion_request_counts: {}, recent_audit_events_24h: 0 });
    listDeletionRequests.mockResolvedValue({ requests: [pendingDeletionRequest] });
    approveDeletionRequest.mockResolvedValue({ deletion_request_id: 'del-pending', status: 'completed', deletion_summary: {} });

    render(AdminApp);
    await fireEvent.click(await screen.findByRole('button', { name: 'delete requests' }));
    await fireEvent.click(await screen.findByRole('button', { name: 'Approve and Delete' }));

    let approveButtons = screen.getAllByRole('button', { name: 'Approve and Delete' });
    expect(approveButtons[approveButtons.length - 1]).toBeDisabled();
    await fireEvent.input(screen.getByLabelText('Confirmation'), { target: { value: 'DELETE USER DATA' } });
    approveButtons = screen.getAllByRole('button', { name: 'Approve and Delete' });
    await fireEvent.click(approveButtons[approveButtons.length - 1]);

    await waitFor(() => expect(approveDeletionRequest).toHaveBeenCalledWith('del-pending', 'DELETE USER DATA', ''));
  });
});
