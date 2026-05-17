interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
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

let corePromise: Promise<TauriCore> | null = null;

async function getCore(): Promise<TauriCore> {
  if (!corePromise) {
    corePromise = import('@tauri-apps/api/core') as Promise<TauriCore>;
  }
  return corePromise;
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
