export interface AppUpdateInfo {
  version: string;
  currentVersion?: string;
  date?: string;
  body?: string;
}

export type AppUpdateCheckResult =
  | { available: true; update: AppUpdateInfo }
  | { available: false; update: null };

export type AppUpdateInstallResult =
  | { installed: true; version: string }
  | { installed: false; reason: 'no_update' | 'unavailable'; message?: string };

interface PluginUpdate {
  version: string;
  currentVersion?: string;
  date?: string;
  body?: string;
  downloadAndInstall?: (onEvent?: (event: unknown) => void) => Promise<void>;
}

interface UpdaterPlugin {
  check(): Promise<PluginUpdate | null>;
}

let updaterPromise: Promise<UpdaterPlugin> | null = null;

async function getUpdaterPlugin(): Promise<UpdaterPlugin> {
  if (!updaterPromise) {
    updaterPromise = import('@tauri-apps/plugin-updater') as Promise<UpdaterPlugin>;
  }
  return updaterPromise;
}

function toUpdateInfo(update: PluginUpdate): AppUpdateInfo {
  return {
    version: update.version,
    currentVersion: update.currentVersion,
    date: update.date,
    body: update.body,
  };
}

export async function checkForAppUpdate(): Promise<AppUpdateCheckResult> {
  const { check } = await getUpdaterPlugin();
  const update = await check();

  if (!update) {
    return { available: false, update: null };
  }

  return {
    available: true,
    update: toUpdateInfo(update),
  };
}

export async function installAppUpdate(onEvent?: (event: unknown) => void): Promise<AppUpdateInstallResult> {
  const { check } = await getUpdaterPlugin();
  const update = await check();

  if (!update) {
    return { installed: false, reason: 'no_update' };
  }

  if (!update.downloadAndInstall) {
    return {
      installed: false,
      reason: 'unavailable',
      message: 'Updater plugin returned an update without an install handler.',
    };
  }

  await update.downloadAndInstall(onEvent);
  return { installed: true, version: update.version };
}
