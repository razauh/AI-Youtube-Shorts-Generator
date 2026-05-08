<script>
  import { runState } from '../lib/stores/runState';
  import { runGenerateAndStream } from '../lib/api/tauriClient';

  let url = '';
  let mode = 'api';
  let numClips = 3;
  let aspectRatio = '9:16';
  let format = '720';
  let language = '';
  let outputJson = '';

  async function submitRun() {
    runState.start();

    try {
      const envelope = await runGenerateAndStream(
        {
          youtube_url: url,
          mode,
          num_clips: numClips,
          aspect_ratio: aspectRatio,
          download_format: format,
          language: language.trim() || undefined,
          output_json: outputJson.trim() || undefined
        },
        (event) => runState.onProgress(event)
      );

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

<main class="app">
  <div class="orb orb-a"></div>
  <div class="orb orb-b"></div>

  <section class="panel hero">
    <p class="eyebrow">Desktop Pipeline</p>
    <h1>AI YouTube Shorts Generator</h1>
    <p class="sub">From one long video to ranked short-form clips with live stage updates.</p>
  </section>

  <section class="panel">
    <form class="form" on:submit|preventDefault={submitRun}>
      <label>URL <input aria-label="URL" bind:value={url} required /></label>
      <label>Mode
        <select aria-label="Mode" bind:value={mode}>
          <option value="api">api</option>
          <option value="local">local</option>
        </select>
      </label>
      <label>Num clips <input aria-label="Num clips" type="number" min="1" bind:value={numClips} /></label>
      <label>Aspect ratio <input aria-label="Aspect ratio" bind:value={aspectRatio} /></label>
      <label>Format <input aria-label="Format" bind:value={format} /></label>
      <label>Language <input aria-label="Language" bind:value={language} /></label>
      <label>Output JSON path <input aria-label="Output JSON path" bind:value={outputJson} /></label>
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
      <h2>Error</h2>
      <p>{$runState.error.error}</p>
    </section>
  {/if}

  {#if $runState.result}
    <section class="panel results">
      <h2>Result</h2>
      <p class="meta">Mode: {$runState.result.mode}</p>
      <p class="meta">Source video: {$runState.result.source_video_url}</p>
      <p class="meta">Highlights: {$runState.result.highlights.length} candidates -> kept top {$runState.result.shorts.length}</p>

      <div class="cards">
        {#each $runState.result.shorts as s, i}
          <article>
            <h3>{s.title}</h3>
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
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: var(--font-body-md);
    background: radial-gradient(
      circle at 8% 12%,
      var(--color-canvas-start) 0%,
      var(--color-canvas-mid) 42%,
      var(--color-canvas-end) 100%
    );
    color: var(--color-text-primary);
  }

  .app {
    min-height: 100vh;
    padding: var(--space-xxl) 1.25rem 3rem;
    max-width: 1060px;
    margin: 0 auto;
    position: relative;
    z-index: 1;
  }

  .orb {
    position: fixed;
    width: 320px;
    height: 320px;
    border-radius: var(--radius-pill);
    filter: blur(70px);
    pointer-events: none;
    z-index: 0;
    opacity: 0.24;
  }

  .orb-a {
    top: -80px;
    left: -90px;
    background: var(--color-primary);
  }

  .orb-b {
    bottom: -70px;
    right: -60px;
    background: var(--color-secondary);
  }

  .panel {
    background: linear-gradient(
      160deg,
      color-mix(in srgb, var(--color-panel-base-1) 95%, transparent),
      color-mix(in srgb, var(--color-panel-base-2) 95%, transparent)
    );
    border: 1px solid color-mix(in srgb, var(--color-border-soft) 20%, transparent);
    border-radius: var(--radius-xl);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.28);
    margin-bottom: 1rem;
    position: relative;
    z-index: 2;
  }

  .hero {
    padding: var(--space-xl) var(--space-xl) 1.05rem;
  }

  .eyebrow {
    margin: 0;
    font-size: 0.8rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--color-tertiary);
  }

  h1 {
    margin: 0.3rem 0 0;
    font-size: clamp(1.45rem, 2vw, 2rem);
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .sub {
    margin: 0.45rem 0 0;
    color: var(--color-text-muted);
  }

  .form {
    padding: 1.1rem;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.85rem;
    align-items: end;
  }

  label {
    display: grid;
    gap: var(--space-xs);
    font-size: 0.87rem;
    color: var(--color-text-label);
  }

  label:first-child {
    grid-column: 1 / -1;
  }

  input,
  select {
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-border-medium) 30%, transparent);
    background: color-mix(in srgb, var(--color-surface-input) 80%, transparent);
    color: var(--color-text-primary);
    padding: 0.64rem var(--space-md);
    outline: none;
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }

  input:focus,
  select:focus {
    border-color: var(--color-focus-ring);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-focus-ring) 20%, transparent);
  }

  button {
    grid-column: 1 / -1;
    border: none;
    border-radius: var(--radius-md);
    padding: var(--space-md) 0.88rem;
    background: linear-gradient(90deg, var(--color-primary), var(--color-secondary));
    color: var(--color-on-accent);
    font-size: 0.93rem;
    font-weight: 700;
    cursor: pointer;
    transition: transform 120ms ease, filter 120ms ease;
  }

  button:hover {
    transform: translateY(-1px);
    filter: brightness(1.05);
  }

  .status {
    padding: 0.85rem 1rem 1rem;
  }

  .status-line {
    margin: 0 0 0.45rem;
    color: var(--color-text-secondary);
  }

  .meter {
    height: 10px;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--color-surface-meter-track) 45%, transparent);
    overflow: hidden;
  }

  .meter span {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--color-primary), var(--color-secondary));
    transition: width 180ms ease;
  }

  .error,
  .results {
    padding: 1rem 1.05rem;
  }

  h2 {
    margin: 0 0 0.55rem;
    font-size: 1.02rem;
  }

  .meta {
    margin: 0.2rem 0;
    color: var(--color-text-tertiary);
  }

  .cards {
    margin-top: 0.75rem;
    display: grid;
    gap: 0.65rem;
  }

  article {
    background: color-mix(in srgb, var(--color-panel-card) 80%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 20%, transparent);
    border-radius: var(--radius-lg);
    padding: var(--space-md) 0.78rem;
  }

  article h3 {
    margin: 0 0 0.35rem;
    font-size: 1rem;
  }

  article p {
    margin: 0.22rem 0;
    color: var(--color-text-secondary);
  }

  @media (max-width: 760px) {
    .app {
      padding: var(--space-lg) 0.8rem 1.6rem;
    }

    .form {
      grid-template-columns: 1fr;
    }
  }
</style>
