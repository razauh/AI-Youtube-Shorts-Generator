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
    if (method === "POST" && path === "/v1/license/webhooks/gumroad") {
      return handleGumroadWebhook(request, env);
    }

    return err("bad_request", "Route not found", requestId(), false, 404);
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
  if (!binding || binding.license_key_hash !== tokenPayload.data.license_key_hash) {
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
  if (!body || !body.purchaser_email || !body.device_public_key || !body.fingerprint || !body.timestamp_ms) {
    return err("invalid_reset_request", "Invalid reset request payload", rid, false, 400);
  }
  if (!String(body.purchaser_email).includes("@")) {
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
  if (body.license_key) {
    const normalizedLicenseKey = normalizeLicenseKey(body.license_key);
    licenseKeyHash = await sha256Hex(`${env?.HASH_PEPPER || ""}:${normalizedLicenseKey}`);
  }
  const requestIdValue = `reset_${crypto.randomUUID().slice(0, 8)}`;
  const response = ok({ request_id: requestIdValue, status: "pending" });
  const responseBody = await response.clone().text();

  try {
    await upsertResetRequest(env.DB, {
      requestId: requestIdValue,
      licenseKeyHash,
      purchaserEmail: String(body.purchaser_email),
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
        purchaser_email: String(body.purchaser_email),
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

  const sale = payload.sale || payload;
  const remoteSaleId = sale.id ?? sale.sale_id;
  const remoteProductId = sale.product_id;
  const remoteEmail = sale.email;
  const refunded = Boolean(sale.refunded);
  const disputed = Boolean(sale.disputed ?? sale.chargebacked);

  if (!remoteSaleId || !remoteProductId || !remoteEmail) {
    return {
      ok: false,
      code: "serialization",
      message: "Gumroad verification payload is missing required sale fields.",
      retryable: false,
      status: 503,
    };
  }

  if (
    String(remoteSaleId) !== String(saleId) ||
    String(remoteProductId) !== String(productId) ||
    String(remoteEmail).toLowerCase() !== String(email).toLowerCase()
  ) {
    return {
      ok: false,
      code: "unauthorized",
      message: "Gumroad verification mismatch for sale_id/product_id/email.",
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
