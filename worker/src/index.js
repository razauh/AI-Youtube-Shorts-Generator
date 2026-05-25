import { err, ok, readForm, readJson, requestId, RESET_STATUS } from "./contracts.js";
import {
  getD1Idempotent,
  putD1Idempotent,
  stableHash,
  writeVerifiedGumroadSale,
  getLicenseByHash,
  upsertDeviceBinding,
  getDeviceBinding,
  writeAuditEvent,
  upsertResetRequest,
  getResetRequest,
  listResetRequestsByStatus,
  updateResetRequestStatus,
  deactivateDeviceBindingsByLicenseHash,
  getAdminOverviewCounts,
  listAdminLicenses,
  listAdminDeviceBindings,
  listAdminAuditEvents,
  listAdminIdempotencyRecords,
  listLicensesByHashPrefix,
  updateLicenseEntitlementStatus,
} from "./store.js";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;
    const method = request.method.toUpperCase();

    if (method === "GET" && path === "/health") {
      return ok({ status: "ok", contract: "v1" });
    }

    if (method === "POST" && path === "/v1/license/activate") {
      return handleActivate(request, env);
    }
    if (method === "POST" && path === "/v1/license/validate") {
      return handleValidate(request, env);
    }
    if (method === "POST" && path === "/v1/license/reset/request") {
      return handleResetRequest(request, env);
    }
    if (method === "POST" && path === "/v1/license/reset/status") {
      return handleResetStatus(request, env);
    }
    if (method === "GET" && path === "/v1/admin/reset/requests") {
      return handleAdminListResetRequests(request, env);
    }
    if (method === "GET" && path === "/v1/admin/overview") {
      return handleAdminOverview(request, env);
    }
    if (method === "GET" && path === "/v1/admin/licenses") {
      return handleAdminLicenses(request, env);
    }
    if (method === "GET" && path === "/v1/admin/device-bindings") {
      return handleAdminDeviceBindings(request, env);
    }
    if (method === "GET" && path === "/v1/admin/audit-events") {
      return handleAdminAuditEvents(request, env);
    }
    if (method === "GET" && path === "/v1/admin/idempotency-records") {
      return handleAdminIdempotencyRecords(request, env);
    }
    if (method === "POST" && path === "/v1/admin/reset/approve") {
      return handleAdminResetDecision(request, env, "approved");
    }
    if (method === "POST" && path === "/v1/admin/reset/reject") {
      return handleAdminResetDecision(request, env, "rejected");
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

async function handleActivate(request, env) {
  const rid = requestId();
  const idem = request.headers.get("x-idempotency-key");
  if (!idem) {
    return err("bad_request", "Missing required header: X-Idempotency-Key", rid, false, 400);
  }
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  if (!env?.TOKEN_SIGNING_SECRET) {
    return err("unauthorized", "TOKEN_SIGNING_SECRET is not configured.", rid, false, 401);
  }

  const body = await readJson(request);
  if (!body || !body.license_key || !body.device_public_key || !body.fingerprint || !body.app_version || !body.timestamp_ms) {
    return err("bad_request", "Invalid activate payload", rid, false, 400);
  }

  const payloadHash = stableHash(body);
  const replay = await getD1Idempotent(env.DB, "activate", idem);
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
    return new Response(replay.response_body, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const now = Date.now();
  const normalizedLicenseKey = normalizeLicenseKey(body.license_key);
  const licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);
  const license = await getLicenseByHash(env.DB, licenseKeyHash);
  if (!license || String(license.entitlement_status).toLowerCase() !== "active") {
    return err("unauthorized", "License is not active.", rid, false, 401);
  }

  const deviceId = `dev_${(await sha256Hex(body.device_public_key)).slice(0, 12)}`;
  const tokenExpiresAtMs = Number(body.timestamp_ms) + 3_600_000;
  const maskedLicenseKey = maskLicenseKey(normalizedLicenseKey);
  const accessToken = await issueAccessToken(
    {
      license_key_hash: licenseKeyHash,
      device_id: deviceId,
      masked_license_key: maskedLicenseKey,
      token_expires_at_ms: tokenExpiresAtMs,
    },
    env.TOKEN_SIGNING_SECRET,
  );

  try {
    await upsertDeviceBinding(env.DB, {
      deviceId,
      licenseKeyHash,
      publicKey: String(body.device_public_key),
      fingerprintJson: JSON.stringify(body.fingerprint),
      status: "active",
      updatedAtMs: now,
    });
    await writeAuditEvent(
      env.DB,
      "license_activated",
      "desktop_client",
      JSON.stringify({
        device_id: deviceId,
        app_version: String(body.app_version),
      }),
      now,
    );
  } catch {
    return err("storage", "Failed to persist activation state.", rid, true, 503);
  }

  const response = ok({
    access_token: accessToken,
    masked_license_key: maskedLicenseKey,
    bound_device: {
      device_id: deviceId,
      public_key: body.device_public_key,
      fingerprint: body.fingerprint,
    },
    entitlement: "active",
    token_expires_at_ms: tokenExpiresAtMs,
  });
  const responseBody = await response.clone().text();

  await putD1Idempotent(
    env.DB,
    "activate",
    idem,
    payloadHash,
    { status: response.status, body: responseBody },
    now,
  );

  return response;
}

async function handleValidate(request, env) {
  const rid = requestId();
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  if (!env?.TOKEN_SIGNING_SECRET) {
    return err("unauthorized", "TOKEN_SIGNING_SECRET is not configured.", rid, false, 401);
  }
  const body = await readJson(request);
  if (!body || !body.access_token) {
    return err("bad_request", "Invalid validate payload", rid, false, 400);
  }

  const tokenPayload = await verifyAccessToken(
    String(body.access_token),
    env.TOKEN_SIGNING_SECRET,
  );
  if (!tokenPayload.ok) {
    return err("reauth_required", "Session token is invalid or expired.", rid, false, 401);
  }

  const license = await getLicenseByHash(env.DB, tokenPayload.data.license_key_hash);
  if (!license || String(license.entitlement_status).toLowerCase() !== "active") {
    return err("reauth_required", "Session entitlement is no longer active.", rid, false, 401);
  }

  const binding = await getDeviceBinding(env.DB, tokenPayload.data.device_id);
  if (
    !binding ||
    binding.license_key_hash !== tokenPayload.data.license_key_hash ||
    String(binding.status).toLowerCase() !== "active"
  ) {
    return err("reauth_required", "Session device binding no longer exists.", rid, false, 401);
  }

  let fingerprint = {};
  try {
    fingerprint = JSON.parse(binding.fingerprint_json || "{}");
  } catch {
    fingerprint = {};
  }

  return ok({
    entitlement: "active",
    masked_license_key: tokenPayload.data.masked_license_key,
    bound_device: {
      device_id: binding.device_id,
      public_key: binding.public_key,
      fingerprint,
    },
    token_expires_at_ms: tokenPayload.data.token_expires_at_ms,
  });
}

async function handleResetRequest(request, env) {
  const rid = requestId();
  const idem = request.headers.get("x-idempotency-key");
  if (!idem) {
    return err("bad_request", "Missing required header: X-Idempotency-Key", rid, false, 400);
  }
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const body = await readJson(request);
  if (!body || !body.device_public_key || !body.fingerprint || !body.timestamp_ms) {
    return err("invalid_reset_request", "Invalid reset request payload", rid, false, 400);
  }
  if (!body.license_key) {
    return err("invalid_reset_request", "Reset request requires license context.", rid, false, 400);
  }
  if (body.purchaser_email && !String(body.purchaser_email).includes("@")) {
    return err("invalid_purchase_email", "Purchaser email format is invalid.", rid, false, 400);
  }

  const payloadHash = stableHash(body);
  const replay = await getD1Idempotent(env.DB, "reset_request", idem);
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
    return new Response(replay.response_body, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const now = Date.now();
  let licenseKeyHash = null;
  let maskedLicenseKey = null;
  if (body.license_key) {
    const normalizedLicenseKey = normalizeLicenseKey(body.license_key);
    licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);
    maskedLicenseKey = maskLicenseKey(normalizedLicenseKey);
  } else if (body.masked_license_key) {
    maskedLicenseKey = String(body.masked_license_key);
  }
  const requestIdValue = `reset_${crypto.randomUUID().slice(0, 8)}`;
  const response = ok({ request_id: requestIdValue, status: "pending" });
  const responseBody = await response.clone().text();

  try {
    await upsertResetRequest(env.DB, {
      requestId: requestIdValue,
      licenseKeyHash,
      maskedLicenseKey,
      purchaserEmail: body.purchaser_email ? String(body.purchaser_email) : null,
      status: "pending",
      createdAtMs: now,
      updatedAtMs: now,
    });
    await writeAuditEvent(
      env.DB,
      "device_reset_requested",
      "desktop_client",
      JSON.stringify({
        request_id: requestIdValue,
        has_license_hash: Boolean(licenseKeyHash),
      }),
      now,
    );
    await putD1Idempotent(
      env.DB,
      "reset_request",
      idem,
      payloadHash,
      { status: response.status, body: responseBody },
      now,
    );
  } catch {
    return err("storage", "Failed to persist reset request state.", rid, true, 503);
  }

  return response;
}

async function handleResetStatus(request, env) {
  const rid = requestId();
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  const body = await readJson(request);
  const reqId = body?.request_id;
  if (!reqId) {
    return err("bad_request", "Invalid reset status payload", rid, false, 400);
  }

  const reset = await getResetRequest(env.DB, reqId);
  if (!reset) {
    return err("reset_request_not_found", "Reset request was not found.", rid, false, 404);
  }
  const status = reset.status;
  if (!RESET_STATUS.has(status)) {
    return err("serialization", "Stored reset status is invalid.", rid, false, 503);
  }

  return ok({ request_id: reqId, status });
}

async function handleAdminListResetRequests(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const url = new URL(request.url);
  const status = url.searchParams.get("status") || "pending";
  if (!RESET_STATUS.has(status)) {
    return err("bad_request", "Invalid reset request status filter.", rid, false, 400);
  }

  try {
    const result = await listResetRequestsByStatus(env.DB, status);
    return ok({
      requests: normalizeD1Results(result).map((row) => adminResetView(row)),
    });
  } catch {
    return err("storage", "Failed to load reset requests.", rid, true, 503);
  }
}

async function handleAdminResetDecision(request, env, decision) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
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
    return err("bad_request", "Invalid admin reset decision payload.", rid, false, 400);
  }

  const payloadHash = stableHash({ decision, request_id: requestIdValue, reason: body.reason || null });
  const replay = await getD1Idempotent(env.DB, `admin_reset_${decision}`, idem);
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
    return new Response(replay.response_body, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const reset = await getResetRequest(env.DB, requestIdValue);
  if (!reset) {
    return err("reset_request_not_found", "Reset request was not found.", rid, false, 404);
  }
  if (reset.status !== "pending") {
    return err("invalid_transition", "Reset request has already been decided.", rid, false, 409);
  }
  if (decision === "approved" && !reset.license_key_hash) {
    return err("invalid_transition", "Reset request cannot be approved without a bound license.", rid, false, 409);
  }

  const now = Date.now();
  try {
    await updateResetRequestStatus(env.DB, reset.request_id, decision, now);
    if (decision === "approved") {
      await deactivateDeviceBindingsByLicenseHash(env.DB, reset.license_key_hash, now);
    }
    await writeAuditEvent(
      env.DB,
      decision === "approved" ? "device_reset_approved" : "device_reset_rejected",
      "admin",
      JSON.stringify({
        request_id: reset.request_id,
        has_license_hash: Boolean(reset.license_key_hash),
        reason_present: Boolean(body.reason),
      }),
      now,
    );
  } catch {
    return err("storage", "Failed to persist admin reset decision.", rid, true, 503);
  }

  const response = ok({
    reset_request_id: reset.request_id,
    status: decision,
    license_state: decision === "approved" ? "UNBOUND" : "BOUND_ACTIVE",
  });
  const responseBody = await response.clone().text();
  await putD1Idempotent(
    env.DB,
    `admin_reset_${decision}`,
    idem,
    payloadHash,
    { status: response.status, body: responseBody },
    now,
  );
  return response;
}

async function handleAdminOverview(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
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
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }

  const idem = request.headers.get("x-idempotency-key");
  if (!idem) {
    return err("bad_request", "Missing required header: X-Idempotency-Key", rid, false, 400);
  }

  const body = await readJson(request);
  const licenseHashPrefix = asOptionalString(body?.license_hash_prefix);
  const reason = asOptionalString(body?.reason);
  const deactivateBindings = Boolean(body?.deactivate_bindings);
  if (!licenseHashPrefix || !reason) {
    return err("bad_request", "license_hash_prefix and reason are required.", rid, false, 400);
  }

  const payloadHash = stableHash({
    license_hash_prefix: licenseHashPrefix,
    reason,
    deactivate_bindings: deactivateBindings,
  });
  const replay = await getD1Idempotent(env.DB, "admin_license_disable", idem);
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
    return new Response(replay.response_body, {
      status: replay.response_status,
      headers: { "content-type": "application/json; charset=utf-8" },
    });
  }

  const matches = normalizeD1Results(await listLicensesByHashPrefix(env.DB, licenseHashPrefix, 2));
  if (matches.length === 0) {
    return err("license_not_found", "License was not found.", rid, false, 404);
  }
  if (matches.length > 1) {
    return err("bad_request", "license_hash_prefix matches multiple licenses.", rid, false, 400);
  }

  const license = matches[0];
  if (String(license.entitlement_status).toLowerCase() !== "active") {
    return err(
      "invalid_transition",
      "This license cannot be disabled from its current state.",
      rid,
      false,
      409,
    );
  }

  const now = Date.now();
  try {
    await updateLicenseEntitlementStatus(env.DB, license.license_key_hash, "disabled", now);
    if (deactivateBindings) {
      await deactivateDeviceBindingsByLicenseHash(env.DB, license.license_key_hash, now);
    }
    await writeAuditEvent(
      env.DB,
      "license_disabled",
      "admin",
      JSON.stringify({
        license_hash_prefix: hashPrefix(license.license_key_hash),
        from_status: String(license.entitlement_status).toLowerCase(),
        to_status: "disabled",
        reason,
        reason_present: true,
        deactivate_bindings: deactivateBindings,
      }),
      now,
    );
  } catch {
    return err("storage", "Failed to disable license.", rid, true, 503);
  }

  const response = ok({
    license_hash_prefix: hashPrefix(license.license_key_hash),
    entitlement_status: "disabled",
    deactivate_bindings: deactivateBindings,
  });
  const responseBody = await response.clone().text();
  await putD1Idempotent(
    env.DB,
    "admin_license_disable",
    idem,
    payloadHash,
    { status: response.status, body: responseBody },
    now,
  );
  return response;
}

async function handleAdminLicenses(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  const url = new URL(request.url);
  const q = asOptionalString(url.searchParams.get("q"));
  const entitlementStatus = asOptionalString(url.searchParams.get("entitlement_status"));
  const provider = asOptionalString(url.searchParams.get("provider"));
  const limit = parseLimit(url.searchParams.get("limit"), 25, 100);
  if (!limit.ok) return err("bad_request", limit.message, rid, false, 400);
  try {
    const rows = await listAdminLicenses(env.DB, {
      q,
      entitlementStatus,
      provider,
      limit: limit.value,
    });
    return ok({
      licenses: rows.map((row) => ({
        license_hash_prefix: hashPrefix(row.license_key_hash),
        purchaser_email_masked: maskEmail(row.purchaser_email),
        entitlement_status: row.entitlement_status,
        provider: row.provider || null,
        provider_sale_id: row.provider_sale_id || null,
        updated_at_ms: row.updated_at_ms,
        active_device_count: Number(row.active_device_count || 0),
        inactive_device_count: Number(row.inactive_device_count || 0),
      })),
    });
  } catch {
    return err("storage", "Failed to load licenses.", rid, true, 503);
  }
}

async function handleAdminDeviceBindings(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
  if (auth) return auth;
  if (!env?.DB) {
    return err("storage", "D1 database binding is not configured.", rid, false, 503);
  }
  const url = new URL(request.url);
  const q = asOptionalString(url.searchParams.get("q"));
  const status = asOptionalString(url.searchParams.get("status"));
  if (status && !["active", "inactive"].includes(status.toLowerCase())) {
    return err("bad_request", "Invalid device binding status filter.", rid, false, 400);
  }
  const licenseHashPrefix = asOptionalString(url.searchParams.get("license_hash_prefix"));
  const limit = parseLimit(url.searchParams.get("limit"), 25, 100);
  if (!limit.ok) return err("bad_request", limit.message, rid, false, 400);
  try {
    const rows = await listAdminDeviceBindings(env.DB, {
      q,
      status,
      licenseHashPrefix,
      limit: limit.value,
    });
    return ok({
      bindings: rows.map((row) => ({
        device_id: row.device_id,
        status: row.status,
        license_hash_prefix: hashPrefix(row.license_key_hash),
        updated_at_ms: row.updated_at_ms,
        purchaser_email_masked: maskEmail(row.purchaser_email),
        public_key_prefix: keyPrefix(row.public_key),
        fingerprint_summary: summarizeFingerprint(row.fingerprint_json),
      })),
    });
  } catch {
    return err("storage", "Failed to load device bindings.", rid, true, 503);
  }
}

async function handleAdminAuditEvents(request, env) {
  const rid = requestId();
  const auth = requireAdminAuth(request, env, rid);
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
  const auth = requireAdminAuth(request, env, rid);
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

  const payloadHash = stableHash(body);
  const replay = await getD1Idempotent(env.DB, "gumroad_webhook", String(body.sale_id));
  if (replay) {
    if (replay.payload_hash !== payloadHash) {
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

  const verification = await verifyGumroadSale({
    saleId: String(body.sale_id),
    productId: String(body.product_id),
    email: String(body.email),
    token: env?.GUMROAD_ACCESS_TOKEN,
  });

  if (!verification.ok) {
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

  const now = Date.now();
  const normalizedLicenseKey = normalizeLicenseKey(verification.sale.license_key);
  const licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);
  const response = ok({
    accepted: true,
    provider: "gumroad",
    sale_id: body.sale_id,
    verified: true,
  });
  const responseBody = await response.clone().text();

  try {
    await writeVerifiedGumroadSale(env.DB, {
      licenseKeyHash,
      purchaserEmail: String(verification.sale.email),
      providerSaleId: String(verification.sale.id ?? verification.sale.sale_id ?? body.sale_id),
      metadataJson: JSON.stringify({
        sale_id: String(verification.sale.id ?? verification.sale.sale_id ?? body.sale_id),
        product_id: String(verification.sale.product_id),
        email: String(verification.sale.email),
        verified: true,
      }),
      updatedAtMs: now,
    });
    await putD1Idempotent(
      env.DB,
      "gumroad_webhook",
      String(body.sale_id),
      payloadHash,
      { status: response.status, body: responseBody },
      now,
    );
  } catch {
    return err("storage", "Failed to persist verified Gumroad sale.", rid, true, 503);
  }

  return response;
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

function requireAdminAuth(request, env, requestIdValue) {
  if (!env?.ADMIN_API_TOKEN) {
    return err("unauthorized", "Admin API token is not configured.", requestIdValue, false, 401);
  }
  const header = request.headers.get("authorization") || "";
  const token = header.startsWith("Bearer ") ? header.slice("Bearer ".length).trim() : "";
  if (!token || token !== env.ADMIN_API_TOKEN) {
    return err("unauthorized", "Admin authorization failed.", requestIdValue, false, 401);
  }
  return null;
}

function normalizeD1Results(result) {
  if (Array.isArray(result)) return result;
  if (Array.isArray(result?.results)) return result.results;
  return [];
}

function adminResetView(row) {
  return {
    reset_request_id: row.request_id,
    status: row.status,
    license_state: row.status === "approved" ? "UNBOUND" : "BOUND_ACTIVE",
    message: row.status,
    masked_license_key: row.masked_license_key || null,
    has_license_hash: Boolean(row.license_key_hash),
    purchaser_email: maskEmail(row.purchaser_email),
    created_at_ms: row.created_at_ms,
    updated_at_ms: row.updated_at_ms,
  };
}

function base64UrlEncode(value) {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  const raw = btoa(binary);
  return raw.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function base64UrlDecode(value) {
  const normalized = value.replace(/-/g, "+").replace(/_/g, "/");
  const padded = normalized + "=".repeat((4 - (normalized.length % 4)) % 4);
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}

async function issueAccessToken(payload, secret) {
  const payloadB64 = base64UrlEncode(JSON.stringify(payload));
  const sig = await sha256Hex(`${secret}:${payloadB64}`);
  return `v1.${payloadB64}.${sig}`;
}

async function verifyAccessToken(token, secret) {
  const parts = String(token).split(".");
  if (parts.length !== 3 || parts[0] !== "v1") {
    return { ok: false };
  }
  const payloadB64 = parts[1];
  const signature = parts[2];
  const expected = await sha256Hex(`${secret}:${payloadB64}`);
  if (signature !== expected) {
    return { ok: false };
  }
  let payload;
  try {
    payload = JSON.parse(base64UrlDecode(payloadB64));
  } catch {
    return { ok: false };
  }
  if (!payload || !payload.license_key_hash || !payload.device_id || !payload.token_expires_at_ms) {
    return { ok: false };
  }
  if (Date.now() >= Number(payload.token_expires_at_ms)) {
    return { ok: false };
  }
  return { ok: true, data: payload };
}

async function sha256Hex(input) {
  const bytes = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, "0")).join("");
}
