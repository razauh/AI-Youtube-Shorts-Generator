CREATE TABLE IF NOT EXISTS user_data_deletion_requests (
  request_id TEXT PRIMARY KEY,
  lookup_token_hash TEXT NOT NULL UNIQUE,
  license_key_hash TEXT,
  masked_license_key TEXT,
  purchaser_email TEXT,
  purchaser_email_masked TEXT,
  status TEXT NOT NULL,
  requested_scope TEXT NOT NULL DEFAULT 'backend_licensing_data',
  request_metadata_json TEXT,
  deletion_summary_json TEXT,
  error_code TEXT,
  error_message_safe TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  decided_at_ms INTEGER,
  completed_at_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_user_data_deletion_requests_status_created
  ON user_data_deletion_requests(status, created_at_ms);

CREATE INDEX IF NOT EXISTS idx_user_data_deletion_requests_license_status
  ON user_data_deletion_requests(license_key_hash, status);

ALTER TABLE licenses ADD COLUMN privacy_deleted_at_ms INTEGER;
