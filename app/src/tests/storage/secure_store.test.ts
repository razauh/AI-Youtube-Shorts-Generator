import { describe, expect, it } from 'vitest';
import { FallbackSecureStore, InMemorySecureStore, type SecureStore } from '../../storage/secure';

describe('test_shared secure store', () => {
  it('test_supports save/load/delete round-trip', async () => {
    const store = new InMemorySecureStore();
    await store.save('license_token', 'jwt-1');
    expect(await store.load('license_token')).toBe('jwt-1');
    expect(await store.exists('license_token')).toBe(true);
    await store.delete('license_token');
    expect(await store.load('license_token')).toBeNull();
  });

  it('test_falls back when primary keychain store is unavailable', async () => {
    const failingPrimary: SecureStore = {
      save: async () => {
        throw new Error('keychain unavailable');
      },
      load: async () => {
        throw new Error('keychain unavailable');
      },
      delete: async () => {
        throw new Error('keychain unavailable');
      },
      exists: async () => {
        throw new Error('keychain unavailable');
      },
    };
    let fallbackCode: string | null = null;
    const fallback = new InMemorySecureStore();
    const store = new FallbackSecureStore(
      failingPrimary,
      fallback,
      (code) => (fallbackCode = code)
    );

    await store.save('license_token', 'jwt-2');
    expect(await store.load('license_token')).toBe('jwt-2');
    expect(fallbackCode).toBe('STOR_KEYCHAIN_UNAVAILABLE');
  });
});
