// In-memory stub store for contract scaffolding only.
// Replace with Durable Objects / D1 / KV in implementation phases.
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

export async function upsertDeviceBinding(
  db,
  { deviceId, licenseKeyHash, publicKey, fingerprintJson, status, updatedAtMs },
) {
  return db
    .prepare(
      `INSERT INTO device_bindings (
         device_id,
         license_key_hash,
         public_key,
         fingerprint_json,
         status,
         updated_at_ms
       ) VALUES (?, ?, ?, ?, ?, ?)
       ON CONFLICT(device_id) DO UPDATE SET
         license_key_hash = excluded.license_key_hash,
         public_key = excluded.public_key,
         fingerprint_json = excluded.fingerprint_json,
         status = excluded.status,
         updated_at_ms = excluded.updated_at_ms`,
    )
    .bind(deviceId, licenseKeyHash, publicKey, fingerprintJson, status, updatedAtMs)
    .run();
}

export async function getDeviceBinding(db, deviceId) {
  return db
    .prepare(
      `SELECT device_id, license_key_hash, public_key, fingerprint_json, status, updated_at_ms
       FROM device_bindings
       WHERE device_id = ?`,
    )
    .bind(deviceId)
    .first();
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

export async function upsertResetRequest(
  db,
  { requestId, licenseKeyHash, maskedLicenseKey, purchaserEmail, status, createdAtMs, updatedAtMs },
) {
  return db
    .prepare(
      `INSERT INTO reset_requests (
         request_id,
         license_key_hash,
         masked_license_key,
         purchaser_email,
         status,
         created_at_ms,
         updated_at_ms
       ) VALUES (?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(request_id) DO UPDATE SET
         license_key_hash = excluded.license_key_hash,
         masked_license_key = excluded.masked_license_key,
         purchaser_email = excluded.purchaser_email,
         status = excluded.status,
         updated_at_ms = excluded.updated_at_ms`,
    )
    .bind(
      requestId,
      licenseKeyHash,
      maskedLicenseKey,
      purchaserEmail,
      status,
      createdAtMs,
      updatedAtMs,
    )
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
