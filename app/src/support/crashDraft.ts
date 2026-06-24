export interface CrashDraftStore {
  load(key: string): Promise<string | null>;
  save(key: string, value: string): Promise<void>;
  delete(key: string): Promise<void>;
}

export interface CrashDraft {
  appVersion: string;
  platform: string;
  timestamp: string;
  errorName: string;
  message: string;
  stack?: string;
}

export const CRASH_DRAFT_KEY = 'shorts.crashDraft.v1';

function redactSensitiveText(value: string): string {
  return value
    .replace(/[A-Fa-f0-9]{8}(?:-[A-Fa-f0-9]{8}){3}/g, '[redacted-license-key]')
    .replace(/(license[_-]?key["'=:\s]+)[^\s"',}]+/gi, '$1[redacted]')
    .replace(/(secret["'=:\s]+)[^\s"',}]+/gi, '$1[redacted]');
}

export function createCrashDraft(error: unknown, input: { appVersion: string; platform: string; now?: Date }): CrashDraft {
  const err = error instanceof Error ? error : new Error(String(error));

  return {
    appVersion: input.appVersion,
    platform: input.platform,
    timestamp: (input.now ?? new Date()).toISOString(),
    errorName: redactSensitiveText(err.name || 'Error'),
    message: redactSensitiveText(err.message || 'Unknown fatal error'),
    stack: err.stack ? redactSensitiveText(err.stack) : undefined,
  };
}

export async function saveCrashDraft(store: CrashDraftStore, draft: CrashDraft): Promise<void> {
  await store.save(CRASH_DRAFT_KEY, JSON.stringify(draft));
}

export async function readPendingCrashDraft(store: CrashDraftStore): Promise<CrashDraft | null> {
  const raw = await store.load(CRASH_DRAFT_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as CrashDraft;
  } catch {
    await store.delete(CRASH_DRAFT_KEY);
    return null;
  }
}

export async function dismissCrashDraft(store: CrashDraftStore): Promise<void> {
  await store.delete(CRASH_DRAFT_KEY);
}
