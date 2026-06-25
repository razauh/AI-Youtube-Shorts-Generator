import '@testing-library/jest-dom/vitest';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';

const loadAdminConfig = vi.fn();
const listDeletionRequests = vi.fn();
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
  listDeletionRequests: (...args: unknown[]) => listDeletionRequests(...args),
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
    listDeletionRequests.mockReset();
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

  it('hides the deprecated reset request queue and overview metric', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: { pending: 2 }, deletion_request_counts: {}, recent_audit_events_24h: 0 });

    render(AdminApp);

    expect(await screen.findByText('Total licenses')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'reset requests' })).toBeNull();
    expect(screen.queryByText('Pending resets')).toBeNull();
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
    listDeletionRequests.mockResolvedValue({ requests: [pendingDeletionRequest] });
    listLicenses.mockResolvedValue({ licenses: [{ license_hash_prefix: 'hash-1', purchaser_email_masked: 'b***@example.com', entitlement_status: 'active', provider: 'gumroad', provider_sale_id: 'sale-1', updated_at_ms: 1, active_device_count: 1, inactive_device_count: 0 }] });
    listDeviceBindings.mockResolvedValue({ bindings: [{ device_id: 'dev-1', status: 'active', license_hash_prefix: 'hash-1', updated_at_ms: 1, purchaser_email_masked: 'b***@example.com', public_key_prefix: 'abc', fingerprint_summary: { os_name: 'linux', platform_family: 'linux', arch: 'x64', app_version: '1.0.0' } }] });
    listAuditEvents.mockResolvedValue({ events: [{ event_type: 'license_disabled', actor: 'admin', created_at_ms: 1, metadata_summary: {} }] });
    listIdempotencyRecords.mockResolvedValue({ records: [{ op: 'admin_license_disable', idempotency_key_prefix: 'idk', payload_hash_prefix: 'ph', response_status: 200, response_body_size: 10, created_at_ms: 1 }] });

    render(AdminApp);

    await fireEvent.click(await screen.findByRole('button', { name: 'delete requests' }));
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

  it('does not show disable button/modal, shows manage on devolens link', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    loadOverview.mockResolvedValue({ total_licenses: 1, entitlement_counts: {}, device_binding_counts: {}, reset_request_counts: {}, deletion_request_counts: {}, recent_audit_events_24h: 0 });
    listLicenses.mockResolvedValue({ licenses: [{ license_hash_prefix: 'hash-1', purchaser_email_masked: 'b***@example.com', entitlement_status: 'active', provider: 'gumroad', provider_sale_id: 'sale-1', updated_at_ms: 1, active_device_count: 1, inactive_device_count: 0 }] });

    render(AdminApp);
    await fireEvent.click(await screen.findByRole('button', { name: 'licenses' }));
    
    // Direct "Disable License" button must not exist
    expect(screen.queryByRole('button', { name: 'Disable License' })).toBeNull();

    // Instead, there must be a "Manage on Devolens" link
    const link = await screen.findByRole('link', { name: 'Manage on Devolens' });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', 'https://console.cryptolens.io');
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
