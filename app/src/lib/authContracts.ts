export type AuthStateView =
  | { status: 'unauthenticated' }
  | {
      status: 'licensed';
      masked_license_key: string;
      device_id: string;
      token_expires_at_ms: number;
      last_validated_at_ms: number;
      next_validation_due_ms: number;
    }
  | {
      status: 'licensed_offline_grace';
      masked_license_key: string;
      device_id: string;
      token_expires_at_ms: number;
      last_validated_at_ms: number;
      next_validation_due_ms: number;
      grace_expires_at_ms: number;
    }
  | { status: 'reauth_required'; masked_license_key?: string | null }
  | {
      status: 'reset_pending';
      request_id: string;
      masked_license_key?: string | null;
    }
  | {
      status: 'reset_approved_unbound';
      request_id: string;
      masked_license_key?: string | null;
      message: string;
    }
  | {
      status: 'reset_rejected';
      request_id: string;
      masked_license_key?: string | null;
    }
  | {
      status: 'reset_expired';
      request_id: string;
      masked_license_key?: string | null;
    };

export interface ActivationView {
  auth_state: AuthStateView;
  masked_license_key: string;
  entitlement: string;
}

export interface SessionView {
  auth_state: AuthStateView;
}

export interface DeviceResetInput {
  purchaser_email: string;
  receipt_reference?: string | null;
}

export interface DeviceResetView {
  request_id: string;
  status: string;
  message?: string | null;
  auth_state: AuthStateView;
}

export interface AuthCommandError {
  code: string;
  message: string;
}
