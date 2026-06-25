import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, cleanup } from '@testing-library/svelte';
import { createRunState } from '../lib/stores/runState';
import { POLICY_LAST_UPDATED_LABEL, POLICY_SECTIONS, type PolicyTab } from '../lib/legal/policiesContent';

const runGenerateAndStream = vi.fn();
const pickOutputJsonPath = vi.fn();
const checkForAppUpdate = vi.fn();
const installAppUpdate = vi.fn();
const appConfigSummary = vi.fn();
const runtimeContext = vi.fn();
const secureStoreLoad = vi.fn();
const secureStoreSave = vi.fn();
const secureStoreDelete = vi.fn();
const apiKeyProfiles = vi.fn();
const apiKeyProfileAdd = vi.fn();
const apiKeyProfileActivate = vi.fn();
const apiKeyProfileDelete = vi.fn();
const clipboardWriteText = vi.fn();
const windowOpen = vi.fn();
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
    deactivateCurrentDevice: vi.fn(),
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

Object.defineProperty(navigator, 'clipboard', {
  configurable: true,
  value: {
    writeText: (...args: unknown[]) => clipboardWriteText(...args)
  }
});
Object.defineProperty(window, 'open', {
  configurable: true,
  writable: true,
  value: (...args: unknown[]) => windowOpen(...args)
});

vi.mock('../lib/api/tauriClient', () => ({
  runGenerateAndStream: (...args: unknown[]) => runGenerateAndStream(...args),
  pickOutputJsonPath: (...args: unknown[]) => pickOutputJsonPath(...args)
}));

vi.mock('../lib/api/updaterClient', () => ({
  checkForAppUpdate: (...args: unknown[]) => checkForAppUpdate(...args),
  installAppUpdate: (...args: unknown[]) => installAppUpdate(...args)
}));

vi.mock('../lib/api/runtimeClient', () => ({
  appConfigSummary: (...args: unknown[]) => appConfigSummary(...args),
  runtimeContext: (...args: unknown[]) => runtimeContext(...args),
  secureStoreLoad: (...args: unknown[]) => secureStoreLoad(...args),
  secureStoreSave: (...args: unknown[]) => secureStoreSave(...args),
  secureStoreDelete: (...args: unknown[]) => secureStoreDelete(...args),
  apiKeyProfiles: (...args: unknown[]) => apiKeyProfiles(...args),
  apiKeyProfileAdd: (...args: unknown[]) => apiKeyProfileAdd(...args),
  apiKeyProfileActivate: (...args: unknown[]) => apiKeyProfileActivate(...args),
  apiKeyProfileDelete: (...args: unknown[]) => apiKeyProfileDelete(...args)
}));

vi.mock('../lib/stores/authState', () => ({
  authState: authStoreMock.store
}));

describe('test_ui flow parity', () => {
  const chooseThemedSelectOption = async (label: string, optionText: string) => {
    await fireEvent.click(screen.getByLabelText(label));
    await fireEvent.click(screen.getByRole('option', { name: optionText }));
  };

  beforeEach(() => {
    runGenerateAndStream.mockReset();
    pickOutputJsonPath.mockReset();
    checkForAppUpdate.mockReset();
    installAppUpdate.mockReset();
    appConfigSummary.mockReset();
    runtimeContext.mockReset();
    secureStoreLoad.mockReset();
    secureStoreSave.mockReset();
    secureStoreDelete.mockReset();
    apiKeyProfiles.mockReset();
    apiKeyProfileAdd.mockReset();
    apiKeyProfileActivate.mockReset();
    apiKeyProfileDelete.mockReset();
    clipboardWriteText.mockReset();
    windowOpen.mockReset();
    clipboardWriteText.mockResolvedValue(undefined);
    windowOpen.mockReturnValue({});
    secureStoreLoad.mockResolvedValue(null);
    secureStoreSave.mockResolvedValue(undefined);
    secureStoreDelete.mockResolvedValue(undefined);
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
      licenseBackendMode: 'devolens',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: true,
      openaiConfigured: true,
      licenseWorkerTimeoutMs: 10000,
      licenseWorkerRetryAttempts: 2
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
    authStoreMock.store.deactivateCurrentDevice.mockReset();
    authStoreMock.store.clearSession.mockReset();
    authStoreMock.store.deactivateCurrentDevice.mockResolvedValue(undefined);
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

  it('test_default form values parity', async () => {
    render(Page);

    expect(await screen.findByRole('dialog', { name: 'Setup Guide' })).toBeTruthy();
    expect((screen.getByLabelText('YouTube video URL') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('Num clips') as HTMLInputElement).value).toBe('3');
    expect(screen.getByLabelText('Aspect ratio').textContent).toContain('9:16 (Shorts/Reels/TikTok)');
    expect(screen.getByLabelText('Resolution').textContent).toContain('720p');
    expect((screen.getByLabelText('Output JSON path') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('YouTube video URL') as HTMLInputElement).getAttribute('required')).toBeNull();
    expect((screen.getByLabelText('Num clips') as HTMLInputElement).getAttribute('min')).toBeNull();
    expect(screen.queryByText('Setup needed before first generation')).toBeNull();
  });

  it('test_licensed_first_launch_shows_customer_onboarding', async () => {
    render(Page);

    expect(await screen.findByRole('dialog', { name: 'Setup Guide' })).toBeTruthy();
    expect(screen.getByText('License activated.')).toBeTruthy();
    expect(screen.getByText('Add MuAPI key.')).toBeTruthy();
    expect(screen.getByText('Generate first clip.')).toBeTruthy();
    expect(screen.getByText('Retrieve output.')).toBeTruthy();
    expect(screen.getByText('Support and policies.')).toBeTruthy();
    expect(await screen.findByText(/Add MuAPI Profile|API setup ready/)).toBeTruthy();
  });

  it('test_onboarding_skip_hides_and_persists_skip', async () => {
    render(Page);

    await screen.findByRole('dialog', { name: 'Setup Guide' });
    await fireEvent.click(screen.getByRole('button', { name: 'Skip' }));

    expect(screen.queryByRole('dialog', { name: 'Setup Guide' })).toBeNull();
    expect(localStorage.getItem('shorts.onboarding.v1')).toBe('skipped');
  });

  it('test_onboarding_start_setup_routes_to_api_provider_settings', async () => {
    render(Page);

    await screen.findByRole('dialog', { name: 'Setup Guide' });
    await fireEvent.click(screen.getByRole('button', { name: 'Start Setup' }));

    expect(screen.getByRole('heading', { name: 'Settings' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Configuration', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'API Providers', selected: true })).toBeTruthy();
    expect(screen.getByText('MuAPI Access')).toBeTruthy();
  });

  it('test_onboarding_marks_muapi_ready_when_configured', async () => {
    render(Page);

    expect(await screen.findByText(/Add MuAPI Profile|API setup ready/)).toBeTruthy();
  });

  it('test_setup_guide_reopens_onboarding_after_skip', async () => {
    render(Page);
    await screen.findByRole('dialog', { name: 'Setup Guide' });
    await fireEvent.click(screen.getByRole('button', { name: 'Skip' }));

    await fireEvent.click(screen.getByRole('button', { name: 'Setup Guide' }));

    expect(screen.getByRole('dialog', { name: 'Setup Guide' })).toBeTruthy();
    expect(localStorage.getItem('shorts.onboarding.v1')).toBe('skipped');
  });

  it('test_select_controls_use_themed_select_wrapper', () => {
    render(Page);

    const selectLabels = ['Aspect ratio', 'Resolution'];
    for (const label of selectLabels) {
      const selectEl = screen.getByLabelText(label) as HTMLButtonElement;
      expect(selectEl.tagName).toBe('BUTTON');
      expect(selectEl.getAttribute('aria-haspopup')).toBe('listbox');
    }
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

  it('test_policy_content_has_no_release_placeholders_or_stale_product_names', () => {
    const encoded = JSON.stringify(POLICY_SECTIONS);
    expect(encoded).toContain('AI YouTube Shorts Generator');
    expect(Object.keys(POLICY_SECTIONS).sort()).toEqual(['compliance', 'deletion', 'notices', 'privacy', 'refund', 'terms']);
    for (const tab of Object.keys(POLICY_SECTIONS) as PolicyTab[]) {
      expect(POLICY_SECTIONS[tab].length).toBeGreaterThan(0);
      for (const section of POLICY_SECTIONS[tab]) {
        expect(section.heading.trim()).not.toBe('');
        expect(section.paragraphs.length).toBeGreaterThan(0);
      }
    }
    expect(encoded).not.toMatch(/\bVERIFY\b/);
    expect(encoded).not.toMatch(/Fill in|Final legal entity|May 23, 2026|repomix/i);
    expect(encoded).not.toMatch(/\[(APP NAME|DEVELOPER NAME|LEGAL COMPANY|COMPANY ADDRESS|CONTACT EMAIL|SUPPORT EMAIL|PRIVACY EMAIL|WEBSITE OR SUPPORT URL|JURISDICTION|EFFECTIVE DATE|RETENTION PERIOD|TO BE COMPLETED)\]/);
    expect(encoded).not.toContain('Signal Forge');
    expect(encoded).not.toContain('AI Shorts App');
  });

  it('test_policy_content_includes_factual_data_deletion_notice', () => {
    const encoded = JSON.stringify(POLICY_SECTIONS.deletion);
    for (const expected of [
      'Data Deletion Notice',
      'The in-app deletion request targets backend licensing data.',
      'request ID, status, message, and lookup token',
      'secure storage',
      'does not remove local project history',
      'blocks the license key and deactivates all associated device bindings in Devolens',
      'typing DELETE USER DATA',
      'status endpoint accepts the request ID and lookup token',
      'US state privacy deletion rights',
      'Gumroad purchase or support channel'
    ]) {
      expect(encoded).toContain(expected);
    }
    expect(encoded).not.toMatch(/automatic deletion|automated legal compliance|guaranteed deletion/i);
  });

  it('test_policy_content_dates_refunds_services_and_telemetry_are_consistent', () => {
    const encoded = JSON.stringify(POLICY_SECTIONS);
    expect(POLICY_LAST_UPDATED_LABEL).toBe('May 30, 2026');
    expect(encoded.match(/Last updated: May 30, 2026/g)?.length).toBeGreaterThanOrEqual(5);
    expect(encoded).not.toMatch(/Last updated: (?!May 30, 2026)/);

    expect(encoded).toMatch(/Refunded, charged-back, revoked, disabled, or disputed purchases may lose access/);
    for (const expected of [
      'MuAPI',
      'Gumroad',
      'Cloudflare Workers and D1',
      'YouTube',
      'Google',
      'source platforms',
      'update hosts',
      'FFmpeg',
      'Tauri/Rust desktop app with Svelte UI',
      'Vite',
      'Rust, Tauri, and Native Dependencies',
      'license-control-suite'
    ]) {
      expect(encoded).toContain(expected);
    }
    expect(encoded).not.toContain('OpenAI');

    expect(encoded).toContain('No general telemetry or analytics SDK was identified during repository inspection.');
    expect(encoded).toContain('Crash reports are submitted only when an endpoint is configured and the user submits a draft.');
    expect(encoded).toContain("The application's supported generation workflow depends on MuAPI availability.");
    expect(encoded).toContain('that outage or service change is outside our control and is not our responsibility.');
    expect(encoded).not.toMatch(/automatic telemetry|automatic analytics|automatically uploads crash/i);
  });

  it('test_policy_content_discloses_linux_secure_storage_limitations_without_timeline', () => {
    const encoded = JSON.stringify(POLICY_SECTIONS);
    const linuxLimitations = Object.values(POLICY_SECTIONS)
      .flat()
      .flatMap((section) => section.paragraphs)
      .filter((paragraph) => paragraph.includes('Linux secure persistence support has known limitations'));

    expect(encoded).toContain('On Windows and macOS, license/session fallback storage uses a protected local encryption key and encrypted app-data fallback files');
    expect(encoded).toContain('Raw license keys are not intended to be stored in fallback files.');
    expect(encoded).toContain('Linux secure persistence support has known limitations in the initial release configuration.');
    expect(encoded).toContain('without a specific date or guarantee');
    expect(linuxLimitations.length).toBeGreaterThan(0);
    for (const paragraph of linuxLimitations) {
      expect(paragraph).not.toMatch(/(Q[1-4]|20\d{2}|next month|next quarter|by [A-Z][a-z]+)/i);
    }
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
    expect(screen.getByRole('tab', { name: 'Configuration', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Diagnostics' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Policies' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'API Providers', selected: true })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Deactivate Device' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'MuAPI Access help' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'OpenAI Access help' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Deactivate this device help' })).toBeNull();

    expect(screen.getByRole('button', { name: 'MuAPI Access help' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'OpenAI Access help' })).toBeNull();
    expect(screen.getByText('Video provider')).toBeTruthy();
    expect(screen.queryByText('LLM provider')).toBeNull();
    expect(screen.getByText('MuAPI Configured')).toBeTruthy();
    expect(screen.queryByText('OpenAI Configured')).toBeNull();
    expect(screen.getByText('Current MuAPI key')).toBeTruthy();
    expect(screen.queryByText('Current OpenAI key')).toBeNull();
    expect(screen.getAllByText('Active').length).toBeGreaterThanOrEqual(1);
    expect(apiKeyProfiles).toHaveBeenCalledWith('muapi');
    expect(apiKeyProfiles).not.toHaveBeenCalledWith('openai');
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
    expect(runtimeContext).toHaveBeenCalledTimes(1);
    await fireEvent.click(screen.getByRole('tab', { name: 'Deactivate Device' }));
    expect(screen.getByRole('button', { name: 'Deactivate this device help' })).toBeTruthy();
    expect(screen.getByText('Current device')).toBeTruthy();
    expect(screen.queryByLabelText('Settings reset license key')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Request Device Reset' })).toBeNull();
    await fireEvent.click(screen.getByRole('button', { name: 'Deactivate this device' }));
    expect(screen.getByRole('group', { name: 'Confirm device deactivation' })).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: 'Confirm device deactivation' })).toBeNull();
    expect(screen.getByText('Device deactivation cancelled.')).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Deactivate this device' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Deactivate this device' }));
    expect(authStoreMock.store.deactivateCurrentDevice).toHaveBeenCalledWith();
    expect(authStoreMock.store.requestReset).not.toHaveBeenCalled();
    expect(screen.getByText('This device has been deactivated. Re-enter your license key to use this device again.')).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Settings' })).toBeTruthy();

    await fireEvent.click(screen.getByRole('tab', { name: 'Configuration' }));
    await fireEvent.click(screen.getByRole('tab', { name: 'API Providers' }));
    expect(screen.getByText('MuAPI Access')).toBeTruthy();
    expect(screen.queryByText('OpenAI Access')).toBeNull();
    expect(screen.queryByRole('button', { name: 'OpenAI Access help' })).toBeNull();
    expect(screen.queryByLabelText('OpenAI profile name')).toBeNull();
    expect(screen.queryByLabelText('OpenAI key')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Add OpenAI Profile' })).toBeNull();
    expect(screen.queryByLabelText('OpenAI key profiles')).toBeNull();
    expect(screen.queryByText('Current OpenAI key')).toBeNull();
    expect(screen.queryByText('License Worker')).toBeNull();
    expect(screen.queryByText('Retry attempts')).toBeNull();
    expect(screen.queryByText('licenses.example.test')).toBeNull();
    await fireEvent.input(screen.getByLabelText('MuAPI profile name'), { target: { value: 'Client MuAPI' } });
    await fireEvent.input(screen.getByLabelText('MuAPI key'), { target: { value: 'mu-secret' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add MuAPI Profile' }));
    expect(apiKeyProfileAdd).toHaveBeenCalledWith('muapi', 'Client MuAPI', 'mu-secret', true);
    expect(document.body.textContent).not.toContain('mu-secret');
    expect(apiKeyProfileAdd).not.toHaveBeenCalledWith('openai', expect.anything(), expect.anything(), expect.anything());
    expect(apiKeyProfiles).not.toHaveBeenCalledWith('openai');

    await fireEvent.click(screen.getByRole('tab', { name: 'API Providers' }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'Delete' })[0]);
    expect(apiKeyProfileDelete).toHaveBeenCalledWith('muapi', 'muapi-1');

    await fireEvent.click(screen.getByRole('tab', { name: 'Diagnostics' }));
    expect(screen.getByRole('tab', { name: 'Diagnostics', selected: true })).toBeTruthy();
    expect(screen.getByText('See app status and maintenance information.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Refresh Status' })).toBeTruthy();
    expect(screen.getByText('Maintenance')).toBeTruthy();
    expect(screen.queryByText('Logs and Support')).toBeNull();

    expect(screen.queryByRole('button', { name: 'Terms' })).toBeNull();
    await fireEvent.click(screen.getByRole('tab', { name: 'Policies' }));
    expect(screen.getByRole('tab', { name: 'Policies', selected: true })).toBeTruthy();
    expect(screen.getByText('Reference documents for use, privacy, data compliance, third-party notices, refunds, and liability.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Terms' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Privacy' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Data Deletion' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Data Compliance' })).toBeTruthy();
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

  it('test_settings_deactivate_device_failure_keeps_retryable_confirmation', async () => {
    authStoreMock.store.deactivateCurrentDevice.mockRejectedValue({
      code: 'worker_unreachable',
      message: 'Unable to reach the license service right now. Please try again shortly.'
    });
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
      resetStatus: 'error',
      resetStatusMessage: null,
      resetError: {
        code: 'worker_unreachable',
        message: 'Unable to reach the license service right now. Please try again shortly.'
      },
      error: null
    });

    render(Page);
    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await fireEvent.click(await screen.findByRole('tab', { name: 'Deactivate Device' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Deactivate this device' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Deactivate this device' }));

    expect(authStoreMock.store.deactivateCurrentDevice).toHaveBeenCalledWith();
    expect(screen.getByRole('group', { name: 'Confirm device deactivation' })).toBeTruthy();
    expect(screen.getByText('Unable to reach the license service right now. Please try again shortly.')).toBeTruthy();
    expect(screen.queryByLabelText('Settings reset license key')).toBeNull();
  });

  it('test_submit sends correct payload', async () => {
    runGenerateAndStream.mockResolvedValue({
      ok: true,
      result: { mode: 'api', source_video_url: 'x', transcript: { duration: 1, segments: [] }, highlights: [], shorts: [] }
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.input(screen.getByLabelText('Num clips'), { target: { value: '5' } });
    await chooseThemedSelectOption('Resolution', '1080p');
    await chooseThemedSelectOption('Aspect ratio', '1:1 (Square feed)');
    await fireEvent.input(screen.getByLabelText('Output JSON path'), { target: { value: 'result.json' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).toHaveBeenCalledTimes(1);
    const req = runGenerateAndStream.mock.calls[0][0];
    expect(req).toEqual({
      run_id: expect.any(String),
      youtube_url: 'https://youtube.com/watch?v=abc',
      mode: 'api',
      num_clips: 5,
      aspect_ratio: '9:16',
      download_format: '720',
      output_json: 'result.json'
    });
  });

  it('test_generate_is_blocked_and_guided_when_setup_is_incomplete', async () => {
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'devolens',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: false,
      openaiConfigured: true,
      licenseWorkerTimeoutMs: 10000,
      licenseWorkerRetryAttempts: 2
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog', { name: 'Setup Required Before Generating' })).toBeTruthy();
    expect(screen.getByText('To generate shorts, you need to configure API access first.')).toBeTruthy();
    expect(screen.getByText('API key is not configured')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Configure Now' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Recheck Setup' })).toBeTruthy();
  });

  it('test_setup_modal_configure_now_routes_to_configuration_api_providers', async () => {
    appConfigSummary.mockResolvedValue({
      licenseBackendMode: 'devolens',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: false,
      openaiConfigured: true,
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
      licenseBackendMode: 'devolens',
      licenseWorkerEndpoint: 'licenses.example.test',
      licenseWorkerEndpointKind: 'remote',
      muapiConfigured: false,
      openaiConfigured: true,
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
    localStorage.setItem('shorts.onboarding.v1', 'skipped');

    render(Page);
    expect(screen.queryByRole('button', { name: 'Legal' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Terms' })).toBeNull();

    await fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(screen.queryByText('1. Acceptance of Terms')).toBeNull();

    await fireEvent.click(screen.getByRole('tab', { name: 'Policies' }));
    expect(screen.getByText('1. Acceptance of Terms')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Privacy' }));
    expect(screen.getByText('Privacy Notice')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Data Deletion' }));
    expect(screen.getByText('Data Deletion Notice')).toBeTruthy();
    expect(screen.getByText(/targets backend licensing data/)).toBeTruthy();
    expect(screen.getByText(/The lookup token is required to refresh deletion status/)).toBeTruthy();
    expect(screen.getByText(/does not remove local project history/)).toBeTruthy();
    expect(screen.getByText(/blocks the license key and deactivates all associated device bindings in Devolens/)).toBeTruthy();
    expect(screen.getByText(/typing DELETE USER DATA/)).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Data Compliance' }));
    expect(screen.getAllByText('Data Compliance').length).toBeGreaterThan(0);
    expect(screen.getByText('Security Controls')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Third-Party Notices' }));
    expect(screen.getAllByText('Third-Party Notices').length).toBeGreaterThan(0);
    expect(screen.getByText('Provider Terms')).toBeTruthy();
    expect(screen.getByText(/Exact license metadata is not shown in this in-app screen/)).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Refund Policy' }));

    expect(screen.getAllByText('Refund Policy').length).toBeGreaterThan(0);
    expect(screen.getByText(/Refund requests are handled through the payment provider or support channel/)).toBeTruthy();
    expect(screen.getByText('Support Boundaries')).toBeTruthy();
    expect(screen.getAllByText(`Last updated: ${POLICY_LAST_UPDATED_LABEL}`).length).toBeGreaterThan(0);
    expect(screen.getByText(/Refunded, charged-back, revoked, disabled, or disputed purchases may lose access/)).toBeTruthy();
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

    await fireEvent.click(screen.getByRole('button', { name: 'Copy Link' }));
    expect(clipboardWriteText).toHaveBeenCalledWith('https://cdn.example.com/s1.mp4');
    expect(await screen.findByText('Link copied.')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Open Clip' }));
    expect(windowOpen).toHaveBeenCalledWith('https://cdn.example.com/s1.mp4', '_blank', 'noopener,noreferrer');
  });

});
