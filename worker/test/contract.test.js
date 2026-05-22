import test from 'node:test';
import assert from 'node:assert/strict';
import app from '../src/index.js';

class MockD1Database {
  constructor() {
    this.idempotency = new Map();
    this.licenses = new Map();
    this.resetRequests = new Map();
    this.deviceBindings = new Map();
    this.auditEvents = [];
  }

  prepare(sql) {
    return new MockStatement(this, sql);
  }

  async batch(statements) {
    const results = [];
    for (const statement of statements) {
      results.push(await statement.run());
    }
    return results;
  }
}

class MockStatement {
  constructor(db, sql) {
    this.db = db;
    this.sql = sql;
    this.args = [];
  }

  bind(...args) {
    this.args = args;
    return this;
  }

  async first() {
    if (this.sql.includes('FROM idempotency_records')) {
      const [op, key] = this.args;
      return this.db.idempotency.get(`${op}:${key}`) ?? null;
    }
    if (this.sql.includes('FROM licenses')) {
      const [licenseKeyHash] = this.args;
      return this.db.licenses.get(licenseKeyHash) ?? null;
    }
    if (this.sql.includes('FROM device_bindings')) {
      const [deviceId] = this.args;
      return this.db.deviceBindings?.get(deviceId) ?? null;
    }
    if (this.sql.includes('FROM reset_requests')) {
      const [requestId] = this.args;
      return this.db.resetRequests.get(requestId) ?? null;
    }
    return null;
  }

  async all() {
    if (this.sql.includes('FROM reset_requests')) {
      const [status] = this.args;
      return {
        results: Array.from(this.db.resetRequests.values()).filter((row) => row.status === status),
      };
    }
    return { results: [] };
  }

  async run() {
    if (this.sql.includes('INSERT INTO idempotency_records')) {
      const [op, key, payloadHash, responseStatus, responseBody] = this.args;
      this.db.idempotency.set(`${op}:${key}`, {
        payload_hash: payloadHash,
        response_status: responseStatus,
        response_body: responseBody,
      });
      return { success: true };
    }

    if (this.sql.includes('INSERT INTO licenses')) {
      const [licenseKeyHash, purchaserEmail, providerSaleId, updatedAtMs] = this.args;
      this.db.licenses.set(licenseKeyHash, {
        license_key_hash: licenseKeyHash,
        purchaser_email: purchaserEmail,
        entitlement_status: 'active',
        provider: 'gumroad',
        provider_sale_id: providerSaleId,
        updated_at_ms: updatedAtMs,
      });
      return { success: true };
    }

    if (this.sql.includes('INSERT INTO device_bindings')) {
      const [deviceId, licenseKeyHash, publicKey, fingerprintJson, status, updatedAtMs] = this.args;
      this.db.deviceBindings.set(deviceId, {
        device_id: deviceId,
        license_key_hash: licenseKeyHash,
        public_key: publicKey,
        fingerprint_json: fingerprintJson,
        status,
        updated_at_ms: updatedAtMs,
      });
      return { success: true };
    }

    if (this.sql.includes('UPDATE reset_requests')) {
      const [status, updatedAtMs, requestId] = this.args;
      const existing = this.db.resetRequests.get(requestId);
      if (existing) {
        this.db.resetRequests.set(requestId, {
          ...existing,
          status,
          updated_at_ms: updatedAtMs,
        });
      }
      return { success: true };
    }

    if (this.sql.includes('UPDATE device_bindings')) {
      const [updatedAtMs, licenseKeyHash] = this.args;
      for (const [deviceId, binding] of this.db.deviceBindings.entries()) {
        if (binding.license_key_hash === licenseKeyHash && binding.status === 'active') {
          this.db.deviceBindings.set(deviceId, {
            ...binding,
            status: 'inactive',
            updated_at_ms: updatedAtMs,
          });
        }
      }
      return { success: true };
    }

    if (this.sql.includes('INSERT INTO reset_requests')) {
      const [requestId, licenseKeyHash, maskedLicenseKey, purchaserEmail, status, createdAtMs, updatedAtMs] = this.args;
      this.db.resetRequests.set(requestId, {
        request_id: requestId,
        license_key_hash: licenseKeyHash,
        masked_license_key: maskedLicenseKey,
        purchaser_email: purchaserEmail,
        status,
        created_at_ms: createdAtMs,
        updated_at_ms: updatedAtMs,
      });
      return { success: true };
    }

    if (this.sql.includes('INSERT INTO audit_events')) {
      const [eventType, actor, metadataJson, createdAtMs] = this.args;
      this.db.auditEvents.push({
        event_type: eventType,
        actor,
        metadata_json: metadataJson,
        created_at_ms: createdAtMs,
      });
      return { success: true };
    }

    throw new Error(`Unhandled SQL in test mock: ${this.sql}`);
  }
}

async function call(path, { method = 'POST', headers = {}, body, env = {} } = {}) {
  const mergedHeaders = new Headers({ 'content-type': 'application/json', ...headers });
  const req = new Request(`http://localhost${path}`, {
    method,
    headers: mergedHeaders,
    body: body
      ? mergedHeaders.get('content-type') === 'application/x-www-form-urlencoded'
        ? new URLSearchParams(body).toString()
        : JSON.stringify(body)
      : undefined,
  });
  return app.fetch(req, env);
}

async function sha256Hex(input) {
  const bytes = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
}

test('health works', async () => {
  const res = await call('/health', { method: 'GET' });
  assert.equal(res.status, 200);
  const json = await res.json();
  assert.equal(json.ok, true);
});

test('activate requires idempotency key', async () => {
  const res = await call('/v1/license/activate', {
    body: {
      license_key: 'AAAA-BBBB-CCCC-DDDD',
      device_public_key: 'pub',
      fingerprint: { os_name: 'linux', platform_family: 'linux', arch: 'x86_64' },
      app_version: '0.1.0',
      timestamp_ms: 1,
    },
  });
  assert.equal(res.status, 400);
  const json = await res.json();
  assert.equal(json.ok, false);
  assert.equal(json.error.code, 'bad_request');
});

test('reset status not found shape', async () => {
  const res = await call('/v1/license/reset/status', {
    body: { request_id: 'missing' },
    env: { DB: new MockD1Database() },
  });
  assert.equal(res.status, 404);
  const json = await res.json();
  assert.equal(json.error.code, 'reset_request_not_found');
});

test('reset request and reset status use D1 authoritative state', async () => {
  const db = new MockD1Database();
  const env = { DB: db, HASH_PEPPER: 'pepper_123', TOKEN_SIGNING_SECRET: 'sign_123' };

  const req = await call('/v1/license/reset/request', {
    headers: { 'x-idempotency-key': 'reset-1' },
    body: {
      license_key: 'AAAA-BBBB-CCCC-DDDD',
      masked_license_key: '••••-DDDD',
      device_public_key: 'pubkey-abc-123',
      fingerprint: { os_name: 'linux', platform_family: 'linux', arch: 'x86_64' },
      app_version: '0.1.0',
      timestamp_ms: Date.now(),
    },
    env,
  });
  assert.equal(req.status, 200);
  const reqJson = await req.json();
  assert.equal(reqJson.ok, true);
  assert.equal(reqJson.data.status, 'pending');

  const status = await call('/v1/license/reset/status', {
    body: { request_id: reqJson.data.request_id },
    env,
  });
  assert.equal(status.status, 200);
  const statusJson = await status.json();
  assert.equal(statusJson.ok, true);
  assert.equal(statusJson.data.status, 'pending');
});

test('admin reset routes require bearer token', async () => {
  const res = await call('/v1/admin/reset/requests?status=pending', {
    method: 'GET',
    env: { DB: new MockD1Database(), ADMIN_API_TOKEN: 'admin-secret' },
  });
  assert.equal(res.status, 401);
  const json = await res.json();
  assert.equal(json.error.code, 'unauthorized');
});

test('admin lists pending reset requests without raw purchaser email', async () => {
  const db = new MockD1Database();
  db.resetRequests.set('reset-1', {
    request_id: 'reset-1',
    license_key_hash: 'hash-1',
    masked_license_key: '••••-DDDD',
    purchaser_email: 'buyer@example.com',
    status: 'pending',
    created_at_ms: 1,
    updated_at_ms: 1,
  });

  const res = await call('/v1/admin/reset/requests?status=pending', {
    method: 'GET',
    headers: { authorization: 'Bearer admin-secret' },
    env: { DB: db, ADMIN_API_TOKEN: 'admin-secret' },
  });
  assert.equal(res.status, 200);
  const json = await res.json();
  assert.equal(json.ok, true);
  assert.equal(json.data.requests[0].status, 'pending');
  assert.equal(json.data.requests[0].masked_license_key, '••••-DDDD');
  assert.equal(json.data.requests[0].has_license_hash, true);
  assert.equal(json.data.requests[0].purchaser_email, 'b***@example.com');
  assert.equal(JSON.stringify(json).includes('buyer@example.com'), false);
});

test('admin approve reset unbinds active device and idempotently replays', async () => {
  const db = new MockD1Database();
  const env = {
    DB: db,
    HASH_PEPPER: 'pepper_123',
    TOKEN_SIGNING_SECRET: 'sign_123',
    ADMIN_API_TOKEN: 'admin-secret',
  };
  const licenseKey = 'AAAA-BBBB-CCCC-DDDD';
  const licenseHash = await sha256Hex(`${env.HASH_PEPPER}:${licenseKey}`);
  db.licenses.set(licenseHash, {
    license_key_hash: licenseHash,
    purchaser_email: 'buyer@example.com',
    entitlement_status: 'active',
    provider: 'gumroad',
    provider_sale_id: 'sale_9xy123',
    updated_at_ms: Date.now(),
  });

  const activate = await call('/v1/license/activate', {
    headers: { 'x-idempotency-key': 'act-admin-approve' },
    body: {
      license_key: licenseKey,
      device_public_key: 'pubkey-abc-123',
      fingerprint: { os_name: 'linux', platform_family: 'linux', arch: 'x86_64' },
      app_version: '0.1.0',
      timestamp_ms: Date.now(),
    },
    env,
  });
  const activateJson = await activate.json();

  const reset = await call('/v1/license/reset/request', {
    headers: { 'x-idempotency-key': 'reset-admin-approve' },
    body: {
      license_key: licenseKey,
      masked_license_key: '••••-DDDD',
      purchaser_email: 'buyer@example.com',
      device_public_key: 'pubkey-abc-123',
      fingerprint: { os_name: 'linux', platform_family: 'linux', arch: 'x86_64' },
      app_version: '0.1.0',
      timestamp_ms: Date.now(),
    },
    env,
  });
  const resetJson = await reset.json();

  const approve = await call('/v1/admin/reset/approve', {
    headers: { authorization: 'Bearer admin-secret', 'x-idempotency-key': 'approve-1' },
    body: { request_id: resetJson.data.request_id },
    env,
  });
  assert.equal(approve.status, 200);
  const approveJson = await approve.json();
  assert.equal(approveJson.data.status, 'approved');
  assert.equal(approveJson.data.license_state, 'UNBOUND');
  assert.equal(Array.from(db.deviceBindings.values())[0].status, 'inactive');

  const replay = await call('/v1/admin/reset/approve', {
    headers: { authorization: 'Bearer admin-secret', 'x-idempotency-key': 'approve-1' },
    body: { request_id: resetJson.data.request_id },
    env,
  });
  assert.equal(replay.status, 200);

  const validate = await call('/v1/license/validate', {
    body: { access_token: activateJson.data.access_token },
    env,
  });
  assert.equal(validate.status, 401);
  const validateJson = await validate.json();
  assert.equal(validateJson.error.code, 'reauth_required');
  assert.equal(db.auditEvents.at(-1).event_type, 'device_reset_approved');
  assert.equal(JSON.stringify(db.auditEvents.at(-1)).includes('buyer@example.com'), false);
});

test('admin reject reset preserves active device binding', async () => {
  const db = new MockD1Database();
  db.resetRequests.set('reset-1', {
    request_id: 'reset-1',
    license_key_hash: 'hash-1',
    masked_license_key: '••••-DDDD',
    purchaser_email: 'buyer@example.com',
    status: 'pending',
    created_at_ms: 1,
    updated_at_ms: 1,
  });
  db.deviceBindings.set('dev-1', {
    device_id: 'dev-1',
    license_key_hash: 'hash-1',
    public_key: 'pub',
    fingerprint_json: '{}',
    status: 'active',
    updated_at_ms: 1,
  });

  const reject = await call('/v1/admin/reset/reject', {
    headers: { authorization: 'Bearer admin-secret', 'x-idempotency-key': 'reject-1' },
    body: { request_id: 'reset-1', reason: 'manual review failed' },
    env: { DB: db, ADMIN_API_TOKEN: 'admin-secret' },
  });
  assert.equal(reject.status, 200);
  const rejectJson = await reject.json();
  assert.equal(rejectJson.data.status, 'rejected');
  assert.equal(db.deviceBindings.get('dev-1').status, 'active');
  assert.equal(db.resetRequests.get('reset-1').status, 'rejected');
  assert.equal(db.auditEvents.at(-1).event_type, 'device_reset_rejected');
  assert.equal(JSON.parse(db.auditEvents.at(-1).metadata_json).reason_present, true);
});

test('gumroad webhook accepts form-encoded ping payloads', async () => {
  const originalFetch = global.fetch;
  const db = new MockD1Database();
  global.fetch = async () =>
    new Response(
      JSON.stringify({
        success: true,
        sale: {
          id: 'sale_9xy123',
          product_id: 'prod_123',
          email: 'buyer@example.com',
          license_key: 'AAAA-BBBB-CCCC-DDDD',
        },
      }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    );
  try {
    const res = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: {
        sale_id: 'sale_9xy123',
        product_id: 'prod_123',
        email: 'buyer@example.com',
      },
      env: { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123', HASH_PEPPER: 'pepper_123' },
    });
    assert.equal(res.status, 200);
    const json = await res.json();
    assert.equal(json.ok, true);
    assert.equal(json.data.provider, 'gumroad');
    assert.equal(json.data.sale_id, 'sale_9xy123');
    assert.equal(json.data.verified, true);
    assert.equal(db.licenses.size, 1);
    assert.equal(db.auditEvents.length, 1);
  } finally {
    global.fetch = originalFetch;
  }
});

test('activate and validate use D1 authoritative state', async () => {
  const db = new MockD1Database();
  const env = { DB: db, HASH_PEPPER: 'pepper_123', TOKEN_SIGNING_SECRET: 'sign_123' };
  const licenseKey = 'AAAA-BBBB-CCCC-DDDD';
  const licenseHash = await sha256Hex(`${env.HASH_PEPPER}:${licenseKey}`);

  // Pre-seed verified active license as webhook flow would.
  db.licenses.set(licenseHash, {
      license_key_hash: licenseHash,
      purchaser_email: 'buyer@example.com',
      entitlement_status: 'active',
      provider: 'gumroad',
      provider_sale_id: 'sale_9xy123',
      updated_at_ms: Date.now(),
    });

  const activate = await call('/v1/license/activate', {
    headers: { 'x-idempotency-key': 'act-1' },
    body: {
      license_key: licenseKey,
      device_public_key: 'pubkey-abc-123',
      fingerprint: { os_name: 'linux', platform_family: 'linux', arch: 'x86_64' },
      app_version: '0.1.0',
      timestamp_ms: Date.now(),
    },
    env,
  });

  assert.equal(activate.status, 200);
  const activateJson = await activate.json();
  assert.equal(activateJson.ok, true);
  assert.equal(activateJson.data.entitlement, 'active');
  assert.equal(db.deviceBindings.size, 1);

  const validate = await call('/v1/license/validate', {
    body: { access_token: activateJson.data.access_token },
    env,
  });
  assert.equal(validate.status, 200);
  const validateJson = await validate.json();
  assert.equal(validateJson.ok, true);
  assert.equal(validateJson.data.entitlement, 'active');
  assert.equal(validateJson.data.bound_device.public_key, 'pubkey-abc-123');
});

test('gumroad webhook fails without access token', async () => {
  const res = await call('/v1/license/webhooks/gumroad', {
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body: { sale_id: 'sale_1', product_id: 'prod_1', email: 'buyer@example.com' },
    env: { DB: new MockD1Database() },
  });
  assert.equal(res.status, 401);
  const json = await res.json();
  assert.equal(json.ok, false);
  assert.equal(json.error.code, 'unauthorized');
});

test('gumroad webhook fails on verification mismatch', async () => {
  const originalFetch = global.fetch;
  const db = new MockD1Database();
  global.fetch = async () =>
    new Response(
      JSON.stringify({
        sale: { id: 'sale_other', product_id: 'prod_123', email: 'buyer@example.com' },
      }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    );
  try {
    const res = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123' },
    });
    assert.equal(res.status, 401);
    const json = await res.json();
    assert.equal(json.ok, false);
    assert.equal(json.error.code, 'unauthorized');
    assert.equal(json.error.message, 'Gumroad verification mismatch for: sale_id.');
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook returns not_found when Gumroad cannot find the sale id', async () => {
  const originalFetch = global.fetch;
  const db = new MockD1Database();
  global.fetch = async () =>
    new Response(
      JSON.stringify({
        success: false,
        message: 'The sale was not found.',
      }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    );
  try {
    const res = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: { sale_id: 'order_123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123' },
    });
    assert.equal(res.status, 404);
    const json = await res.json();
    assert.equal(json.ok, false);
    assert.equal(json.error.code, 'not_found');
    assert.equal(json.error.message, 'Gumroad sale was not found for the provided sale_id.');
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook replays stored response for identical sale payload', async () => {
  const originalFetch = global.fetch;
  const db = new MockD1Database();
  let fetchCalls = 0;
  global.fetch = async () => {
    fetchCalls += 1;
    return new Response(
      JSON.stringify({
        sale: {
          id: 'sale_9xy123',
          product_id: 'prod_123',
          email: 'buyer@example.com',
          license_key: 'AAAA-BBBB-CCCC-DDDD',
        },
      }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    );
  };

  try {
    const env = { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123', HASH_PEPPER: 'pepper_123' };
    const first = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env,
    });
    const second = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env,
    });

    assert.equal(first.status, 200);
    assert.equal(second.status, 200);
    assert.equal(fetchCalls, 1);
    assert.equal(db.auditEvents.length, 1);
  } finally {
    global.fetch = originalFetch;
  }
});
