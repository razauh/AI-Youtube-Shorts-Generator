import { describe, expect, it, vi } from 'vitest';
import {
  CRASH_DRAFT_KEY,
  createCrashDraft,
  dismissCrashDraft,
  readPendingCrashDraft,
  saveCrashDraft,
  type CrashDraftStore,
} from '../../support/crashDraft';

function memoryStore(initial?: string): CrashDraftStore & { values: Map<string, string> } {
  const values = new Map<string, string>();
  if (initial) {
    values.set(CRASH_DRAFT_KEY, initial);
  }
  return {
    values,
    load: vi.fn(async (key: string) => values.get(key) ?? null),
    save: vi.fn(async (key: string, value: string) => {
      values.set(key, value);
    }),
    delete: vi.fn(async (key: string) => {
      values.delete(key);
    }),
  };
}

describe('test_crash_draft_launch_observability', () => {
  it('test_creates_minimal_local_crash_draft', () => {
    const draft = createCrashDraft(new Error('boom'), {
      appVersion: '0.1.0',
      platform: 'linux',
      now: new Date('2026-05-13T12:00:00Z'),
    });

    expect(draft).toMatchObject({
      appVersion: '0.1.0',
      platform: 'linux',
      timestamp: '2026-05-13T12:00:00.000Z',
      errorName: 'Error',
      message: 'boom',
    });
  });

  it('test_redacts_license_keys_and_secrets', () => {
    const draft = createCrashDraft(
      new Error('license_key=AAAA1111-BBBB2222-CCCC3333-DDDD4444 secret=top-secret-value'),
      { appVersion: '0.1.0', platform: 'linux' }
    );

    expect(JSON.stringify(draft)).not.toContain('AAAA1111-BBBB2222-CCCC3333-DDDD4444');
    expect(JSON.stringify(draft)).not.toContain('top-secret-value');
  });

  it('test_saves_and_reads_pending_crash_draft', async () => {
    const store = memoryStore();
    const draft = createCrashDraft(new Error('boom'), { appVersion: '0.1.0', platform: 'linux' });

    await saveCrashDraft(store, draft);

    expect(await readPendingCrashDraft(store)).toEqual(draft);
  });

  it('test_corrupt_draft_is_deleted', async () => {
    const store = memoryStore('{bad-json');

    expect(await readPendingCrashDraft(store)).toBeNull();
    expect(store.values.has(CRASH_DRAFT_KEY)).toBe(false);
  });

  it('test_dismiss_deletes_pending_draft', async () => {
    const store = memoryStore(JSON.stringify({ message: 'boom' }));

    await dismissCrashDraft(store);

    expect(store.values.has(CRASH_DRAFT_KEY)).toBe(false);
  });
});
