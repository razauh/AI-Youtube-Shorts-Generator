import { describe, expect, it } from 'vitest';
import { EncryptedProtectedStore, type ProtectedStoreAdapter } from '../../storage/protected';

describe('test_shared protected store', () => {
  it('test_stores encrypted payload (not plaintext) and round-trips', async () => {
    const files = new Map<string, string>();
    const adapter: ProtectedStoreAdapter = {
      read: async (path) => files.get(path) ?? null,
      write: async (path, value) => {
        files.set(path, value);
      },
      remove: async (path) => {
        files.delete(path);
      },
    };
    const store = new EncryptedProtectedStore(adapter, 'machine-secret');
    await store.save('license_meta', { grace_until: '2026-06-01T00:00:00Z' });

    const raw = files.get('protected/license_meta.enc') ?? '';
    expect(raw).not.toContain('grace_until');
    const loaded = await store.load<{ grace_until: string }>('license_meta');
    expect(loaded?.grace_until).toBe('2026-06-01T00:00:00Z');
  });

  it('test_treats corrupt encrypted payload as empty and removes file', async () => {
    const files = new Map<string, string>([['protected/device_info.enc', 'bad-data']]);
    const adapter: ProtectedStoreAdapter = {
      read: async (path) => files.get(path) ?? null,
      write: async (path, value) => {
        files.set(path, value);
      },
      remove: async (path) => {
        files.delete(path);
      },
    };
    const store = new EncryptedProtectedStore(adapter, 'machine-secret');
    const loaded = await store.load('device_info');
    expect(loaded).toBeNull();
    expect(files.has('protected/device_info.enc')).toBe(false);
  });
});
