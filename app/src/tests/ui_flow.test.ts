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
const authStoreMock = vi.hoisted(() => {
  let value: any = {
    lifecycle: 'licensed',
    authState: { status: 'licensed', masked_license_key: '****-1234', device_id: 'dev', token_expires_at_ms: 1 },
    resetRequestId: null,
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
    pollResetStatus: vi.fn()
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
  runtimeContext: (...args: unknown[]) => runtimeContext(...args)
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
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'hosted',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: true,
      openaiConfigured: false,
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
      tools: [{ tool: 'python', ok: true, path: '/usr/bin/python3', source: 'path', message: 'ok' }]
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
    authStoreMock.set({
      lifecycle: 'licensed',
      authState: { status: 'licensed', masked_license_key: '****-1234', device_id: 'dev', token_expires_at_ms: 1 },
      resetRequestId: null,
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
  });

  it('test_unauthenticated_state_hides_generator_and_submits_license', async () => {
    authStoreMock.set({
      lifecycle: 'unauthenticated',
      authState: { status: 'unauthenticated' },
      resetRequestId: null,
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

    expect((screen.getByLabelText('License key') as HTMLInputElement).type).toBe('password');

    await fireEvent.input(screen.getByLabelText('License key'), { target: { value: 'LICENSE-1234' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Activate' }));

    expect(authStoreMock.store.activate).toHaveBeenCalledWith('LICENSE-1234');
    expect(JSON.stringify(localStorage)).not.toContain('LICENSE-1234');
  });

  it('test_settings_screen_shows_redacted_runtime_and_device_status', async () => {
    authStoreMock.set({
      lifecycle: 'licensed',
      authState: {
        status: 'licensed',
        masked_license_key: '****-1234',
        device_id: 'raw-device-id',
        token_expires_at_ms: 1_700_000_000_000
      },
      resetRequestId: null,
      error: null
    });

    render(Page);
    expect(screen.queryByRole('button', { name: 'Help & Trust' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Terms' })).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));

    expect(await screen.findByText('Runtime, license, and device status for this installation.')).toBeTruthy();
    expect(screen.getByText('Registered on this device')).toBeTruthy();
    expect(screen.getByText('****-1234')).toBeTruthy();
    expect(screen.getByText('licenses.example.test')).toBeTruthy();
    expect(screen.getByText('Available')).toBeTruthy();
    expect(screen.queryByText('raw-device-id')).toBeNull();
    expect(document.body.textContent).not.toContain('/home/test');
    expect(appConfigSummary).toHaveBeenCalledTimes(1);
    expect(validateRuntime).toHaveBeenCalledTimes(1);
    expect(runtimeContext).toHaveBeenCalledTimes(1);
  });

  it('test_device_bound_state_shows_reset_request_form', async () => {
    authStoreMock.set({
      lifecycle: 'device_bound_elsewhere',
      authState: null,
      resetRequestId: null,
      error: { code: 'device_already_bound', message: 'license is already bound to another device' }
    });

    render(Page);
    expect(screen.getByText('License Required')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Generate' })).toBeNull();

    await fireEvent.input(screen.getByLabelText('Purchaser email'), { target: { value: 'buyer@example.com' } });
    await fireEvent.input(screen.getByLabelText('Receipt reference'), { target: { value: 'receipt-1' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Request Reset' }));

    expect(authStoreMock.store.requestReset).toHaveBeenCalledWith({
      purchaser_email: 'buyer@example.com',
      receipt_reference: 'receipt-1'
    });
  });

  it('test_submit sends correct payload', async () => {
    runGenerateAndStream.mockResolvedValue({
      ok: true,
      result: { mode: 'api', source_video_url: 'x', transcript: { duration: 1, segments: [] }, highlights: [], shorts: [] }
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.change(screen.getByLabelText('Mode'), { target: { value: 'local' } });
    await fireEvent.input(screen.getByLabelText('Num clips'), { target: { value: '5' } });
    await fireEvent.input(screen.getByLabelText('Resolution'), { target: { value: '1080' } });
    await fireEvent.change(screen.getByLabelText('Aspect ratio'), { target: { value: '1:1' } });
    await fireEvent.input(screen.getByLabelText('Output JSON path'), { target: { value: 'result.json' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).toHaveBeenCalledTimes(1);
    const req = runGenerateAndStream.mock.calls[0][0];
    expect(req).toEqual({
      youtube_url: 'https://youtube.com/watch?v=abc',
      mode: 'local',
      num_clips: 5,
      aspect_ratio: '1:1',
      download_format: '1080',
      output_json: 'result.json'
    });
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

    expect(await screen.findByText('Crash Report Draft')).toBeTruthy();
    expect(screen.getByText('Error: boom')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss Crash Report' }));

    expect(screen.queryByText('Crash Report Draft')).toBeNull();
    expect(localStorage.getItem('shorts.crashDraft.v1')).toBeNull();
  });

  it('test_legal_page_states_manual_7_day_refund_policy', async () => {
    render(Page);
    expect(screen.queryByRole('button', { name: 'Legal' })).toBeNull();
    await fireEvent.click(screen.getByRole('button', { name: 'Terms' }));

    expect(screen.getByText('Refund Policy')).toBeTruthy();
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
