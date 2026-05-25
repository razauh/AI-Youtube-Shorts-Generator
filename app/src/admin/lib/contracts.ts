export type ResetRequestStatus = 'pending' | 'approved' | 'rejected' | 'expired';
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
