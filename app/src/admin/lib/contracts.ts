export type ResetRequestStatus = 'pending' | 'approved' | 'rejected' | 'expired';
export type DeletionRequestStatus = 'pending' | 'approved' | 'processing' | 'rejected' | 'completed' | 'failed';
export type LicenseState = 'BOUND_ACTIVE' | 'UNBOUND';

export interface AdminConfigView {
  baseUrl: string | null;
  tokenConfigured: boolean;
  tokenRedacted: string | null;
}

export interface AdminResetRequestItem {
  reset_request_id: string;
  status: ResetRequestStatus;
  license_state: LicenseState;
  message: string;
  masked_license_key: string | null;
  has_license_hash: boolean;
  purchaser_email: string | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminResetListData {
  requests: AdminResetRequestItem[];
}

export interface AdminResetDecisionData {
  reset_request_id: string;
  status: ResetRequestStatus;
  license_state: LicenseState;
}

export interface AdminDeletionRequestItem {
  deletion_request_id: string;
  status: DeletionRequestStatus;
  masked_license_key: string | null;
  has_license_hash: boolean;
  license_hash_prefix: string | null;
  purchaser_email: string | null;
  requested_scope: string;
  deletion_preview: Record<string, unknown> | null;
  deletion_summary: Record<string, unknown> | null;
  error_code: string | null;
  error_message_safe: string | null;
  created_at_ms: number;
  updated_at_ms: number;
  decided_at_ms: number | null;
  completed_at_ms: number | null;
}

export interface AdminDeletionListData {
  requests: AdminDeletionRequestItem[];
}

export interface AdminDeletionDecisionData {
  deletion_request_id: string;
  status: DeletionRequestStatus;
  deletion_summary: Record<string, unknown> | null;
}

export interface AdminOverviewData {
  total_licenses: number;
  entitlement_counts: Record<string, number>;
  device_binding_counts: Record<string, number>;
  reset_request_counts: Record<string, number>;
  deletion_request_counts: Record<string, number>;
  recent_audit_events_24h: number;
}

export interface AdminLicenseItem {
  license_hash_prefix: string;
  purchaser_email_masked: string;
  entitlement_status: string;
  provider: string | null;
  provider_sale_id: string | null;
  updated_at_ms: number;
  active_device_count: number;
  inactive_device_count: number;
}

export interface AdminDisableLicenseData {
  license_hash_prefix: string;
  entitlement_status: string;
  deactivate_bindings: boolean;
}

export interface AdminLicenseListData {
  licenses: AdminLicenseItem[];
}

export interface FingerprintSummary {
  os_name: string | null;
  platform_family: string | null;
  arch: string | null;
  app_version: string | null;
}

export interface AdminDeviceBindingItem {
  device_id: string;
  status: string;
  license_hash_prefix: string;
  updated_at_ms: number;
  purchaser_email_masked: string;
  public_key_prefix: string;
  fingerprint_summary: FingerprintSummary;
}

export interface AdminDeviceBindingListData {
  bindings: AdminDeviceBindingItem[];
}

export interface AdminAuditEventItem {
  event_type: string;
  actor: string | null;
  created_at_ms: number;
  metadata_summary: Record<string, unknown>;
}

export interface AdminAuditEventListData {
  events: AdminAuditEventItem[];
}

export interface AdminIdempotencyRecordItem {
  op: string;
  idempotency_key_prefix: string;
  payload_hash_prefix: string;
  response_status: number;
  response_body_size: number;
  created_at_ms: number;
}

export interface AdminIdempotencyRecordListData {
  records: AdminIdempotencyRecordItem[];
}

export interface AdminCommandError {
  code: string;
  message: string;
  request_id?: string | null;
  retryable: boolean;
}

export interface AdminNotice {
  kind: 'success' | 'error' | 'info';
  message: string;
  requestId?: string | null;
}
