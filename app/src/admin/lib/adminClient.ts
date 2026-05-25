import type {
  AdminConfigView,
  AdminResetDecisionData,
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

export function testAdminConnection(): Promise<AdminResetListData> {
  return invoke<AdminResetListData>('admin_test_connection');
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
