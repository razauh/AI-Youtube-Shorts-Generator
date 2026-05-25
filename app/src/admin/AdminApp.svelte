<script>
  import {
    approveResetRequest,
    clearAdminConfig,
    listAuditEvents,
    listDeviceBindings,
    listIdempotencyRecords,
    listLicenses,
    listResetRequests,
    loadAdminConfig,
    loadOverview,
    rejectResetRequest,
    saveAdminConfig,
    testAdminConnection
  } from './lib/adminClient';
  import { friendlyAdminError, validateAdminConfig } from './lib/messages';

  const SECTIONS = ['overview', 'reset_requests', 'licenses', 'device_bindings', 'audit_events', 'idempotency'];
  const STATUSES = ['pending', 'approved', 'rejected', 'expired'];
  const DEVICE_STATUSES = ['all', 'active', 'inactive'];

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
  let resetFilter = 'pending';
  let resetRequests = [];
  let licenseQ = '';
  let licenseStatus = '';
  let licenses = [];
  let bindingQ = '';
  let bindingStatus = 'all';
  let deviceBindings = [];
  let auditEventType = '';
  let auditActor = '';
  let auditEvents = [];
  let idempotencyOp = '';
  let idempotencyRecords = [];

  let actionBusyFor = null;
  let confirmAction = null;
  let confirmRequest = null;
  let confirmReason = '';

  $: configured = Boolean(config.baseUrl && config.tokenConfigured);

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
      if (configured) await refreshCurrentSection();
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
      if (!configured) {
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
    if (!configured) return;
    loading = true;
    notice = null;
    try {
      if (section === 'overview') {
        overview = await loadOverview();
      } else if (section === 'reset_requests') {
        resetRequests = (await listResetRequests(resetFilter)).requests;
      } else if (section === 'licenses') {
        licenses = (await listLicenses({ q: licenseQ, entitlementStatus: licenseStatus, limit: 30 })).licenses;
      } else if (section === 'device_bindings') {
        deviceBindings = (await listDeviceBindings({
          q: bindingQ,
          status: bindingStatus === 'all' ? '' : bindingStatus,
          limit: 30
        })).bindings;
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
  }

  function closeConfirm() {
    if (actionBusyFor) return;
    confirmAction = null;
    confirmRequest = null;
    confirmReason = '';
  }

  async function submitDecision() {
    if (!confirmAction || !confirmRequest || actionBusyFor) return;
    const requestId = confirmRequest.reset_request_id;
    actionBusyFor = `${confirmAction}:${requestId}`;
    try {
      const result = confirmAction === 'approve'
        ? await approveResetRequest(requestId, confirmReason)
        : await rejectResetRequest(requestId, confirmReason);
      notice = { kind: 'success', message: `Reset request ${result.reset_request_id} ${result.status}.` };
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
          <article><h3>Pending resets</h3><strong>{overview?.reset_request_counts?.pending ?? 0}</strong></article>
        </div>
      {:else if section === 'reset_requests'}
        <div class="actions">
          <label>Status
            <select bind:value={resetFilter} on:change={refreshCurrentSection}>
              {#each STATUSES as s}<option value={s}>{s}</option>{/each}
            </select>
          </label>
        </div>
        {#if !loading && resetRequests.length === 0}<div class="empty-state">No results.</div>{/if}
        {#each resetRequests as item (item.reset_request_id)}
          <div class="table-row">
            <span>{item.reset_request_id}</span><span>{item.status}</span><span>{item.masked_license_key ?? 'Unavailable'}</span><span>{item.purchaser_email ?? 'Unavailable'}</span>
            <span>{formatDate(item.updated_at_ms)}</span>
            <span class="row-actions">
              <button class="secondary" on:click={() => openDetail('Reset Request', item)}>Details</button>
              {#if item.status === 'pending'}
                <button on:click={() => openConfirm('approve', item)}>Approve</button>
                <button class="danger" on:click={() => openConfirm('reject', item)}>Reject</button>
              {/if}
            </span>
          </div>
        {/each}
      {:else if section === 'licenses'}
        <div class="actions">
          <input placeholder="Search email/sale/hash prefix" bind:value={licenseQ} />
          <input placeholder="entitlement_status" bind:value={licenseStatus} />
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && licenses.length === 0}<div class="empty-state">No results.</div>{/if}
        {#each licenses as item (item.license_hash_prefix)}
          <div class="table-row">
            <span>{item.license_hash_prefix}</span><span>{item.purchaser_email_masked || 'Unavailable'}</span><span>{item.entitlement_status}</span><span>{item.provider ?? 'n/a'}</span>
            <span>{item.active_device_count}/{item.inactive_device_count}</span>
            <span class="row-actions"><button class="secondary" on:click={() => openDetail('License', item)}>Details</button></span>
          </div>
        {/each}
      {:else if section === 'device_bindings'}
        <div class="actions">
          <input placeholder="Search device/email/hash prefix" bind:value={bindingQ} />
          <select bind:value={bindingStatus}><option value="all">all</option>{#each DEVICE_STATUSES.slice(1) as s}<option value={s}>{s}</option>{/each}</select>
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && deviceBindings.length === 0}<div class="empty-state">No results.</div>{/if}
        {#each deviceBindings as item (item.device_id)}
          <div class="table-row">
            <span>{item.device_id}</span><span>{item.status}</span><span>{item.license_hash_prefix}</span><span>{item.purchaser_email_masked || 'Unavailable'}</span>
            <span>{item.fingerprint_summary?.os_name ?? 'n/a'} / {item.fingerprint_summary?.arch ?? 'n/a'}</span>
            <span class="row-actions"><button class="secondary" on:click={() => openDetail('Device Binding', item)}>Details</button></span>
          </div>
        {/each}
      {:else if section === 'audit_events'}
        <div class="actions">
          <input placeholder="event_type" bind:value={auditEventType} />
          <input placeholder="actor" bind:value={auditActor} />
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && auditEvents.length === 0}<div class="empty-state">No results.</div>{/if}
        {#each auditEvents as item, index (`${item.event_type}:${item.created_at_ms}:${index}`)}
          <div class="table-row">
            <span>{item.event_type}</span><span>{item.actor ?? 'unknown'}</span><span>{formatDate(item.created_at_ms)}</span><span>[redacted summary]</span>
            <span></span>
            <span class="row-actions"><button class="secondary" on:click={() => openDetail('Audit Event', item)}>Details</button></span>
          </div>
        {/each}
      {:else}
        <div class="actions">
          <input placeholder="operation name" bind:value={idempotencyOp} />
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && idempotencyRecords.length === 0}<div class="empty-state">No results.</div>{/if}
        {#each idempotencyRecords as item, index (`${item.op}:${item.created_at_ms}:${index}`)}
          <div class="table-row">
            <span>{item.op}</span><span>{item.idempotency_key_prefix}</span><span>{item.payload_hash_prefix}</span><span>{item.response_status}</span><span>{formatDate(item.created_at_ms)}</span>
            <span class="row-actions"><button class="secondary" on:click={() => openDetail('Idempotency Record', item)}>Details</button></span>
          </div>
        {/each}
      {/if}
    </section>
  {/if}
</main>

{#if detailItem}
  <div class="modal-backdrop" on:click={closeDetail}>
    <section class="modal" on:click|stopPropagation>
      <header><h2>{detailTitle}</h2><button class="ghost" on:click={closeDetail}>Close</button></header>
      <pre>{JSON.stringify(detailItem, null, 2)}</pre>
    </section>
  </div>
{/if}

{#if confirmAction && confirmRequest}
  <div class="modal-backdrop" on:click={closeConfirm}>
    <section class="modal decision-modal" on:click|stopPropagation>
      <header><h2>{confirmAction === 'approve' ? 'Approve reset request?' : 'Reject reset request?'}</h2></header>
      <label>Optional reason <textarea bind:value={confirmReason} rows="4" /></label>
      <div class="actions">
        <button class={confirmAction === 'reject' ? 'danger' : ''} on:click={submitDecision} disabled={Boolean(actionBusyFor)}>
          {actionBusyFor ? 'Submitting...' : confirmAction === 'approve' ? 'Confirm approve' : 'Confirm reject'}
        </button>
        <button class="secondary" on:click={closeConfirm} disabled={Boolean(actionBusyFor)}>Cancel</button>
      </div>
    </section>
  </div>
{/if}
