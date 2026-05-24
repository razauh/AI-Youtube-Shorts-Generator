import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, cleanup } from '@testing-library/svelte';
import { createRunState } from '../lib/stores/runState';

const runGenerateAndStream = vi.fn();
const pickLocalVideoFile = vi.fn();
const pickOutputJsonPath = vi.fn();
const openInFileManager = vi.fn();
const checkForAppUpdate = vi.fn();
const installAppUpdate = vi.fn();
const appConfigSummary = vi.fn();
const validateRuntime = vi.fn();
const runtimeContext = vi.fn();
const secureStoreSave = vi.fn();
const secureStoreDelete = vi.fn();
const localModelProfiles = vi.fn();
const localModelDownloadStatus = vi.fn();
const localModelProfileAdd = vi.fn();
const localModelProfileActivate = vi.fn();
const localModelProfileDelete = vi.fn();
const localModelProfileRetryDownload = vi.fn();
const listenLocalModelDownloadProgress = vi.fn();
const localRuntimePackStatus = vi.fn();
const localRuntimePackPrepare = vi.fn();
const localRuntimePackRetry = vi.fn();
const localRuntimePackRepair = vi.fn();
const listenLocalRuntimePackProgress = vi.fn();
const apiKeyProfiles = vi.fn();
const apiKeyProfileAdd = vi.fn();
const apiKeyProfileActivate = vi.fn();
const apiKeyProfileDelete = vi.fn();
const authStoreMock = vi.hoisted(() => {
  let value: any = {
    lifecycle: 'licensed',
    authState: {
      status: 'licensed',
      masked_license_key: '****-1234',
      device_id: 'dev',
      token_expires_at_ms: 1,
      last_validated_at_ms: 1,
      next_validation_due_ms: 2
    },
    resetRequestId: null,
    resetStatus: 'idle',
    resetStatusMessage: null,
    resetError: null,
    error: null
  };
  let subscribers: Array<(value: any) => void> = [];
  const store = {
    subscribe: (fn: (value: any) => void) => {
      subscribers.push(fn);
      fn(value);
      return () => {
        subscribers = subscribers.filter((subscriber) => subscriber !== fn);
      };
    },
    bootstrap: vi.fn(),
    activate: vi.fn(),
    requestReset: vi.fn(),
    pollResetStatus: vi.fn(),
    clearSession: vi.fn()
  };
  return {
    store,
    set: (next: any) => {
      value = next;
      subscribers.forEach((fn) => fn(value));
    }
  };
});
let Page: Awaited<typeof import('../routes/+page.svelte')>['default'];

vi.mock('../lib/api/tauriClient', () => ({
  runGenerateAndStream: (...args: unknown[]) => runGenerateAndStream(...args),
  pickLocalVideoFile: (...args: unknown[]) => pickLocalVideoFile(...args),
  pickOutputJsonPath: (...args: unknown[]) => pickOutputJsonPath(...args),
  openInFileManager: (...args: unknown[]) => openInFileManager(...args)
}));

vi.mock('../lib/api/updaterClient', () => ({
  checkForAppUpdate: (...args: unknown[]) => checkForAppUpdate(...args),
  installAppUpdate: (...args: unknown[]) => installAppUpdate(...args)
}));

vi.mock('../lib/api/runtimeClient', () => ({
  appConfigSummary: (...args: unknown[]) => appConfigSummary(...args),
  validateRuntime: (...args: unknown[]) => validateRuntime(...args),
  runtimeContext: (...args: unknown[]) => runtimeContext(...args),
  secureStoreSave: (...args: unknown[]) => secureStoreSave(...args),
  secureStoreDelete: (...args: unknown[]) => secureStoreDelete(...args),
  localModelProfiles: (...args: unknown[]) => localModelProfiles(...args),
  localModelDownloadStatus: (...args: unknown[]) => localModelDownloadStatus(...args),
  localModelProfileAdd: (...args: unknown[]) => localModelProfileAdd(...args),
  localModelProfileActivate: (...args: unknown[]) => localModelProfileActivate(...args),
  localModelProfileDelete: (...args: unknown[]) => localModelProfileDelete(...args),
  localModelProfileRetryDownload: (...args: unknown[]) => localModelProfileRetryDownload(...args),
  listenLocalModelDownloadProgress: (...args: unknown[]) => listenLocalModelDownloadProgress(...args),
  localRuntimePackStatus: (...args: unknown[]) => localRuntimePackStatus(...args),
  localRuntimePackPrepare: (...args: unknown[]) => localRuntimePackPrepare(...args),
  localRuntimePackRetry: (...args: unknown[]) => localRuntimePackRetry(...args),
  localRuntimePackRepair: (...args: unknown[]) => localRuntimePackRepair(...args),
  listenLocalRuntimePackProgress: (...args: unknown[]) => listenLocalRuntimePackProgress(...args),
  apiKeyProfiles: (...args: unknown[]) => apiKeyProfiles(...args),
  apiKeyProfileAdd: (...args: unknown[]) => apiKeyProfileAdd(...args),
  apiKeyProfileActivate: (...args: unknown[]) => apiKeyProfileActivate(...args),
  apiKeyProfileDelete: (...args: unknown[]) => apiKeyProfileDelete(...args)
}));

vi.mock('../lib/stores/authState', () => ({
  authState: authStoreMock.store
}));

describe('test_ui flow parity', () => {
  beforeEach(() => {
    runGenerateAndStream.mockReset();
    pickLocalVideoFile.mockReset();
    pickOutputJsonPath.mockReset();
    openInFileManager.mockReset();
    checkForAppUpdate.mockReset();
    installAppUpdate.mockReset();
    appConfigSummary.mockReset();
    validateRuntime.mockReset();
    runtimeContext.mockReset();
    secureStoreSave.mockReset();
    secureStoreDelete.mockReset();
    localModelProfiles.mockReset();
    localModelDownloadStatus.mockReset();
    localModelProfileAdd.mockReset();
    localModelProfileActivate.mockReset();
    localModelProfileDelete.mockReset();
    localModelProfileRetryDownload.mockReset();
    listenLocalModelDownloadProgress.mockReset();
    localRuntimePackStatus.mockReset();
    localRuntimePackPrepare.mockReset();
    localRuntimePackRetry.mockReset();
    localRuntimePackRepair.mockReset();
    listenLocalRuntimePackProgress.mockReset();
    apiKeyProfiles.mockReset();
    apiKeyProfileAdd.mockReset();
    apiKeyProfileActivate.mockReset();
    apiKeyProfileDelete.mockReset();
    secureStoreSave.mockResolvedValue(undefined);
    secureStoreDelete.mockResolvedValue(undefined);
    localModelProfiles.mockResolvedValue({
      envOverride: false,
      activeProfileId: 'local-1',
      profiles: [{ id: 'local-1', label: 'Base local', model: 'base', device: 'auto', active: true, downloadStatus: 'ready', error: null, createdAtMs: 1, updatedAtMs: 1 }]
    });
    localModelDownloadStatus.mockResolvedValue({
      active: false,
      profileId: null,
      model: null,
      device: null,
      phase: 'idle',
      progress: 0,
      message: 'No local model download is running.',
      error: null
    });
    localModelProfileAdd.mockImplementation((label: string, model: string, device: string) => Promise.resolve({
      envOverride: false,
      activeProfileId: 'local-new',
      profiles: [{ id: 'local-new', label, model, device, active: true, downloadStatus: 'downloading', error: null, createdAtMs: 2, updatedAtMs: 2 }]
    }));
    localModelProfileActivate.mockImplementation((profileId: string) => Promise.resolve({
      envOverride: false,
      activeProfileId: profileId,
      profiles: [{ id: profileId, label: 'Activated local', model: 'small', device: 'cpu', active: true, downloadStatus: 'ready', error: null, createdAtMs: 1, updatedAtMs: 2 }]
    }));
    localModelProfileDelete.mockResolvedValue({ envOverride: false, activeProfileId: null, profiles: [] });
    localModelProfileRetryDownload.mockImplementation((profileId: string) => Promise.resolve({
      envOverride: false,
      activeProfileId: profileId,
      profiles: [{ id: profileId, label: 'Retry local', model: 'small', device: 'cpu', active: true, downloadStatus: 'downloading', error: null, createdAtMs: 1, updatedAtMs: 2 }]
    }));
    listenLocalModelDownloadProgress.mockResolvedValue(() => {});
    localRuntimePackStatus.mockResolvedValue({
      status: 'ready',
      version: '1.0.0',
      platform: 'linux',
      arch: 'x86_64',
      installDir: '/home/test/.local/share/app/runtime-pack/current',
      manifestUrl: 'https://example.test/manifest.json',
      requiredSizeBytes: 1,
      message: 'Local processing runtime is ready.',
      errorCode: null
    });
    localRuntimePackPrepare.mockImplementation(() => localRuntimePackStatus());
    localRuntimePackRetry.mockImplementation(() => localRuntimePackStatus());
    localRuntimePackRepair.mockImplementation(() => localRuntimePackStatus());
    listenLocalRuntimePackProgress.mockResolvedValue(() => {});
    apiKeyProfiles.mockImplementation((provider: 'muapi' | 'openai') => Promise.resolve({
      provider,
      envOverride: false,
      profiles: provider === 'muapi'
        ? [{ id: 'muapi-1', label: 'Current MuAPI key', lastFour: '1111', active: true, createdAtMs: 1, updatedAtMs: 1 }]
        : [{ id: 'openai-1', label: 'Current OpenAI key', lastFour: '2222', active: true, createdAtMs: 1, updatedAtMs: 1 }]
    }));
    apiKeyProfileAdd.mockImplementation((provider: 'muapi' | 'openai', label: string) => Promise.resolve({
      provider,
      envOverride: false,
      profiles: [{ id: `${provider}-new`, label, lastFour: provider === 'muapi' ? 'cret' : 'cret', active: true, createdAtMs: 2, updatedAtMs: 2 }]
    }));
    apiKeyProfileActivate.mockImplementation((provider: 'muapi' | 'openai', profileId: string) => Promise.resolve({
      provider,
      envOverride: false,
      profiles: [{ id: profileId, label: 'Activated profile', lastFour: '9999', active: true, createdAtMs: 1, updatedAtMs: 2 }]
    }));
    apiKeyProfileDelete.mockImplementation((provider: 'muapi' | 'openai') => Promise.resolve({
      provider,
      envOverride: false,
      profiles: []
    }));
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'hosted',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: true,
      openaiConfigured: true,
      localWhisperModel: 'base',
      localWhisperDevice: 'auto',
      licenseWorkerTimeoutMs: 10000,
      licenseWorkerRetryAttempts: 2
    });
    validateRuntime.mockResolvedValue({
      runtime: 'python:python3',
      bridge_entry: '../../python_legacy/bridge_entry.py',
      bridge_entry_exists: true,
      ok: true,
      local_runtime_ready: true,
      tools: [
        { tool: 'python', ok: true, path: '/usr/bin/python3', source: 'path', message: 'ok' },
        { tool: 'ffmpeg', ok: true, path: '/usr/bin/ffmpeg', source: 'path', message: 'ok' },
        { tool: 'yt-dlp', ok: true, path: '/usr/bin/yt-dlp', source: 'path', message: 'ok' }
      ],
      python_packages: [{ tool: 'faster_whisper', ok: true, message: 'ok' }]
    });
    runtimeContext.mockResolvedValue({
      appVersion: '0.1.0',
      platform: 'linux',
      runtimeRoot: '/home/test/.local/share/app',
      logDir: '/home/test/.local/share/app/logs',
      logPath: '/home/test/.local/share/app/logs/app.log',
      crashLogPath: '/home/test/.local/share/app/logs/crash.log',
      configPath: '/home/test/.local/share/app/config/config.json',
      protectedBasePath: '/home/test/.local/share/app/protected',
      secureFallbackBasePath: '/home/test/.local/share/app/secure-fallback'
    });
    authStoreMock.store.bootstrap.mockReset();
    authStoreMock.store.activate.mockReset();
    authStoreMock.store.requestReset.mockReset();
    authStoreMock.store.pollResetStatus.mockReset();
    authStoreMock.store.clearSession.mockReset();
    authStoreMock.set({
      lifecycle: 'licensed',
      authState: {
        status: 'licensed',
        masked_license_key: '****-1234',
        device_id: 'dev',
        token_expires_at_ms: 1,
        last_validated_at_ms: 1,
        next_validation_due_ms: 2
      },
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: null
    });
    localStorage.clear();
  });

  beforeEach(async () => {
    vi.resetModules();
    Page = (await import('../routes/+page.svelte')).default;
  });

  afterEach(() => {
    cleanup();
  });

  it('test_default form values parity', () => {
    render(Page);

    expect((screen.getByLabelText('YouTube video URL') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('Mode') as HTMLSelectElement).value).toBe('api');
    expect((screen.getByLabelText('Num clips') as HTMLInputElement).value).toBe('3');
    expect((screen.getByLabelText('Aspect ratio') as HTMLSelectElement).value).toBe('9:16');
    expect((screen.getByLabelText('Resolution') as HTMLSelectElement).value).toBe('720');
    expect((screen.getByLabelText('Output JSON path') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('YouTube video URL') as HTMLInputElement).getAttribute('required')).toBeNull();
    expect((screen.getByLabelText('Num clips') as HTMLInputElement).getAttribute('min')).toBeNull();
    expect(screen.queryByText('Setup needed before first generation')).toBeNull();
  });

  it('test_unauthenticated_state_hides_generator_and_submits_license', async () => {
    authStoreMock.set({
      lifecycle: 'unauthenticated',
      authState: { status: 'unauthenticated' },
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: null
    });

    render(Page);

    expect(screen.getByText('License Required')).toBeTruthy();
    expect(screen.queryByLabelText('YouTube video URL')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Generate' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Shorts Library' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Settings' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Help & Trust' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Terms' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Privacy' })).toBeNull();
    expect(screen.getByRole('switch', { name: 'Toggle theme' })).toBeTruthy();

    expect((screen.getByLabelText('License key') as HTMLInputElement).type).toBe('password');

    await fireEvent.input(screen.getByLabelText('License key'), { target: { value: 'LICENSE-1234' } });
    await fireEvent.click(screen.getByLabelText('Accept terms and conditions'));
    await fireEvent.click(screen.getByRole('button', { name: 'Activate' }));

    expect(authStoreMock.store.activate).toHaveBeenCalledWith('LICENSE-1234');
    expect(JSON.stringify(localStorage)).not.toContain('LICENSE-1234');
  });

  it('test_login_requires_accepting_terms_before_activation', async () => {
    authStoreMock.set({
      lifecycle: 'unauthenticated',
      authState: { status: 'unauthenticated' },
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: null
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('License key'), { target: { value: 'LICENSE-1234' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Activate' }));

    expect(authStoreMock.store.activate).not.toHaveBeenCalled();
    expect(screen.getByText('You must accept the Terms and Conditions to continue.')).toBeTruthy();
  });

  it('test_login_terms_link_opens_policy_modal_from_shared_source', async () => {
    authStoreMock.set({
      lifecycle: 'unauthenticated',
      authState: { status: 'unauthenticated' },
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: null
    });

    render(Page);
    await fireEvent.click(screen.getByRole('button', { name: 'Terms and Conditions' }));
    expect(screen.getByRole('dialog', { name: 'Terms and Conditions' })).toBeTruthy();
    expect(screen.getAllByText('Terms and Conditions').length).toBeGreaterThan(0);
    expect(screen.getByText('16. Prohibited Uses')).toBeTruthy();
    expect(screen.getByText('18. Limitation of Liability')).toBeTruthy();
  });

  it('test_global_theme_switch_toggles_checked_state', async () => {
    render(Page);
    const themeSwitch = screen.getByRole('switch', { name: 'Toggle theme' });

    expect(themeSwitch.getAttribute('aria-checked')).toBe('true');
    await fireEvent.click(themeSwitch);
    expect(themeSwitch.getAttribute('aria-checked')).toBe('false');
  });

  it('test_license_form_uses_persistent_app_error_for_empty_key', async () => {
    authStoreMock.set({
      lifecycle: 'unauthenticated',
      authState: { status: 'unauthenticated' },
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: null
    });

    render(Page);
    await fireEvent.click(screen.getByRole('button', { name: 'Activate' }));

    expect(authStoreMock.store.activate).not.toHaveBeenCalled();
    const message = screen.getByText('Enter your license key to continue.');
    expect(message.className).toContain('form-status');
    expect(message.className).toContain('form-status--error');
  });

  it('test_settings_screen_shows_redacted_runtime_and_device_status', async () => {
    authStoreMock.set({
      lifecycle: 'licensed',
      authState: {
        status: 'licensed',
        masked_license_key: '****-1234',
        device_id: 'raw-device-id',
        token_expires_at_ms: 1_700_000_000_000,
        last_validated_at_ms: 1_699_999_000_000,
        next_validation_due_ms: 1_700_086_400_000
      },
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: null
    });

    render(Page);
    expect(screen.getAllByRole('switch', { name: 'Toggle theme' })).toHaveLength(1);
    expect(screen.queryByRole('button', { name: 'Help & Trust' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Terms' })).toBeNull();

    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));

    expect(await screen.findByRole('heading', { name: 'Settings' })).toBeTruthy();
    expect(screen.queryByText('Configure API access, manage device licensing, and review diagnostics and policies.')).toBeNull();
    expect(screen.queryByText('License, device, and runtime status for this installation.')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Refresh' })).toBeNull();
    expect(screen.queryByText('Open Folder is available for locally generated shorts.')).toBeNull();
    expect(screen.getByRole('tab', { name: 'Configuration', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Diagnostics' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Policies' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Local Processing', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'API Providers' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Device Reset' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Local Processing help' })).toBeTruthy();
    expect(screen.getByText('On-device pipeline')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'MuAPI Access help' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'OpenAI Access help' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Device Reset help' })).toBeNull();

    await fireEvent.click(screen.getByRole('tab', { name: 'API Providers' }));
    expect(screen.getByRole('button', { name: 'MuAPI Access help' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'OpenAI Access help' })).toBeTruthy();
    expect(screen.getByText('Video provider')).toBeTruthy();
    expect(screen.getByText('LLM provider')).toBeTruthy();
    expect(screen.getByText('MuAPI Configured')).toBeTruthy();
    expect(screen.getByText('OpenAI Configured')).toBeTruthy();
    expect(screen.getByText('Current MuAPI key')).toBeTruthy();
    expect(screen.getByText('Current OpenAI key')).toBeTruthy();
    expect(screen.getAllByText('Active').length).toBeGreaterThanOrEqual(2);
    expect(screen.queryByText('License support')).toBeNull();
    expect(screen.queryByText('Model base')).toBeNull();
    expect(screen.queryByText('Device auto')).toBeNull();
    expect(screen.queryByText('licenses.example.test')).toBeNull();
    expect(screen.queryByText('Endpoint type')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Clear Local Session' })).toBeNull();
    expect(screen.queryByText('App version')).toBeNull();
    expect(screen.queryByText('Bridge entry')).toBeNull();
    expect(screen.queryByText('raw-device-id')).toBeNull();
    expect(document.body.textContent).not.toContain('/home/test');
    expect(appConfigSummary.mock.calls.length).toBeGreaterThanOrEqual(1);
    expect(validateRuntime.mock.calls.length).toBeGreaterThanOrEqual(1);
    expect(runtimeContext).toHaveBeenCalledTimes(1);
    await fireEvent.click(screen.getByRole('tab', { name: 'Device Reset' }));
    expect(screen.getByRole('button', { name: 'Device Reset help' })).toBeTruthy();
    expect(screen.getByText('License support')).toBeTruthy();
    await fireEvent.input(screen.getByLabelText('Settings reset license key'), { target: { value: 'LICENSE-1234' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Request Device Reset' }));
    expect(authStoreMock.store.requestReset).toHaveBeenCalledWith({ license_key: 'LICENSE-1234' }, { preserveLicensedSession: true });
    expect(screen.getByRole('heading', { name: 'Settings' })).toBeTruthy();

    await fireEvent.click(screen.getByRole('tab', { name: 'Configuration' }));
    await fireEvent.click(screen.getByRole('tab', { name: 'API Providers' }));
    expect(screen.getByText('MuAPI Access')).toBeTruthy();
    expect(screen.getByText('OpenAI Access')).toBeTruthy();
    expect(screen.queryByRole('heading', { name: 'Local Processing' })).toBeNull();
    expect(screen.queryByText('License Worker')).toBeNull();
    expect(screen.queryByText('Retry attempts')).toBeNull();
    expect(screen.queryByText('licenses.example.test')).toBeNull();
    await fireEvent.input(screen.getByLabelText('MuAPI profile name'), { target: { value: 'Client MuAPI' } });
    await fireEvent.input(screen.getByLabelText('MuAPI key'), { target: { value: 'mu-secret' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add MuAPI Profile' }));
    expect(apiKeyProfileAdd).toHaveBeenCalledWith('muapi', 'Client MuAPI', 'mu-secret', true);
    await fireEvent.input(screen.getByLabelText('OpenAI profile name'), { target: { value: 'Client OpenAI' } });
    await fireEvent.input(screen.getByLabelText('OpenAI key'), { target: { value: 'openai-secret' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add OpenAI Profile' }));
    expect(apiKeyProfileAdd).toHaveBeenCalledWith('openai', 'Client OpenAI', 'openai-secret', true);
    expect(document.body.textContent).not.toContain('mu-secret');
    expect(document.body.textContent).not.toContain('openai-secret');

    await fireEvent.click(screen.getByRole('tab', { name: 'Local Processing' }));
    await fireEvent.change(screen.getByLabelText('Whisper model'), { target: { value: 'small' } });
    await fireEvent.input(screen.getByLabelText('Local model profile name'), { target: { value: 'Small CPU' } });
    await fireEvent.change(screen.getByLabelText('Processing device'), { target: { value: 'cpu' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Save and Download' }));
    expect(localModelProfileAdd).toHaveBeenCalledWith('Small CPU', 'small', 'cpu', true);

    await fireEvent.click(screen.getByRole('tab', { name: 'API Providers' }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'Delete' })[0]);
    expect(apiKeyProfileDelete).toHaveBeenCalledWith('muapi', 'muapi-1');

    await fireEvent.click(screen.getByRole('tab', { name: 'Diagnostics' }));
    expect(screen.getByRole('tab', { name: 'Diagnostics', selected: true })).toBeTruthy();
    expect(screen.getByText('See system health and take action when setup issues are detected.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Recheck Dependencies' })).toBeTruthy();
    expect(screen.getByText('Required Dependencies')).toBeTruthy();
    expect(screen.getAllByText('Available').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Maintenance')).toBeTruthy();
    expect(screen.queryByText('Logs and Support')).toBeNull();

    expect(screen.queryByRole('button', { name: 'Terms' })).toBeNull();
    await fireEvent.click(screen.getByRole('tab', { name: 'Policies' }));
    expect(screen.getByRole('tab', { name: 'Policies', selected: true })).toBeTruthy();
    expect(screen.getByText('Reference documents for use, privacy, third-party notices, refunds, and liability.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Terms' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Privacy' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Third-Party Notices' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Refund Policy' })).toBeTruthy();
  });

  it('test_device_bound_state_shows_reset_request_form', async () => {
    authStoreMock.set({
      lifecycle: 'device_bound_elsewhere',
      authState: null,
      resetRequestId: null,
      resetStatus: 'idle',
      resetStatusMessage: null,
      resetError: null,
      error: { code: 'device_already_bound', message: 'license is already bound to another device' }
    });

    render(Page);
    expect(screen.getByText('License Required')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Generate' })).toBeNull();

    await fireEvent.input(screen.getByLabelText('Reset license key'), { target: { value: 'LICENSE-1234' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Request Reset' }));

    expect(authStoreMock.store.requestReset).toHaveBeenCalledWith({ license_key: 'LICENSE-1234' });
  });

  it('test_submit sends correct payload', async () => {
    runGenerateAndStream.mockResolvedValue({
      ok: true,
      result: { mode: 'api', source_video_url: 'x', transcript: { duration: 1, segments: [] }, highlights: [], shorts: [] }
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.change(screen.getByLabelText('Mode'), { target: { value: 'api' } });
    await fireEvent.input(screen.getByLabelText('Num clips'), { target: { value: '5' } });
    await fireEvent.input(screen.getByLabelText('Resolution'), { target: { value: '1080' } });
    await fireEvent.change(screen.getByLabelText('Aspect ratio'), { target: { value: '1:1' } });
    await fireEvent.input(screen.getByLabelText('Output JSON path'), { target: { value: 'result.json' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).toHaveBeenCalledTimes(1);
    const req = runGenerateAndStream.mock.calls[0][0];
    expect(req).toEqual({
      youtube_url: 'https://youtube.com/watch?v=abc',
      mode: 'api',
      num_clips: 5,
      aspect_ratio: '1:1',
      download_format: '1080',
      output_json: 'result.json'
    });
  });

  it('test_generate_is_blocked_and_guided_when_setup_is_incomplete', async () => {
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'hosted',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: false,
      openaiConfigured: true,
      localWhisperModel: 'base',
      localWhisperDevice: 'auto',
      licenseWorkerTimeoutMs: 10000,
      licenseWorkerRetryAttempts: 2
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog', { name: 'Setup Required Before Generating' })).toBeTruthy();
    expect(screen.getByText('To generate shorts, you need to configure either an API-based setup or a local model first.')).toBeTruthy();
    expect(screen.getByText('API key is not configured')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Configure Now' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Recheck Setup' })).toBeTruthy();
  });

  it('test_setup_modal_configure_now_routes_to_configuration_api_providers', async () => {
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'hosted',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: false,
      openaiConfigured: true,
      localWhisperModel: 'base',
      localWhisperDevice: 'auto',
      licenseWorkerTimeoutMs: 10000,
      licenseWorkerRetryAttempts: 2
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Configure Now' }));

    expect(screen.getByRole('heading', { name: 'Settings' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Configuration', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'API Providers', selected: true })).toBeTruthy();
  });

  it('test_setup_modal_cancel_closes_and_keeps_form_values', async () => {
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'hosted',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: false,
      openaiConfigured: true,
      localWhisperModel: 'base',
      localWhisperDevice: 'auto',
      licenseWorkerTimeoutMs: 10000,
      licenseWorkerRetryAttempts: 2
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('Project title'), { target: { value: 'My Project' } });
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.input(screen.getByLabelText('Num clips'), { target: { value: '4' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(screen.getByRole('dialog', { name: 'Setup Required Before Generating' })).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('dialog', { name: 'Setup Required Before Generating' })).toBeNull();
    expect((screen.getByLabelText('Project title') as HTMLInputElement).value).toBe('My Project');
    expect((screen.getByLabelText('YouTube video URL') as HTMLInputElement).value).toBe('https://youtube.com/watch?v=abc');
    expect((screen.getByLabelText('Num clips') as HTMLInputElement).value).toBe('4');
  });

  it('test_setup_modal_configure_now_routes_to_configuration_local_processing_for_local_blockers', async () => {
    localModelProfiles.mockResolvedValue({
      envOverride: false,
      activeProfileId: null,
      profiles: []
    });

    render(Page);
    await fireEvent.change(screen.getByLabelText('Mode'), { target: { value: 'local' } });
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    expect(screen.getByText('Local processing runtime is not ready. Download it from Settings before running local mode.')).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));

    expect(screen.getByRole('heading', { name: 'Settings' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Configuration', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Local Processing', selected: true })).toBeTruthy();
  });

  it('test_generate_form_shows_persistent_error_for_empty_source', async () => {
    render(Page);
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).not.toHaveBeenCalled();
    const message = screen.getByText('Enter a YouTube video URL before running.');
    expect(message.className).toContain('form-status');
    expect(message.className).toContain('form-status--error');
  });

  it('test_generate_form_blocks_invalid_num_clips_with_app_error', async () => {
    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.input(screen.getByLabelText('Num clips'), { target: { value: '0' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).not.toHaveBeenCalled();
    const message = screen.getByText('Num clips must be at least 1.');
    expect(message.className).toContain('form-status');
    expect(message.className).toContain('form-status--error');
  });

  it('test_event updates state lifecycle and progress', async () => {
    const state = createRunState();
    state.start();
    state.onProgress({ event: 'progress', stage: 'download:start', progress: 0.1 });
    state.onProgress({ event: 'progress', stage: 'clip:end', progress: 1.0 });

    let latest: any;
    const unsub = state.subscribe((v) => {
      latest = v;
    });

    expect(latest.lifecycle).toBe('running');
    expect(latest.progress.stage).toBe('clip:end');
    expect(latest.progress.value).toBe(1);
    unsub();
  });

  it('test_update_panel_checks_and_installs_official_plugin_update', async () => {
    checkForAppUpdate.mockResolvedValue({
      available: true,
      update: { version: '0.2.0' }
    });
    installAppUpdate.mockResolvedValue({
      installed: true,
      version: '0.2.0'
    });

    render(Page);
    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await fireEvent.click(screen.getByRole('tab', { name: 'Diagnostics' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Check for Updates' }));

    expect(await screen.findByText('Update 0.2.0 is available.')).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Install Update 0.2.0' }));

    expect(await screen.findByText('Update 0.2.0 installed. Restart the app to finish.')).toBeTruthy();
    expect(checkForAppUpdate).toHaveBeenCalledTimes(1);
    expect(installAppUpdate).toHaveBeenCalledTimes(1);
  });

  it('test_pending_crash_draft_is_local_and_dismissible', async () => {
    localStorage.setItem(
      'shorts.crashDraft.v1',
      JSON.stringify({
        appVersion: '0.1.0',
        platform: 'linux',
        timestamp: '2026-05-13T12:00:00.000Z',
        errorName: 'Error',
        message: 'boom'
      })
    );

    render(Page);
    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await fireEvent.click(screen.getByRole('tab', { name: 'Diagnostics' }));

    expect(await screen.findByText('Crash Report Draft')).toBeTruthy();
    expect(screen.getByText('Error: boom')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss Crash Report' }));

    expect(screen.queryByText('Crash Report Draft')).toBeNull();
    expect(localStorage.getItem('shorts.crashDraft.v1')).toBeNull();
  });

  it('test_legal_page_states_manual_7_day_refund_policy', async () => {
    render(Page);
    expect(screen.queryByRole('button', { name: 'Legal' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Terms' })).toBeNull();

    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(screen.queryByText('1. Acceptance of Terms')).toBeNull();

    await fireEvent.click(screen.getByRole('tab', { name: 'Policies' }));
    expect(screen.getByText('1. Acceptance of Terms')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Privacy' }));
    expect(screen.getByText('Privacy Policy')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Third-Party Notices' }));
    expect(screen.getAllByText('Third-Party Notices').length).toBeGreaterThan(0);
    expect(screen.getByText('3. FFmpeg')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Refund Policy' }));

    expect(screen.getAllByText('Refund Policy').length).toBeGreaterThan(0);
    expect(screen.getByText(/within 7 days from purchase/)).toBeTruthy();
    expect(screen.getByText('No automated refund engine is built into this app.')).toBeTruthy();
  });

  it('test_render short entries and failures', async () => {
    runGenerateAndStream.mockResolvedValue({
      ok: true,
      result: {
        mode: 'api',
        source_video_url: 'https://cdn.example.com/video.mp4',
        transcript: { duration: 1, segments: [] },
        highlights: [{ title: 'H1', start_time: 1, end_time: 2, score: 90, hook_sentence: 'hook', virality_reason: 'reason' }],
        shorts: [
          { title: 'Hit', start_time: 1, end_time: 2, score: 90, hook_sentence: 'hook', virality_reason: 'reason', clip_url: 'https://cdn.example.com/s1.mp4' },
          { title: 'Miss', start_time: 3, end_time: 4, score: 60, hook_sentence: 'hook2', virality_reason: 'reason2', clip_url: null, error: 'render failed' }
        ]
      }
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(await screen.findByText('Hit')).toBeTruthy();
    expect(await screen.findByText('Miss')).toBeTruthy();
    expect(await screen.findByText(/FAILED \(render failed\)/)).toBeTruthy();
  });
});
