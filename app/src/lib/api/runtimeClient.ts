interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

interface TauriEventApi {
  listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void>;
}

export interface DesktopRuntimeContext {
  appVersion: string;
  platform: string;
  runtimeRoot: string;
  logDir: string;
  logPath: string;
  crashLogPath: string;
  configPath: string;
  protectedBasePath: string;
  secureFallbackBasePath: string;
}

export interface RuntimeToolStatus {
  tool: string;
  ok: boolean;
  path?: string | null;
  source?: string | null;
  message: string;
}

export interface RuntimeValidation {
  runtime: string;
  bridge_entry: string;
  bridge_entry_exists: boolean;
  ok: boolean;
  tools: RuntimeToolStatus[];
  python_packages: RuntimeToolStatus[];
  local_runtime_ready: boolean;
}

export interface AppConfigSummary {
  licenseBackendMode: string;
  licenseWorkerEndpoint: string;
  licenseWorkerEndpointKind: string;
  muapiConfigured: boolean;
  openaiConfigured: boolean;
  localWhisperModel: string;
  localWhisperDevice: string;
  licenseWorkerTimeoutMs: number;
  licenseWorkerRetryAttempts: number;
}

export type ApiKeyProvider = 'muapi' | 'openai';

export interface ApiKeyProfile {
  id: string;
  label: string;
  lastFour: string;
  active: boolean;
  createdAtMs: number;
  updatedAtMs: number;
}

export interface ApiKeyProfilesView {
  provider: ApiKeyProvider;
  envOverride: boolean;
  profiles: ApiKeyProfile[];
}

export interface LocalModelProfile {
  id: string;
  label: string;
  model: string;
  device: string;
  active: boolean;
  downloadStatus: string;
  error?: string | null;
  errorCode?: string | null;
  debugRef?: string | null;
  createdAtMs: number;
  updatedAtMs: number;
}

export interface LocalModelProfilesView {
  envOverride: boolean;
  activeProfileId?: string | null;
  profiles: LocalModelProfile[];
}

export interface LocalModelDownloadStatus {
  active: boolean;
  profileId?: string | null;
  model?: string | null;
  device?: string | null;
  phase: string;
  progress: number;
  message: string;
  error?: string | null;
  errorCode?: string | null;
  debugRef?: string | null;
}

let corePromise: Promise<TauriCore> | null = null;
let eventPromise: Promise<TauriEventApi> | null = null;

async function getCore(): Promise<TauriCore> {
  if (!corePromise) {
    corePromise = import('@tauri-apps/api/core') as Promise<TauriCore>;
  }
  return corePromise;
}

async function getEventApi(): Promise<TauriEventApi> {
  if (!eventPromise) {
    eventPromise = import('@tauri-apps/api/event') as Promise<TauriEventApi>;
  }
  return eventPromise;
}

export async function isDesktopRuntime(): Promise<boolean> {
  try {
    await getCore();
    return true;
  } catch {
    return false;
  }
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const core = await getCore();
  return core.invoke<T>(command, args);
}

export function runtimeContext(): Promise<DesktopRuntimeContext> {
  return invoke<DesktopRuntimeContext>('runtime_context');
}

export function validateRuntime(): Promise<RuntimeValidation> {
  return invoke<RuntimeValidation>('validate_runtime');
}

export function appConfigSummary(): Promise<AppConfigSummary> {
  return invoke<AppConfigSummary>('app_config_summary');
}

export function runtimeMachineSecret(): Promise<string> {
  return invoke<string>('runtime_machine_secret');
}

export function runtimeReadText(path: string): Promise<string | null> {
  return invoke<string | null>('runtime_fs_read_text', { path });
}

export function runtimeWriteText(path: string, value: string): Promise<void> {
  return invoke<void>('runtime_fs_write_text', { path, value });
}

export function runtimeAppendLine(path: string, line: string): Promise<void> {
  return invoke<void>('runtime_fs_append_line', { path, line });
}

export function runtimeRemove(path: string): Promise<void> {
  return invoke<void>('runtime_fs_remove', { path });
}

export function runtimeExists(path: string): Promise<boolean> {
  return invoke<boolean>('runtime_fs_exists', { path });
}

export function runtimeList(path: string): Promise<string[]> {
  return invoke<string[]>('runtime_fs_list', { prefix: path });
}

export function runtimeRename(from: string, to: string): Promise<void> {
  return invoke<void>('runtime_fs_rename', { from, to });
}

export function runtimeChmodReadonly(path: string): Promise<void> {
  return invoke<void>('runtime_fs_chmod_readonly', { path });
}

export function runtimeFileSize(path: string): Promise<number> {
  return invoke<number>('runtime_fs_size', { path });
}

export function secureStoreSave(key: string, value: string): Promise<void> {
  return invoke<void>('secure_store_save', { key, value });
}

export function secureStoreLoad(key: string): Promise<string | null> {
  return invoke<string | null>('secure_store_load', { key });
}

export function secureStoreDelete(key: string): Promise<void> {
  return invoke<void>('secure_store_delete', { key });
}

export function secureStoreExists(key: string): Promise<boolean> {
  return invoke<boolean>('secure_store_exists', { key });
}

export function apiKeyProfiles(provider: ApiKeyProvider): Promise<ApiKeyProfilesView> {
  return invoke<ApiKeyProfilesView>('api_key_profiles', { provider });
}

export function apiKeyProfileAdd(
  provider: ApiKeyProvider,
  label: string,
  key: string,
  activate = true
): Promise<ApiKeyProfilesView> {
  return invoke<ApiKeyProfilesView>('api_key_profile_add', { provider, label, key, activate });
}

export function apiKeyProfileActivate(provider: ApiKeyProvider, profileId: string): Promise<ApiKeyProfilesView> {
  return invoke<ApiKeyProfilesView>('api_key_profile_activate', { provider, profileId });
}

export function apiKeyProfileDelete(provider: ApiKeyProvider, profileId: string): Promise<ApiKeyProfilesView> {
  return invoke<ApiKeyProfilesView>('api_key_profile_delete', { provider, profileId });
}

export function localModelProfiles(): Promise<LocalModelProfilesView> {
  return invoke<LocalModelProfilesView>('local_model_profiles');
}

export function localModelDownloadStatus(): Promise<LocalModelDownloadStatus> {
  return invoke<LocalModelDownloadStatus>('local_model_download_status');
}

export function localModelProfileAdd(
  label: string,
  model: string,
  device: string,
  activate = true
): Promise<LocalModelProfilesView> {
  return invoke<LocalModelProfilesView>('local_model_profile_add', { label, model, device, activate });
}

export function localModelProfileActivate(profileId: string): Promise<LocalModelProfilesView> {
  return invoke<LocalModelProfilesView>('local_model_profile_activate', { profileId });
}

export function localModelProfileDelete(profileId: string): Promise<LocalModelProfilesView> {
  return invoke<LocalModelProfilesView>('local_model_profile_delete', { profileId });
}

export function localModelProfileRetryDownload(profileId: string): Promise<LocalModelProfilesView> {
  return invoke<LocalModelProfilesView>('local_model_profile_retry_download', { profileId });
}

export async function listenLocalModelDownloadProgress(
  handler: (status: LocalModelDownloadStatus) => void
): Promise<() => void> {
  const events = await getEventApi();
  return events.listen<LocalModelDownloadStatus>('local-model-download-progress', (event) => handler(event.payload));
}
