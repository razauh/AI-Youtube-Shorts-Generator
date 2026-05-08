<script>
  import { onMount } from 'svelte';
  import { runState } from '../lib/stores/runState';
  import { pickLocalVideoFile, runGenerateAndStream } from '../lib/api/tauriClient';
  const LS = {
    projects: 'shorts.projects.v1',
    media: 'shorts.media.v1',
    presets: 'shorts.presets.v1',
    theme: 'shorts.theme.v1'
  };

  let active = 'generate';

  let url = '';
  let sourceType = 'youtube';
  let mode = 'api';
  let numClips = 3;
  let aspectRatio = '9:16';
  let format = '720';
  let outputJson = '';

  let projectName = '';
  let projectSearch = '';
  let mediaSearch = '';
  let selectedType = 'all';
  let selectedFolder = 'all';

  let hookTopic = '';
  let hookResult = '';
  let studioPrompt = '';
  let studioTone = 'energetic';
  let studioLanguage = 'English';
  let presetName = '';

  let supportMessage = '';
  let supportLog = '';
  let theme = 'dark';

  let projects = [];
  let assets = [];
  let presets = [];

  const sampleProject = {
    id: 'sample-project',
    name: 'Sample: Podcast Growth Clip',
    status: 'exported',
    updatedAt: new Date().toISOString(),
    sourceUrl: 'https://www.youtube.com/watch?v=sample123',
    clipCount: 3
  };

  const sampleAssets = [
    {
      id: 'asset-1',
      name: 'podcast_episode.mp4',
      type: 'video',
      folder: 'raw-footage',
      tags: ['podcast', 'long-form'],
      path: '/Users/local/raw-footage/podcast_episode.mp4',
      usedIn: ['Sample: Podcast Growth Clip'],
      updatedAt: new Date().toISOString()
    },
    {
      id: 'asset-2',
      name: 'brand_logo.png',
      type: 'logo',
      folder: 'brand',
      tags: ['logo', 'branding'],
      path: '/Users/local/brand/brand_logo.png',
      usedIn: [],
      updatedAt: new Date().toISOString()
    }
  ];

  const samplePresets = [
    {
      id: 'preset-1',
      name: 'Educational Hook',
      prompt: 'Generate a curiosity-first hook with one concrete takeaway for creators.',
      tone: 'educational',
      language: 'English'
    },
    {
      id: 'preset-2',
      name: 'Bold Viral CTA',
      prompt: 'Write a short, punchy title and opening line with urgency.',
      tone: 'bold',
      language: 'English'
    }
  ];

  $: draftCount = projects.filter((p) => p.status === 'draft').length;
  $: exportedCount = projects.filter((p) => p.status === 'exported').length;
  $: recentProjects = [...projects]
    .sort((a, b) => +new Date(b.updatedAt) - +new Date(a.updatedAt))
    .slice(0, 6);
  $: filteredProjects = projects.filter((p) =>
    [p.name, p.sourceUrl].join(' ').toLowerCase().includes(projectSearch.toLowerCase())
  );
  $: folders = Array.from(new Set(assets.map((a) => a.folder))).sort();
  $: filteredAssets = assets.filter((a) => {
    const matchesText = [a.name, a.folder, a.tags.join(' '), a.path]
      .join(' ')
      .toLowerCase()
      .includes(mediaSearch.toLowerCase());
    const matchesType = selectedType === 'all' || a.type === selectedType;
    const matchesFolder = selectedFolder === 'all' || a.folder === selectedFolder;
    return matchesText && matchesType && matchesFolder;
  });
  $: sourceLabel = sourceType === 'local' ? 'Local video file path' : 'YouTube video URL';
  $: sourcePlaceholder =
    sourceType === 'local'
      ? '/home/user/Videos/interview.mp4'
      : 'https://www.youtube.com/watch?v=dQw4w9WgXcQ';

  onMount(() => {
    loadState();
  });

  function loadState() {
    const p = localStorage.getItem(LS.projects);
    const m = localStorage.getItem(LS.media);
    const s = localStorage.getItem(LS.presets);
    const t = localStorage.getItem(LS.theme);

    projects = p ? JSON.parse(p) : [sampleProject];
    assets = m ? JSON.parse(m) : sampleAssets;
    presets = s ? JSON.parse(s) : samplePresets;
    theme = t === 'light' ? 'light' : 'dark';
    applyTheme(theme);

    if (!p) persistProjects();
    if (!m) persistMedia();
    if (!s) persistPresets();
  }

  function persistProjects() {
    localStorage.setItem(LS.projects, JSON.stringify(projects));
  }

  function persistMedia() {
    localStorage.setItem(LS.media, JSON.stringify(assets));
  }

  function persistPresets() {
    localStorage.setItem(LS.presets, JSON.stringify(presets));
  }

  function applyTheme(nextTheme) {
    document.documentElement.setAttribute('data-theme', nextTheme);
  }

  function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    localStorage.setItem(LS.theme, theme);
    applyTheme(theme);
  }

  function useSampleProject() {
    const sample = projects.find((p) => p.id === 'sample-project') ?? sampleProject;
    projectName = sample.name;
    url = sample.sourceUrl;
    numClips = sample.clipCount;
    active = 'generate';
  }

  function continueProject(project) {
    projectName = project.name;
    url = project.sourceUrl;
    numClips = project.clipCount;
    active = 'generate';
  }

  function makeHook() {
    if (!hookTopic.trim()) {
      hookResult = 'Add a topic to generate hooks.';
      return;
    }
    hookResult = `Stop scrolling: ${hookTopic.trim()} mistakes are killing your growth. Here is the 20-second fix.`;
  }

  function savePreset() {
    if (!presetName.trim() || !studioPrompt.trim()) {
      return;
    }
    presets = [
      {
        id: crypto.randomUUID(),
        name: presetName.trim(),
        prompt: studioPrompt.trim(),
        tone: studioTone,
        language: studioLanguage
      },
      ...presets
    ];
    presetName = '';
    persistPresets();
  }

  function applyPreset(preset) {
    studioPrompt = preset.prompt;
    studioTone = preset.tone;
    studioLanguage = preset.language;
  }

  function exportDebugLog() {
    const debug = {
      time: new Date().toISOString(),
      activeScreen: active,
      runLifecycle: $runState.lifecycle,
      progress: $runState.progress,
      lastError: $runState.error,
      recentProjects: recentProjects.slice(0, 3).map((p) => p.name)
    };
    supportLog = JSON.stringify(debug, null, 2);
  }

  async function chooseLocalFile() {
    const picked = await pickLocalVideoFile();
    if (picked) {
      sourceType = 'local';
      mode = 'local';
      url = picked;
    }
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
        clipCount: numClips
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
    <h1>Signal Forge</h1>
    <nav>
      <button class:active={active === 'dashboard'} on:click={() => (active = 'dashboard')}>Dashboard</button>
      <button class:active={active === 'generate'} on:click={() => (active = 'generate')}>Generate</button>
      <button class:active={active === 'library'} on:click={() => (active = 'library')}>Media Library</button>
      <button class:active={active === 'studio'} on:click={() => (active = 'studio')}>Prompt Studio</button>
      <button class:active={active === 'help'} on:click={() => (active = 'help')}>Help & Trust</button>
      <button class:active={active === 'legal'} on:click={() => (active = 'legal')}>Legal</button>
    </nav>
    <button class="theme-toggle" type="button" on:click={toggleTheme}>
      {theme === 'dark' ? 'Switch to Light' : 'Switch to Dark'}
    </button>
  </aside>

  <section class="content">
    {#if active === 'dashboard'}
      <section class="panel hero">
        <p class="eyebrow">Project Dashboard</p>
        <h2>Recent projects and quick resume</h2>
        <p class="sub">Track drafts vs exported and continue where you left off.</p>
      </section>

      <section class="stats-grid">
        <article class="panel card"><h3>Total Projects</h3><p>{projects.length}</p></article>
        <article class="panel card"><h3>Drafts</h3><p>{draftCount}</p></article>
        <article class="panel card"><h3>Exported</h3><p>{exportedCount}</p></article>
      </section>

      <section class="panel">
        <div class="toolbar">
          <input bind:value={projectSearch} placeholder="Search projects" />
        </div>
        <div class="list">
          {#each filteredProjects as project}
            <article class="list-item">
              <div>
                <h3>{project.name}</h3>
                <p>{project.sourceUrl}</p>
                <p class="meta">{project.status} | {new Date(project.updatedAt).toLocaleString()} | {project.clipCount} clips</p>
              </div>
              <button type="button" on:click={() => continueProject(project)}>Continue</button>
            </article>
          {/each}
        </div>
      </section>
    {/if}

    {#if active === 'generate'}
      <section class="panel hero">
        <p class="eyebrow">Desktop Pipeline</p>
        <h2>Generate Shorts</h2>
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

    {#if active === 'library'}
      <section class="panel hero">
        <p class="eyebrow">Media Library</p>
        <h2>Local assets, tags, folders, reuse map</h2>
      </section>

      <section class="panel">
        <div class="toolbar split">
          <input bind:value={mediaSearch} placeholder="Search assets, tags, paths" />
          <select bind:value={selectedType}>
            <option value="all">All types</option>
            <option value="video">Video</option>
            <option value="audio">Audio</option>
            <option value="captions">Captions</option>
            <option value="logo">Logo</option>
          </select>
          <select bind:value={selectedFolder}>
            <option value="all">All folders</option>
            {#each folders as folder}
              <option value={folder}>{folder}</option>
            {/each}
          </select>
        </div>

        <div class="list">
          {#each filteredAssets as asset}
            <article class="list-item">
              <div>
                <h3>{asset.name}</h3>
                <p class="meta">{asset.type} | folder: {asset.folder}</p>
                <p>{asset.path}</p>
                <p class="meta">tags: {asset.tags.join(', ') || 'none'} | used in: {asset.usedIn.join(', ') || 'none'}</p>
              </div>
            </article>
          {/each}
        </div>
      </section>
    {/if}

    {#if active === 'studio'}
      <section class="panel hero">
        <p class="eyebrow">Script & Prompt Studio</p>
        <h2>Presets, hook/title generation, tone and language templates</h2>
      </section>

      <section class="panel">
        <div class="form one-col">
          <label>Hook topic <input bind:value={hookTopic} placeholder="e.g. email marketing for coaches" /></label>
          <button type="button" on:click={makeHook}>Generate Hook</button>
          {#if hookResult}
            <p class="meta">{hookResult}</p>
          {/if}
        </div>
      </section>

      <section class="panel">
        <div class="form one-col">
          <label>Preset name <input bind:value={presetName} placeholder="My preset" /></label>
          <label>Prompt template <textarea bind:value={studioPrompt} rows="4"></textarea></label>
          <label>Tone
            <select bind:value={studioTone}>
              <option value="energetic">energetic</option>
              <option value="educational">educational</option>
              <option value="bold">bold</option>
              <option value="storytelling">storytelling</option>
            </select>
          </label>
          <label>Language template <input bind:value={studioLanguage} /></label>
          <button type="button" on:click={savePreset}>Save Preset</button>
        </div>
      </section>

      <section class="panel">
        <h3>Saved presets</h3>
        <div class="list">
          {#each presets as preset}
            <article class="list-item">
              <div>
                <h4>{preset.name}</h4>
                <p class="meta">tone: {preset.tone} | language: {preset.language}</p>
                <p>{preset.prompt}</p>
              </div>
              <button type="button" on:click={() => applyPreset(preset)}>Apply</button>
            </article>
          {/each}
        </div>
      </section>
    {/if}

    {#if active === 'help'}
      <section class="panel hero">
        <p class="eyebrow">Help & Trust</p>
        <h2>How it works, privacy, support and logs</h2>
      </section>

      <section class="panel">
        <h3>How it works</h3>
        <p>1. Add a source URL. 2. Choose mode and clip settings. 3. Run pipeline. 4. Review exported clips.</p>
        <p class="meta">Progress events stream live so users can track each stage of the generation pipeline.</p>
      </section>

      <section class="panel">
        <h3>Privacy</h3>
        <p>Project history, media library metadata, and prompt presets are stored locally on this machine (browser local storage in this build).</p>
        <p class="meta">No cloud sync is used for these screens unless you implement explicit remote storage later.</p>
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
    {/if}

    {#if active === 'legal'}
      <section class="panel hero">
        <p class="eyebrow">Legal</p>
        <h2>Terms and Conditions</h2>
      </section>

      <section class="panel legal-copy">
        <h3>Terms of Use</h3>
        <p>By using this software, you confirm you have rights to process input media and comply with platform policies and local laws.</p>
        <p>You are responsible for generated outputs, publication decisions, copyright compliance, and any third-party claims.</p>

        <h3>Acceptable Use</h3>
        <p>Do not use the app to generate unlawful, deceptive, harmful, or infringing content. Abuse may result in access suspension for paid services.</p>

        <h3>Privacy Notice</h3>
        <p>Local workspace data for dashboard, library, and prompt presets is stored on-device in this release build. Cloud operations only occur when API mode is selected for generation.</p>

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
    min-height: 100vh;
    padding: var(--space-lg);
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    gap: var(--space-lg);
    position: relative;
  }

  .content {
    display: grid;
    gap: var(--space-md);
    align-content: start;
  }

  .panel {
    background: linear-gradient(160deg, color-mix(in srgb, var(--color-panel-base-1) 95%, transparent), color-mix(in srgb, var(--color-panel-base-2) 95%, transparent));
    border: 1px solid color-mix(in srgb, var(--color-border-soft) 20%, transparent);
    border-radius: var(--radius-xl);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.28);
    padding: 1rem;
    z-index: 1;
    position: relative;
  }

  .hero { padding: var(--space-xl); }
  .eyebrow { margin: 0; font-size: 0.8rem; letter-spacing: 0.12em; text-transform: uppercase; color: var(--color-tertiary); }
  .sub, .meta { color: var(--color-text-tertiary); }
  h1, h2, h3, h4 { margin: 0 0 .45rem; }
  p { margin: .2rem 0; }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: .75rem;
  }
  .sidebar h1 { font-size: 1.25rem; }
  nav { display: grid; gap: .4rem; }
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
  .stats-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--space-md); }
  .card p { font-size: 1.3rem; color: var(--color-secondary); }

  .form { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .7rem; align-items: end; }
  .form.one-col { grid-template-columns: 1fr; }
  label { display: grid; gap: .3rem; }
  .advanced { grid-column: 1 / -1; }
  .advanced summary {
    cursor: pointer;
    color: var(--color-text-tertiary);
    margin-bottom: .45rem;
  }
  textarea { resize: vertical; }

  .toolbar { display: grid; gap: .55rem; margin-bottom: .75rem; }
  .toolbar.split { grid-template-columns: 1.6fr .7fr .7fr; }

  .list { display: grid; gap: .55rem; }
  .list-item {
    background: color-mix(in srgb, var(--color-panel-card) 80%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent);
    border-radius: var(--radius-lg);
    padding: .7rem;
    display: flex;
    justify-content: space-between;
    gap: .8rem;
    align-items: center;
  }

  .row { display: flex; gap: .55rem; flex-wrap: wrap; }
  .picker-row { grid-column: 1 / -1; }

  .status-line { margin: 0 0 .45rem; color: var(--color-text-secondary); }
  .meter { height: 10px; border-radius: var(--radius-pill); background: color-mix(in srgb, var(--color-surface-meter-track) 45%, transparent); overflow: hidden; }
  .meter span { display: block; height: 100%; background: linear-gradient(90deg, var(--color-primary), var(--color-secondary)); transition: width 180ms ease; }

  .cards { display: grid; gap: .5rem; }
  .cards article { background: color-mix(in srgb, var(--color-panel-card) 80%, transparent); border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent); border-radius: var(--radius-lg); padding: .65rem; }

  .orb { position: fixed; width: 320px; height: 320px; border-radius: var(--radius-pill); filter: blur(70px); pointer-events: none; z-index: 0; opacity: .24; }
  .orb-a { top: -80px; left: -90px; background: var(--color-primary); }
  .orb-b { bottom: -70px; right: -60px; background: var(--color-secondary); }

  .legal-copy h3 { margin-top: .8rem; }

  @media (max-width: 900px) {
    .app-shell { grid-template-columns: 1fr; }
    .stats-grid { grid-template-columns: 1fr; }
    .form, .toolbar.split { grid-template-columns: 1fr; }
    .list-item { flex-direction: column; align-items: stretch; }
  }
</style>
