import { beforeEach, describe, expect, it, vi } from 'vitest';

const check = vi.fn();

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: () => check(),
}));

describe('test_updater_client_off_the_shelf_wrapper', () => {
  beforeEach(() => {
    vi.resetModules();
    check.mockReset();
  });

  it('test_check_returns_no_update_when_plugin_returns_null', async () => {
    check.mockResolvedValue(null);
    const { checkForAppUpdate } = await import('../lib/api/updaterClient');

    await expect(checkForAppUpdate()).resolves.toEqual({
      available: false,
      update: null,
    });
  });

  it('test_check_maps_available_update_metadata', async () => {
    check.mockResolvedValue({
      version: '0.2.0',
      currentVersion: '0.1.0',
      date: '2026-05-13T12:00:00Z',
      body: 'Bug fixes',
    });
    const { checkForAppUpdate } = await import('../lib/api/updaterClient');

    await expect(checkForAppUpdate()).resolves.toEqual({
      available: true,
      update: {
        version: '0.2.0',
        currentVersion: '0.1.0',
        date: '2026-05-13T12:00:00Z',
        body: 'Bug fixes',
      },
    });
  });

  it('test_install_uses_plugin_download_and_install', async () => {
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    const onEvent = vi.fn();
    check.mockResolvedValue({
      version: '0.2.0',
      downloadAndInstall,
    });
    const { installAppUpdate } = await import('../lib/api/updaterClient');

    await expect(installAppUpdate(onEvent)).resolves.toEqual({
      installed: true,
      version: '0.2.0',
    });
    expect(downloadAndInstall).toHaveBeenCalledWith(onEvent);
  });

  it('test_install_reports_no_update_without_custom_updater_fallback', async () => {
    check.mockResolvedValue(null);
    const { installAppUpdate } = await import('../lib/api/updaterClient');

    await expect(installAppUpdate()).resolves.toEqual({
      installed: false,
      reason: 'no_update',
    });
  });
});
