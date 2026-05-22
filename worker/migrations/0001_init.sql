CREATE TABLE IF NOT EXISTS licenses (
  license_key_hash TEXT PRIMARY KEY,
  purchaser_email TEXT,
  entitlement_status TEXT NOT NULL,
  provider TEXT,
  provider_sale_id TEXT,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS device_bindings (
  device_id TEXT PRIMARY KEY,
  license_key_hash TEXT NOT NULL,
  public_key TEXT NOT NULL,
  fingerprint_json TEXT NOT NULL,
  status TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS reset_requests (
  request_id TEXT PRIMARY KEY,
  license_key_hash TEXT,
  masked_license_key TEXT,
  purchaser_email TEXT,
  status TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS idempotency_records (
  op TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  payload_hash TEXT NOT NULL,
  response_status INTEGER NOT NULL,
  response_body TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY (op, idempotency_key)
);

CREATE TABLE IF NOT EXISTS audit_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_type TEXT NOT NULL,
  actor TEXT,
  metadata_json TEXT,
  created_at_ms INTEGER NOT NULL
);
