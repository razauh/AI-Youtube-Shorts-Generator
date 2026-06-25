<script>
  import {
    approveDeletionRequest,
    clearAdminConfig,
    listAuditEvents,
    listIdempotencyRecords,
    listDeletionRequests,
    loadAdminConfig,
    loadOverview,
    rejectDeletionRequest,
    saveAdminConfig,
    testAdminConnection
  } from './lib/adminClient';
  import { friendlyAdminError, validateAdminConfig } from './lib/messages';

  const SECTIONS = ['overview', 'delete_requests', 'audit_events', 'idempotency'];
  const DELETION_STATUSES = ['pending', 'approved', 'processing', 'rejected', 'completed', 'failed'];

  let section = 'overview';
  let config = { baseUrl: null, tokenConfigured: false, tokenRedacted: null };
  let configLoaded = false;
  let setupBaseUrl = '';
  let setupToken = '';
  let setupBusy = false;
  let setupMessage = null;

  let loading = false;
  let notice = null;
  let detailItem = null;
  let detailTitle = '';

  let overview = null;
  let deletionFilter = 'pending';
  let deletionRequests = [];
  let auditEventType = '';
  let auditActor = '';
  let auditEvents = [];
  let idempotencyOp = '';
  let idempotencyRecords = [];

  let actionBusyFor = null;
  let confirmAction = null;
  let confirmRequest = null;
  let confirmReason = '';
  let deletionConfirmText = '';


  $: configured = Boolean(config.baseUrl && config.tokenConfigured);

  function isConfigured(value = config) {
    return Boolean(value.baseUrl && value.tokenConfigured);
  }

  function formatDate(value) {
    if (!Number.isFinite(value) || value <= 0) return 'Unavailable';
    return new Date(value).toLocaleString();
  }

  function showError(error, target = 'main') {
    const friendly = friendlyAdminError(error);
    const next = { kind: 'error', message: friendly.message, requestId: friendly.request_id };
    if (target === 'setup') setupMessage = next;
    else notice = next;
  }



  async function bootstrap() {
    try {
      config = await loadAdminConfig();
      setupBaseUrl = config.baseUrl ?? '';
      if (isConfigured(config)) await refreshCurrentSection();
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
      await refreshCurrentSection();
    } catch (error) {
      showError(error, 'setup');
    } finally {
      setupBusy = false;
    }
  }

  async function checkConnection() {
    setupBusy = true;
    setupMessage = null;
    try {
      if (!isConfigured()) {
        const validation = validateAdminConfig(setupBaseUrl, setupToken);
        if (validation) {
          setupMessage = { kind: 'error', message: validation };
          return;
        }
        config = await saveAdminConfig(setupBaseUrl, setupToken);
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
      notice = null;
      setupMessage = { kind: 'info', message: 'Admin configuration cleared.' };
    } catch (error) {
      showError(error, 'setup');
    } finally {
      setupBusy = false;
    }
  }

  async function refreshCurrentSection() {
    if (!isConfigured()) return;
    loading = true;
    notice = null;
    try {
      if (section === 'overview') {
        overview = await loadOverview();
      } else if (section === 'delete_requests') {
        deletionRequests = (await listDeletionRequests(deletionFilter)).requests;
      } else if (section === 'audit_events') {
        auditEvents = (await listAuditEvents({ eventType: auditEventType, actor: auditActor, limit: 30 })).events;
      } else {
        idempotencyRecords = (await listIdempotencyRecords({ op: idempotencyOp, limit: 30 })).records;
      }
    } catch (error) {
      showError(error);
    } finally {
      loading = false;
    }
  }

  async function switchSection(next) {
    section = next;
    await refreshCurrentSection();
  }

  function openDetail(title, item) {
    detailTitle = title;
    detailItem = item;
  }

  function closeDetail() {
    detailItem = null;
    detailTitle = '';
  }

  function openConfirm(action, request) {
    confirmAction = action;
    confirmRequest = request;
    confirmReason = '';
    deletionConfirmText = '';
  }

  function closeConfirm() {
    if (actionBusyFor) return;
    confirmAction = null;
    confirmRequest = null;
    confirmReason = '';
    deletionConfirmText = '';
  }

  async function submitDeletionDecision() {
    if (!confirmAction || !confirmRequest || actionBusyFor) return;
    if (!confirmReason.trim()) {
      notice = { kind: 'error', message: 'Support decision reason is required.' };
      return;
    }
    const requestId = confirmRequest.deletion_request_id;
    actionBusyFor = `${confirmAction}:${requestId}`;
    try {
      const result = confirmAction === 'approve_deletion'
        ? await approveDeletionRequest(requestId, deletionConfirmText, confirmReason)
        : await rejectDeletionRequest(requestId, confirmReason);
      notice = { kind: 'success', message: `Deletion request ${result.deletion_request_id} ${result.status}.` };
      closeConfirm();
      await refreshCurrentSection();
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
      <h1>Worker Operations Console</h1>
    </div>
    {#if configured}
      <div class="config-card">
        <span>Worker</span><strong>{config.baseUrl}</strong>
        <span>Token</span><strong>{config.tokenRedacted ?? '[redacted]'}</strong>
      </div>
    {/if}
  </section>

  {#if !configLoaded}
    <section class="panel centered">Loading admin configuration...</section>
  {:else if !configured}
    <section class="panel setup-panel">
      <h2>Initial Setup</h2>
      <label>Worker API base URL <input bind:value={setupBaseUrl} /></label>
      <label>Admin API token <input bind:value={setupToken} type="password" autocomplete="off" /></label>
      {#if setupMessage}<div class="notice {setupMessage.kind}">{setupMessage.message}</div>{/if}
      <div class="actions">
        <button on:click={saveConfig} disabled={setupBusy}>Save</button>
        <button class="secondary" on:click={checkConnection} disabled={setupBusy}>Test connection</button>
      </div>
    </section>
  {:else}
    <section class="panel toolbar">
      <div class="tabs">
        {#each SECTIONS as tab}
          <button class:active={section === tab} class="secondary" on:click={() => switchSection(tab)} disabled={loading}>
            {tab.replaceAll('_', ' ')}
          </button>
        {/each}
      </div>
      <div class="actions">
        <button class="secondary" on:click={refreshCurrentSection} disabled={loading}>{loading ? 'Refreshing...' : 'Refresh'}</button>
        <button class="ghost" on:click={resetConfig} disabled={setupBusy}>Clear config</button>
      </div>
    </section>

    {#if notice}<div class="notice {notice.kind}">{notice.message}</div>{/if}

    <section class="panel queue-panel">
      {#if section === 'overview'}
        <div class="metrics">
          <article><h3>Total licenses</h3><strong>{overview?.total_licenses ?? 0}</strong></article>
          <article><h3>Recent audit events (24h)</h3><strong>{overview?.recent_audit_events_24h ?? 0}</strong></article>
          <article><h3>Active bindings</h3><strong>{overview?.device_binding_counts?.active ?? 0}</strong></article>
          <article><h3>Pending deletions</h3><strong>{overview?.deletion_request_counts?.pending ?? 0}</strong></article>
        </div>
      {:else if section === 'delete_requests'}
        <div class="actions">
          <label>Status
            <select bind:value={deletionFilter} on:change={refreshCurrentSection}>
              {#each DELETION_STATUSES as s}<option value={s}>{s}</option>{/each}
            </select>
          </label>
        </div>
        {#if !loading && deletionRequests.length === 0}<div class="empty-state">No results.</div>{/if}
        {#if deletionRequests.length > 0}
          <div class="table-scroll" role="region" aria-label="Delete user data requests table">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">Request ID</th>
                  <th scope="col">Status</th>
                  <th scope="col">License</th>
                  <th scope="col">Purchaser Email</th>
                  <th scope="col">Scope</th>
                  <th scope="col">Preview</th>
                  <th scope="col">Created</th>
                  <th scope="col">Updated</th>
                  <th scope="col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each deletionRequests as item (item.deletion_request_id)}
                  <tr>
                    <td>{item.deletion_request_id}</td>
                    <td>{item.status}</td>
                    <td>{item.masked_license_key ?? item.license_hash_prefix ?? 'Unavailable'}</td>
                    <td>{item.purchaser_email ?? 'Unavailable'}</td>
                    <td>{item.requested_scope}</td>
                    <td>
                      {item.deletion_preview?.licenses ?? 0} license /
                      {item.deletion_preview?.device_bindings ?? 0} devices /
                      {item.deletion_preview?.reset_requests ?? 0} resets
                    </td>
                    <td>{formatDate(item.created_at_ms)}</td>
                    <td>{formatDate(item.updated_at_ms)}</td>
                    <td class="row-actions">
                      <button class="secondary" on:click={() => openDetail('Delete User Data Request', item)}>Details</button>
                      {#if item.status === 'pending' || item.status === 'failed'}
                        <button class="danger" on:click={() => openConfirm('approve_deletion', item)}>Approve and Delete</button>
                        {#if item.status === 'pending'}
                          <button class="secondary" on:click={() => openConfirm('reject_deletion', item)}>Reject</button>
                        {/if}
                      {:else}
                        <span class="muted">No action available</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {:else if section === 'audit_events'}
        <div class="actions">
          <input placeholder="event_type" bind:value={auditEventType} />
          <input placeholder="actor" bind:value={auditActor} />
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && auditEvents.length === 0}<div class="empty-state">No results.</div>{/if}
        {#if auditEvents.length > 0}
          <div class="table-scroll" role="region" aria-label="Audit events table">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">Event Type</th>
                  <th scope="col">Actor</th>
                  <th scope="col">Metadata Summary</th>
                  <th scope="col">Created</th>
                  <th scope="col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each auditEvents as item, index (`${item.event_type}:${item.created_at_ms}:${index}`)}
                  <tr>
                    <td>{item.event_type}</td>
                    <td>{item.actor ?? 'unknown'}</td>
                    <td>[redacted summary]</td>
                    <td>{formatDate(item.created_at_ms)}</td>
                    <td class="row-actions"><button class="secondary" on:click={() => openDetail('Audit Event', item)}>Details</button></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {:else}
        <div class="actions">
          <input placeholder="operation name" bind:value={idempotencyOp} />
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && idempotencyRecords.length === 0}<div class="empty-state">No results.</div>{/if}
        {#if idempotencyRecords.length > 0}
          <div class="table-scroll" role="region" aria-label="Idempotency records table">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">Operation</th>
                  <th scope="col">Idempotency Key</th>
                  <th scope="col">Payload Hash</th>
                  <th scope="col">Response Status</th>
                  <th scope="col">Created</th>
                  <th scope="col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each idempotencyRecords as item, index (`${item.op}:${item.created_at_ms}:${index}`)}
                  <tr>
                    <td>{item.op}</td>
                    <td>{item.idempotency_key_prefix}</td>
                    <td>{item.payload_hash_prefix}</td>
                    <td>{item.response_status}</td>
                    <td>{formatDate(item.created_at_ms)}</td>
                    <td class="row-actions"><button class="secondary" on:click={() => openDetail('Idempotency Record', item)}>Details</button></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {/if}
    </section>
  {/if}
</main>

{#if detailItem}
  <div class="modal-backdrop" role="presentation" on:click|self={closeDetail}>
    <section class="modal" role="dialog" aria-modal="true" aria-labelledby="detail-dialog-title">
      <header><h2 id="detail-dialog-title">{detailTitle}</h2><button class="ghost" on:click={closeDetail}>Close</button></header>
      <pre>{JSON.stringify(detailItem, null, 2)}</pre>
    </section>
  </div>
{/if}

{#if confirmRequest && (confirmAction === 'approve_deletion' || confirmAction === 'reject_deletion')}
  <div class="modal-backdrop" role="presentation" on:click|self={closeConfirm}>
    <section class="modal decision-modal" role="dialog" aria-modal="true" aria-labelledby="confirm-dialog-title">
      {#if confirmAction === 'approve_deletion'}
        <header><h2 id="confirm-dialog-title">Approve and delete user data?</h2></header>
        <p>This will delete device bindings and anonymize backend licensing records for the selected request. Type DELETE USER DATA to continue.</p>
        <label>Confirmation <input bind:value={deletionConfirmText} autocomplete="off" /></label>
      {:else if confirmAction === 'reject_deletion'}
        <header><h2 id="confirm-dialog-title">Reject deletion request?</h2></header>
      {/if}
      <label>Support reason <textarea bind:value={confirmReason} rows="4" /></label>
      <div class="actions">
        <button
          class={confirmAction === 'approve_deletion' ? 'danger' : ''}
          on:click={submitDeletionDecision}
          disabled={Boolean(actionBusyFor) || !confirmReason.trim() || (confirmAction === 'approve_deletion' && deletionConfirmText.trim() !== 'DELETE USER DATA')}
        >
          {actionBusyFor ? 'Submitting...' : confirmAction === 'approve_deletion' ? 'Approve and Delete' : 'Confirm reject'}
        </button>
        <button class="secondary" on:click={closeConfirm} disabled={Boolean(actionBusyFor)}>Cancel</button>
      </div>
    </section>
  </div>
{/if}
