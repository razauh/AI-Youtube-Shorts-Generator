import { describe, expect, it } from 'vitest';
import { JsonConfigStore, type ConfigStoreAdapter } from '../../storage/config';

describe('test_shared config store', () => {
  it('test_creates defaults when config file is missing', async () => {
    const files = new Map<string, string>();
    const adapter: ConfigStoreAdapter = {
      read: async (path) => files.get(path) ?? null,
      write: async (path, value) => {
        files.set(path, value);
      },
    };
    const store = new JsonConfigStore(adapter, 'config.json', { log_level: 'INFO' });
    expect(await store.get('log_level')).toBe('INFO');
    expect(files.has('config.json')).toBe(true);
  });

  it('test_resets to defaults on corrupt config file', async () => {
    const files = new Map<string, string>([['config.json', '{bad-json']]);
    const adapter: ConfigStoreAdapter = {
      read: async (path) => files.get(path) ?? null,
      write: async (path, value) => {
        files.set(path, value);
      },
    };
    const store = new JsonConfigStore(adapter, 'config.json', { update_mode: 'manual' });
    expect(await store.get('update_mode')).toBe('manual');
    expect(files.get('config.json')).toContain('update_mode');
  });
});
