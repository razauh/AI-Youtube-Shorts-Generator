import { createDevolensKey, blockDevolensKey } from "./devolensBridge.js";
import { DELETION_STATUS, err, json, ok, readForm, readJson, requestId } from "./contracts.js";
import {
  anonymizeLicenseForPrivacyDeletion,
  anonymizeResetRequestsByLicenseHash,
  createDeletionRequest,
  deleteDeviceBindingsByLicenseHash,
  getD1Idempotent,
  getDeletionPreviewByLicenseHash,
  getDeletionRequest,
  putD1Idempotent,
  stableHash,
  writeVerifiedGumroadSale,
  getLicenseByHash,
  writeAuditEvent,
  listDeletionRequestsByStatus,
  updateDeletionRequestStatus,
  sanitizeCompletedDeletionRequest,
  getAdminOverviewCounts,
  listAdminAuditEvents,
  listAdminIdempotencyRecords,
  listLicensesByHashPrefix,
  updateLicenseEntitlementStatus,
  getOpenDeletionRequestByLicenseHash,
  updateDeletionRequestMetadata,
} from "./store.js";

const ADMIN_AUTH_SCOPE = {
  LEGACY: "legacy",
  READ: "read",
  SUPPORT: "support",
};
const adminSupportRateLimits = new Map();

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;
    const method = request.method.toUpperCase();

    if (method === "GET" && path === "/health") {
      return ok({ status: "ok", contract: "v1" });
    }

    if (method === "GET" && path === "/readyz") {
      return handleReadyz(request, env);
    }

    const updateMatch = path.match(/^\/updates\/([^/]+)\/([^/]+)\/([^/]+)$/);
    if (method === "GET" && updateMatch) {
      return handleUpdateCheck(env, {
        target: updateMatch[1],
        arch: updateMatch[2],
        currentVersion: decodeURIComponent(updateMatch[3]),
      });
    }

    if (method === "POST" && path === "/v1/privacy/delete/request") {
      return handleDeletionRequest(request, env);
    }
    if (method === "POST" && path === "/v1/privacy/delete/status") {
      return handleDeletionStatus(request, env);
    }
    if (method === "GET" && path === "/v1/admin/reset/requests") {
      return handleAdminListResetRequests(request, env);
    }
    if (method === "GET" && path === "/v1/admin/overview") {
      return handleAdminOverview(request, env);
    }
    if (method === "GET" && path === "/v1/admin/audit-events") {
      return handleAdminAuditEvents(request, env);
    }
    if (method === "GET" && path === "/v1/admin/idempotency-records") {
      return handleAdminIdempotencyRecords(request, env);
    }
    if (method === "POST" && path === "/v1/admin/reset/approve") {
      return handleAdminResetDecision(request, env);
    }
    if (method === "POST" && path === "/v1/admin/reset/reject") {
      return handleAdminResetDecision(request, env);
    }
    if (method === "GET" && path === "/v1/admin/privacy/delete-requests") {
      return handleAdminListDeletionRequests(request, env);
    }
    if (method === "POST" && path === "/v1/admin/privacy/delete/approve") {
      return handleAdminDeletionApprove(request, env);
    }
    if (method === "POST" && path === "/v1/admin/privacy/delete/reject") {
      return handleAdminDeletionReject(request, env);
    }
    if (method === "POST" && path === "/v1/admin/licenses/disable") {
      return handleAdminDisableLicense(request, env);
    }
    if (method === "POST" && path === "/v1/license/webhooks/gumroad") {
      return handleGumroadWebhook(request, env);
    }

    return err("route_not_found", "Route not found", requestId(), false, 404);
  },
};

function nonEmpty(value) {
  return typeof value === "string" ? value.trim().length > 0 : Boolean(value);
}

async function handleReadyz(request, env) {
  const url = new URL(request.url);
  const deep = url.searchParams.get("deep") === "1";

  const checks = {
    d1: { ok: false, tables: {}, schema: { reset_requests_masked_license_key: false } },
    secrets: {
      ok: false,
      core_ok: false,
      admin_ok: false,
      gumroad_ok: false,
      hash_pepper: nonEmpty(env?.HASH_PEPPER),
      admin_api_token: nonEmpty(env?.ADMIN_API_TOKEN),
      admin_read_token: nonEmpty(env?.ADMIN_READ_TOKEN),
      admin_support_token: nonEmpty(env?.ADMIN_SUPPORT_TOKEN),
      gumroad_access_token: nonEmpty(env?.GUMROAD_ACCESS_TOKEN),
    },
    config: {
      license_contract_version: String(env?.LICENSE_CONTRACT_VERSION || "").trim() || null,
      update_manifest_url_configured: nonEmpty(env?.UPDATE_MANIFEST_URL),
    },
    deep: {
      enabled: deep,
      updater_manifest_ok: null,
    },
  };

  let ready = true;

  checks.secrets.core_ok = checks.secrets.hash_pepper;
  checks.secrets.admin_ok = checks.secrets.admin_read_token && checks.secrets.admin_support_token;
  if (!checks.secrets.admin_ok && checks.secrets.admin_api_token) {
    checks.secrets.admin_ok = true;
  }
  checks.secrets.gumroad_ok = checks.secrets.gumroad_access_token;
  checks.secrets.ok = checks.secrets.core_ok && checks.secrets.admin_ok;
  if (!checks.secrets.ok) ready = false;

  if (!env?.DB) {
    checks.d1.ok = false;
    ready = false;
  } else {
    const requiredTables = [
      "licenses",
      "device_bindings",
      "reset_requests",
      "idempotency_records",
      "audit_events",
      "user_data_deletion_requests",
    ];
    for (const table of requiredTables) {
      try {
        await env.DB.prepare(`SELECT COUNT(*) AS count FROM ${table}`).first();
        checks.d1.tables[table] = true;
      } catch {
        checks.d1.tables[table] = false;
        ready = false;
      }
    }

    if (checks.d1.tables.reset_requests) {
      try {
        await env.DB.prepare("SELECT masked_license_key FROM reset_requests LIMIT 1").first();
        checks.d1.schema.reset_requests_masked_license_key = true;
      } catch {
        checks.d1.schema.reset_requests_masked_license_key = false;
        ready = false;
      }
    }

    checks.d1.ok =
      Object.values(checks.d1.tables).every(Boolean) && checks.d1.schema.reset_requests_masked_license_key;
  }

  if (deep) {
    if (checks.config.update_manifest_url_configured) {
      try {
        const res = await fetch(String(env.UPDATE_MANIFEST_URL).trim());
        checks.deep.updater_manifest_ok = res.ok;
        if (!res.ok) ready = false;
      } catch {
        checks.deep.updater_manifest_ok = false;
        ready = false;
      }
    } else {
      checks.deep.updater_manifest_ok = true;
    }
  }

  return json(
    {
      status: ready ? "ready" : "not_ready",
      contract: "v1",
      checks,
    },
    { status: ready ? 200 : 503 },
  );
}

async function handleUpdateCheck(env, request) {
  const manifestUrl = String(env?.UPDATE_MANIFEST_URL || "").trim();
  if (!manifestUrl) {
    return new Response(null, { status: 204 });
  }

  const current = parseSemver(request.currentVersion);
  if (!current) {
    return new Response(null, { status: 204 });
  }

  const platformKey = platformManifestKey(request.target, request.arch);
  if (!platformKey) {
    return new Response(null, { status: 204 });
  }

  let manifest;
  try {
    const response = await fetch(manifestUrl, {
      headers: { accept: "application/json" },
    });
    if (!response.ok) {
      return err("storage", "Update manifest could not be loaded.", requestId(), true, 503);
    }
    manifest = await response.json();
  } catch {
    return err("storage", "Update manifest could not be loaded.", requestId(), true, 503);
  }

  const latest = parseSemver(manifest?.version);
  if (!latest) {
    return err("storage", "Update manifest has an invalid version.", requestId(), true, 503);
  }

  if (compareSemver(latest, current) <= 0) {
    return new Response(null, { status: 204 });
  }

  const platform = manifest?.platforms?.[platformKey];
  if (!platform) {
    return new Response(null, { status: 204 });
  }

  if (!isHttpsUrl(platform.url) || !isNonEmptyString(platform.signature)) {
    return err("storage", "Update manifest is missing a valid URL or signature.", requestId(), true, 503);
  }

  const body = {
    version: String(manifest.version),
    url: String(platform.url),
    signature: String(platform.signature),
  };
  if (isNonEmptyString(manifest.notes)) {
    body.notes = String(manifest.notes);
  }
  if (isNonEmptyString(manifest.pub_date)) {
    body.pub_date = String(manifest.pub_date);
  }

  return json(body, {
    headers: {
      "cache-control": "public, max-age=300",
    },
  });
}

function platformManifestKey(target, arch) {
  const normalizedTarget = normalizeTarget(target);
  const normalizedArch = normalizeArch(arch);
  if (!normalizedTarget || !normalizedArch) {
    return null;
  }
  return `${normalizedTarget}-${normalizedArch}`;
}

function normalizeTarget(target) {
  const value = String(target || "").toLowerCase();
  if (["windows", "linux", "darwin"].includes(value)) {
    return value;
  }
  if (value === "macos") {
    return "darwin";
  }
  return null;
}

function normalizeArch(arch) {
  const value = String(arch || "").toLowerCase();
  if (value === "x64" || value === "amd64") {
    return "x86_64";
  }
  if (["x86_64", "aarch64", "i686", "armv7"].includes(value)) {
    return value;
  }
  return null;
}

function parseSemver(version) {
  const value = String(version || "").trim().replace(/^v/i, "");
  const match = value.match(/^(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$/);
  if (!match) {
    return null;
  }
  return match.slice(1, 4).map((part) => Number.parseInt(part, 10));
}

function compareSemver(left, right) {
  for (let i = 0; i < 3; i += 1) {
    if (left[i] > right[i]) return 1;
    if (left[i] < right[i]) return -1;
  }
  return 0;
}

function isHttpsUrl(value) {
  try {
    return new URL(String(value)).protocol === "https:";
  } catch {
    return false;
  }
}

function isNonEmptyString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

async function handleDeletionRequest(request, env) {
  const rid = requestId();
  const idem = request.headers.get("x-idempotency-key");
  if (!idem) {
    return err("bad_request", "Missing required header: X-Idempotency-Key", rid, false, 400);
  }
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const body = await readJson(request);
  if (!body || !body.license_key || body.confirmation !== "DELETE" || !body.timestamp_ms) {
    return err("invalid_deletion_request", "Invalid deletion request payload.", rid, false, 400);
  }
  if (body.purchaser_email && !String(body.purchaser_email).includes("@")) {
    return err("invalid_purchase_email", "Purchaser email format is invalid.", rid, false, 400);
  }

  const normalizedLicenseKey = normalizeLicenseKey(body.license_key);
  const licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);
  const lookupToken = `dlk_${(await sha256Hex(`${env?.HASH_PEPPER || ""}:${idem}:${licenseKeyHash}`)).slice(0, 32)}`;
  const lookupTokenHash = await sha256Hex(lookupToken);
  const payloadHash = stableHash({
    license_key: normalizeLicenseKey(body.license_key),
    purchaser_email: body.purchaser_email ? String(body.purchaser_email).trim().toLowerCase() : null,
    confirmation: body.confirmation,
    app_version: body.app_version || null,
    timestamp_ms: body.timestamp_ms,
  });
  const replay = await getD1Idempotent(env.DB, "privacy_delete_request", idem);
  if (replay) {
    if (replay.payload_hash !== payloadHash) {
      return err(
        "invalid_transition",
        "Idempotency key reuse does not match original request payload.",
        rid,
        false,
        409,
      );
    }
    const replayBody = withDeletionLookupToken(replay.response_body, lookupToken);
    return new Response(replayBody, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const now = Date.now();
  const license = await getLicenseByHash(env.DB, licenseKeyHash);
  const suppliedEmail = body.purchaser_email ? String(body.purchaser_email).trim() : null;
  const storedEmail = license?.purchaser_email ? String(license.purchaser_email) : null;
  if (suppliedEmail && storedEmail && suppliedEmail.toLowerCase() !== storedEmail.toLowerCase()) {
    return err("invalid_purchase_email", "Purchaser email does not match this license.", rid, false, 400);
  }

  const open = await getOpenDeletionRequestByLicenseHash(env.DB, licenseKeyHash);
  if (open) {
    return err("invalid_transition", "A deletion request is already open for this license.", rid, false, 409);
  }

  const requestIdValue = `del_${crypto.randomUUID().slice(0, 12)}`;
  const purchaserEmail = storedEmail || suppliedEmail;
  const response = ok({
    request_id: requestIdValue,
    lookup_token: lookupToken,
    status: "pending",
    message: "Deletion request submitted for admin review.",
  });
  const idempotentResponse = ok({
    request_id: requestIdValue,
    status: "pending",
    message: "Deletion request submitted for admin review.",
  });
  const idempotentResponseBody = await idempotentResponse.clone().text();

  try {
    await createDeletionRequest(env.DB, {
      requestId: requestIdValue,
      lookupTokenHash,
      licenseKeyHash,
      maskedLicenseKey: maskLicenseKey(normalizedLicenseKey),
      purchaserEmail,
      purchaserEmailMasked: maskEmail(purchaserEmail),
      status: "pending",
      requestedScope: "backend_licensing_data",
      requestMetadataJson: JSON.stringify({
        app_version: asOptionalString(body.app_version),
        has_purchase_email: Boolean(purchaserEmail),
        license_match: Boolean(license),
      }),
      createdAtMs: now,
      updatedAtMs: now,
    });
    await writeAuditEvent(
      env.DB,
      "user_data_deletion_requested",
      "desktop_client",
      JSON.stringify({
        request_id: requestIdValue,
        has_license_hash: true,
        has_purchase_email: Boolean(purchaserEmail),
        license_match: Boolean(license),
      }),
      now,
    );
    await putD1Idempotent(
      env.DB,
      "privacy_delete_request",
      idem,
      payloadHash,
      { status: response.status, body: idempotentResponseBody },
      now,
    );
  } catch {
    return err("storage", "Failed to persist deletion request state.", rid, true, 503);
  }

  return response;
}

async function handleDeletionStatus(request, env) {
  const rid = requestId();
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  const body = await readJson(request);
  const reqId = body?.request_id;
  const lookupToken = body?.lookup_token;
  if (!reqId || !lookupToken) {
    return err("bad_request", "Invalid deletion status payload.", rid, false, 400);
  }

  const deletion = await getDeletionRequest(env.DB, reqId);
  if (!deletion) {
    return err("deletion_request_not_found", "Deletion request was not found.", rid, false, 404);
  }
  const tokenHash = await sha256Hex(String(lookupToken));
  if (tokenHash !== deletion.lookup_token_hash) {
    return err("invalid_deletion_lookup_token", "Deletion status token is invalid.", rid, false, 401);
  }
  if (!DELETION_STATUS.has(deletion.status)) {
    return err("serialization", "Stored deletion status is invalid.", rid, false, 503);
  }

  return ok(deletionStatusView(deletion));
}

async function handleAdminListResetRequests(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
  if (auth) return auth;
  return deprecatedAdminResetResponse(rid);
}

async function handleAdminResetDecision(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
  if (auth) return auth;
  return deprecatedAdminResetResponse(rid);
}

function deprecatedAdminResetResponse(rid) {
  return err(
    "gone",
    "Admin reset approval routes are deprecated. Deactivate devices through the Devolens-backed app flow instead.",
    rid,
    false,
    410,
  );
}

async function handleAdminListDeletionRequests(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid, ADMIN_AUTH_SCOPE.SUPPORT);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const url = new URL(request.url);
  const status = url.searchParams.get("status") || "pending";
  if (!DELETION_STATUS.has(status)) {
    return err("bad_request", "Invalid deletion request status filter.", rid, false, 400);
  }

  try {
    const rows = normalizeD1Results(await listDeletionRequestsByStatus(env.DB, status));
    const requests = [];
    for (const row of rows) {
      const preview = row.license_key_hash
        ? await getDeletionPreviewByLicenseHash(env.DB, row.license_key_hash)
        : { licenses: 0, device_bindings: 0, reset_requests: 0 };
      requests.push(adminDeletionView(row, preview));
    }
    return ok({ requests });
  } catch {
    return err("storage", "Failed to load deletion requests.", rid, true, 503);
  }
}

async function handleAdminDeletionReject(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid, ADMIN_AUTH_SCOPE.SUPPORT);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const idem = request.headers.get("x-idempotency-key");
  if (!idem) {
    return err("bad_request", "Missing required header: X-Idempotency-Key", rid, false, 400);
  }
  const body = await readJson(request);
  const requestIdValue = body?.request_id;
  if (!requestIdValue) {
    return err("bad_request", "Invalid admin deletion decision payload.", rid, false, 400);
  }
  const supportContext = requireSupportDecisionContext(request, body, rid);
  if (supportContext instanceof Response) return supportContext;
  const rateLimit = enforceAdminSupportRateLimit(request, env, "privacy_delete_reject", rid);
  if (rateLimit) return rateLimit;

  const payloadHash = stableHash({ decision: "rejected", request_id: requestIdValue, reason: supportContext.reason });
  const replay = await getD1Idempotent(env.DB, "admin_privacy_delete_reject", idem);
  if (replay) {
    if (replay.payload_hash !== payloadHash) {
      return err("invalid_transition", "Idempotency key reuse does not match original request payload.", rid, false, 409);
    }
    return new Response(replay.response_body, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const deletion = await getDeletionRequest(env.DB, requestIdValue);
  if (!deletion) {
    return err("deletion_request_not_found", "Deletion request was not found.", rid, false, 404);
  }
  if (deletion.status !== "pending") {
    return err("invalid_transition", "Deletion request cannot be rejected from its current state.", rid, false, 409);
  }

  const now = Date.now();
  try {
    await updateDeletionRequestStatus(env.DB, {
      requestId: deletion.request_id,
      status: "rejected",
      updatedAtMs: now,
      decidedAtMs: now,
      errorCode: null,
      errorMessageSafe: null,
    });
    await writeAuditEvent(
      env.DB,
      "user_data_deletion_rejected",
      supportContext.actor,
      JSON.stringify({
        request_id: deletion.request_id,
        has_license_hash: Boolean(deletion.license_key_hash),
        reason_present: true,
        reason: redactAuditText(supportContext.reason),
      }),
      now,
    );
  } catch {
    return err("storage", "Failed to persist deletion rejection.", rid, true, 503);
  }

  const response = ok({
    deletion_request_id: deletion.request_id,
    status: "rejected",
    deletion_summary: null,
  });
  const responseBody = await response.clone().text();
  await putD1Idempotent(
    env.DB,
    "admin_privacy_delete_reject",
    idem,
    payloadHash,
    { status: response.status, body: responseBody },
    now,
  );
  return response;
}

async function handleAdminDeletionApprove(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid, ADMIN_AUTH_SCOPE.SUPPORT);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const idem = request.headers.get("x-idempotency-key");
  if (!idem) {
    return err("bad_request", "Missing required header: X-Idempotency-Key", rid, false, 400);
  }
  const body = await readJson(request);
  const requestIdValue = body?.request_id;
  if (!requestIdValue || body.confirmation !== "DELETE USER DATA") {
    return err("bad_request", "Deletion approval requires request_id and confirmation.", rid, false, 400);
  }
  const supportContext = requireSupportDecisionContext(request, body, rid);
  if (supportContext instanceof Response) return supportContext;
  const rateLimit = enforceAdminSupportRateLimit(request, env, "privacy_delete_approve", rid);
  if (rateLimit) return rateLimit;

  const payloadHash = stableHash({
    decision: "approved",
    request_id: requestIdValue,
    confirmation: body.confirmation,
    reason: supportContext.reason,
  });
  const replay = await getD1Idempotent(env.DB, "admin_privacy_delete_approve", idem);
  if (replay) {
    if (replay.payload_hash !== payloadHash) {
      return err("invalid_transition", "Idempotency key reuse does not match original request payload.", rid, false, 409);
    }
    return new Response(replay.response_body, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const deletion = await getDeletionRequest(env.DB, requestIdValue);
  if (!deletion) {
    return err("deletion_request_not_found", "Deletion request was not found.", rid, false, 404);
  }
  if (!["pending", "failed", "approved", "processing"].includes(deletion.status)) {
    return err("invalid_transition", "Deletion request cannot be approved from its current state.", rid, false, 409);
  }
  if (!deletion.license_key_hash) {
    return err("invalid_transition", "Deletion request cannot be approved without license context.", rid, false, 409);
  }

  const now = Date.now();
  let summary;
  let phaseForError = "start";
  try {
    const meta = safeJsonObject(deletion.request_metadata_json);
    const attempt = Number(meta.attempt || 0) + 1;
    let phase = typeof meta.phase === "string" ? meta.phase : "start";
    phaseForError = phase;

    const writePhase = async (nextPhase) => {
      const next = { ...meta, attempt, phase: nextPhase, phase_updated_at_ms: Date.now() };
      await updateDeletionRequestMetadata(env.DB, {
        requestId: deletion.request_id,
        updatedAtMs: Date.now(),
        requestMetadataJson: JSON.stringify(next),
      });
      phase = nextPhase;
      phaseForError = nextPhase;
    };
    const maybeFail = (phaseName) => {
      phaseForError = phaseName;
      if (String(env?.DELETION_FAIL_PHASE || "").trim() === phaseName) {
        throw new Error(`forced_failure:${phaseName}`);
      }
    };

    if (phase === "start") {
      summary = await getDeletionPreviewByLicenseHash(env.DB, deletion.license_key_hash);
      await writePhase("previewed");
    } else {
      summary = await getDeletionPreviewByLicenseHash(env.DB, deletion.license_key_hash);
    }

    if (["start", "previewed"].includes(phase) || deletion.status === "pending" || deletion.status === "failed") {
      await updateDeletionRequestStatus(env.DB, {
        requestId: deletion.request_id,
        status: "approved",
        updatedAtMs: Date.now(),
        decidedAtMs: deletion.decided_at_ms ? null : Date.now(),
        errorCode: null,
        errorMessageSafe: null,
      });
      await writePhase("approved");
    }

    if (["start", "previewed", "approved"].includes(phase) || deletion.status === "approved") {
      await updateDeletionRequestStatus(env.DB, {
        requestId: deletion.request_id,
        status: "processing",
        updatedAtMs: Date.now(),
        errorCode: null,
        errorMessageSafe: null,
      });
      await writePhase("processing");
    }

    if (!["device_bindings_deleted", "reset_requests_anonymized", "license_anonymized", "completed"].includes(phase)) {
      maybeFail("delete_device_bindings");
      await deleteDeviceBindingsByLicenseHash(env.DB, deletion.license_key_hash);
      await writePhase("device_bindings_deleted");
    }

    if (!["reset_requests_anonymized", "license_anonymized", "completed"].includes(phase)) {
      maybeFail("anonymize_reset_requests");
      await anonymizeResetRequestsByLicenseHash(env.DB, deletion.license_key_hash, now);
      await writePhase("reset_requests_anonymized");
    }

    if (!["license_anonymized", "completed"].includes(phase)) {
      maybeFail("anonymize_license");
      await anonymizeLicenseForPrivacyDeletion(env.DB, deletion.license_key_hash, now);
      await writePhase("license_anonymized");
    }
    const completedSummary = {
      ...summary,
      action: "backend_licensing_data_deleted_or_anonymized",
    };
    maybeFail("mark_completed");
    await updateDeletionRequestStatus(env.DB, {
      requestId: deletion.request_id,
      status: "completed",
      updatedAtMs: now,
      completedAtMs: now,
      summaryJson: JSON.stringify(completedSummary),
      errorCode: null,
      errorMessageSafe: null,
    });
    await writePhase("completed");
    await sanitizeCompletedDeletionRequest(env.DB, deletion.request_id, now);
    await writeAuditEvent(
      env.DB,
      "user_data_deletion_completed",
      supportContext.actor,
      JSON.stringify({
        request_id: deletion.request_id,
        reason_present: true,
        reason: redactAuditText(supportContext.reason),
        summary: completedSummary,
      }),
      now,
    );
    summary = completedSummary;
  } catch {
    await updateDeletionRequestStatus(env.DB, {
      requestId: deletion.request_id,
      status: "failed",
      updatedAtMs: Date.now(),
      errorCode: "storage",
      errorMessageSafe: `Deletion execution failed at phase=${phaseForError}. Retry after checking Worker storage.`,
    });
    return err("storage", "Failed to execute deletion request.", rid, true, 503);
  }

  const response = ok({
    deletion_request_id: deletion.request_id,
    status: "completed",
    deletion_summary: summary,
  });
  const responseBody = await response.clone().text();
  await putD1Idempotent(
    env.DB,
    "admin_privacy_delete_approve",
    idem,
    payloadHash,
    { status: response.status, body: responseBody },
    now,
  );
  return response;
}

async function handleAdminOverview(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid, ADMIN_AUTH_SCOPE.READ);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  try {
    return ok(await getAdminOverviewCounts(env.DB));
  } catch {
    return err("storage", "Failed to load overview.", rid, true, 503);
  }
}

async function handleAdminDisableLicense(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
  if (auth) return auth;
  return err(
    "gone",
    "Direct license disabling is deprecated. Please manage licenses directly in the Devolens management portal.",
    rid,
    false,
    410,
  );
}

async function handleAdminAuditEvents(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid, ADMIN_AUTH_SCOPE.READ);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  const url = new URL(request.url);
  const eventType = asOptionalString(url.searchParams.get("event_type"));
  const actor = asOptionalString(url.searchParams.get("actor"));
  const limit = parseLimit(url.searchParams.get("limit"), 25, 100);
  if (!limit.ok) return err("bad_request", limit.message, rid, false, 400);
  try {
    const rows = await listAdminAuditEvents(env.DB, {
      eventType,
      actor,
      limit: limit.value,
    });
    return ok({
      events: rows.map((row) => ({
        event_type: row.event_type,
        actor: row.actor || null,
        created_at_ms: row.created_at_ms,
        metadata_summary: summarizeAuditMetadata(row.metadata_json),
      })),
    });
  } catch {
    return err("storage", "Failed to load audit events.", rid, true, 503);
  }
}

async function handleAdminIdempotencyRecords(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid, ADMIN_AUTH_SCOPE.READ);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  const url = new URL(request.url);
  const op = asOptionalString(url.searchParams.get("op"));
  const limit = parseLimit(url.searchParams.get("limit"), 25, 100);
  if (!limit.ok) return err("bad_request", limit.message, rid, false, 400);
  try {
    const rows = await listAdminIdempotencyRecords(env.DB, { op, limit: limit.value });
    return ok({
      records: rows.map((row) => ({
        op: row.op,
        idempotency_key_prefix: keyPrefix(row.idempotency_key),
        payload_hash_prefix: hashPrefix(row.payload_hash),
        response_status: row.response_status,
        response_body_size: String(row.response_body || "").length,
        created_at_ms: row.created_at_ms,
      })),
    });
  } catch {
    return err("storage", "Failed to load idempotency records.", rid, true, 503);
  }
}

async function handleGumroadWebhook(request, env) {
  const rid = requestId();
  const prepared = await prepareGumroadWebhookRequest(request, env, rid);
  if (prepared instanceof Response) return prepared;

  const replay = await resolveGumroadWebhookReplay(env.DB, prepared, rid);
  if (replay) return replay;

  const verification = await verifyGumroadSale({
    saleId: prepared.saleId,
    productId: prepared.productId,
    email: prepared.email,
    token: env?.GUMROAD_ACCESS_TOKEN,
  });

  if (!verification.ok) {
    await handleVerifiedGumroadTerminalSale(env, prepared.saleId, verification);

    return err(
      verification.code,
      verification.message,
      rid,
      verification.retryable,
      verification.status,
    );
  }

  if (!verification.sale.license_key) {
    return err(
      "invalid_transition",
      "Verified Gumroad sale does not include a license key.",
      rid,
      false,
      409,
    );
  }

  const normalizedLicenseKey = normalizeLicenseKey(verification.sale.license_key);
  const devolensResult = await provisionVerifiedGumroadSaleInDevolens(env, normalizedLicenseKey);
  if (!devolensResult.ok) {
    return err(
      devolensResult.code,
      devolensResult.message,
      rid,
      devolensResult.retryable,
      devolensResult.status,
    );
  }

  const response = ok({
    accepted: true,
    provider: "gumroad",
    sale_id: prepared.saleId,
    verified: true,
  });
  const responseBody = await response.clone().text();

  try {
    await persistVerifiedGumroadSaleMapping(env, prepared, verification.sale, normalizedLicenseKey, {
      status: response.status,
      body: responseBody,
    });
  } catch {
    return err("storage", "Failed to persist verified Gumroad sale.", rid, true, 503);
  }

  return response;
}

async function prepareGumroadWebhookRequest(request, env, rid) {
  const contentType = request.headers.get("content-type") || "";
  if (!contentType.toLowerCase().includes("application/x-www-form-urlencoded")) {
    return err(
      "bad_request",
      "Gumroad Ping must use application/x-www-form-urlencoded payloads.",
      rid,
      false,
      400,
    );
  }

  const body = await readForm(request);
  if (!body || !body.sale_id || !body.product_id || !body.email) {
    return err("bad_request", "Invalid webhook payload", rid, false, 400);
  }
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  return {
    body,
    saleId: String(body.sale_id),
    productId: String(body.product_id),
    email: String(body.email),
    payloadHash: stableHash(body),
  };
}

async function resolveGumroadWebhookReplay(db, prepared, rid) {
  const replay = await getD1Idempotent(db, "gumroad_webhook", prepared.saleId);
  if (!replay) return null;

  if (replay.payload_hash !== prepared.payloadHash) {
    return err(
      "invalid_transition",
      "Webhook replay does not match original Gumroad payload.",
      rid,
      false,
      409,
    );
  }

  return new Response(replay.response_body, {
    status: replay.response_status,
    headers: { "content-type": "application/json; charset=utf-8" },
  });
}

async function handleVerifiedGumroadTerminalSale(env, saleId, verification) {
  if (verification.code !== "invalid_transition" || !verification.license_key) return;

  const normalizedLicenseKey = normalizeLicenseKey(verification.license_key);
  const licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);

  if (hasDevolensWebhookConfig(env)) {
    await blockDevolensKey(env, normalizedLicenseKey);
  }

  const now = Date.now();
  try {
    await updateLicenseEntitlementStatus(env.DB, licenseKeyHash, "disabled", now);
    await writeAuditEvent(
      env.DB,
      "license_disabled",
      "gumroad",
      JSON.stringify({
        sale_id: saleId,
        reason: "refunded_or_disputed",
      }),
      now,
    );
  } catch {
    // Keep terminal Gumroad verification failures non-retryable even if compatibility mapping fails.
  }
}

async function provisionVerifiedGumroadSaleInDevolens(env, normalizedLicenseKey) {
  if (!hasDevolensWebhookConfig(env)) return { ok: true };
  return createDevolensKey(env, normalizedLicenseKey);
}

async function persistVerifiedGumroadSaleMapping(env, prepared, sale, normalizedLicenseKey, responseRecord) {
  const now = Date.now();
  const providerSaleId = String(sale.id ?? sale.sale_id ?? prepared.saleId);
  const licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);

  await writeVerifiedGumroadSale(env.DB, {
    licenseKeyHash,
    purchaserEmail: String(sale.email),
    providerSaleId,
    metadataJson: JSON.stringify({
      sale_id: providerSaleId,
      product_id: String(sale.product_id),
      email_masked: maskEmail(sale.email),
      verified: true,
      mapping_only: true,
    }),
    updatedAtMs: now,
  });
  await putD1Idempotent(
    env.DB,
    "gumroad_webhook",
    prepared.saleId,
    prepared.payloadHash,
    responseRecord,
    now,
  );
}

function hasDevolensWebhookConfig(env) {
  return Boolean((env?.DEVOLENS_WEBHOOK_TOKEN || env?.DEVOLENS_ACCESS_TOKEN) && env?.DEVOLENS_PRODUCT_ID);
}

async function verifyGumroadSale({ saleId, productId, email, token }) {
  if (!token) {
    return {
      ok: false,
      code: "unauthorized",
      message: "Gumroad access token is not configured.",
      retryable: false,
      status: 401,
    };
  }

  const url = new URL(`https://api.gumroad.com/v2/sales/${encodeURIComponent(saleId)}`);
  url.searchParams.set("access_token", token);

  let response;
  try {
    response = await fetch(url.toString(), { method: "GET" });
  } catch {
    return {
      ok: false,
      code: "worker_unreachable",
      message: "Gumroad verification request failed.",
      retryable: true,
      status: 503,
    };
  }

  let payload = null;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok || !payload) {
    return {
      ok: false,
      code: "worker_unreachable",
      message: "Gumroad verification service returned an invalid response.",
      retryable: true,
      status: 503,
    };
  }

  if (payload.success === false) {
    const remoteMessage = String(payload.message || "").trim();
    if (remoteMessage === "The sale was not found.") {
      return {
        ok: false,
        code: "not_found",
        message: "Gumroad sale was not found for the provided sale_id.",
        retryable: false,
        status: 404,
      };
    }
    return {
      ok: false,
      code: "unauthorized",
      message: remoteMessage || "Gumroad verification was rejected.",
      retryable: false,
      status: response.status || 401,
    };
  }

  const sale = payload.sale || payload;
  const remoteSaleId = sale.id ?? sale.sale_id;
  const remoteProductId = sale.product_id;
  const remoteEmail = sale.email;
  const refunded = Boolean(sale.refunded);
  const disputed = Boolean(sale.disputed ?? sale.chargebacked);

  const missingFields = [];
  if (!remoteSaleId) missingFields.push("sale.id");
  if (!remoteProductId) missingFields.push("sale.product_id");
  if (!remoteEmail) missingFields.push("sale.email");

  if (missingFields.length > 0) {
    return {
      ok: false,
      code: "serialization",
      message: `Gumroad verification payload is missing required sale fields: ${missingFields.join(", ")}.`,
      retryable: false,
      status: 503,
    };
  }

  const mismatchedFields = [];
  if (String(remoteSaleId) !== String(saleId)) mismatchedFields.push("sale_id");
  if (String(remoteProductId) !== String(productId)) mismatchedFields.push("product_id");
  if (String(remoteEmail).toLowerCase() !== String(email).toLowerCase()) mismatchedFields.push("email");

  if (mismatchedFields.length > 0) {
    return {
      ok: false,
      code: "unauthorized",
      message: `Gumroad verification mismatch for: ${mismatchedFields.join(", ")}.`,
      retryable: false,
      status: 401,
    };
  }

  if (refunded || disputed) {
    return {
      ok: false,
      code: "invalid_transition",
      message: "Sale is refunded or disputed and is not eligible for activation.",
      retryable: false,
      status: 409,
      license_key: sale.license_key,
    };
  }

  return { ok: true, sale };
}

function normalizeLicenseKey(value) {
  return String(value).trim().toUpperCase();
}

function maskLicenseKey(licenseKey) {
  const tail = String(licenseKey).slice(-4).toUpperCase();
  return `••••-${tail}`;
}

function maskEmail(value) {
  const email = String(value || "");
  const [local, domain] = email.split("@");
  if (!local || !domain) return "";
  const prefix = local.slice(0, 1);
  return `${prefix}***@${domain}`;
}

function hashPrefix(value) {
  const input = String(value || "");
  if (!input) return null;
  return input.slice(0, 12);
}

function keyPrefix(value) {
  const input = String(value || "");
  if (!input) return null;
  return `${input.slice(0, 12)}...`;
}

function asOptionalString(value) {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function safeJsonObject(value) {
  if (!value) return {};
  try {
    const parsed = JSON.parse(String(value));
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

function parseLimit(value, fallback, max) {
  if (value == null || String(value).trim() === "") {
    return { ok: true, value: fallback };
  }
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0 || parsed > max) {
    return { ok: false, message: `limit must be an integer between 1 and ${max}.` };
  }
  return { ok: true, value: parsed };
}

function summarizeFingerprint(fingerprintJson) {
  try {
    const parsed = JSON.parse(String(fingerprintJson || "{}"));
    return {
      os_name: asOptionalString(parsed.os_name) || null,
      platform_family: asOptionalString(parsed.platform_family) || null,
      arch: asOptionalString(parsed.arch) || null,
      app_version: asOptionalString(parsed.app_version) || null,
    };
  } catch {
    return {
      os_name: null,
      platform_family: null,
      arch: null,
      app_version: null,
    };
  }
}

function summarizeAuditMetadata(metadataJson) {
  let parsed = {};
  try {
    parsed = JSON.parse(String(metadataJson || "{}"));
  } catch {
    return { invalid_json: true };
  }
  const summary = {};
  for (const [key, value] of Object.entries(parsed)) {
    if (key.toLowerCase().includes("email")) {
      summary[key] = maskEmail(value);
      continue;
    }
    if (typeof value === "string") {
      summary[key] = value.length > 80 ? `${value.slice(0, 80)}...` : value;
      continue;
    }
    if (typeof value === "number" || typeof value === "boolean") {
      summary[key] = value;
      continue;
    }
    if (value === null) {
      summary[key] = null;
      continue;
    }
    summary[key] = "[redacted]";
  }
  return summary;
}

function bearerToken(request) {
  const header = request.headers.get("authorization") || "";
  return header.startsWith("Bearer ") ? header.slice("Bearer ".length).trim() : "";
}

function requireAdminAuth(request, env, requestIdValue, scope = ADMIN_AUTH_SCOPE.LEGACY) {
  const token = bearerToken(request);

  if (scope === ADMIN_AUTH_SCOPE.READ) {
    const readToken = String(env?.ADMIN_READ_TOKEN || "").trim();
    const supportToken = String(env?.ADMIN_SUPPORT_TOKEN || "").trim();
    const legacyToken = String(env?.ADMIN_API_TOKEN || "").trim();
    if (!readToken && !supportToken && !legacyToken) {
      return err("unauthorized", "Admin read token is not configured.", requestIdValue, false, 401);
    }
    if (!token || (token !== readToken && token !== supportToken && token !== legacyToken)) {
      return err("unauthorized", "Admin read authorization failed.", requestIdValue, false, 401);
    }
    return null;
  }

  if (scope === ADMIN_AUTH_SCOPE.SUPPORT) {
    const supportToken = String(env?.ADMIN_SUPPORT_TOKEN || "").trim();
    if (!supportToken) {
      return err("unauthorized", "Admin support token is not configured.", requestIdValue, false, 401);
    }
    if (!token || token !== supportToken) {
      return err("forbidden", "Admin support authorization failed.", requestIdValue, false, 403);
    }
    return null;
  }

  if (!env?.ADMIN_API_TOKEN) {
    return err("unauthorized", "Admin API token is not configured.", requestIdValue, false, 401);
  }
  if (!token || token !== env.ADMIN_API_TOKEN) {
    return err("unauthorized", "Admin authorization failed.", requestIdValue, false, 401);
  }
  return null;
}

function requireSupportDecisionContext(request, body, requestIdValue) {
  const actor = String(request.headers.get("x-admin-actor") || "").trim();
  const reason = typeof body?.reason === "string" ? body.reason.trim() : "";
  if (!actor) {
    return err("bad_request", "Missing required header: X-Admin-Actor", requestIdValue, false, 400);
  }
  if (!reason) {
    return err("bad_request", "Support decision reason is required.", requestIdValue, false, 400);
  }
  return { actor: redactAuditText(actor), reason };
}

function enforceAdminSupportRateLimit(request, env, action, requestIdValue) {
  const token = bearerToken(request);
  const max = parsePositiveInteger(env?.ADMIN_SUPPORT_RATE_LIMIT_MAX, 10);
  const windowMs = parsePositiveInteger(env?.ADMIN_SUPPORT_RATE_LIMIT_WINDOW_MS, 60_000);
  const now = Date.now();
  const bucketKey = `${action}:${hashPrefix(stableHash(token))}`;
  const bucket = adminSupportRateLimits.get(bucketKey);
  if (!bucket || bucket.resetAtMs <= now) {
    adminSupportRateLimits.set(bucketKey, { count: 1, resetAtMs: now + windowMs });
    return null;
  }
  if (bucket.count >= max) {
    return err("rate_limited", "Admin support action rate limit exceeded.", requestIdValue, true, 429);
  }
  bucket.count += 1;
  return null;
}

function parsePositiveInteger(value, fallback) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : fallback;
}

function redactAuditText(value) {
  return String(value || "")
    .trim()
    .replace(/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi, "[redacted-email]")
    .slice(0, 160);
}

function normalizeD1Results(result) {
  if (Array.isArray(result)) return result;
  if (Array.isArray(result?.results)) return result.results;
  return [];
}

function deletionStatusView(row) {
  return {
    request_id: row.request_id,
    status: row.status,
    message: deletionStatusMessage(row.status),
    completed_at_ms: row.completed_at_ms || null,
    error_code: row.error_code || null,
  };
}

function adminDeletionView(row, preview) {
  return {
    deletion_request_id: row.request_id,
    status: row.status,
    masked_license_key: row.masked_license_key || null,
    has_license_hash: Boolean(row.license_key_hash),
    license_hash_prefix: hashPrefix(row.license_key_hash),
    purchaser_email: row.purchaser_email_masked || maskEmail(row.purchaser_email),
    requested_scope: row.requested_scope || "backend_licensing_data",
    deletion_preview: preview || parseJsonObject(row.deletion_summary_json),
    deletion_summary: parseJsonObject(row.deletion_summary_json),
    error_code: row.error_code || null,
    error_message_safe: row.error_message_safe || null,
    created_at_ms: row.created_at_ms,
    updated_at_ms: row.updated_at_ms,
    decided_at_ms: row.decided_at_ms || null,
    completed_at_ms: row.completed_at_ms || null,
  };
}

function deletionStatusMessage(status) {
  switch (status) {
    case "pending":
      return "Deletion request is pending admin review.";
    case "approved":
    case "processing":
      return "Deletion request is being processed.";
    case "completed":
      return "Deletion request completed.";
    case "rejected":
      return "Deletion request was rejected after review.";
    case "failed":
      return "Deletion request failed and needs admin review.";
    default:
      return "Deletion request status is unavailable.";
  }
}

function parseJsonObject(value) {
  if (!value) return null;
  try {
    const parsed = JSON.parse(String(value));
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function withDeletionLookupToken(responseBody, lookupToken) {
  try {
    const parsed = JSON.parse(String(responseBody || "{}"));
    if (parsed?.ok && parsed.data && typeof parsed.data === "object") {
      parsed.data.lookup_token = lookupToken;
      return JSON.stringify(parsed);
    }
  } catch {
    // Fall through to a safe generic response below.
  }
  return JSON.stringify({
    ok: true,
    data: {
      lookup_token: lookupToken,
      status: "pending",
      message: "Deletion request submitted for admin review.",
    },
  });
}

async function sha256Hex(input) {
  const bytes = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, "0")).join("");
}
