<script>
  import {
    approveResetRequest,
    clearAdminConfig,
    listResetRequests,
    loadAdminConfig,
    rejectResetRequest,
    saveAdminConfig,
    testAdminConnection
  } from './lib/adminClient';
  import { friendlyAdminError, validateAdminConfig } from './lib/messages';

  const STATUSES = ['pending', 'approved', 'rejected', 'expired'];

  let config = { baseUrl: null, tokenConfigured: false, tokenRedacted: null };
  let configLoaded = false;
  let setupBaseUrl = '';
  let setupToken = '';
  let setupBusy = false;
  let setupMessage = null;

  let statusFilter = 'pending';
  let requests = [];
  let selectedRequest = null;
  let notice = null;
  let loading = false;
  let actionBusyFor = null;

  let confirmAction = null;
  let confirmRequest = null;
  let confirmReason = '';

  $: configured = Boolean(config.baseUrl && config.tokenConfigured);

  function formatDate(value) {
    if (!Number.isFinite(value) || value <= 0) return 'Unavailable';
    return new Date(value).toLocaleString();
  }

  function statusLabel(value) {
    return value.replaceAll('_', ' ');
  }

  function showError(error, target = 'queue') {
    const friendly = friendlyAdminError(error);
    const next = {
      kind: 'error',
      message: friendly.message,
      requestId: friendly.request_id
    };
    if (target === 'setup') setupMessage = next;
    else notice = next;
  }

  async function bootstrap() {
    try {
      config = await loadAdminConfig();
      setupBaseUrl = config.baseUrl ?? '';
      if (config.baseUrl && config.tokenConfigured) {
        await refreshQueue();
      }
    } catch (error) {
      showError(error, 'setup');
    } finally {
      configLoaded = true;
    }
  }

  async function saveConfig() {
    setupMessage = null;
    const validation = validateAdminConfig(setupBaseUrl, setupToken);
    if (validation) {
      setupMessage = { kind: 'error', message: validation };
      return;
    }
    setupBusy = true;
    try {
      config = await saveAdminConfig(setupBaseUrl, setupToken);
      setupToken = '';
      setupMessage = { kind: 'success', message: 'Admin configuration saved.' };
      await refreshQueue();
    } catch (error) {
      showError(error, 'setup');
    } finally {
      setupBusy = false;
    }
  }

  async function testConnection() {
    setupMessage = null;
    const validation = configured ? null : validateAdminConfig(setupBaseUrl, setupToken);
    if (validation) {
      setupMessage = { kind: 'error', message: validation };
      return;
    }
    setupBusy = true;
    try {
      if (!configured) {
        config = await saveAdminConfig(setupBaseUrl, setupToken);
        setupToken = '';
      }
      await testAdminConnection();
      setupMessage = { kind: 'success', message: 'Connection succeeded.' };
    } catch (error) {
      showError(error, 'setup');
    } finally {
      setupBusy = false;
    }
  }

  async function resetConfig() {
    setupBusy = true;
    try {
      config = await clearAdminConfig();
      setupBaseUrl = '';
      setupToken = '';
      requests = [];
      notice = null;
      setupMessage = { kind: 'info', message: 'Admin configuration cleared.' };
    } catch (error) {
      showError(error, 'setup');
    } finally {
      setupBusy = false;
    }
  }

  async function refreshQueue() {
    if (!config.baseUrl || !config.tokenConfigured) return;
    loading = true;
    notice = null;
    try {
      const data = await listResetRequests(statusFilter);
      requests = data.requests;
    } catch (error) {
      showError(error);
    } finally {
      loading = false;
    }
  }

  async function changeStatus(next) {
    statusFilter = next;
    await refreshQueue();
  }

  function openConfirm(action, request) {
    confirmAction = action;
    confirmRequest = request;
    confirmReason = '';
  }

  function closeConfirm() {
    if (actionBusyFor) return;
    confirmAction = null;
    confirmRequest = null;
    confirmReason = '';
  }

  function stopModalPropagation(event) {
    event.stopPropagation();
  }

  async function submitDecision() {
    if (!confirmAction || !confirmRequest || actionBusyFor) return;
    const requestId = confirmRequest.reset_request_id;
    actionBusyFor = `${confirmAction}:${requestId}`;
    notice = null;
    try {
      const result = confirmAction === 'approve'
        ? await approveResetRequest(requestId, confirmReason)
        : await rejectResetRequest(requestId, confirmReason);
      requests = requests.map((item) =>
        item.reset_request_id === requestId
          ? { ...item, status: result.status, license_state: result.license_state, updated_at_ms: Date.now() }
          : item
      );
      confirmAction = null;
      confirmRequest = null;
      confirmReason = '';
      await refreshQueue();
      notice = {
        kind: 'success',
        message: `Reset request ${result.reset_request_id} ${result.status}.`
      };
    } catch (error) {
      showError(error);
    } finally {
      actionBusyFor = null;
    }
  }

  bootstrap();
</script>

<main class="admin-shell">
  <section class="hero">
    <div>
      <p class="eyebrow">Desktop Admin Console</p>
      <h1>License Reset Review Queue</h1>
      <p class="lede">Review reset requests directly against the configured Worker API. Tokens stay in secure desktop storage and are never displayed after saving.</p>
    </div>
    {#if configured}
      <div class="config-card" aria-label="Saved admin configuration">
        <span>Worker</span>
        <strong>{config.baseUrl}</strong>
        <span>Token</span>
        <strong>{config.tokenRedacted ?? '[redacted]'}</strong>
      </div>
    {/if}
  </section>

  {#if !configLoaded}
    <section class="panel centered">Loading admin configuration...</section>
  {:else if !configured}
    <section class="panel setup-panel" aria-label="Admin setup">
      <h2>Initial Setup</h2>
      <p>Configure the Worker base URL and admin bearer token before opening the reset queue.</p>
      <label>
        Worker API base URL
        <input bind:value={setupBaseUrl} placeholder="https://license-worker.example.com" autocomplete="url" />
      </label>
      <label>
        Admin API token
        <input bind:value={setupToken} placeholder="Paste admin token" type="password" autocomplete="off" />
      </label>
      {#if setupMessage}
        <div class:success={setupMessage.kind === 'success'} class:error={setupMessage.kind === 'error'} class:info={setupMessage.kind === 'info'} class="notice">
          {setupMessage.message}
          {#if setupMessage.requestId}<small>Worker request ID: {setupMessage.requestId}</small>{/if}
        </div>
      {/if}
      <div class="actions">
        <button on:click={saveConfig} disabled={setupBusy}>Save</button>
        <button class="secondary" on:click={testConnection} disabled={setupBusy}>Test connection</button>
      </div>
    </section>
  {:else}
    <section class="toolbar panel" aria-label="Queue controls">
      <div>
        <label for="status-filter">Status filter</label>
        <select id="status-filter" bind:value={statusFilter} on:change={() => changeStatus(statusFilter)} disabled={loading}>
          {#each STATUSES as status}
            <option value={status}>{status}</option>
          {/each}
        </select>
      </div>
      <div class="toolbar-actions">
        <button class="secondary" on:click={refreshQueue} disabled={loading}>{loading ? 'Refreshing...' : 'Refresh'}</button>
        <button class="ghost" on:click={resetConfig} disabled={setupBusy}>Clear config</button>
      </div>
    </section>

    {#if notice}
      <div class:success={notice.kind === 'success'} class:error={notice.kind === 'error'} class:info={notice.kind === 'info'} class="notice wide">
        {notice.message}
        {#if notice.requestId}<small>Worker request ID: {notice.requestId}</small>{/if}
      </div>
    {/if}

    <section class="panel queue-panel">
      <div class="queue-head">
        <h2>{statusLabel(statusFilter)} requests</h2>
        <span>{requests.length} item{requests.length === 1 ? '' : 's'}</span>
      </div>

      {#if loading}
        <div class="centered">Loading reset requests...</div>
      {:else if requests.length === 0}
        <div class="empty-state">No {statusFilter} reset requests.</div>
      {:else}
        <div class="request-table" role="table" aria-label="Reset requests">
          <div class="table-row table-head" role="row">
            <span>Request</span>
            <span>Status</span>
            <span>License</span>
            <span>Purchaser</span>
            <span>Updated</span>
            <span>Actions</span>
          </div>
          {#each requests as request (request.reset_request_id)}
            <div class="table-row" role="row">
              <button class="linklike" on:click={() => (selectedRequest = request)}>{request.reset_request_id}</button>
              <span class="badge {request.status}">{request.status}</span>
              <span>{request.masked_license_key ?? 'Unavailable'} · {request.license_state}</span>
              <span>{request.purchaser_email ?? 'Unavailable'}</span>
              <span>{formatDate(request.updated_at_ms)}</span>
              <span class="row-actions">
                <button class="secondary" on:click={() => (selectedRequest = request)}>Details</button>
                {#if request.status === 'pending'}
                  <button on:click={() => openConfirm('approve', request)} disabled={Boolean(actionBusyFor)}>Approve</button>
                  <button class="danger" on:click={() => openConfirm('reject', request)} disabled={Boolean(actionBusyFor)}>Reject</button>
                {:else}
                  <span class="muted">Decision already final</span>
                {/if}
              </span>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</main>

{#if selectedRequest}
  <div class="modal-backdrop" role="presentation" on:click={() => (selectedRequest = null)}>
    <section class="modal" role="dialog" aria-modal="true" aria-label="Reset request details" on:mousedown={stopModalPropagation}>
      <header>
        <h2>Request details</h2>
        <button class="ghost" on:click={() => (selectedRequest = null)}>Close</button>
      </header>
      <dl class="details-grid">
        <dt>Request ID</dt><dd>{selectedRequest.reset_request_id}</dd>
        <dt>Status</dt><dd>{selectedRequest.status}</dd>
        <dt>License state</dt><dd>{selectedRequest.license_state}</dd>
        <dt>Masked license key</dt><dd>{selectedRequest.masked_license_key ?? 'Unavailable'}</dd>
        <dt>License hash exists</dt><dd>{selectedRequest.has_license_hash ? 'Yes' : 'No'}</dd>
        <dt>Masked purchaser email</dt><dd>{selectedRequest.purchaser_email ?? 'Unavailable'}</dd>
        <dt>Message</dt><dd>{selectedRequest.message}</dd>
        <dt>Created</dt><dd>{formatDate(selectedRequest.created_at_ms)}</dd>
        <dt>Updated</dt><dd>{formatDate(selectedRequest.updated_at_ms)}</dd>
      </dl>
    </section>
  </div>
{/if}

{#if confirmAction && confirmRequest}
  <div class="modal-backdrop" role="presentation" on:click={closeConfirm}>
    <section class="modal decision-modal" role="dialog" aria-modal="true" aria-label="Confirm reset decision" on:mousedown={stopModalPropagation}>
      <header>
        <h2>{confirmAction === 'approve' ? 'Approve reset request?' : 'Reject reset request?'}</h2>
        <button class="ghost" on:click={closeConfirm} disabled={Boolean(actionBusyFor)}>Close</button>
      </header>
      <p>
        This will {confirmAction === 'approve' ? 'unbind active device bindings for' : 'keep the active binding for'}
        <strong>{confirmRequest.reset_request_id}</strong>.
      </p>
      <label>
        Optional reason
        <textarea bind:value={confirmReason} rows="4" placeholder="Internal audit note, optional"></textarea>
      </label>
      <div class="actions">
        <button class={confirmAction === 'reject' ? 'danger' : ''} on:click={submitDecision} disabled={Boolean(actionBusyFor)}>
          {actionBusyFor ? 'Submitting...' : confirmAction === 'approve' ? 'Confirm approve' : 'Confirm reject'}
        </button>
        <button class="secondary" on:click={closeConfirm} disabled={Boolean(actionBusyFor)}>Cancel</button>
      </div>
    </section>
  </div>
{/if}
