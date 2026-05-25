<script>
  import {
    approveResetRequest,
    clearAdminConfig,
    disableLicense,
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
  let disableTarget = null;
  let disableReason = '';
  let disableDeactivateBindings = true;

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

  function canDisableLicense(item) {
    return String(item?.entitlement_status || '').toLowerCase() === 'active';
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

  function openDisable(item) {
    disableTarget = item;
    disableReason = '';
    disableDeactivateBindings = true;
  }

  function closeDisable() {
    if (actionBusyFor) return;
    disableTarget = null;
    disableReason = '';
    disableDeactivateBindings = true;
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

  async function submitDisable() {
    if (!disableTarget || actionBusyFor) return;
    const reason = disableReason.trim();
    if (!reason) {
      notice = { kind: 'error', message: 'Reason for disabling is required.' };
      return;
    }
    actionBusyFor = `disable:${disableTarget.license_hash_prefix}`;
    try {
      const result = await disableLicense(disableTarget.license_hash_prefix, reason, disableDeactivateBindings);
      notice = {
        kind: 'success',
        message: `License ${result.license_hash_prefix} is now ${result.entitlement_status}.`
      };
      closeDisable();
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
        {#if resetRequests.length > 0}
          <div class="table-scroll" role="region" aria-label="Reset requests table">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">Request ID</th>
                  <th scope="col">Status</th>
                  <th scope="col">License State</th>
                  <th scope="col">Masked License Key</th>
                  <th scope="col">Purchaser Email</th>
                  <th scope="col">Created</th>
                  <th scope="col">Updated</th>
                  <th scope="col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each resetRequests as item (item.reset_request_id)}
                  <tr>
                    <td>{item.reset_request_id}</td>
                    <td>{item.status}</td>
                    <td>{item.license_state}</td>
                    <td>{item.masked_license_key ?? 'Unavailable'}</td>
                    <td>{item.purchaser_email ?? 'Unavailable'}</td>
                    <td>{formatDate(item.created_at_ms)}</td>
                    <td>{formatDate(item.updated_at_ms)}</td>
                    <td class="row-actions">
                      <button class="secondary" on:click={() => openDetail('Reset Request', item)}>Details</button>
                      {#if item.status === 'pending'}
                        <button on:click={() => openConfirm('approve', item)}>Approve</button>
                        <button class="danger" on:click={() => openConfirm('reject', item)}>Reject</button>
                      {:else}
                        <span class="muted">Decision already final</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {:else if section === 'licenses'}
        <div class="actions">
          <input placeholder="Search email/sale/hash prefix" bind:value={licenseQ} />
          <input placeholder="entitlement_status" bind:value={licenseStatus} />
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && licenses.length === 0}<div class="empty-state">No results.</div>{/if}
        {#if licenses.length > 0}
          <div class="table-scroll" role="region" aria-label="Licenses table">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">License</th>
                  <th scope="col">Purchaser Email</th>
                  <th scope="col">Entitlement Status</th>
                  <th scope="col">Provider</th>
                  <th scope="col">Provider Sale ID</th>
                  <th scope="col">Active Devices</th>
                  <th scope="col">Updated</th>
                  <th scope="col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each licenses as item (item.license_hash_prefix)}
                  <tr>
                    <td>{item.license_hash_prefix}</td>
                    <td>{item.purchaser_email_masked || 'Unavailable'}</td>
                    <td>{item.entitlement_status}</td>
                    <td>{item.provider ?? 'n/a'}</td>
                    <td>{item.provider_sale_id ?? 'n/a'}</td>
                    <td>{item.active_device_count}/{item.inactive_device_count}</td>
                    <td>{formatDate(item.updated_at_ms)}</td>
                    <td class="row-actions">
                      <button class="secondary" on:click={() => openDetail('License', item)}>Details</button>
                      {#if canDisableLicense(item)}
                        <button class="danger" on:click={() => openDisable(item)}>Disable License</button>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {:else if section === 'device_bindings'}
        <div class="actions">
          <input placeholder="Search device/email/hash prefix" bind:value={bindingQ} />
          <select bind:value={bindingStatus}><option value="all">all</option>{#each DEVICE_STATUSES.slice(1) as s}<option value={s}>{s}</option>{/each}</select>
          <button class="secondary" on:click={refreshCurrentSection}>Apply</button>
        </div>
        {#if !loading && deviceBindings.length === 0}<div class="empty-state">No results.</div>{/if}
        {#if deviceBindings.length > 0}
          <div class="table-scroll" role="region" aria-label="Device bindings table">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">Device ID</th>
                  <th scope="col">License</th>
                  <th scope="col">Purchaser Email</th>
                  <th scope="col">Status</th>
                  <th scope="col">Fingerprint Summary</th>
                  <th scope="col">Updated</th>
                  <th scope="col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each deviceBindings as item (item.device_id)}
                  <tr>
                    <td>{item.device_id}</td>
                    <td>{item.license_hash_prefix}</td>
                    <td>{item.purchaser_email_masked || 'Unavailable'}</td>
                    <td>{item.status}</td>
                    <td>{item.fingerprint_summary?.os_name ?? 'n/a'} / {item.fingerprint_summary?.arch ?? 'n/a'}</td>
                    <td>{formatDate(item.updated_at_ms)}</td>
                    <td class="row-actions"><button class="secondary" on:click={() => openDetail('Device Binding', item)}>Details</button></td>
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

{#if confirmAction && confirmRequest}
  <div class="modal-backdrop" role="presentation" on:click|self={closeConfirm}>
    <section class="modal decision-modal" role="dialog" aria-modal="true" aria-labelledby="confirm-dialog-title">
      <header><h2 id="confirm-dialog-title">{confirmAction === 'approve' ? 'Approve reset request?' : 'Reject reset request?'}</h2></header>
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

{#if disableTarget}
  <div class="modal-backdrop" role="presentation" on:click|self={closeDisable}>
    <section class="modal decision-modal" role="dialog" aria-modal="true" aria-labelledby="disable-dialog-title">
      <header><h2 id="disable-dialog-title">Disable License?</h2></header>
      <p>This action may prevent the customer from activating or using the application with this license. This does not delete the license record. The action will be recorded in the audit log.</p>
      <label>Reason for disabling <textarea bind:value={disableReason} rows="4" /></label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={disableDeactivateBindings} />
        Deactivate active device bindings now
      </label>
      <div class="actions">
        <button class="danger" on:click={submitDisable} disabled={Boolean(actionBusyFor) || !disableReason.trim()}>
          {actionBusyFor ? 'Disabling...' : 'Disable License'}
        </button>
        <button class="secondary" on:click={closeDisable} disabled={Boolean(actionBusyFor)}>Cancel</button>
      </div>
    </section>
  </div>
{/if}
