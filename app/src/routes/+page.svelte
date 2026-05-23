<script>
  import { onMount } from 'svelte';
  import FormStatus from '../lib/components/FormStatus.svelte';
  import { runState } from '../lib/stores/runState';
  import { authState } from '../lib/stores/authState';
  import { openInFileManager, pickLocalVideoFile, pickOutputJsonPath, runGenerateAndStream } from '../lib/api/tauriClient';
  import {
    apiKeyProfileActivate,
    apiKeyProfileAdd,
    apiKeyProfileDelete,
    apiKeyProfiles,
    appConfigSummary,
    listenLocalModelDownloadProgress,
    localModelDownloadStatus,
    localModelProfileActivate,
    localModelProfileAdd,
    localModelProfileDelete,
    localModelProfileRetryDownload,
    localModelProfiles,
    runtimeContext,
    validateRuntime
  } from '../lib/api/runtimeClient';
  import { checkForAppUpdate, installAppUpdate } from '../lib/api/updaterClient';
  import { CRASH_DRAFT_KEY, createCrashDraft, dismissCrashDraft, saveCrashDraft } from '../support/crashDraft';
  import { POLICY_COMMON_SECTIONS, POLICY_LAST_UPDATED_LABEL, POLICY_SECTIONS } from '../lib/legal/policiesContent';
  const LS = {
    projects: 'shorts.projects.v1',
    theme: 'shorts.theme.v1'
  };
  const APP_VERSION = import.meta.env?.VITE_APP_VERSION ?? '0.1.0';
  const CRASH_REPORT_ENDPOINT = import.meta.env?.VITE_CRASH_REPORT_ENDPOINT ?? '';
  const WHISPER_MODEL_OPTIONS = [
    { value: 'tiny', label: 'Tiny - fastest, lowest accuracy' },
    { value: 'base', label: 'Base - default lightweight model' },
    { value: 'small', label: 'Small - better accuracy, still practical on CPU' },
    { value: 'medium', label: 'Medium - slower, higher accuracy' },
    { value: 'large-v3', label: 'Large v3 - best quality, heavy' },
    { value: 'large-v3-turbo', label: 'Large v3 Turbo - high quality, faster than large' },
    { value: 'tiny.en', label: 'Tiny English - fastest for English only' },
    { value: 'base.en', label: 'Base English - lightweight English only' },
    { value: 'small.en', label: 'Small English - balanced English only' },
    { value: 'medium.en', label: 'Medium English - accurate English only' }
  ];
  const WHISPER_DEVICE_OPTIONS = [
    { value: 'auto', label: 'Auto - choose CUDA when available' },
    { value: 'cpu', label: 'CPU - most compatible' },
    { value: 'cuda', label: 'CUDA - NVIDIA GPU' }
  ];

  let active = 'generate';

  let url = '';
  let sourceType = 'youtube';
  let mode = 'api';
  let numClips = 3;
  let aspectRatio = '9:16';
  let format = '720';
  let outputJson = '';
  let licenseKey = '';
  let resetLicenseKey = '';
  let termsAccepted = false;
  let showTermsModal = false;

  let projectName = '';
  let shortsSearch = '';

  let updaterStatus = 'Updater idle.';
  let updateAvailable = false;
  let updateVersion = '';
  let updaterBusy = false;
  let crashDraft = null;
  let crashStatus = '';
  let diagnosticsLastCheckedAt = '';
  let diagnosticsShowAdvanced = false;
  let diagnosticsInstallHelpFor = '';
  let settingsBusy = false;
  let settingsError = '';
  let settingsConfig = null;
  let settingsRuntime = null;
  let settingsContext = null;
  let settingsTab = 'configuration';
  let settingsConfigTab = 'local';
  let policiesTab = 'terms';
  let apiProfiles = { muapi: null, openai: null };
  let localProfiles = null;
  let localModelDownload = null;
  let localProfileLabel = '';
  let muapiProfileLabel = '';
  let muapiKeyInput = '';
  let openaiProfileLabel = '';
  let openaiKeyInput = '';
  let whisperModelInput = 'base';
  let whisperDeviceInput = 'auto';
  let settingsActionStatus = '';
  let settingsActionTarget = '';
  let settingsActionKind = 'success';
  let settingsActionBusy = false;
  let localDownloadActionStatus = '';
  let settingsResetLicenseKey = '';
  let authResetActionStatus = '';
  let authResetActionKind = 'success';
  let licenseFormStatus = '';
  let licenseFormStatusKind = 'info';
  let generateFormStatus = '';
  let generateFormStatusKind = 'info';
  let theme = 'dark';
  let mobileNavOpen = false;
  let openLocalDropdown = '';
  let setupStatus = {
    checkedAt: '',
    busy: false,
    readyApi: false,
    readyLocal: false,
    blockersApi: [],
    blockersLocal: [],
    dependencyBlockersLocal: [],
    mode: 'api'
  };
  let setupRequiredModalOpen = false;

  let projects = [];
  const localDraftStore = {
    load: async (key) => localStorage.getItem(key),
    save: async (key, value) => localStorage.setItem(key, value),
    delete: async (key) => localStorage.removeItem(key)
  };
  $: filteredProjectsWithShorts = projects
    .filter((p) => (p.shorts || []).length > 0)
    .filter((p) =>
      [p.name, ...(p.shorts || []).map((s) => s.title || ''), ...(p.shorts || []).map((s) => s.clip_url || '')]
        .join(' ')
        .toLowerCase()
        .includes(shortsSearch.toLowerCase())
    );
  $: sourceLabel = sourceType === 'local' ? 'Local video file path' : 'YouTube video URL';
  $: sourcePlaceholder =
    sourceType === 'local'
      ? '/home/user/Videos/interview.mp4'
      : 'https://www.youtube.com/watch?v=dQw4w9WgXcQ';
  $: isResetStatus =
    $authState.lifecycle === 'reset_pending' ||
    $authState.lifecycle === 'reset_approved_unbound' ||
    $authState.lifecycle === 'reset_rejected' ||
    $authState.lifecycle === 'reset_expired';
  $: canShowActivationForm =
    $authState.lifecycle !== 'checking' &&
    $authState.lifecycle !== 'reset_pending' &&
    $authState.lifecycle !== 'reset_rejected' &&
    $authState.lifecycle !== 'reset_expired';
  $: trimmedMuapiKey = muapiKeyInput.trim();
  $: trimmedOpenaiKey = openaiKeyInput.trim();
  $: trimmedLocalProfileLabel = localProfileLabel.trim();
  $: trimmedMuapiProfileLabel = muapiProfileLabel.trim();
  $: trimmedOpenaiProfileLabel = openaiProfileLabel.trim();
  $: trimmedWhisperModel = whisperModelInput.trim();
  $: trimmedWhisperDevice = whisperDeviceInput.trim();
  $: canSaveMuapiKey = Boolean(trimmedMuapiKey && trimmedMuapiProfileLabel);
  $: canSaveOpenaiKey = Boolean(trimmedOpenaiKey && trimmedOpenaiProfileLabel);
  $: canSaveLocalProcessing = Boolean(trimmedLocalProfileLabel && trimmedWhisperModel && trimmedWhisperDevice);
  $: whisperModelOptions = optionListWithCurrent(WHISPER_MODEL_OPTIONS, settingsConfig?.localWhisperModel, 'Current custom model');
  $: whisperDeviceOptions = optionListWithCurrent(WHISPER_DEVICE_OPTIONS, settingsConfig?.localWhisperDevice, 'Current custom device');
  $: activeLocalModelProfile = localProfiles?.profiles?.find((profile) => profile.active) ?? null;
  $: activeLocalModelDownloading =
    Boolean(localModelDownload?.active && localModelDownload?.profileId === activeLocalModelProfile?.id) ||
    activeLocalModelProfile?.downloadStatus === 'downloading' ||
    activeLocalModelProfile?.downloadStatus === 'queued';
  $: activeLocalModelFailed = activeLocalModelProfile?.downloadStatus === 'failed';
  $: activeLocalModelNotReady =
    Boolean(activeLocalModelProfile) &&
    !['ready', 'downloading', 'queued'].includes(activeLocalModelProfile?.downloadStatus || '');
  $: localRunBlocked = mode === 'local' && (activeLocalModelDownloading || activeLocalModelFailed || activeLocalModelNotReady);
  $: localRunBlockedMessage = activeLocalModelDownloading
    ? 'Local model is still downloading. You can use API mode now or wait for the local model to finish.'
    : 'Active local model is not ready. Retry the download from Settings or use API mode.';
  $: activeSetupBlockers = mode === 'local' ? setupStatus.blockersLocal : setupStatus.blockersApi;
  $: setupModalBlockerMessages = friendlySetupBlockers(activeSetupBlockers);

  onMount(() => {
    let unlistenLocalModel = null;
    try {
      loadState();
      loadCrashDraftFromLocalStorage();
      authState.bootstrap();
      loadLocalModelStatus();
      loadSetupStatus();
      listenLocalModelDownloadProgress((status) => {
        localModelDownload = status;
        loadLocalModelProfilesOnly();
        loadSetupStatus();
      }).then((unlisten) => {
        unlistenLocalModel = unlisten;
      }).catch(() => {
        localModelDownload = null;
      });
    } catch (_err) {
      projects = [];
      theme = 'dark';
      applyTheme(theme);
    }

    window.addEventListener('error', captureWindowError);
    window.addEventListener('unhandledrejection', captureUnhandledRejection);

    return () => {
      window.removeEventListener('error', captureWindowError);
      window.removeEventListener('unhandledrejection', captureUnhandledRejection);
      if (unlistenLocalModel) {
        unlistenLocalModel();
      }
    };
  });

  function loadState() {
    const p = localStorage.getItem(LS.projects);
    const t = localStorage.getItem(LS.theme);

    projects = p ? JSON.parse(p) : [];
    theme = t === 'light' ? 'light' : 'dark';
    applyTheme(theme);

    if (!p) persistProjects();
  }

  function persistProjects() {
    localStorage.setItem(LS.projects, JSON.stringify(projects));
  }

  function applyTheme(nextTheme) {
    document.documentElement.setAttribute('data-theme', nextTheme);
  }

  function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    localStorage.setItem(LS.theme, theme);
    applyTheme(theme);
  }

  function selectScreen(screen) {
    if (screen === 'settings') {
      settingsTab = 'configuration';
      settingsConfigTab = 'local';
      policiesTab = 'terms';
      loadSettingsStatus();
      loadCrashDraftFromLocalStorage();
    }
    active = screen;
    mobileNavOpen = false;
  }

  async function loadSettingsStatus() {
    settingsBusy = true;
    settingsError = '';
    try {
      const [config, runtime, context, muapiProfiles, openaiProfiles, modelProfiles, downloadStatus] = await Promise.all([
        appConfigSummary(),
        validateRuntime(),
        runtimeContext(),
        apiKeyProfiles('muapi'),
        apiKeyProfiles('openai'),
        localModelProfiles(),
        localModelDownloadStatus()
      ]);
      settingsConfig = config;
      settingsRuntime = runtime;
      settingsContext = context;
      apiProfiles = { muapi: muapiProfiles, openai: openaiProfiles };
      localProfiles = modelProfiles;
      localModelDownload = downloadStatus;
      whisperModelInput = config.localWhisperModel || 'base';
      whisperDeviceInput = config.localWhisperDevice || 'auto';
      diagnosticsLastCheckedAt = new Date().toISOString();
    } catch (err) {
      settingsError = err instanceof Error ? err.message : 'Unable to load settings status.';
    } finally {
      settingsBusy = false;
    }
  }

  async function loadLocalModelStatus() {
    try {
      const [modelProfiles, downloadStatus] = await Promise.all([
        localModelProfiles(),
        localModelDownloadStatus()
      ]);
      localProfiles = modelProfiles;
      localModelDownload = downloadStatus;
    } catch (_err) {
      localProfiles = null;
      localModelDownload = null;
    }
  }

  async function loadLocalModelProfilesOnly() {
    try {
      localProfiles = await localModelProfiles();
      await loadSetupStatus();
    } catch (_err) {
      // Keep the last known status; this is only a UI refresh helper.
    }
  }

  function setupBlocker(id, message, action) {
    return { id, message, action };
  }

  function toolStatusMap(runtime) {
    const map = new Map();
    for (const tool of runtime?.tools || []) {
      map.set(tool.tool, tool);
    }
    return map;
  }

  function computeSetupStatus(config, runtime, models) {
    const blockersApi = [];
    const blockersLocal = [];
    const dependencyBlockersLocal = [];

    if (!config?.muapiConfigured) {
      blockersApi.push(setupBlocker('muapi_key', 'MuAPI key is not configured.', 'Open API setup'));
    }

    const localTools = toolStatusMap(runtime);
    const python = localTools.get('python');
    const ffmpeg = localTools.get('ffmpeg');
    const ytdlp = localTools.get('yt-dlp');

    if (!config?.openaiConfigured) {
      blockersLocal.push(setupBlocker('openai_key', 'OpenAI key is not configured for local mode.', 'Open API setup'));
    }
    if (!runtime?.bridge_entry_exists) {
      blockersLocal.push(setupBlocker('bridge_entry', 'Local runtime bridge is unavailable.', 'Open diagnostics'));
    }
    if (!python?.ok) {
      const blocker = setupBlocker('python', 'Python 3 is not available for local mode.', 'Open diagnostics');
      blockersLocal.push(blocker);
      dependencyBlockersLocal.push(blocker);
    }
    if (!ffmpeg?.ok) {
      const blocker = setupBlocker('ffmpeg', 'FFmpeg is not available for local mode.', 'Open diagnostics');
      blockersLocal.push(blocker);
      dependencyBlockersLocal.push(blocker);
    }
    if (!ytdlp?.ok) {
      const blocker = setupBlocker('ytdlp', 'yt-dlp is not available for local YouTube downloads.', 'Open diagnostics');
      blockersLocal.push(blocker);
      dependencyBlockersLocal.push(blocker);
    }
    for (const pkg of runtime?.python_packages || []) {
      if (!pkg?.ok) {
        const blocker = setupBlocker(
          `py_pkg_${pkg.tool}`,
          `Python package '${pkg.tool}' is unavailable for local mode.`,
          'Open diagnostics'
        );
        blockersLocal.push(blocker);
        dependencyBlockersLocal.push(blocker);
      }
    }
    if (runtime?.bridge_entry_exists && runtime?.tools?.length && runtime?.local_runtime_ready === false) {
      blockersLocal.push(setupBlocker('bundled_runtime_incomplete', 'Bundled local runtime is incomplete.', 'Open diagnostics'));
    }

    const activeModelProfile = models?.profiles?.find((profile) => profile.active) ?? null;
    if (!activeModelProfile) {
      blockersLocal.push(setupBlocker('local_profile', 'No active local model profile is configured.', 'Open local setup'));
    } else if (activeModelProfile.downloadStatus !== 'ready') {
      blockersLocal.push(setupBlocker('local_model_not_ready', 'Active local model is not ready yet.', 'Open local setup'));
    }

    return {
      checkedAt: new Date().toISOString(),
      busy: false,
      readyApi: blockersApi.length === 0,
      readyLocal: blockersLocal.length === 0,
      blockersApi,
      blockersLocal,
      dependencyBlockersLocal,
      mode
    };
  }

  async function loadSetupStatus() {
    setupStatus = { ...setupStatus, busy: true };
    try {
      const [config, runtime, models] = await Promise.all([
        appConfigSummary(),
        validateRuntime(),
        localModelProfiles()
      ]);
      setupStatus = computeSetupStatus(config, runtime, models);
    } catch (_err) {
      setupStatus = {
        ...setupStatus,
        busy: false,
        checkedAt: new Date().toISOString(),
        readyApi: false,
        readyLocal: false,
        blockersApi: [setupBlocker('setup_unavailable', 'Unable to load setup status.', 'Recheck')],
        blockersLocal: [setupBlocker('setup_unavailable', 'Unable to load setup status.', 'Recheck')]
      };
    }
  }

  function openSetupConfiguration(target) {
    active = 'settings';
    settingsTab = 'configuration';
    if (target === 'api') {
      settingsConfigTab = 'api';
    } else {
      settingsConfigTab = 'local';
    }
    loadSettingsStatus();
  }

  function setupTargetFromBlockers(blockers) {
    if (blockers.some((blocker) => blocker.id === 'muapi_key' || blocker.id === 'openai_key')) {
      return 'api';
    }
    return 'local';
  }

  function friendlySetupBlockers(blockers) {
    const friendly = [];
    const add = (id, message) => {
      if (!friendly.some((item) => item.id === id)) {
        friendly.push({ id, message });
      }
    };
    for (const blocker of blockers || []) {
      if (blocker.id === 'muapi_key' || blocker.id === 'openai_key') {
        add('api_key', 'API key is not configured');
      } else if (blocker.id === 'local_profile' || blocker.id === 'local_model_not_ready') {
        add('local_model', 'Local model is not selected or downloaded');
      } else if (blocker.id === 'ffmpeg') {
        add('ffmpeg', 'FFmpeg is not available');
      } else if (blocker.id === 'python') {
        add('python', 'Python 3 is not available for local processing');
      } else if (blocker.id === 'ytdlp') {
        add('ytdlp', 'yt-dlp is not available for local YouTube downloads');
      } else if (blocker.id === 'bridge_entry') {
        add('bridge_entry', 'Local runtime bridge is unavailable');
      } else if (blocker.id === 'bundled_runtime_incomplete') {
        add('bundled_runtime_incomplete', 'Bundled local runtime is missing or incomplete');
      } else if (blocker.id?.startsWith('py_pkg_')) {
        add(blocker.id, blocker.message.replace(/\.$/, ''));
      } else if (blocker.message) {
        add(blocker.id || `blocker-${friendly.length}`, blocker.message.replace(/\.$/, ''));
      }
    }
    return friendly;
  }

  function closeSetupRequiredModal() {
    setupRequiredModalOpen = false;
  }

  function handleSetupConfigureNow() {
    const target = setupTargetFromBlockers(activeSetupBlockers);
    setupRequiredModalOpen = false;
    openSetupConfiguration(target);
  }

  async function recheckSetupFromModal() {
    await loadSetupStatus();
    if ((mode === 'local' ? setupStatus.blockersLocal : setupStatus.blockersApi).length === 0) {
      setupRequiredModalOpen = false;
    }
  }

  async function ensureSetupForRun() {
    await loadSetupStatus();
    const blockers = mode === 'local' ? setupStatus.blockersLocal : setupStatus.blockersApi;
    if (blockers.length === 0) {
      return true;
    }
    setupRequiredModalOpen = true;
    return false;
  }

  function configuredLabel(value) {
    return value ? 'Configured' : 'Not configured';
  }

  async function submitSettingsResetRequest() {
    if (settingsActionBusy) {
      return;
    }
    const key = settingsResetLicenseKey.trim();
    if (!key) {
      settingsActionStatus = 'Enter your license key to request a device reset.';
      settingsActionTarget = 'reset';
      settingsActionKind = 'error';
      return;
    }
    settingsActionStatus = '';
    settingsActionTarget = '';
    settingsActionKind = 'success';
    settingsActionBusy = true;
    try {
      await authState.requestReset({ license_key: key }, { preserveLicensedSession: true });
      settingsResetLicenseKey = '';
      settingsActionStatus = 'Device reset request sent.';
      settingsActionTarget = 'reset';
      settingsActionKind = 'success';
    } catch (err) {
      // The auth store surfaces safe messages via `$authState.resetError`.
      // Avoid duplicating the same error message in both places.
      settingsActionStatus = '';
      settingsActionTarget = 'reset';
      settingsActionKind = 'error';
    } finally {
      settingsActionBusy = false;
    }
  }

  async function saveMuapiKey() {
    if (settingsActionBusy) {
      return;
    }
    if (!canSaveMuapiKey) {
      settingsActionStatus = 'Enter a profile name and MuAPI key before continuing.';
      settingsActionTarget = 'muapi';
      settingsActionKind = 'error';
      return;
    }
    settingsActionStatus = '';
    settingsActionTarget = '';
    settingsActionKind = 'success';
    settingsActionBusy = true;
    try {
      apiProfiles = {
        ...apiProfiles,
        muapi: await apiKeyProfileAdd('muapi', trimmedMuapiProfileLabel, trimmedMuapiKey, true)
      };
      muapiKeyInput = '';
      muapiProfileLabel = '';
      settingsActionStatus = 'MuAPI profile saved and set active.';
      settingsActionTarget = 'muapi';
      settingsActionKind = 'success';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function saveOpenaiKey() {
    if (settingsActionBusy) {
      return;
    }
    if (!canSaveOpenaiKey) {
      settingsActionStatus = 'Enter a profile name and OpenAI key before continuing.';
      settingsActionTarget = 'openai';
      settingsActionKind = 'error';
      return;
    }
    settingsActionStatus = '';
    settingsActionTarget = '';
    settingsActionKind = 'success';
    settingsActionBusy = true;
    try {
      apiProfiles = {
        ...apiProfiles,
        openai: await apiKeyProfileAdd('openai', trimmedOpenaiProfileLabel, trimmedOpenaiKey, true)
      };
      openaiKeyInput = '';
      openaiProfileLabel = '';
      settingsActionStatus = 'OpenAI profile saved and set active.';
      settingsActionTarget = 'openai';
      settingsActionKind = 'success';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function activateApiProfile(provider, profileId) {
    if (settingsActionBusy) {
      return;
    }
    settingsActionStatus = '';
    settingsActionTarget = '';
    settingsActionKind = 'success';
    settingsActionBusy = true;
    try {
      apiProfiles = {
        ...apiProfiles,
        [provider]: await apiKeyProfileActivate(provider, profileId)
      };
      settingsActionStatus = `${provider === 'muapi' ? 'MuAPI' : 'OpenAI'} active profile updated.`;
      settingsActionTarget = provider;
      settingsActionKind = 'success';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function deleteApiProfile(provider, profileId) {
    if (settingsActionBusy) {
      return;
    }
    settingsActionStatus = '';
    settingsActionTarget = '';
    settingsActionKind = 'success';
    settingsActionBusy = true;
    try {
      apiProfiles = {
        ...apiProfiles,
        [provider]: await apiKeyProfileDelete(provider, profileId)
      };
      settingsActionStatus = `${provider === 'muapi' ? 'MuAPI' : 'OpenAI'} profile deleted.`;
      settingsActionTarget = provider;
      settingsActionKind = 'success';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function saveLocalProcessing() {
    if (settingsActionBusy) {
      return;
    }
    if (!canSaveLocalProcessing) {
      settingsActionStatus = 'Enter a profile name, model, and processing device before continuing.';
      settingsActionTarget = 'local';
      settingsActionKind = 'error';
      return;
    }
    settingsActionStatus = '';
    settingsActionTarget = '';
    settingsActionKind = 'success';
    settingsActionBusy = true;
    try {
      localProfiles = await localModelProfileAdd(
        trimmedLocalProfileLabel,
        trimmedWhisperModel,
        trimmedWhisperDevice,
        true
      );
      localProfileLabel = '';
      settingsActionStatus = 'Local model profile saved. Download started in the background.';
      settingsActionTarget = 'local';
      settingsActionKind = 'success';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function activateLocalModelProfile(profileId) {
    if (settingsActionBusy) {
      return;
    }
    settingsActionBusy = true;
    settingsActionTarget = 'local';
    settingsActionKind = 'success';
    settingsActionStatus = '';
    try {
      localProfiles = await localModelProfileActivate(profileId);
      settingsActionStatus = 'Active local model profile updated.';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function retryLocalModelDownload(profileId) {
    if (settingsActionBusy) {
      return;
    }
    settingsActionBusy = true;
    settingsActionTarget = 'local';
    settingsActionKind = 'success';
    settingsActionStatus = '';
    try {
      localProfiles = await localModelProfileRetryDownload(profileId);
      settingsActionStatus = 'Local model download restarted.';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  async function recheckLocalSetup() {
    await loadSettingsStatus();
    await loadSetupStatus();
    localDownloadActionStatus = 'Setup rechecked.';
  }

  async function openLocalDownloadLog() {
    const path = settingsContext?.logPath;
    if (!path) {
      localDownloadActionStatus = 'Log path is unavailable.';
      return;
    }
    try {
      await openInFileManager(path);
      localDownloadActionStatus = 'Opened log folder.';
    } catch (_err) {
      localDownloadActionStatus = 'Unable to open log folder.';
    }
  }

  async function copyLocalDownloadDetails() {
    const errorCode = localModelDownload?.errorCode || activeLocalModelProfile?.errorCode || 'unknown';
    const debugRef = localModelDownload?.debugRef || activeLocalModelProfile?.debugRef || 'n/a';
    const details = [
      `model=${localModelDownload?.model || activeLocalModelProfile?.model || 'n/a'}`,
      `device=${localModelDownload?.device || activeLocalModelProfile?.device || 'n/a'}`,
      `error_code=${errorCode}`,
      `debug_ref=${debugRef}`,
      `message=${localModelDownload?.error || activeLocalModelProfile?.error || 'n/a'}`,
      `log_path=${settingsContext?.logPath || 'n/a'}`
    ].join('\n');
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(details);
      } else {
        throw new Error('clipboard unavailable');
      }
      localDownloadActionStatus = 'Error details copied.';
    } catch (_err) {
      localDownloadActionStatus = 'Unable to copy details automatically.';
    }
  }

  async function deleteLocalModelProfile(profileId) {
    if (settingsActionBusy) {
      return;
    }
    settingsActionBusy = true;
    settingsActionTarget = 'local';
    settingsActionKind = 'success';
    settingsActionStatus = '';
    try {
      localProfiles = await localModelProfileDelete(profileId);
      settingsActionStatus = 'Local model profile deleted.';
    } finally {
      settingsActionBusy = false;
    }
    await loadSettingsStatus();
  }

  function localModelStatusLabel(status) {
    const normalized = status || 'not_downloaded';
    if (normalized === 'ready') return 'Ready';
    if (normalized === 'downloading') return 'Downloading';
    if (normalized === 'queued') return 'Queued';
    if (normalized === 'failed') return 'Failed';
    return 'Not downloaded';
  }

  function optionListWithCurrent(options, currentValue, currentLabel) {
    const current = (currentValue || '').trim();
    if (!current || options.some((option) => option.value === current)) {
      return options;
    }
    return [{ value: current, label: currentLabel }, ...options];
  }

  function optionLabel(options, value) {
    return options.find((option) => option.value === value)?.label ?? value;
  }

  function toggleLocalDropdown(name) {
    openLocalDropdown = openLocalDropdown === name ? '' : name;
  }

  function chooseLocalOption(name, value) {
    if (name === 'model') {
      whisperModelInput = value;
    } else {
      whisperDeviceInput = value;
    }
    openLocalDropdown = '';
  }

  function closeLocalDropdown() {
    openLocalDropdown = '';
  }

  function closeLocalDropdownOnEscape(event) {
    if (event.key === 'Escape') {
      openLocalDropdown = '';
    }
  }

  function formatLastChecked(timestamp) {
    if (!timestamp) {
      return 'Not checked yet';
    }
    const date = new Date(timestamp);
    if (Number.isNaN(date.getTime())) {
      return 'Not checked yet';
    }
    return date.toLocaleString();
  }

  function toolDisplayName(tool) {
    if (tool === 'python') {
      return 'Python 3';
    }
    if (tool === 'ffmpeg') {
      return 'FFmpeg';
    }
    return 'yt-dlp';
  }

  function toolPurpose(tool) {
    if (tool === 'python') {
      return 'Runs local processing bridge';
    }
    if (tool === 'ffmpeg') {
      return 'Cuts and exports clips';
    }
    return 'Downloads source videos';
  }

  function installInstructions(tool) {
    if (tool === 'python') {
      return 'Install Python 3 and ensure `python3` is available in your PATH.';
    }
    if (tool === 'ffmpeg') {
      return 'Install FFmpeg and ensure `ffmpeg` is available in your PATH.';
    }
    return 'Install yt-dlp and ensure `yt-dlp` is available in your PATH.';
  }

  function platformLabel() {
    return navigator.platform || 'unknown';
  }

  function loadCrashDraftFromLocalStorage() {
    const raw = localStorage.getItem(CRASH_DRAFT_KEY);
    if (!raw) {
      crashDraft = null;
      return;
    }

    try {
      crashDraft = JSON.parse(raw);
    } catch (_err) {
      localStorage.removeItem(CRASH_DRAFT_KEY);
      crashDraft = null;
    }
  }

  function captureWindowError(event) {
    const draft = createCrashDraft(event.error ?? event.message, {
      appVersion: APP_VERSION,
      platform: platformLabel()
    });
    saveCrashDraft(localDraftStore, draft);
  }

  function captureUnhandledRejection(event) {
    const draft = createCrashDraft(event.reason ?? 'Unhandled promise rejection', {
      appVersion: APP_VERSION,
      platform: platformLabel()
    });
    saveCrashDraft(localDraftStore, draft);
  }

  async function dismissPendingCrashDraft() {
    await dismissCrashDraft(localDraftStore);
    crashDraft = null;
    crashStatus = '';
  }

  async function submitPendingCrashDraft() {
    if (!crashDraft) {
      return;
    }
    if (!CRASH_REPORT_ENDPOINT) {
      crashStatus = 'Crash report endpoint is not configured. No report was sent.';
      return;
    }

    try {
      const response = await fetch(CRASH_REPORT_ENDPOINT, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(crashDraft)
      });
      if (!response.ok) {
        crashStatus = 'Crash report submission failed. You can dismiss this draft or try again later.';
        return;
      }
      await dismissPendingCrashDraft();
    } catch (_err) {
      crashStatus = 'Crash report submission failed. You can dismiss this draft or try again later.';
    }
  }

  async function checkForUpdates() {
    updaterBusy = true;
    updaterStatus = 'Checking for updates...';
    try {
      const result = await checkForAppUpdate();
      updateAvailable = result.available;
      updateVersion = result.available ? result.update.version : '';
      updaterStatus = result.available
        ? `Update ${result.update.version} is available.`
        : 'Signal Forge is up to date.';
    } catch (err) {
      updateAvailable = false;
      updateVersion = '';
      updaterStatus = err instanceof Error ? err.message : 'Updater is unavailable.';
    } finally {
      updaterBusy = false;
    }
  }

  async function installUpdate() {
    updaterBusy = true;
    updaterStatus = updateVersion ? `Installing update ${updateVersion}...` : 'Installing update...';
    try {
      const result = await installAppUpdate();
      if (result.installed) {
        updaterStatus = `Update ${result.version} installed. Restart the app to finish.`;
        updateAvailable = false;
        updateVersion = '';
      } else {
        updaterStatus = result.message || 'No update is available to install.';
      }
    } catch (err) {
      updaterStatus = err instanceof Error ? err.message : 'Update installation failed.';
    } finally {
      updaterBusy = false;
    }
  }

  async function chooseLocalFile() {
    const picked = await pickLocalVideoFile();
    if (picked) {
      sourceType = 'local';
      mode = 'local';
      url = picked;
    }
  }

  async function submitLicense() {
    const key = licenseKey.trim();
    if (!key) {
      licenseFormStatus = 'Enter your license key to continue.';
      licenseFormStatusKind = 'error';
      return;
    }
    if (!termsAccepted) {
      licenseFormStatus = 'You must accept the Terms and Conditions to continue.';
      licenseFormStatusKind = 'error';
      return;
    }
    licenseFormStatus = '';
    licenseFormStatusKind = 'info';
    await authState.activate(key);
    licenseKey = '';
  }

  async function submitResetRequest() {
    const key = resetLicenseKey.trim();
    if (!key) {
      authResetActionStatus = 'Enter your license key to request a device reset.';
      authResetActionKind = 'error';
      return;
    }
    authResetActionStatus = '';
    authResetActionKind = 'success';
    await authState.requestReset({ license_key: key });
    resetLicenseKey = '';
  }

  async function refreshResetStatus() {
    if ($authState.resetRequestId) {
      await authState.pollResetStatus($authState.resetRequestId);
    }
  }

  async function chooseOutputJsonPath() {
    const picked = await pickOutputJsonPath();
    if (picked) {
      outputJson = picked;
    }
  }

  async function openClipFolder(path) {
    if (!path || typeof path !== 'string' || path.startsWith('http')) {
      return;
    }
    try {
      await openInFileManager(path);
    } catch (_e) {
      // no-op for now
    }
  }

  function removeProject(projectId) {
    projects = projects.filter((p) => p.id !== projectId);
    persistProjects();
  }

  function clearShortsLibrary() {
    projects = [];
    persistProjects();
  }

  async function submitRun() {
    const trimmedUrl = url.trim();
    if (!trimmedUrl) {
      generateFormStatus = sourceType === 'local'
        ? 'Select a local video file path before running.'
        : 'Enter a YouTube video URL before running.';
      generateFormStatusKind = 'error';
      return;
    }

    const normalizedNumClips = Number(numClips);
    if (!Number.isFinite(normalizedNumClips) || normalizedNumClips < 1) {
      generateFormStatus = 'Num clips must be at least 1.';
      generateFormStatusKind = 'error';
      return;
    }

    generateFormStatus = '';
    generateFormStatusKind = 'info';
    if (sourceType === 'local') {
      mode = 'local';
    }
    if (mode === 'local' && localRunBlocked) {
      runState.onError({
        error: localRunBlockedMessage,
        mode,
        source_video_url: url,
        details: { stage: 'local_model_download' }
      });
      return;
    }
    const setupReady = await ensureSetupForRun();
    if (!setupReady) {
      return;
    }
    runState.start();

    try {
      const envelope = await runGenerateAndStream(
        {
          youtube_url: trimmedUrl,
          mode,
          num_clips: normalizedNumClips,
          aspect_ratio: aspectRatio,
          download_format: format,
          output_json: outputJson.trim() || undefined
        },
        (event) => runState.onProgress(event)
      );

      const status = envelope.ok ? 'exported' : 'draft';
      const existing = projects.find((p) => p.name.toLowerCase() === projectName.trim().toLowerCase());
      const next = {
        id: existing?.id ?? crypto.randomUUID(),
        name: projectName.trim() || 'Untitled Project',
        status,
        updatedAt: new Date().toISOString(),
        sourceUrl: trimmedUrl,
        clipCount: normalizedNumClips,
        shorts: envelope.ok ? envelope.result.shorts : existing?.shorts || []
      };

      projects = [next, ...projects.filter((p) => p.id !== next.id)];
      persistProjects();

      if (envelope.ok) {
        runState.onSuccess(envelope.result);
      } else {
        runState.onError(envelope.error);
      }
    } catch (e) {
      runState.onError({
        error: e instanceof Error ? e.message : 'unknown error',
        mode,
        source_video_url: trimmedUrl
      });
    }
  }
</script>

<svelte:body on:click|capture={closeLocalDropdown} />
<svelte:window on:keydown={closeLocalDropdownOnEscape} />

<div class="global-theme-toggle">
  <button
    type="button"
    class="theme-switch"
    role="switch"
    aria-label="Toggle theme"
    aria-checked={theme === 'dark'}
    on:click={toggleTheme}
  >
    <span class="theme-switch-icon" aria-hidden="true">{theme === 'dark' ? '🌙' : '☀️'}</span>
    <span class="theme-switch-track" aria-hidden="true">
      <span class="theme-switch-thumb"></span>
    </span>
  </button>
</div>

{#if $authState.lifecycle !== 'licensed' && $authState.lifecycle !== 'licensed_offline_grace'}
  <main class="license-shell">
    <div class="orb orb-a"></div>
    <div class="orb orb-b"></div>

    <section class="panel license-card" aria-labelledby="license-title">
      <div class="license-brand">
        <p class="brand-mark">Signal Forge</p>
      </div>

      <div class="license-copy">
        <h1 id="license-title">License Required</h1>
        <p class="meta">Enter your Gumroad license key to unlock this device.</p>
      </div>

      {#if $authState.lifecycle === 'checking'}
        <div class="status auth-status">
          <p class="status-line">Checking license...</p>
        </div>
      {/if}

      {#if isResetStatus}
        <div class="auth-status">
          <h2>Device Reset</h2>
          <p class="meta">Status: {$authState.lifecycle.replaceAll('_', ' ')}</p>
          {#if $authState.resetRequestId}
            <p class="meta">Request: {$authState.resetRequestId}</p>
          {/if}
          {#if $authState.lifecycle === 'reset_approved_unbound'}
            <p>Device reset complete.</p>
            <p class="meta">This license is now unbound. You can:</p>
            <ul class="meta">
              <li>Activate again on this device using your license key.</li>
              <li>Or activate on a different device if you’re moving.</li>
            </ul>
          {/if}
          {#if $authState.lifecycle === 'reset_pending'}
            <button type="button" on:click={refreshResetStatus}>Refresh Reset Status</button>
          {/if}
        </div>
      {/if}

      {#if canShowActivationForm}
        <form class="form license-form" novalidate on:submit|preventDefault={submitLicense}>
          <label>License key <input aria-label="License key" type="password" bind:value={licenseKey} autocomplete="off" spellcheck="false" /></label>
          <button type="submit" disabled={$authState.lifecycle === 'activating'}>
            {$authState.lifecycle === 'activating' ? 'Activating...' : 'Activate'}
          </button>
          <label class="terms-accept-row">
            <input aria-label="Accept terms and conditions" type="checkbox" bind:checked={termsAccepted} />
            <span class="terms-copy">
              By logging in, I accept the
              <button class="inline-link" type="button" on:click={() => (showTermsModal = true)}>Terms and Conditions</button>.
            </span>
          </label>
          <FormStatus message={licenseFormStatus} kind={licenseFormStatusKind} />
        </form>
      {/if}

      {#if $authState.lifecycle === 'reauth_required'}
        <p class="meta">{$authState.reauthMessage || 'Session expired. Re-enter your license key to continue.'}</p>
      {/if}

      {#if $authState.error}
        <p class="meta error-text">{$authState.error.message}</p>
      {/if}

      {#if $authState.lifecycle === 'device_bound_elsewhere'}
        <div class="reset-box">
          <h2>Request Device Reset</h2>
          <form class="form reset-form" novalidate on:submit|preventDefault={submitResetRequest}>
            <label>
              License key
              <input
                aria-label="Reset license key"
                type="password"
                bind:value={resetLicenseKey}
                autocomplete="off"
                spellcheck="false"
              />
            </label>
            <button type="submit">Request Reset</button>
          </form>
          <FormStatus message={authResetActionStatus} kind={authResetActionKind} />
        </div>
      {/if}
    </section>

    {#if showTermsModal}
      <div class="policy-modal-backdrop" role="presentation" on:click|self={() => (showTermsModal = false)}>
        <section
          class="panel policy-modal"
          role="dialog"
          aria-modal="true"
          aria-labelledby="terms-modal-title"
        >
          <div class="policy-modal-head">
            <h2 id="terms-modal-title">Terms and Conditions</h2>
            <button type="button" class="button-secondary" on:click={() => (showTermsModal = false)}>Close</button>
          </div>
          <div class="policy-modal-content">
            {#each POLICY_SECTIONS.terms as section}
              <h3>{section.heading}</h3>
              {#each section.paragraphs as paragraph}
                <p>{paragraph}</p>
              {/each}
            {/each}
            {#each POLICY_COMMON_SECTIONS as section}
              <h3>{section.heading}</h3>
              {#each section.paragraphs as paragraph}
                <p>{paragraph}</p>
              {/each}
            {/each}
            <p class="meta">Last updated: {POLICY_LAST_UPDATED_LABEL}</p>
          </div>
        </section>
      </div>
    {/if}
  </main>
{:else}
  <main class="app-shell">
    <div class="orb orb-a"></div>
    <div class="orb orb-b"></div>

    <aside class="sidebar panel">
      <div class="sidebar-head">
        <h1>Signal Forge</h1>
        <button
          type="button"
          class="menu-toggle"
          aria-label="Toggle navigation"
          aria-expanded={mobileNavOpen}
          on:click={() => (mobileNavOpen = !mobileNavOpen)}
        >
          Menu
        </button>
      </div>
      <nav class:nav-open={mobileNavOpen}>
        <button class:active={active === 'generate'} on:click={() => selectScreen('generate')}>Generate</button>
        <button class:active={active === 'library'} on:click={() => selectScreen('library')}>Shorts Library</button>
        <button class:active={active === 'settings'} on:click={() => selectScreen('settings')}>Settings</button>
      </nav>
    </aside>

    <section class="content">
      {#if localModelDownload && (localModelDownload.active || localModelDownload.phase === 'failed' || localModelDownload.phase === 'ready')}
        <section class="panel local-download-banner" aria-live="polite">
          <div>
            <p class="status-line">
              Local model: {localModelDownload.message}
              {#if localModelDownload.phase === 'failed' && localModelDownload.error}
                <span class="error-text">{localModelDownload.error}</span>
              {/if}
            </p>
            {#if localModelDownload.phase === 'failed'}
              <div class="row">
                <button type="button" on:click={() => retryLocalModelDownload(localModelDownload.profileId)} disabled={settingsActionBusy || !localModelDownload.profileId}>Try Again</button>
                <button type="button" class="button-secondary" on:click={recheckLocalSetup}>Recheck Setup</button>
                <button type="button" class="button-secondary" on:click={openLocalDownloadLog}>Open Logs</button>
                <button type="button" class="button-secondary" on:click={copyLocalDownloadDetails}>Copy Error Details</button>
              </div>
              {#if localDownloadActionStatus}
                <p class="meta">{localDownloadActionStatus}</p>
              {/if}
            {/if}
            <div class="meter">
              <span style={`width:${Math.max(0, Math.min(100, Math.round((localModelDownload.progress || 0) * 100)))}%`}></span>
            </div>
          </div>
        </section>
      {/if}

      {#if active === 'generate'}
      <section class="screen-header">
        <h2 class="screen-title">Generate Shorts</h2>
        <p class="meta">Create and export short clips from YouTube URLs or local videos.</p>
      </section>

      <section class="panel">
        <form class="form" novalidate on:submit|preventDefault={submitRun}>
          <label>Project title <input aria-label="Project title" bind:value={projectName} placeholder="My Product Launch Highlights" /></label>
          <label>Source type
            <div class="select-wrap">
              <select aria-label="Source type" bind:value={sourceType}>
                <option value="youtube">YouTube URL</option>
                <option value="local">Local video file</option>
              </select>
            </div>
          </label>
          <label>{sourceLabel} <input aria-label="YouTube video URL" bind:value={url} placeholder={sourcePlaceholder} /></label>
          {#if sourceType === 'local'}
            <div class="row picker-row">
              <button type="button" on:click={chooseLocalFile}>Choose File</button>
            </div>
          {/if}
          <label>Mode
            <div class="select-wrap">
              <select aria-label="Mode" bind:value={mode}>
                <option value="api">api</option>
                <option value="local">local</option>
              </select>
            </div>
          </label>
          <label>Num clips <input aria-label="Num clips" type="number" bind:value={numClips} /></label>
          <label>Aspect ratio
            <div class="select-wrap">
              <select
                aria-label="Aspect ratio"
                bind:value={aspectRatio}
                on:input={(event) => (aspectRatio = event.currentTarget.value)}
              >
                <option value="9:16">9:16 (Shorts/Reels/TikTok)</option>
                <option value="1:1">1:1 (Square feed)</option>
                <option value="4:5">4:5 (Instagram portrait)</option>
                <option value="16:9">16:9 (YouTube landscape)</option>
                <option value="3:4">3:4 (Portrait classic)</option>
              </select>
            </div>
          </label>
          <label>Resolution
            <div class="select-wrap">
              <select
                aria-label="Resolution"
                bind:value={format}
                on:input={(event) => (format = event.currentTarget.value)}
              >
                <option value="360">360p</option>
                <option value="480">480p</option>
                <option value="720">720p</option>
                <option value="1080">1080p</option>
                <option value="1440">1440p</option>
                <option value="2160">4K (2160p)</option>
              </select>
            </div>
          </label>
          <details class="advanced">
            <summary>Advanced</summary>
            <label>Save detailed report to file (optional) <input aria-label="Output JSON path" bind:value={outputJson} /></label>
            <div class="row advanced-actions">
              <button type="button" on:click={chooseOutputJsonPath}>Choose Save Location</button>
            </div>
          </details>
          <div class="form-action-row">
            <button type="submit" disabled={localRunBlocked}>Run</button>
          </div>
          {#if localRunBlocked}
            <p class="meta warn-text form-full">{localRunBlockedMessage}</p>
          {/if}
          <div class="form-full">
            <FormStatus message={generateFormStatus} kind={generateFormStatusKind} />
          </div>
        </form>
      </section>

      {#if setupRequiredModalOpen}
        <div class="policy-modal-backdrop" role="presentation" on:click|self={closeSetupRequiredModal}>
          <section
            class="panel policy-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="setup-required-modal-title"
          >
            <div class="policy-modal-head">
              <h2 id="setup-required-modal-title">Setup Required Before Generating</h2>
            </div>
            <div class="policy-modal-content">
              <p>To generate shorts, you need to configure either an API-based setup or a local model first.</p>
              {#if setupModalBlockerMessages.length > 0}
                <ul class="meta">
                  {#each setupModalBlockerMessages as blocker}
                    <li>{blocker.message}</li>
                  {/each}
                </ul>
              {/if}
              <div class="row">
                <button type="button" on:click={handleSetupConfigureNow}>Configure Now</button>
                <button type="button" class="button-secondary" on:click={closeSetupRequiredModal}>Cancel</button>
                <button type="button" class="button-secondary" on:click={recheckSetupFromModal} disabled={setupStatus.busy}>
                  {setupStatus.busy ? 'Checking...' : 'Recheck Setup'}
                </button>
              </div>
            </div>
          </section>
        </div>
      {/if}

      {#if $runState.lifecycle === 'running'}
        <section class="panel status">
          <p class="status-line">Running: {$runState.progress.stage} ({Math.round($runState.progress.value * 100)}%)</p>
          <div class="meter">
            <span style={`width:${Math.max(0, Math.min(100, Math.round($runState.progress.value * 100)))}%`}></span>
          </div>
        </section>
      {/if}

      {#if $runState.error}
        <section class="panel error">
          <h3>Error</h3>
          <p>{$runState.error.error}</p>
        </section>
      {/if}

      {#if $runState.result}
        <section class="panel results">
          <h3>Result</h3>
          <p class="meta">Highlights: {$runState.result.highlights.length} -> kept {$runState.result.shorts.length}</p>
          <div class="cards">
            {#each $runState.result.shorts as s, i}
              <article>
                <h4>{s.title}</h4>
                <p>#{i + 1} score={s.score} {s.start_time}s -> {s.end_time}s</p>
                <p>hook: {s.hook_sentence}</p>
                {#if s.clip_url}
                  <p>clip: {s.clip_url}</p>
                {:else}
                  <p>clip: FAILED ({s.error})</p>
                {/if}
              </article>
            {/each}
          </div>
        </section>
      {/if}
    {/if}

    {#if active === 'library'}
      <section class="screen-header">
        <h2 class="screen-title">Shorts Library</h2>
        <p class="meta">Open Folder is available for locally generated shorts.</p>
      </section>

      <section class="panel">
        <div class="toolbar">
          <input bind:value={shortsSearch} placeholder="Search by project, short title, or clip path" />
          <div class="row">
            <button type="button" on:click={clearShortsLibrary}>Clear All</button>
          </div>
        </div>

        {#if filteredProjectsWithShorts.length === 0}
          <p class="meta">No shorts yet. Run generation and completed clips will appear here.</p>
        {:else}
          <div class="list">
            {#each filteredProjectsWithShorts as project}
              <article class="list-item">
                <div>
                  <h3>{project.name}</h3>
                  <p class="meta">{(project.shorts || []).length} shorts | {new Date(project.updatedAt).toLocaleString()}</p>
                  <div class="list">
                    {#each project.shorts || [] as short}
                      <div class="row">
                        <span>{short.title}</span>
                        {#if short.clip_url}
                          <button type="button" on:click={() => openClipFolder(short.clip_url)}>Open Folder</button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
                <button type="button" on:click={() => removeProject(project.id)}>Delete</button>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    {/if}

    {#if active === 'settings'}
      <section class="screen-header">
        <h2 class="screen-title">Settings</h2>
      </section>

      <section class="panel settings-nav-panel">
        <div class="settings-tabs" role="tablist" aria-label="Settings sections">
          <button
            id="settings-tab-configuration"
            type="button"
            role="tab"
            aria-selected={settingsTab === 'configuration'}
            aria-controls="settings-panel-configuration"
            class:active-tab={settingsTab === 'configuration'}
            on:click={() => {
              settingsTab = 'configuration';
              settingsConfigTab = 'local';
            }}
          >Configuration</button>
          <button
            id="settings-tab-diagnostics"
            type="button"
            role="tab"
            aria-selected={settingsTab === 'diagnostics'}
            aria-controls="settings-panel-diagnostics"
            class:active-tab={settingsTab === 'diagnostics'}
            on:click={() => (settingsTab = 'diagnostics')}
          >Diagnostics</button>
          <button
            id="settings-tab-policies"
            type="button"
            role="tab"
            aria-selected={settingsTab === 'policies'}
            aria-controls="settings-panel-policies"
            class:active-tab={settingsTab === 'policies'}
            on:click={() => (settingsTab = 'policies')}
          >Policies</button>
        </div>
        {#if settingsError}
          <p class="meta error-text">{settingsError}</p>
        {/if}
      </section>

      {#if settingsTab === 'configuration'}
        <div
          id="settings-panel-configuration"
          class="configuration-panel"
          role="tabpanel"
          aria-labelledby="settings-tab-configuration"
        >
          <div class="config-subtabs" role="tablist" aria-label="Configuration sections">
            <button
              type="button"
              role="tab"
              aria-selected={settingsConfigTab === 'local'}
              class:active-tab={settingsConfigTab === 'local'}
              on:click={() => (settingsConfigTab = 'local')}
            >Local Processing</button>
            <button
              type="button"
              role="tab"
              aria-selected={settingsConfigTab === 'api'}
              class:active-tab={settingsConfigTab === 'api'}
              on:click={() => (settingsConfigTab = 'api')}
            >API Providers</button>
            <button
              type="button"
              role="tab"
              aria-selected={settingsConfigTab === 'reset'}
              class:active-tab={settingsConfigTab === 'reset'}
              on:click={() => (settingsConfigTab = 'reset')}
            >Device Reset</button>
          </div>

          <div class="configuration-grid" class:configuration-grid-single={settingsConfigTab !== 'api'}>
          {#if settingsConfigTab === 'api'}
          <article class="panel config-card">
            <div class="config-card-head">
              <div>
                <p class="eyebrow">Video provider</p>
                <div class="config-title-row">
                  <h3>MuAPI Access</h3>
                  <span class="help-wrap">
                    <button class="help-button" type="button" aria-label="MuAPI Access help" aria-describedby="help-muapi">?</button>
                    <span id="help-muapi" class="help-tooltip" role="tooltip">Store the MuAPI key used for hosted video processing.</span>
                  </span>
                </div>
              </div>
              <div class="status-chips" aria-label="MuAPI key status">
                <span class:ok={settingsConfig?.muapiConfigured} class:warn={!settingsConfig?.muapiConfigured}>MuAPI {configuredLabel(settingsConfig?.muapiConfigured)}</span>
              </div>
            </div>
            {#if apiProfiles.muapi?.envOverride}
              <p class="meta warn-text">Environment variable override is active. Saved profiles are available, but runtime will use the environment key.</p>
            {/if}
            <form class="form config-form api-profile-form" novalidate on:submit|preventDefault={saveMuapiKey}>
              <label>Profile name <input aria-label="MuAPI profile name" autocomplete="off" bind:value={muapiProfileLabel} placeholder="Personal MuAPI" /></label>
              <label>MuAPI key <input aria-label="MuAPI key" type="password" autocomplete="off" bind:value={muapiKeyInput} /></label>
              <div class="settings-actions">
                <button type="submit" disabled={settingsActionBusy || !canSaveMuapiKey}>Add MuAPI Profile</button>
              </div>
            </form>
            <div class="api-profile-list" aria-label="MuAPI key profiles">
              {#if apiProfiles.muapi?.profiles?.length}
                {#each apiProfiles.muapi.profiles as profile}
                  <div class="api-profile-row">
                    <div>
                      <strong>{profile.label}</strong>
                      <span class="meta">•••• {profile.lastFour}</span>
                    </div>
                    <div class="api-profile-actions">
                      {#if profile.active}
                        <span class="profile-active-badge">Active</span>
                      {:else}
                        <button class="button-secondary" type="button" on:click={() => activateApiProfile('muapi', profile.id)} disabled={settingsActionBusy}>Set active</button>
                      {/if}
                      <button class="button-secondary" type="button" on:click={() => deleteApiProfile('muapi', profile.id)} disabled={settingsActionBusy}>Delete</button>
                    </div>
                  </div>
                {/each}
              {:else}
                <p class="meta">No MuAPI profiles yet. Add a named profile so saving does not silently replace another key.</p>
              {/if}
            </div>
            {#if settingsActionTarget === 'muapi'}
              <FormStatus message={settingsActionStatus} kind={settingsActionKind} />
            {/if}
          </article>

          <article class="panel config-card">
            <div class="config-card-head">
              <div>
                <p class="eyebrow">LLM provider</p>
                <div class="config-title-row">
                  <h3>OpenAI Access</h3>
                  <span class="help-wrap">
                    <button class="help-button" type="button" aria-label="OpenAI Access help" aria-describedby="help-openai">?</button>
                    <span id="help-openai" class="help-tooltip" role="tooltip">Store the OpenAI key used for transcript and highlight generation.</span>
                  </span>
                </div>
              </div>
              <div class="status-chips" aria-label="OpenAI key status">
                <span class:ok={settingsConfig?.openaiConfigured} class:warn={!settingsConfig?.openaiConfigured}>OpenAI {configuredLabel(settingsConfig?.openaiConfigured)}</span>
              </div>
            </div>
            {#if apiProfiles.openai?.envOverride}
              <p class="meta warn-text">Environment variable override is active. Saved profiles are available, but runtime will use the environment key.</p>
            {/if}
            <form class="form config-form api-profile-form" novalidate on:submit|preventDefault={saveOpenaiKey}>
              <label>Profile name <input aria-label="OpenAI profile name" autocomplete="off" bind:value={openaiProfileLabel} placeholder="Personal OpenAI" /></label>
              <label>OpenAI key <input aria-label="OpenAI key" type="password" autocomplete="off" bind:value={openaiKeyInput} /></label>
              <div class="settings-actions">
                <button type="submit" disabled={settingsActionBusy || !canSaveOpenaiKey}>Add OpenAI Profile</button>
              </div>
            </form>
            <div class="api-profile-list" aria-label="OpenAI key profiles">
              {#if apiProfiles.openai?.profiles?.length}
                {#each apiProfiles.openai.profiles as profile}
                  <div class="api-profile-row">
                    <div>
                      <strong>{profile.label}</strong>
                      <span class="meta">•••• {profile.lastFour}</span>
                    </div>
                    <div class="api-profile-actions">
                      {#if profile.active}
                        <span class="profile-active-badge">Active</span>
                      {:else}
                        <button class="button-secondary" type="button" on:click={() => activateApiProfile('openai', profile.id)} disabled={settingsActionBusy}>Set active</button>
                      {/if}
                      <button class="button-secondary" type="button" on:click={() => deleteApiProfile('openai', profile.id)} disabled={settingsActionBusy}>Delete</button>
                    </div>
                  </div>
                {/each}
              {:else}
                <p class="meta">No OpenAI profiles yet. Add a named profile so saving does not silently replace another key.</p>
              {/if}
            </div>
            {#if settingsActionTarget === 'openai'}
              <FormStatus message={settingsActionStatus} kind={settingsActionKind} />
            {/if}
          </article>
          {/if}

          {#if settingsConfigTab === 'local'}
          <article class="panel config-card">
            <div class="config-card-head">
              <div>
                <p class="eyebrow">On-device pipeline</p>
                <div class="config-title-row">
                  <h3>Local Processing</h3>
                  <span class="help-wrap">
                    <button class="help-button" type="button" aria-label="Local Processing help" aria-describedby="help-local-processing">?</button>
                    <span id="help-local-processing" class="help-tooltip" role="tooltip">Tune the local transcription defaults used when generation runs without API mode.</span>
                  </span>
                </div>
              </div>
            </div>
            {#if localProfiles?.envOverride}
              <p class="meta warn-text">Environment variable override is active. Saved model profiles are available, but runtime will use the environment model/device.</p>
            {/if}
            <form class="form config-form local-processing-form" novalidate on:submit|preventDefault={saveLocalProcessing}>
              <label>Profile name <input aria-label="Local model profile name" autocomplete="off" bind:value={localProfileLabel} placeholder="Balanced local model" /></label>
              <label class="select-field">
                <span class="field-label-row">
                  Whisper model
                  <span class="help-wrap field-help">
                    <button class="help-button" type="button" aria-label="Whisper model help" aria-describedby="help-whisper-model">?</button>
                    <span id="help-whisper-model" class="help-tooltip" role="tooltip">Small is the safest quality/speed upgrade from Base.</span>
                  </span>
                </span>
                <div class="local-select" class:open={openLocalDropdown === 'model'}>
                  <button
                    class="local-select-button"
                    type="button"
                    aria-label="Whisper model"
                    aria-haspopup="listbox"
                    aria-expanded={openLocalDropdown === 'model'}
                    on:click|preventDefault|stopPropagation={() => toggleLocalDropdown('model')}
                  >
                    <span>{optionLabel(whisperModelOptions, whisperModelInput)}</span>
                  </button>
                  {#if openLocalDropdown === 'model'}
                    <div class="local-select-menu" role="listbox" aria-label="Whisper model options">
                    {#each whisperModelOptions as option}
                      <button
                        class:active={whisperModelInput === option.value}
                        type="button"
                        role="option"
                        aria-selected={whisperModelInput === option.value}
                        on:click|preventDefault|stopPropagation={() => chooseLocalOption('model', option.value)}
                      >
                        {option.label}
                      </button>
                    {/each}
                    </div>
                  {/if}
                </div>
              </label>
              <label class="select-field">
                <span class="field-label-row">
                  Processing device
                  <span class="help-wrap field-help">
                    <button class="help-button" type="button" aria-label="Processing device help" aria-describedby="help-processing-device">?</button>
                    <span id="help-processing-device" class="help-tooltip" role="tooltip">This device choice is saved with the model profile and becomes active when that profile is selected.</span>
                  </span>
                </span>
                <div class="local-select" class:open={openLocalDropdown === 'device'}>
                  <button
                    class="local-select-button"
                    type="button"
                    aria-label="Processing device"
                    aria-haspopup="listbox"
                    aria-expanded={openLocalDropdown === 'device'}
                    on:click|preventDefault|stopPropagation={() => toggleLocalDropdown('device')}
                  >
                    <span>{optionLabel(whisperDeviceOptions, whisperDeviceInput)}</span>
                  </button>
                  {#if openLocalDropdown === 'device'}
                    <div class="local-select-menu compact" role="listbox" aria-label="Processing device options">
                    {#each whisperDeviceOptions as option}
                      <button
                        class:active={whisperDeviceInput === option.value}
                        type="button"
                        role="option"
                        aria-selected={whisperDeviceInput === option.value}
                        on:click|preventDefault|stopPropagation={() => chooseLocalOption('device', option.value)}
                      >
                        {option.label}
                      </button>
                    {/each}
                    </div>
                  {/if}
                </div>
              </label>
              <div class="settings-actions">
                <button type="submit" disabled={settingsActionBusy || !canSaveLocalProcessing}>Save and Download</button>
              </div>
            </form>
            <div class="api-profile-list local-model-list" aria-label="Local model profiles">
              {#if localProfiles?.profiles?.length}
                {#each localProfiles.profiles as profile}
                  <div class="api-profile-row local-model-row">
                    <div>
                      <strong>{profile.label}</strong>
                      <span class="meta">{profile.model} | {optionLabel(whisperDeviceOptions, profile.device)}</span>
                      {#if profile.error}
                        <span class="meta error-text">{profile.error}</span>
                      {/if}
                    </div>
                    <div class="api-profile-actions">
                      {#if profile.active}
                        <span class="profile-active-badge">Active</span>
                      {:else}
                        <button class="button-secondary" type="button" on:click={() => activateLocalModelProfile(profile.id)} disabled={settingsActionBusy}>Set active</button>
                      {/if}
                      <span class:profile-ready-badge={profile.downloadStatus === 'ready'} class:profile-failed-badge={profile.downloadStatus === 'failed'} class="profile-status-badge">{localModelStatusLabel(profile.downloadStatus)}</span>
                      {#if profile.downloadStatus === 'failed' || profile.downloadStatus === 'not_downloaded'}
                        <button class="button-secondary" type="button" on:click={() => retryLocalModelDownload(profile.id)} disabled={settingsActionBusy || Boolean(localModelDownload?.active && localModelDownload?.profileId === profile.id)}>Retry download</button>
                      {/if}
                      <button class="button-secondary" type="button" on:click={() => deleteLocalModelProfile(profile.id)} disabled={settingsActionBusy}>Delete</button>
                    </div>
                  </div>
                {/each}
              {:else}
                <p class="meta">No local model profiles yet. Save a named profile to download and reuse a model.</p>
              {/if}
            </div>
            {#if settingsActionTarget === 'local'}
              <FormStatus message={settingsActionStatus} kind={settingsActionKind} />
            {/if}
          </article>
          {/if}

          {#if settingsConfigTab === 'reset'}
          <article class="panel config-card config-card-caution">
            <div class="config-card-head">
              <div>
                <p class="eyebrow">License support</p>
                <div class="config-title-row">
                  <h3>Device Reset</h3>
                  <span class="help-wrap">
                    <button class="help-button" type="button" aria-label="Device Reset help" aria-describedby="help-device-reset">?</button>
                    <span id="help-device-reset" class="help-tooltip" role="tooltip">Request a reset only when this license needs to move to a different device.</span>
                  </span>
                </div>
              </div>
            </div>
            <form class="form reset-form" novalidate on:submit|preventDefault={submitSettingsResetRequest}>
              <label>
                License key
                <input
                  aria-label="Settings reset license key"
                  type="password"
                  bind:value={settingsResetLicenseKey}
                  autocomplete="off"
                  spellcheck="false"
                />
              </label>
              <button class="button-danger" type="submit" disabled={settingsActionBusy}>Request Device Reset</button>
            </form>
            {#if $authState.resetRequestId}
              <p class="meta">Request: {$authState.resetRequestId}</p>
            {/if}
            {#if $authState.resetStatus !== 'idle' && $authState.resetStatus !== 'error'}
              <p class="meta">Status: {$authState.resetStatus}</p>
              {#if $authState.resetStatusMessage}
                <p class="meta">{$authState.resetStatusMessage}</p>
              {/if}
            {/if}
            {#if $authState.resetError}
              <p class="error-text">{$authState.resetError.message}</p>
            {/if}
            {#if settingsActionTarget === 'reset'}
              <FormStatus message={settingsActionStatus} kind={settingsActionKind} />
            {/if}
          </article>
          {/if}
          </div>
        </div>
      {/if}

      {#if settingsTab === 'diagnostics'}
        <div id="settings-panel-diagnostics" class="panel" role="tabpanel" aria-labelledby="settings-tab-diagnostics">
          <h3>Diagnostics</h3>
          <p class="meta">See system health and take action when setup issues are detected.</p>
          <p>
            Runtime status:
            <span class:ok={settingsRuntime?.ok} class:warn={!settingsRuntime?.ok}>
              {settingsRuntime?.ok ? 'All required dependencies are available.' : 'Action needed for one or more dependencies.'}
            </span>
          </p>
          <p class="meta">Last checked: {formatLastChecked(diagnosticsLastCheckedAt)}</p>
          <button type="button" on:click={loadSettingsStatus} disabled={settingsBusy}>
            {settingsBusy ? 'Rechecking...' : 'Recheck Dependencies'}
          </button>
        </div>
        {#if settingsRuntime?.tools?.length}
          <section class="panel">
            <h3>Required Dependencies</h3>
            <div class="tool-list">
              {#each settingsRuntime.tools as tool}
                <div class="tool-row">
                  <div>
                    <p><strong>{toolDisplayName(tool.tool)}</strong></p>
                    <p class="meta">{toolPurpose(tool.tool)}</p>
                    <p class:ok={tool.ok} class:warn={!tool.ok}>{tool.ok ? 'Available' : tool.message}</p>
                  </div>
                  <div class="row">
                    <button type="button" class="button-secondary" on:click={loadSettingsStatus} disabled={settingsBusy}>
                      Recheck
                    </button>
                    <button
                      type="button"
                      class="button-secondary"
                      on:click={() => (diagnosticsInstallHelpFor = diagnosticsInstallHelpFor === tool.tool ? '' : tool.tool)}
                    >
                      {diagnosticsInstallHelpFor === tool.tool ? 'Hide instructions' : 'Install instructions'}
                    </button>
                  </div>
                </div>
                {#if diagnosticsInstallHelpFor === tool.tool}
                  <p class="meta">{installInstructions(tool.tool)}</p>
                {/if}
              {/each}
            </div>
          </section>
        {/if}
        {#if settingsRuntime?.python_packages?.length}
          <section class="panel">
            <h3>Local Python Packages</h3>
            <div class="tool-list">
              {#each settingsRuntime.python_packages as pkg}
                <div class="tool-row">
                  <div>
                    <p><strong>{pkg.tool}</strong></p>
                    <p class:ok={pkg.ok} class:warn={!pkg.ok}>{pkg.ok ? 'Available' : pkg.message}</p>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <section class="panel">
          <h3>Maintenance</h3>
          <p class="meta">Update checks use the official Tauri updater plugin and signed release artifacts.</p>
          <p>{updaterStatus}</p>
          <div class="row">
            <button type="button" on:click={checkForUpdates} disabled={updaterBusy}>
              {updaterBusy ? 'Working...' : 'Check for Updates'}
            </button>
            {#if updateAvailable}
              <button type="button" on:click={installUpdate} disabled={updaterBusy}>
                Install Update {updateVersion}
              </button>
            {/if}
          </div>
        </section>

        <section class="panel">
          <h3>Advanced Diagnostics</h3>
          <button type="button" class="button-secondary" on:click={() => (diagnosticsShowAdvanced = !diagnosticsShowAdvanced)}>
            {diagnosticsShowAdvanced ? 'Hide advanced details' : 'Show advanced details'}
          </button>
          {#if diagnosticsShowAdvanced}
            <div class="tool-list">
              <div class="tool-row">
                <span>App version</span>
                <span>{settingsContext?.appVersion || APP_VERSION}</span>
              </div>
              <div class="tool-row">
                <span>Platform</span>
                <span>{settingsContext?.platform || platformLabel()}</span>
              </div>
              <div class="tool-row">
                <span>Bridge entry</span>
                <span>{settingsRuntime?.bridge_entry || 'Unavailable'}</span>
              </div>
              <div class="tool-row">
                <span>Bridge entry exists</span>
                <span class:ok={settingsRuntime?.bridge_entry_exists} class:warn={!settingsRuntime?.bridge_entry_exists}>
                  {settingsRuntime?.bridge_entry_exists ? 'Yes' : 'No'}
                </span>
              </div>
              <div class="tool-row">
                <span>Config path</span>
                <span>{settingsContext?.configPath || 'Unavailable'}</span>
              </div>
              <div class="tool-row">
                <span>Log path</span>
                <span>{settingsContext?.logPath || 'Unavailable'}</span>
              </div>
              {#each settingsRuntime?.tools ?? [] as tool}
                <div class="tool-row">
                  <span>{toolDisplayName(tool.tool)} source/path</span>
                  <span>{tool.source || 'n/a'} {tool.path || ''}</span>
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/if}

      {#if settingsTab === 'diagnostics' && crashDraft}
        <section class="panel">
          <h3>Crash Report Draft</h3>
          <p class="meta">A previous fatal error was saved locally. Nothing is uploaded unless you choose to submit it.</p>
          <p>{crashDraft.errorName}: {crashDraft.message}</p>
          {#if crashStatus}
            <p class="meta">{crashStatus}</p>
          {/if}
          <div class="row">
            <button type="button" on:click={submitPendingCrashDraft}>Submit Crash Report</button>
            <button type="button" on:click={dismissPendingCrashDraft}>Dismiss Crash Report</button>
          </div>
        </section>
      {/if}
      {#if settingsTab === 'policies'}
        <div id="settings-panel-policies" class="panel" role="tabpanel" aria-labelledby="settings-tab-policies">
          <h3>Policies</h3>
          <p class="meta">Reference documents for use, privacy, third-party notices, refunds, and liability.</p>
          <div class="row">
            <button type="button" class:active-tab={policiesTab === 'terms'} on:click={() => (policiesTab = 'terms')}>Terms</button>
            <button type="button" class:active-tab={policiesTab === 'privacy'} on:click={() => (policiesTab = 'privacy')}>Privacy</button>
            <button type="button" class:active-tab={policiesTab === 'notices'} on:click={() => (policiesTab = 'notices')}>Third-Party Notices</button>
            <button type="button" class:active-tab={policiesTab === 'refund'} on:click={() => (policiesTab = 'refund')}>Refund Policy</button>
          </div>
        </div>
        <div class="panel legal-copy" role="tabpanel">
          {#each POLICY_SECTIONS[policiesTab] as section}
            <h3>{section.heading}</h3>
            {#each section.paragraphs as paragraph}
              <p>{paragraph}</p>
            {/each}
          {/each}
          {#each POLICY_COMMON_SECTIONS as section}
            <h3>{section.heading}</h3>
            {#each section.paragraphs as paragraph}
              <p>{paragraph}</p>
            {/each}
          {/each}
          <p class="meta">Last updated: {POLICY_LAST_UPDATED_LABEL}</p>
        </div>
      {/if}
    {/if}

  </section>
</main>
{/if}

<style>
  :global(html),
  :global(body),
  :global(#app) {
    margin: 0;
    min-height: 100%;
    font-family: var(--font-body-md);
    background: radial-gradient(circle at 8% 12%, var(--color-canvas-start) 0%, var(--color-canvas-mid) 42%, var(--color-canvas-end) 100%);
    color: var(--color-text-primary);
    color-scheme: var(--ui-color-scheme, dark);
  }

  .app-shell {
    height: 100vh;
    box-sizing: border-box;
    padding: var(--space-lg);
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    gap: var(--space-lg);
    position: relative;
    overflow: hidden;
    background: radial-gradient(circle at 8% 12%, var(--color-canvas-start) 0%, var(--color-canvas-mid) 42%, var(--color-canvas-end) 100%);
  }

  .license-shell {
    min-height: 100vh;
    box-sizing: border-box;
    padding: var(--space-lg);
    display: grid;
    place-items: center;
    position: relative;
    overflow: hidden;
    background: radial-gradient(circle at 8% 12%, var(--color-canvas-start) 0%, var(--color-canvas-mid) 42%, var(--color-canvas-end) 100%);
  }

  .license-card {
    width: min(100%, 520px);
    display: grid;
    gap: var(--space-lg);
    padding: var(--space-xl);
  }

  .license-brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
  }

  .brand-mark {
    margin: 0;
    color: var(--color-text-secondary);
    font-weight: 700;
  }

  .license-copy {
    display: grid;
    gap: var(--space-xs);
  }

  .license-copy h1 {
    margin: 0;
    font-size: 1.55rem;
  }

  .license-form,
  .reset-form {
    grid-template-columns: 1fr;
  }

  .terms-accept-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-xs);
    margin-top: calc(var(--space-xs) * -1);
  }

  .terms-accept-row input[type="checkbox"] {
    margin-top: .15rem;
  }

  .terms-copy {
    color: var(--color-text-tertiary);
    line-height: 1.4;
  }

  .inline-link {
    background: none;
    border: none;
    padding: 0;
    margin: 0 .1rem;
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
    color: var(--color-focus-ring);
    cursor: pointer;
  }

  .policy-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.46);
    z-index: 30;
    display: grid;
    place-items: center;
    padding: var(--space-md);
  }

  .policy-modal {
    width: min(760px, 96vw);
    max-height: min(86vh, 900px);
    display: grid;
    gap: var(--space-sm);
    overflow: hidden;
  }

  .policy-modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-sm);
  }

  .policy-modal-content {
    overflow-y: auto;
    padding-right: .3rem;
  }

  .auth-status,
  .reset-box {
    display: grid;
    gap: var(--space-sm);
    padding: var(--space-md);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--color-panel-card) 72%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent);
  }

  .auth-status h2,
  .reset-box h2 {
    margin: 0;
    font-size: 1rem;
  }

  .error-text {
    color: var(--color-state-error);
  }

  .warn-text {
    color: var(--color-state-warning);
  }

  .content {
    display: grid;
    gap: var(--space-lg);
    align-content: start;
    min-height: 0;
    overflow-y: auto;
    padding-right: 0.2rem;
  }

  .panel {
    background: linear-gradient(160deg, color-mix(in srgb, var(--color-panel-base-1) 95%, transparent), color-mix(in srgb, var(--color-panel-base-2) 95%, transparent));
    border: 1px solid color-mix(in srgb, var(--color-border-soft) 20%, transparent);
    border-radius: var(--radius-xl);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.28);
    padding: var(--space-lg);
    z-index: 1;
    position: relative;
  }

  .screen-header {
    display: grid;
    gap: .2rem;
    padding: 0;
    margin: 0;
  }
  .screen-title { margin: 0; font-size: clamp(1.15rem, 1.8vw, 1.45rem); font-weight: 700; }
  .meta { color: var(--color-text-tertiary); }
  h1, h2, h3, h4 { margin: 0 0 var(--space-sm); line-height: 1.3; }
  p { margin: 0 0 var(--space-sm); line-height: 1.45; }
  p:last-child { margin-bottom: 0; }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: .75rem;
    min-height: 0;
    height: calc(100vh - (var(--space-lg) * 2));
    position: sticky;
    top: var(--space-lg);
  }
  .sidebar-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-sm);
  }
  .sidebar h1 { font-size: 1.25rem; }
  .menu-toggle {
    display: none;
    position: relative;
    padding-right: 2rem;
    text-align: left;
    min-width: 110px;
  }
  .menu-toggle::after {
    content: "";
    position: absolute;
    right: .75rem;
    top: 50%;
    width: .45rem;
    height: .45rem;
    border-right: 2px solid var(--color-text-tertiary);
    border-bottom: 2px solid var(--color-text-tertiary);
    transform: translateY(-62%) rotate(45deg);
    pointer-events: none;
  }
  nav {
    display: grid;
    gap: var(--space-sm);
  }
  .global-theme-toggle {
    position: fixed;
    top: max(var(--space-md), env(safe-area-inset-top));
    right: max(var(--space-md), env(safe-area-inset-right));
    z-index: 20;
  }

  .theme-switch {
    display: inline-flex;
    align-items: center;
    gap: .45rem;
    padding: .36rem .5rem;
    border-radius: var(--radius-pill);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 26%, transparent);
    background: color-mix(in srgb, var(--color-panel-card) 82%, transparent);
    color: var(--color-text-primary);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.22);
    font-weight: 600;
  }

  .theme-switch-icon {
    font-size: .95rem;
    line-height: 1;
  }

  .theme-switch-track {
    width: 2.2rem;
    height: 1.2rem;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--color-surface-input) 72%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-soft) 30%, transparent);
    position: relative;
    display: inline-flex;
    align-items: center;
    padding: 0 .12rem;
    box-sizing: border-box;
  }

  .theme-switch-thumb {
    width: .86rem;
    height: .86rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
    transform: translateX(0);
    transition: transform 160ms ease;
  }

  .theme-switch[aria-checked="true"] .theme-switch-thumb {
    transform: translateX(.94rem);
  }

  button, input, select {
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-medium);
    border: 1px solid color-mix(in srgb, var(--color-border-medium) 30%, transparent);
    background: var(--color-surface-input);
    background: color-mix(in srgb, var(--color-surface-input) 80%, transparent);
    color: var(--color-text-primary);
    padding: .6rem .7rem;
    font-family: inherit;
  }
  select option {
    background: var(--color-surface-input);
    color: var(--color-text-primary);
  }
  select {
    appearance: none;
    -webkit-appearance: none;
    background-color: var(--color-surface-input);
    color: var(--color-text-primary);
  }
  .select-wrap {
    position: relative;
  }
  .select-wrap::after {
    content: "";
    position: absolute;
    right: .75rem;
    top: 50%;
    width: .45rem;
    height: .45rem;
    border-right: 2px solid var(--color-text-tertiary);
    border-bottom: 2px solid var(--color-text-tertiary);
    transform: translateY(-62%) rotate(45deg);
    pointer-events: none;
  }
  .select-wrap select {
    width: 100%;
    padding-right: 2rem;
  }

  button { cursor: pointer; background: linear-gradient(90deg, var(--color-primary), var(--color-secondary)); color: var(--color-on-accent); border: none; font-weight: 700; }
  button:disabled {
    cursor: not-allowed;
    opacity: .58;
  }
  nav button { text-align: left; background: color-mix(in srgb, var(--color-panel-card) 80%, transparent); color: var(--color-text-primary); border: 1px solid color-mix(in srgb, var(--color-border-strong) 25%, transparent); }
  nav button.active { border-color: var(--color-focus-ring); box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-focus-ring) 25%, transparent); }
  .form { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-md); align-items: end; }
  label { display: grid; gap: var(--space-xs); }
  .form-full { grid-column: 1 / -1; }
  .form-action-row {
    grid-column: 1 / -1;
    display: flex;
    justify-content: flex-start;
    align-items: center;
  }
  .advanced { grid-column: 1 / -1; }
  .advanced summary {
    cursor: pointer;
    color: var(--color-text-tertiary);
    margin-bottom: var(--space-sm);
  }
  .advanced[open] {
    display: grid;
    gap: var(--space-sm);
  }
  .advanced-actions {
    margin-top: var(--space-sm);
  }
  .toolbar { display: grid; gap: var(--space-sm); margin-bottom: var(--space-md); }

  .settings-nav-panel {
    padding: var(--space-sm);
  }

  .settings-tabs {
    display: flex;
    gap: var(--space-xs);
    flex-wrap: wrap;
    padding: .22rem;
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--color-surface-input) 52%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 16%, transparent);
  }
  .settings-tabs button,
  .row button.active-tab {
    background: color-mix(in srgb, var(--color-panel-card) 80%, transparent);
    color: var(--color-text-primary);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 25%, transparent);
  }
  .settings-tabs button {
    flex: 1 1 145px;
    position: relative;
    transition: transform 160ms ease, border-color 160ms ease, background 160ms ease;
  }
  .settings-tabs button.active-tab,
  .row button.active-tab {
    border-color: var(--color-focus-ring);
    background: linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 28%, var(--color-panel-card)), color-mix(in srgb, var(--color-secondary) 18%, var(--color-panel-card)));
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-focus-ring) 22%, transparent);
  }

  .settings-tabs button:not(.active-tab):hover {
    transform: translateY(-1px);
    border-color: color-mix(in srgb, var(--color-focus-ring) 45%, transparent);
  }

  .configuration-panel {
    display: grid;
    gap: var(--space-md);
  }

  .config-subtabs {
    display: inline-flex;
    width: fit-content;
    max-width: 100%;
    gap: .18rem;
    flex-wrap: wrap;
    padding: .2rem;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--color-surface-input) 24%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 12%, transparent);
  }

  .config-subtabs button {
    flex: 0 1 auto;
    padding: .48rem .72rem;
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    background: transparent;
    border: 1px solid transparent;
    font-size: .82rem;
    font-weight: 800;
    letter-spacing: .03em;
    text-transform: uppercase;
    transition: color 160ms ease, border-color 160ms ease, background 160ms ease;
  }

  .config-subtabs button.active-tab {
    color: var(--color-secondary);
    border-color: color-mix(in srgb, var(--color-secondary) 32%, transparent);
    background: color-mix(in srgb, var(--color-secondary) 10%, transparent);
  }

  .config-subtabs button:not(.active-tab):hover {
    color: var(--color-text-primary);
    border-color: color-mix(in srgb, var(--color-border-strong) 28%, transparent);
    background: color-mix(in srgb, var(--color-panel-card) 52%, transparent);
  }

  .configuration-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-lg);
    align-items: start;
  }

  .configuration-grid-single {
    grid-template-columns: minmax(0, 720px);
  }

  .config-card {
    display: grid;
    gap: var(--space-md);
    overflow: visible;
    z-index: 1;
  }

  .config-card:hover,
  .config-card:focus-within {
    z-index: 8;
  }

  .config-card-caution {
    border-color: color-mix(in srgb, var(--color-state-warning) 38%, transparent);
  }

  .config-card-head {
    display: flex;
    justify-content: space-between;
    gap: var(--space-md);
    align-items: flex-start;
  }

  .eyebrow {
    margin: 0 0 var(--space-xs);
    color: var(--color-secondary);
    font-size: .74rem;
    font-weight: 700;
    letter-spacing: .08em;
    text-transform: uppercase;
  }

  .config-title-row {
    display: flex;
    gap: var(--space-xs);
    align-items: center;
  }

  .config-title-row h3 {
    margin-bottom: 0;
  }

  .help-wrap {
    position: relative;
    display: inline-flex;
  }

  .help-button {
    width: 1.35rem;
    height: 1.35rem;
    display: inline-grid;
    place-items: center;
    padding: 0;
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    background: color-mix(in srgb, var(--color-panel-card) 78%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 30%, transparent);
    font-size: .78rem;
    line-height: 1;
  }

  .help-tooltip {
    position: absolute;
    left: 50%;
    bottom: calc(100% + var(--space-sm));
    width: max-content;
    max-width: 250px;
    padding: .45rem .55rem;
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    background: color-mix(in srgb, var(--color-surface-input) 94%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 28%, transparent);
    box-shadow: 0 14px 36px rgba(0, 0, 0, .32);
    font-size: .82rem;
    line-height: 1.35;
    opacity: 0;
    pointer-events: none;
    transform: translate(-50%, .2rem);
    transition: opacity 140ms ease, transform 140ms ease;
    z-index: 20;
  }

  .help-wrap:hover .help-tooltip,
  .help-button:focus-visible + .help-tooltip {
    opacity: 1;
    transform: translate(-50%, 0);
  }

  .status-chips {
    display: flex;
    gap: var(--space-xs);
    flex-wrap: wrap;
    justify-content: flex-end;
    min-width: 180px;
  }

  .status-chips span {
    padding: .32rem .52rem;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--color-panel-card) 76%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 18%, transparent);
    color: var(--color-text-secondary);
    font-size: .82rem;
    font-weight: 700;
    white-space: nowrap;
  }

  .status-chips span.ok {
    color: var(--color-state-success);
    border-color: color-mix(in srgb, var(--color-state-success) 34%, transparent);
  }

  .status-chips span.warn {
    color: var(--color-state-warning);
    border-color: color-mix(in srgb, var(--color-state-warning) 32%, transparent);
    text-align: left;
  }

  .config-form {
    align-items: end;
  }

  .local-processing-form {
    grid-template-columns: 1fr;
    align-items: start;
  }

  .local-download-banner {
    padding: var(--space-md);
  }

  .local-model-row > div:first-child {
    display: grid;
    gap: .18rem;
  }

  .profile-status-badge {
    padding: .34rem .55rem;
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 26%, transparent);
    background: color-mix(in srgb, var(--color-panel-card) 80%, transparent);
    font-size: .78rem;
    font-weight: 800;
  }

  .profile-ready-badge {
    color: var(--color-state-success);
    border-color: color-mix(in srgb, var(--color-state-success) 32%, transparent);
  }

  .profile-failed-badge {
    color: var(--color-state-error);
    border-color: color-mix(in srgb, var(--color-state-error) 32%, transparent);
  }

  .select-field {
    min-width: 0;
  }

  .field-label-row {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    min-width: 0;
  }

  .field-help .help-tooltip {
    max-width: 230px;
    white-space: normal;
  }

  .local-select {
    position: relative;
    z-index: 1;
    width: 100%;
    min-width: 0;
  }

  .local-select.open {
    z-index: 30;
  }

  .local-select-button {
    position: relative;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    min-height: 46px;
    padding: .66rem 2.25rem .66rem .78rem;
    text-align: left;
    color: var(--color-text-primary);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--color-surface-input) 92%, var(--color-panel-card)), var(--color-surface-input));
    border: 1px solid color-mix(in srgb, var(--color-border-medium) 34%, transparent);
    box-shadow: inset 0 1px 0 color-mix(in srgb, white 7%, transparent);
    font-weight: 700;
    line-height: 1.25;
  }

  .local-select-button::after {
    content: "";
    position: absolute;
    right: .85rem;
    top: 50%;
    width: .48rem;
    height: .48rem;
    border-right: 2px solid var(--color-text-tertiary);
    border-bottom: 2px solid var(--color-text-tertiary);
    transform: translateY(-62%) rotate(45deg);
  }

  .local-select-button span {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .local-select-menu {
    position: absolute;
    top: calc(100% + .35rem);
    left: 0;
    right: 0;
    display: grid;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    max-height: 220px;
    overflow-y: auto;
    padding: .35rem;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 40%, transparent);
    background: color-mix(in srgb, var(--color-panel-card) 94%, var(--color-surface-input));
    box-shadow: 0 18px 44px rgba(0, 0, 0, .34);
  }

  .local-select-menu.compact {
    max-height: 168px;
  }

  .local-select-menu button {
    width: 100%;
    min-width: 0;
    padding: .58rem .62rem;
    color: var(--color-text-secondary);
    background: transparent;
    border: 0;
    border-radius: calc(var(--radius-sm) - 4px);
    text-align: left;
    font-weight: 650;
    line-height: 1.25;
  }

  .local-select-menu button:hover,
  .local-select-menu button:focus-visible {
    color: var(--color-text-primary);
    background: color-mix(in srgb, var(--color-primary) 18%, var(--color-panel-card));
    outline: none;
  }

  .local-select-menu button.active {
    color: var(--color-on-accent);
    background: linear-gradient(90deg, var(--color-primary), var(--color-secondary));
  }

  .api-profile-form label {
    min-width: 0;
  }

  .api-profile-list {
    display: grid;
    gap: var(--space-sm);
  }

  .api-profile-row {
    display: flex;
    justify-content: space-between;
    gap: var(--space-sm);
    align-items: center;
    padding: .68rem;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-surface-input) 44%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 16%, transparent);
  }

  .api-profile-row strong {
    display: block;
    color: var(--color-text-primary);
  }

  .api-profile-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-xs);
    flex-wrap: wrap;
  }

  .profile-active-badge {
    padding: .34rem .55rem;
    border-radius: var(--radius-pill);
    color: var(--color-state-success);
    border: 1px solid color-mix(in srgb, var(--color-state-success) 32%, transparent);
    background: color-mix(in srgb, var(--color-state-success) 8%, transparent);
    font-size: .78rem;
    font-weight: 800;
  }

  .settings-actions {
    grid-column: 1 / -1;
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .button-secondary,
  .button-danger {
    color: var(--color-text-primary);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 28%, transparent);
    background: color-mix(in srgb, var(--color-panel-card) 82%, transparent);
  }

  .button-danger {
    color: var(--color-state-warning);
    border-color: color-mix(in srgb, var(--color-state-warning) 36%, transparent);
  }

  .tool-row {
    display: flex;
    justify-content: space-between;
    gap: var(--space-md);
    align-items: center;
  }

  .tool-list {
    display: grid;
    gap: var(--space-sm);
  }

  .tool-row {
    padding: var(--space-sm);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-panel-card) 80%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent);
  }

  .ok {
    color: var(--color-state-success);
  }

  .warn {
    color: var(--color-state-error);
    text-align: right;
  }

  .list { display: grid; gap: var(--space-sm); }
  .list-item {
    background: color-mix(in srgb, var(--color-panel-card) 80%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent);
    border-radius: var(--radius-lg);
    padding: var(--space-md);
    display: flex;
    justify-content: space-between;
    gap: .8rem;
    align-items: center;
  }

  .row { display: flex; gap: var(--space-sm); flex-wrap: wrap; }
  .picker-row { grid-column: 1 / -1; }

  .status-line { margin: 0 0 var(--space-sm); color: var(--color-text-secondary); }
  .meter { height: 10px; border-radius: var(--radius-pill); background: color-mix(in srgb, var(--color-surface-meter-track) 45%, transparent); overflow: hidden; }
  .meter span { display: block; height: 100%; background: linear-gradient(90deg, var(--color-primary), var(--color-secondary)); transition: width 180ms ease; }

  .cards { display: grid; gap: .5rem; }
  .cards article { background: color-mix(in srgb, var(--color-panel-card) 80%, transparent); border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent); border-radius: var(--radius-lg); padding: .65rem; }

  .orb { position: fixed; width: 320px; height: 320px; border-radius: var(--radius-pill); filter: blur(70px); pointer-events: none; z-index: 0; opacity: .24; }
  .orb-a { top: -80px; left: -90px; background: var(--color-primary); }
  .orb-b { bottom: -70px; right: -60px; background: var(--color-secondary); }

  .legal-copy h3 { margin-top: .8rem; }

  @media (max-width: 900px) {
    .app-shell {
      height: auto;
      min-height: 100vh;
      overflow: visible;
      grid-template-columns: 1fr;
    }
    .sidebar {
      height: auto;
      position: static;
      top: auto;
    }
    .menu-toggle {
      display: inline-flex;
      width: auto;
    }
    nav {
      display: none;
    }
    nav.nav-open {
      display: grid;
    }
    .global-theme-toggle {
      top: max(var(--space-sm), env(safe-area-inset-top));
      right: max(var(--space-sm), env(safe-area-inset-right));
    }
    .content {
      min-height: auto;
      overflow: visible;
      padding-right: 0;
    }
    .configuration-grid {
      grid-template-columns: 1fr;
    }
    .config-card-head {
      display: grid;
    }
    .help-tooltip {
      left: auto;
      right: 0;
      transform: translate(0, .2rem);
    }
    .help-wrap:hover .help-tooltip,
    .help-button:focus-visible + .help-tooltip {
      transform: translate(0, 0);
    }
    .status-chips {
      justify-content: flex-start;
      min-width: 0;
    }
    .settings-actions {
      display: grid;
    }
    .tool-row {
      align-items: stretch;
    }
    .form { grid-template-columns: 1fr; }
    .list-item { flex-direction: column; align-items: stretch; }
  }

  @media (max-height: 760px) {
    .app-shell {
      height: auto;
      min-height: 100vh;
      overflow: visible;
    }
    .sidebar {
      height: auto;
      position: static;
      top: auto;
    }
    .content {
      overflow: visible;
      min-height: auto;
    }
  }
</style>
