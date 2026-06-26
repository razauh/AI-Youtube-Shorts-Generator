// MemoryStore is retained for contract tests; production routes use the D1 helpers below.
export class MemoryStore {
  constructor() {
    this.idempotency = new Map();
    this.resetStatuses = new Map();
  }

  idempotencyKey(op, key) {
    return `${op}:${key}`;
  }

  getIdempotent(op, key) {
    return this.idempotency.get(this.idempotencyKey(op, key));
  }

  putIdempotent(op, key, payloadHash, response) {
    this.idempotency.set(this.idempotencyKey(op, key), { payloadHash, response });
  }

  getResetStatus(requestId) {
    return this.resetStatuses.get(requestId);
  }

  putResetStatus(requestId, status) {
    this.resetStatuses.set(requestId, status);
  }
}

export function stableHash(value) {
  // Deterministic string hash for scaffold behavior only.
  // Swap with stronger canonical hashing in production implementation.
  const src = JSON.stringify(value, Object.keys(value).sort());
  let h = 0;
  for (let i = 0; i < src.length; i += 1) {
    h = (h * 31 + src.charCodeAt(i)) >>> 0;
  }
  return `h${h.toString(16)}`;
}

export async function getD1Idempotent(db, op, key) {
  return db
    .prepare(
      `SELECT payload_hash, response_status, response_body
       FROM idempotency_records
       WHERE op = ? AND idempotency_key = ?`,
    )
    .bind(op, key)
    .first();
}

export async function putD1Idempotent(db, op, key, payloadHash, response, createdAtMs) {
  return db
    .prepare(
      `INSERT INTO idempotency_records (
         op,
         idempotency_key,
         payload_hash,
         response_status,
         response_body,
         created_at_ms
       ) VALUES (?, ?, ?, ?, ?, ?)`,
    )
    .bind(op, key, payloadHash, response.status, response.body, createdAtMs)
    .run();
}

export async function writeVerifiedGumroadSale(
  db,
  { licenseKeyHash, purchaserEmail, providerSaleId, metadataJson, updatedAtMs },
) {
  return db.batch([
    db.prepare(
      `INSERT INTO licenses (
         license_key_hash,
         purchaser_email,
         entitlement_status,
         provider,
         provider_sale_id,
         updated_at_ms
       ) VALUES (?, ?, 'active', 'gumroad', ?, ?)
       ON CONFLICT(license_key_hash) DO UPDATE SET
         purchaser_email = excluded.purchaser_email,
         entitlement_status = 'active',
         provider = 'gumroad',
         provider_sale_id = excluded.provider_sale_id,
         updated_at_ms = excluded.updated_at_ms`,
    ).bind(licenseKeyHash, purchaserEmail, providerSaleId, updatedAtMs),
    db.prepare(
      `INSERT INTO audit_events (
         event_type,
         actor,
         metadata_json,
         created_at_ms
       ) VALUES (?, ?, ?, ?)`,
    ).bind("gumroad_sale_verified", "gumroad", metadataJson, updatedAtMs),
  ]);
}

export async function getLicenseByHash(db, licenseKeyHash) {
  return db
    .prepare(
      `SELECT license_key_hash, entitlement_status, purchaser_email, provider, provider_sale_id, updated_at_ms
       FROM licenses
       WHERE license_key_hash = ?`,
    )
    .bind(licenseKeyHash)
    .first();
}

export async function listLicensesByHashPrefix(db, licenseHashPrefix, limit = 5) {
  return db
    .prepare(
      `SELECT license_key_hash, entitlement_status, purchaser_email, provider, provider_sale_id, updated_at_ms
       FROM licenses
       WHERE LOWER(license_key_hash) LIKE ?
       ORDER BY updated_at_ms DESC
       LIMIT ?`,
    )
    .bind(`${String(licenseHashPrefix).toLowerCase()}%`, limit)
    .all();
}

export async function updateLicenseEntitlementStatus(db, licenseKeyHash, entitlementStatus, updatedAtMs) {
  return db
    .prepare(
      `UPDATE licenses
       SET entitlement_status = ?, updated_at_ms = ?
       WHERE license_key_hash = ?`,
    )
    .bind(entitlementStatus, updatedAtMs, licenseKeyHash)
    .run();
}

export async function listResetRequestsByStatus(db, status) {
  return db
    .prepare(
      `SELECT request_id, license_key_hash, masked_license_key, purchaser_email, status, created_at_ms, updated_at_ms
       FROM reset_requests
       WHERE status = ?
       ORDER BY created_at_ms ASC`,
    )
    .bind(status)
    .all();
}

export async function listDeletionRequestsByStatus(db, status) {
  return db
    .prepare(
      `SELECT request_id, lookup_token_hash, license_key_hash, masked_license_key, purchaser_email,
              purchaser_email_masked, status, requested_scope, request_metadata_json,
              deletion_summary_json, error_code, error_message_safe, created_at_ms, updated_at_ms,
              decided_at_ms, completed_at_ms
       FROM user_data_deletion_requests
       WHERE status = ?
       ORDER BY created_at_ms ASC`,
    )
    .bind(status)
    .all();
}

export async function writeAuditEvent(db, eventType, actor, metadataJson, createdAtMs) {
  return db
    .prepare(
      `INSERT INTO audit_events (
         event_type,
         actor,
         metadata_json,
         created_at_ms
       ) VALUES (?, ?, ?, ?)`,
    )
    .bind(eventType, actor, metadataJson, createdAtMs)
    .run();
}

export async function createDeletionRequest(
  db,
  {
    requestId,
    lookupTokenHash,
    licenseKeyHash,
    maskedLicenseKey,
    purchaserEmail,
    purchaserEmailMasked,
    status,
    requestedScope,
    requestMetadataJson,
    createdAtMs,
    updatedAtMs,
  },
) {
  return db
    .prepare(
      `INSERT INTO user_data_deletion_requests (
         request_id,
         lookup_token_hash,
         license_key_hash,
         masked_license_key,
         purchaser_email,
         purchaser_email_masked,
         status,
         requested_scope,
         request_metadata_json,
         created_at_ms,
         updated_at_ms
       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    )
    .bind(
      requestId,
      lookupTokenHash,
      licenseKeyHash,
      maskedLicenseKey,
      purchaserEmail,
      purchaserEmailMasked,
      status,
      requestedScope,
      requestMetadataJson,
      createdAtMs,
      updatedAtMs,
    )
    .run();
}

export async function getDeletionRequest(db, requestId) {
  return db
    .prepare(
      `SELECT request_id, lookup_token_hash, license_key_hash, masked_license_key, purchaser_email,
              purchaser_email_masked, status, requested_scope, request_metadata_json,
              deletion_summary_json, error_code, error_message_safe, created_at_ms, updated_at_ms,
              decided_at_ms, completed_at_ms
       FROM user_data_deletion_requests
       WHERE request_id = ?`,
    )
    .bind(requestId)
    .first();
}

export async function getOpenDeletionRequestByLicenseHash(db, licenseKeyHash) {
  return db
    .prepare(
      `SELECT request_id, lookup_token_hash, license_key_hash, masked_license_key, purchaser_email,
              purchaser_email_masked, status, requested_scope, request_metadata_json,
              deletion_summary_json, error_code, error_message_safe, created_at_ms, updated_at_ms,
              decided_at_ms, completed_at_ms
       FROM user_data_deletion_requests
       WHERE license_key_hash = ?
         AND status IN ('pending', 'approved', 'processing', 'failed')
       ORDER BY created_at_ms ASC
       LIMIT 1`,
    )
    .bind(licenseKeyHash)
    .first();
}

export async function updateDeletionRequestStatus(
  db,
  { requestId, status, updatedAtMs, decidedAtMs = null, completedAtMs = null, summaryJson = null, errorCode = null, errorMessageSafe = null },
) {
  return db
    .prepare(
      `UPDATE user_data_deletion_requests
       SET status = ?,
           updated_at_ms = ?,
           decided_at_ms = COALESCE(?, decided_at_ms),
           completed_at_ms = COALESCE(?, completed_at_ms),
           deletion_summary_json = COALESCE(?, deletion_summary_json),
           error_code = ?,
           error_message_safe = ?
       WHERE request_id = ?`,
    )
    .bind(status, updatedAtMs, decidedAtMs, completedAtMs, summaryJson, errorCode, errorMessageSafe, requestId)
    .run();
}

export async function updateDeletionRequestMetadata(db, { requestId, updatedAtMs, requestMetadataJson }) {
  return db
    .prepare(
      `UPDATE user_data_deletion_requests
       SET request_metadata_json = ?,
           updated_at_ms = ?
       WHERE request_id = ?`,
    )
    .bind(requestMetadataJson, updatedAtMs, requestId)
    .run();
}

export async function sanitizeCompletedDeletionRequest(db, requestId, updatedAtMs) {
  return db
    .prepare(
      `UPDATE user_data_deletion_requests
       SET purchaser_email = NULL,
           masked_license_key = NULL,
           updated_at_ms = ?
       WHERE request_id = ?`,
    )
    .bind(updatedAtMs, requestId)
    .run();
}

export async function getResetRequest(db, requestId) {
  return db
    .prepare(
      `SELECT request_id, license_key_hash, masked_license_key, purchaser_email, status, created_at_ms, updated_at_ms
       FROM reset_requests
       WHERE request_id = ?`,
    )
    .bind(requestId)
    .first();
}

export async function updateResetRequestStatus(db, requestId, status, updatedAtMs) {
  return db
    .prepare(
      `UPDATE reset_requests
       SET status = ?, updated_at_ms = ?
       WHERE request_id = ?`,
    )
    .bind(status, updatedAtMs, requestId)
    .run();
}

export async function deactivateDeviceBindingsByLicenseHash(db, licenseKeyHash, updatedAtMs) {
  return db
    .prepare(
      `UPDATE device_bindings
       SET status = 'inactive', updated_at_ms = ?
       WHERE license_key_hash = ? AND status = 'active'`,
    )
    .bind(updatedAtMs, licenseKeyHash)
    .run();
}

export async function getDeletionPreviewByLicenseHash(db, licenseKeyHash) {
  const [licenseCount, deviceCount, resetCount] = await Promise.all([
    db.prepare("SELECT COUNT(*) AS count FROM licenses WHERE license_key_hash = ?").bind(licenseKeyHash).first(),
    db.prepare("SELECT COUNT(*) AS count FROM device_bindings WHERE license_key_hash = ?").bind(licenseKeyHash).first(),
    db.prepare("SELECT COUNT(*) AS count FROM reset_requests WHERE license_key_hash = ?").bind(licenseKeyHash).first(),
  ]);
  return {
    licenses: Number(licenseCount?.count || 0),
    device_bindings: Number(deviceCount?.count || 0),
    reset_requests: Number(resetCount?.count || 0),
  };
}

export async function deleteDeviceBindingsByLicenseHash(db, licenseKeyHash) {
  return db
    .prepare("DELETE FROM device_bindings WHERE license_key_hash = ?")
    .bind(licenseKeyHash)
    .run();
}

export async function anonymizeResetRequestsByLicenseHash(db, licenseKeyHash, updatedAtMs) {
  return db
    .prepare(
      `UPDATE reset_requests
       SET license_key_hash = NULL,
           masked_license_key = NULL,
           purchaser_email = NULL,
           updated_at_ms = ?
       WHERE license_key_hash = ?`,
    )
    .bind(updatedAtMs, licenseKeyHash)
    .run();
}

export async function anonymizeLicenseForPrivacyDeletion(db, licenseKeyHash, updatedAtMs) {
  return db
    .prepare(
      `UPDATE licenses
       SET purchaser_email = NULL,
           updated_at_ms = ?,
           privacy_deleted_at_ms = ?
       WHERE license_key_hash = ?`,
    )
    .bind(updatedAtMs, updatedAtMs, licenseKeyHash)
    .run();
}

export async function getAdminOverviewCounts(db) {
  const queries = [
    db.prepare("SELECT COUNT(*) AS count FROM licenses").first(),
    db.prepare("SELECT entitlement_status AS key, COUNT(*) AS count FROM licenses GROUP BY entitlement_status").all(),
    db.prepare("SELECT status AS key, COUNT(*) AS count FROM device_bindings GROUP BY status").all(),
    db.prepare("SELECT status AS key, COUNT(*) AS count FROM reset_requests GROUP BY status").all(),
    db.prepare("SELECT status AS key, COUNT(*) AS count FROM user_data_deletion_requests GROUP BY status").all(),
    db.prepare("SELECT COUNT(*) AS count FROM audit_events WHERE created_at_ms >= ?").bind(Date.now() - 86_400_000).first(),
  ];
  const [licensesTotal, entitlementRows, deviceRows, resetRows, deletionRows, recentAuditCount] = await Promise.all(queries);
  return {
    total_licenses: Number(licensesTotal?.count || 0),
    entitlement_counts: normalizeCountRows(entitlementRows),
    device_binding_counts: normalizeCountRows(deviceRows),
    reset_request_counts: normalizeCountRows(resetRows),
    deletion_request_counts: normalizeCountRows(deletionRows),
    recent_audit_events_24h: Number(recentAuditCount?.count || 0),
  };
}

export async function listAdminAuditEvents(db, { eventType, actor, limit }) {
  const where = [];
  const args = [];
  if (eventType) {
    where.push("LOWER(event_type) = ?");
    args.push(eventType.toLowerCase());
  }
  if (actor) {
    where.push("LOWER(actor) = ?");
    args.push(actor.toLowerCase());
  }
  const whereSql = where.length > 0 ? `WHERE ${where.join(" AND ")}` : "";
  const result = await db
    .prepare(
      `SELECT event_type, actor, metadata_json, created_at_ms
       FROM audit_events
       ${whereSql}
       ORDER BY created_at_ms DESC
       LIMIT ?`,
    )
    .bind(...args, limit)
    .all();
  return normalizeRows(result);
}

export async function listAdminIdempotencyRecords(db, { op, limit }) {
  const where = [];
  const args = [];
  if (op) {
    where.push("LOWER(op) = ?");
    args.push(op.toLowerCase());
  }
  const whereSql = where.length > 0 ? `WHERE ${where.join(" AND ")}` : "";
  const result = await db
    .prepare(
      `SELECT op, idempotency_key, payload_hash, response_status, response_body, created_at_ms
       FROM idempotency_records
       ${whereSql}
       ORDER BY created_at_ms DESC
       LIMIT ?`,
    )
    .bind(...args, limit)
    .all();
  return normalizeRows(result);
}

function normalizeRows(result) {
  if (Array.isArray(result)) return result;
  if (Array.isArray(result?.results)) return result.results;
  return [];
}

function normalizeCountRows(result) {
  const rows = normalizeRows(result);
  const counts = {};
  for (const row of rows) {
    const key = String(row.key || "unknown").toLowerCase();
    counts[key] = Number(row.count || 0);
  }
  return counts;
}
