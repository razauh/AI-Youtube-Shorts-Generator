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

vi.mock('../../admin/lib/adminClient', () => ({
  loadAdminConfig: (...args: unknown[]) => loadAdminConfig(...args),
  listResetRequests: (...args: unknown[]) => listResetRequests(...args),
  approveResetRequest: (...args: unknown[]) => approveResetRequest(...args),
  rejectResetRequest: (...args: unknown[]) => rejectResetRequest(...args),
  saveAdminConfig: (...args: unknown[]) => saveAdminConfig(...args),
  clearAdminConfig: (...args: unknown[]) => clearAdminConfig(...args),
  testAdminConnection: (...args: unknown[]) => testAdminConnection(...args)
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
    listResetRequests.mockResolvedValue({ requests: [pendingRequest, approvedRequest] });

    render(AdminApp);

    expect(await screen.findByText('reset-pending')).toBeInTheDocument();
    expect(screen.getByText('reset-approved')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Approve' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Reject' })).toBeInTheDocument();
    expect(screen.getByText('Decision already final')).toBeInTheDocument();
  });

  it('confirms approve actions and sends the optional reason', async () => {
    loadAdminConfig.mockResolvedValue({ baseUrl: 'https://worker.example.test', tokenConfigured: true, tokenRedacted: '[redacted]...1234' });
    listResetRequests.mockResolvedValue({ requests: [pendingRequest] });
    approveResetRequest.mockResolvedValue({ reset_request_id: 'reset-pending', status: 'approved', license_state: 'UNBOUND' });

    render(AdminApp);

    await fireEvent.click(await screen.findByRole('button', { name: 'Approve' }));
    await fireEvent.input(screen.getByLabelText('Optional reason'), { target: { value: 'verified by support' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Confirm approve' }));

    await waitFor(() => expect(approveResetRequest).toHaveBeenCalledWith('reset-pending', 'verified by support'));
  });
});
