export const ERROR_CODES = new Set([
  "invalid_license_key",
  "invalid_purchase_email",
  "invalid_device_identity",
  "invalid_reset_request",
  "invalid_deletion_request",
  "deletion_request_not_found",
  "invalid_deletion_lookup_token",
  "device_already_bound",
  "reauth_required",
  "worker_unreachable",
  "not_found",
  "route_not_found",
  "reset_request_not_found",
  "license_not_found",
  "unauthorized",
  "storage",
  "serialization",
  "invalid_transition",
  "bad_request",
]);

export const RESET_STATUS = new Set(["pending", "approved", "rejected", "expired"]);
export const DELETION_STATUS = new Set(["pending", "approved", "processing", "rejected", "completed", "failed"]);

export function ok(data, init = {}) {
  return json({ ok: true, data }, init);
}

export function err(code, message, requestId, retryable, status = 400) {
  const safeCode = ERROR_CODES.has(code) ? code : "serialization";
  return json(
    {
      ok: false,
      error: {
        code: safeCode,
        message,
        request_id: requestId,
        retryable: Boolean(retryable),
      },
    },
    { status },
  );
}

export function json(body, init = {}) {
  const headers = new Headers(init.headers || {});
  if (!headers.has("content-type")) {
    headers.set("content-type", "application/json; charset=utf-8");
  }
  return new Response(JSON.stringify(body), { ...init, headers });
}

export function requestId() {
  return `req_${crypto.randomUUID().replaceAll("-", "")}`;
}

export async function readJson(request) {
  try {
    return await request.json();
  } catch {
    return null;
  }
}

export async function readForm(request) {
  try {
    const form = await request.formData();
    const body = {};
    for (const [key, value] of form.entries()) {
      body[key] = typeof value === "string" ? value : value.name;
    }
    return body;
  } catch {
    return null;
  }
}
