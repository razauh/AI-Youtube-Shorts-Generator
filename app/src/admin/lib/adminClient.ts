import type {
  AdminConfigView,
  AdminOverviewData,
  AdminLicenseListData,
  AdminDeviceBindingListData,
  AdminAuditEventListData,
  AdminIdempotencyRecordListData,
  AdminResetDecisionData,
  AdminDisableLicenseData,
  AdminResetListData,
  ResetRequestStatus
} from './contracts';

interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

let corePromise: Promise<TauriCore> | null = null;

async function getCore(): Promise<TauriCore> {
  if (!corePromise) {
    corePromise = import('@tauri-apps/api/core') as Promise<TauriCore>;
  }
  return corePromise;
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const core = await getCore();
  return core.invoke<T>(command, args);
}

export function loadAdminConfig(): Promise<AdminConfigView> {
  return invoke<AdminConfigView>('admin_config_load');
}

export function saveAdminConfig(baseUrl: string, token: string): Promise<AdminConfigView> {
  return invoke<AdminConfigView>('admin_config_save', { baseUrl, token });
}

export function clearAdminConfig(): Promise<AdminConfigView> {
  return invoke<AdminConfigView>('admin_config_clear');
}

export function testAdminConnection(): Promise<AdminOverviewData> {
  return invoke<AdminOverviewData>('admin_test_connection');
}

export function loadOverview(): Promise<AdminOverviewData> {
  return invoke<AdminOverviewData>('admin_overview');
}

export function listLicenses(filters: {
  q?: string;
  entitlementStatus?: string;
  provider?: string;
  limit?: number;
}): Promise<AdminLicenseListData> {
  return invoke<AdminLicenseListData>('admin_list_licenses', {
    q: filters.q?.trim() || null,
    entitlementStatus: filters.entitlementStatus?.trim() || null,
    provider: filters.provider?.trim() || null,
    limit: filters.limit ?? null
  });
}

export function listDeviceBindings(filters: {
  q?: string;
  status?: string;
  licenseHashPrefix?: string;
  limit?: number;
}): Promise<AdminDeviceBindingListData> {
  return invoke<AdminDeviceBindingListData>('admin_list_device_bindings', {
    q: filters.q?.trim() || null,
    status: filters.status?.trim() || null,
    licenseHashPrefix: filters.licenseHashPrefix?.trim() || null,
    limit: filters.limit ?? null
  });
}

export function listAuditEvents(filters: {
  eventType?: string;
  actor?: string;
  limit?: number;
}): Promise<AdminAuditEventListData> {
  return invoke<AdminAuditEventListData>('admin_list_audit_events', {
    eventType: filters.eventType?.trim() || null,
    actor: filters.actor?.trim() || null,
    limit: filters.limit ?? null
  });
}

export function listIdempotencyRecords(filters: {
  op?: string;
  limit?: number;
}): Promise<AdminIdempotencyRecordListData> {
  return invoke<AdminIdempotencyRecordListData>('admin_list_idempotency_records', {
    op: filters.op?.trim() || null,
    limit: filters.limit ?? null
  });
}

export function listResetRequests(status: ResetRequestStatus): Promise<AdminResetListData> {
  return invoke<AdminResetListData>('admin_list_reset_requests', { status });
}

export function approveResetRequest(requestId: string, reason?: string): Promise<AdminResetDecisionData> {
  return invoke<AdminResetDecisionData>('admin_approve_reset_request', {
    requestId,
    reason: reason?.trim() ? reason.trim() : null
  });
}

export function rejectResetRequest(requestId: string, reason?: string): Promise<AdminResetDecisionData> {
  return invoke<AdminResetDecisionData>('admin_reject_reset_request', {
    requestId,
    reason: reason?.trim() ? reason.trim() : null
  });
}

export function disableLicense(
  licenseHashPrefix: string,
  reason: string,
  deactivateBindings: boolean
): Promise<AdminDisableLicenseData> {
  return invoke<AdminDisableLicenseData>('admin_disable_license', {
    licenseHashPrefix: licenseHashPrefix.trim(),
    reason: reason.trim(),
    deactivateBindings
  });
}
