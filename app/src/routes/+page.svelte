<script>
  import { onMount } from 'svelte';
  import { runState } from '../lib/stores/runState';
  import { authState } from '../lib/stores/authState';
  import { openInFileManager, pickLocalVideoFile, pickOutputJsonPath, runGenerateAndStream } from '../lib/api/tauriClient';
  import { checkForAppUpdate, installAppUpdate } from '../lib/api/updaterClient';
  import { CRASH_DRAFT_KEY, createCrashDraft, dismissCrashDraft, saveCrashDraft } from '../support/crashDraft';
  const LS = {
    projects: 'shorts.projects.v1',
    theme: 'shorts.theme.v1'
  };
  const APP_VERSION = import.meta.env?.VITE_APP_VERSION ?? '0.1.0';
  const CRASH_REPORT_ENDPOINT = import.meta.env?.VITE_CRASH_REPORT_ENDPOINT ?? '';

  let active = 'generate';

  let url = '';
  let sourceType = 'youtube';
  let mode = 'api';
  let numClips = 3;
  let aspectRatio = '9:16';
  let format = '720';
  let outputJson = '';
  let licenseKey = '';
  let resetEmail = '';
  let resetReceipt = '';

  let projectName = '';
  let shortsSearch = '';

  let supportMessage = '';
  let supportLog = '';
  let updaterStatus = 'Updater idle.';
  let updateAvailable = false;
  let updateVersion = '';
  let updaterBusy = false;
  let crashDraft = null;
  let crashStatus = '';
  let theme = 'dark';
  let mobileNavOpen = false;

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

  onMount(() => {
    try {
      loadState();
      loadCrashDraftFromLocalStorage();
      authState.bootstrap();
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
    if (screen === 'help') {
      loadCrashDraftFromLocalStorage();
    }
    active = screen;
    mobileNavOpen = false;
  }

  function exportDebugLog() {
    const debug = {
      time: new Date().toISOString(),
      activeScreen: active,
      runLifecycle: $runState.lifecycle,
      progress: $runState.progress,
      lastError: $runState.error,
      recentProjects: projects.slice(0, 3).map((p) => p.name)
    };
    supportLog = JSON.stringify(debug, null, 2);
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
      return;
    }
    await authState.activate(key);
    licenseKey = '';
  }

  async function submitResetRequest() {
    const purchaser_email = resetEmail.trim();
    if (!purchaser_email) {
      return;
    }
    await authState.requestReset({
      purchaser_email,
      receipt_reference: resetReceipt.trim() || null
    });
    resetReceipt = '';
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
    if (sourceType === 'local') {
      mode = 'local';
    }
    runState.start();

    try {
      const envelope = await runGenerateAndStream(
        {
          youtube_url: url,
          mode,
          num_clips: numClips,
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
        sourceUrl: url,
        clipCount: numClips,
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
        source_video_url: url
      });
    }
  }
</script>

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
      <button class:active={active === 'help'} on:click={() => selectScreen('help')}>Help & Trust</button>
      <button class:active={active === 'legal'} on:click={() => selectScreen('legal')}>Legal</button>
    </nav>
    <button class="theme-toggle" class:nav-open={mobileNavOpen} type="button" on:click={toggleTheme}>
      {theme === 'dark' ? 'Switch to Light' : 'Switch to Dark'}
    </button>
  </aside>

  <section class="content">
    {#if active === 'generate'}
      {#if $authState.lifecycle !== 'licensed'}
        <section class="panel hero">
          <h2 class="screen-title">License Required</h2>
          <p class="meta">Enter your license key to unlock generation on this device.</p>
        </section>

        {#if $authState.lifecycle === 'checking'}
          <section class="panel status">
            <p class="status-line">Checking license...</p>
          </section>
        {:else if $authState.lifecycle === 'reset_pending' || $authState.lifecycle === 'reset_approved_unbound' || $authState.lifecycle === 'reset_rejected' || $authState.lifecycle === 'reset_expired'}
          <section class="panel">
            <h3>Device Reset</h3>
            <p class="meta">Status: {$authState.lifecycle.replaceAll('_', ' ')}</p>
            {#if $authState.resetRequestId}
              <p class="meta">Request: {$authState.resetRequestId}</p>
            {/if}
            {#if $authState.lifecycle === 'reset_approved_unbound'}
              <p>Device reset approved. You can now use this license key to activate a device.</p>
              <p class="meta">Your license is currently unbound. The next device activated with this license key will become the registered device.</p>
            {/if}
            {#if $authState.lifecycle === 'reset_pending'}
              <button type="button" on:click={refreshResetStatus}>Refresh Reset Status</button>
            {/if}
          </section>
        {:else}
          <section class="panel">
            <form class="form" on:submit|preventDefault={submitLicense}>
              <label>License key <input aria-label="License key" bind:value={licenseKey} autocomplete="off" required /></label>
              <button type="submit" disabled={$authState.lifecycle === 'activating'}>
                {$authState.lifecycle === 'activating' ? 'Activating...' : 'Activate'}
              </button>
            </form>
            {#if $authState.lifecycle === 'reauth_required'}
              <p class="meta">Session expired. Re-enter your license key to continue.</p>
            {/if}
            {#if $authState.error}
              <p class="meta">{$authState.error.message}</p>
            {/if}
          </section>

          {#if $authState.lifecycle === 'device_bound_elsewhere'}
            <section class="panel">
              <h3>Request Device Reset</h3>
              <form class="form" on:submit|preventDefault={submitResetRequest}>
                <label>Purchaser email <input aria-label="Purchaser email" type="email" bind:value={resetEmail} required /></label>
                <label>Receipt reference <input aria-label="Receipt reference" bind:value={resetReceipt} /></label>
                <button type="submit">Request Reset</button>
              </form>
            </section>
          {/if}
        {/if}
      {:else}
      <section class="panel hero">
        <h2 class="screen-title">Generate Shorts</h2>
      </section>

      <section class="panel">
        <form class="form" on:submit|preventDefault={submitRun}>
          <label>Project title <input aria-label="Project title" bind:value={projectName} placeholder="My Product Launch Highlights" /></label>
          <label>Source type
            <div class="select-wrap">
              <select aria-label="Source type" bind:value={sourceType}>
                <option value="youtube">YouTube URL</option>
                <option value="local">Local video file</option>
              </select>
            </div>
          </label>
          <label>{sourceLabel} <input aria-label="YouTube video URL" bind:value={url} placeholder={sourcePlaceholder} required /></label>
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
          <label>Num clips <input aria-label="Num clips" type="number" min="1" bind:value={numClips} /></label>
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
          <button type="submit">Run</button>
        </form>
      </section>

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
    {/if}

    {#if active === 'library'}
      <section class="panel hero">
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

    {#if active === 'help'}
      <section class="panel hero">
        <h2 class="screen-title">Help & Trust</h2>
        <p class="meta">How it works, privacy and support.</p>
      </section>

      <section class="panel">
        <h3>How it works</h3>
        <p>1. Add a source URL. 2. Choose mode and clip settings. 3. Run pipeline. 4. Review exported clips.</p>
        <p class="meta">Progress events stream live so users can track each stage of the generation pipeline.</p>
      </section>

      <section class="panel">
        <h3>Privacy</h3>
        <p>Project history and media library metadata are stored locally on this machine (browser local storage in this build).</p>
        <p class="meta">No cloud sync is used for these screens unless you implement explicit remote storage later.</p>
      </section>

      <section class="panel">
        <h3>Updates</h3>
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
        <h3>Support</h3>
        <label>Message to support
          <textarea rows="4" bind:value={supportMessage} placeholder="Describe your issue"></textarea>
        </label>
        <div class="row">
          <button type="button" on:click={exportDebugLog}>Generate Debug Log</button>
        </div>
        {#if supportLog}
          <label>Debug log
            <textarea rows="8" readonly value={supportLog}></textarea>
          </label>
        {/if}
      </section>

      {#if crashDraft}
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
    {/if}

    {#if active === 'legal'}
      <section class="panel hero">
        <h2 class="screen-title">Legal</h2>
        <p class="meta">Terms and Conditions.</p>
      </section>

      <section class="panel legal-copy">
        <h3>Terms of Use</h3>
        <p>By using this software, you confirm you have rights to process input media and comply with platform policies and local laws.</p>
        <p>You are responsible for generated outputs, publication decisions, copyright compliance, and any third-party claims.</p>

        <h3>Acceptable Use</h3>
        <p>Do not use the app to generate unlawful, deceptive, harmful, or infringing content. Abuse may result in access suspension for paid services.</p>

        <h3>Privacy Notice</h3>
        <p>Local workspace data for generation history and library is stored on-device in this release build. Cloud operations only occur when API mode is selected for generation.</p>

        <h3>Refund Policy</h3>
        <p>Refund requests are handled manually within 7 days from purchase, subject to purchase records and platform dispute rules.</p>
        <p>No automated refund engine is built into this app.</p>

        <h3>Warranty and Liability</h3>
        <p>The software is provided as-is without guarantees of uninterrupted service. Liability is limited to the maximum extent permitted by law.</p>

        <h3>Support and Contact</h3>
        <p>For enterprise support and data requests, use the Support section and attach generated debug logs.</p>

        <p class="meta">Last updated: May 8, 2026</p>
      </section>
    {/if}

  </section>
</main>

<style>
  :global(body) {
    margin: 0;
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

  .hero { padding: var(--space-xl); }
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
  nav { display: grid; gap: var(--space-sm); }
  .theme-toggle {
    margin-top: auto;
  }

  button, input, select, textarea {
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
  nav button { text-align: left; background: color-mix(in srgb, var(--color-panel-card) 80%, transparent); color: var(--color-text-primary); border: 1px solid color-mix(in srgb, var(--color-border-strong) 25%, transparent); }
  nav button.active { border-color: var(--color-focus-ring); box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-focus-ring) 25%, transparent); }
  .form { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-md); align-items: end; }
  label { display: grid; gap: var(--space-xs); }
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
  textarea { resize: vertical; }

  .toolbar { display: grid; gap: var(--space-sm); margin-bottom: var(--space-md); }

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
    nav,
    .theme-toggle {
      display: none;
    }
    nav.nav-open,
    .theme-toggle.nav-open {
      display: grid;
    }
    .theme-toggle.nav-open {
      margin-top: 0;
    }
    .content {
      min-height: auto;
      overflow: visible;
      padding-right: 0;
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
