import type {
  ActivationView,
  AuthStateView,
  DeviceResetInput,
  DeviceResetView,
  SessionView,
  UserDataDeletionInput,
  UserDataDeletionStatusInput,
  UserDataDeletionView,
} from '../authContracts';

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

export function activateLicense(licenseKey: string): Promise<ActivationView> {
  return invoke<ActivationView>('activate_license', { licenseKey });
}

export function validateSession(): Promise<SessionView> {
  return invoke<SessionView>('validate_session');
}

export function deactivateCurrentDevice(): Promise<SessionView> {
  return invoke<SessionView>('deactivate_current_device');
}

export function requestDeviceReset(input: DeviceResetInput): Promise<DeviceResetView> {
  return invoke<DeviceResetView>('request_device_reset', { input });
}

export function getDeviceResetStatus(requestId: string): Promise<DeviceResetView> {
  return invoke<DeviceResetView>('get_device_reset_status', { requestId });
}

export function requestUserDataDeletion(input: UserDataDeletionInput): Promise<UserDataDeletionView> {
  return invoke<UserDataDeletionView>('request_user_data_deletion', { input });
}

export function getUserDataDeletionStatus(input: UserDataDeletionStatusInput): Promise<UserDataDeletionView> {
  return invoke<UserDataDeletionView>('get_user_data_deletion_status', { input });
}

export function clearLocalSession(): Promise<void> {
  return invoke<void>('clear_local_session');
}

export function getAuthState(): Promise<AuthStateView> {
  return invoke<AuthStateView>('get_auth_state');
}
