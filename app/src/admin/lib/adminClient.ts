import type {
  AdminConfigView,
  AdminOverviewData,
  AdminAuditEventListData,
  AdminIdempotencyRecordListData,
  AdminDeletionDecisionData,
  AdminDeletionListData,
  AdminDisableLicenseData,
  DeletionRequestStatus
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

export function listDeletionRequests(status: DeletionRequestStatus): Promise<AdminDeletionListData> {
  return invoke<AdminDeletionListData>('admin_list_deletion_requests', { status });
}

export function approveDeletionRequest(
  requestId: string,
  confirmation: string,
  reason?: string
): Promise<AdminDeletionDecisionData> {
  return invoke<AdminDeletionDecisionData>('admin_approve_deletion_request', {
    requestId,
    confirmation: confirmation.trim(),
    reason: reason?.trim() ? reason.trim() : null
  });
}

export function rejectDeletionRequest(requestId: string, reason?: string): Promise<AdminDeletionDecisionData> {
  return invoke<AdminDeletionDecisionData>('admin_reject_deletion_request', {
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
