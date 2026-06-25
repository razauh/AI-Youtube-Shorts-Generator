import test from 'node:test';
import assert from 'node:assert/strict';
import app from '../src/index.js';

class WebhookD1 {
  constructor({ failBatch = false } = {}) {
    this.failBatch = failBatch;
    this.idempotency = new Map();
    this.licenses = new Map();
    this.auditEvents = [];
  }

  prepare(sql) {
    return new WebhookStatement(this, sql);
  }

  async batch(statements) {
    if (this.failBatch) {
      throw new Error('simulated d1 batch failure');
    }
    const results = [];
    for (const statement of statements) {
      results.push(await statement.run());
    }
    return results;
  }
}

class WebhookStatement {
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
    return null;
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

    if (this.sql.includes('UPDATE licenses')) {
      const [entitlementStatus, updatedAtMs, licenseKeyHash] = this.args;
      const existing = this.db.licenses.get(licenseKeyHash) || { license_key_hash: licenseKeyHash };
      this.db.licenses.set(licenseKeyHash, {
        ...existing,
        entitlement_status: entitlementStatus,
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

    throw new Error(`Unhandled SQL in webhook failure test mock: ${this.sql}`);
  }
}

async function callWebhook({ headers = {}, body, env = {}, method = 'POST' } = {}) {
  const mergedHeaders = new Headers({ 'content-type': 'application/x-www-form-urlencoded', ...headers });
  const request = new Request('http://localhost/v1/license/webhooks/gumroad', {
    method,
    headers: mergedHeaders,
    body: body ? new URLSearchParams(body).toString() : undefined,
  });
  return app.fetch(request, env);
}

function activeSalePayload(overrides = {}) {
  return {
    success: true,
    sale: {
      id: 'sale_9xy123',
      product_id: 'prod_123',
      email: 'buyer@example.com',
      license_key: 'GUMROAD-VAL-KEY',
      ...overrides,
    },
  };
}

test('gumroad webhook fails with non-retryable 400 for invalid content type', async () => {
  const originalFetch = global.fetch;
  let fetchCalls = 0;
  global.fetch = async () => {
    fetchCalls += 1;
    return new Response('{}', { status: 200 });
  };

  try {
    const res = await callWebhook({
      headers: { 'content-type': 'application/json' },
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: new WebhookD1(), GUMROAD_ACCESS_TOKEN: 'token_123' },
    });

    assert.equal(res.status, 400);
    const json = await res.json();
    assert.equal(json.error.code, 'bad_request');
    assert.equal(json.error.retryable, false);
    assert.equal(fetchCalls, 0);
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook fails with non-retryable 409 for duplicate sale ID with mismatched payload', async () => {
  const originalFetch = global.fetch;
  const db = new WebhookD1();
  global.fetch = async () =>
    new Response(JSON.stringify(activeSalePayload()), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });

  try {
    const env = { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123', HASH_PEPPER: 'pepper_123' };
    const first = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env,
    });
    const second = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_other', email: 'buyer@example.com' },
      env,
    });

    assert.equal(first.status, 200);
    assert.equal(second.status, 409);
    const json = await second.json();
    assert.equal(json.error.code, 'invalid_transition');
    assert.equal(json.error.retryable, false);
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook distinguishes retryable and terminal Gumroad verification failures', async () => {
  const missingToken = await callWebhook({
    body: { sale_id: 'sale_1', product_id: 'prod_123', email: 'buyer@example.com' },
    env: { DB: new WebhookD1() },
  });
  assert.equal(missingToken.status, 401);
  assert.equal((await missingToken.json()).error.retryable, false);

  const originalFetch = global.fetch;
  global.fetch = async () => {
    throw new Error('network unavailable');
  };

  try {
    const networkFailure = await callWebhook({
      body: { sale_id: 'sale_1', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: new WebhookD1(), GUMROAD_ACCESS_TOKEN: 'token_123' },
    });
    assert.equal(networkFailure.status, 503);
    const networkJson = await networkFailure.json();
    assert.equal(networkJson.error.code, 'worker_unreachable');
    assert.equal(networkJson.error.retryable, true);
  } finally {
    global.fetch = originalFetch;
  }

  global.fetch = async () =>
    new Response(JSON.stringify({ success: false, message: 'The sale was not found.' }), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });

  try {
    const notFound = await callWebhook({
      body: { sale_id: 'sale_1', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: new WebhookD1(), GUMROAD_ACCESS_TOKEN: 'token_123' },
    });
    assert.equal(notFound.status, 404);
    const notFoundJson = await notFound.json();
    assert.equal(notFoundJson.error.code, 'not_found');
    assert.equal(notFoundJson.error.retryable, false);
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook audits refund BlockKey failure and keeps refund response non-retryable', async () => {
  const originalFetch = global.fetch;
  const db = new WebhookD1();
  global.fetch = async (url) => {
    if (url.includes('api.gumroad.com')) {
      return new Response(JSON.stringify(activeSalePayload({ refunded: true })), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      });
    }
    return new Response(JSON.stringify({ result: 1, message: 'block failed' }), { status: 200 });
  };

  try {
    const res = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: {
        DB: db,
        GUMROAD_ACCESS_TOKEN: 'token_123',
        HASH_PEPPER: 'pepper_123',
        DEVOLENS_WEBHOOK_TOKEN: 'devolens_tok',
        DEVOLENS_PRODUCT_ID: 'dev_prod',
      },
    });

    assert.equal(res.status, 409);
    const json = await res.json();
    assert.equal(json.error.code, 'invalid_transition');
    assert.equal(json.error.retryable, false);
    assert.ok(db.auditEvents.some((event) => event.event_type === 'gumroad_refund_devolens_block_failed'));
    assert.ok(db.auditEvents.some((event) => event.event_type === 'license_disabled'));
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook returns retryable 503 if Devolens key creation is unreachable', async () => {
  const originalFetch = global.fetch;
  const db = new WebhookD1();
  global.fetch = async (url) => {
    if (url.includes('api.gumroad.com')) {
      return new Response(JSON.stringify(activeSalePayload()), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      });
    }
    return new Response('unavailable', { status: 503 });
  };

  try {
    const res = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: {
        DB: db,
        GUMROAD_ACCESS_TOKEN: 'token_123',
        HASH_PEPPER: 'pepper_123',
        DEVOLENS_WEBHOOK_TOKEN: 'devolens_tok',
        DEVOLENS_PRODUCT_ID: 'dev_prod',
      },
    });

    assert.equal(res.status, 503);
    const json = await res.json();
    assert.equal(json.error.code, 'worker_unreachable');
    assert.equal(json.error.retryable, true);
    assert.equal(db.licenses.size, 0);
    assert.equal(db.idempotency.size, 0);
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook rejects missing license key and invalid provider payload without mutation', async () => {
  const originalFetch = global.fetch;
  const fetchUrls = [];
  global.fetch = async (url) => {
    fetchUrls.push(url);
    return new Response(JSON.stringify(activeSalePayload({ license_key: undefined })), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });
  };

  try {
    const db = new WebhookD1();
    const missingLicense = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: {
        DB: db,
        GUMROAD_ACCESS_TOKEN: 'token_123',
        HASH_PEPPER: 'pepper_123',
        DEVOLENS_WEBHOOK_TOKEN: 'devolens_tok',
        DEVOLENS_PRODUCT_ID: 'dev_prod',
      },
    });
    assert.equal(missingLicense.status, 409);
    assert.equal((await missingLicense.json()).error.retryable, false);
    assert.equal(fetchUrls.filter((url) => url.includes('/api/key/CreateKey')).length, 0);
    assert.equal(db.licenses.size, 0);
  } finally {
    global.fetch = originalFetch;
  }

  global.fetch = async () =>
    new Response(JSON.stringify({ success: true, sale: { id: 'sale_9xy123' } }), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });

  try {
    const db = new WebhookD1();
    const invalidPayload = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123', HASH_PEPPER: 'pepper_123' },
    });
    assert.equal(invalidPayload.status, 503);
    const json = await invalidPayload.json();
    assert.equal(json.error.code, 'serialization');
    assert.equal(json.error.retryable, false);
    assert.equal(db.licenses.size, 0);
    assert.equal(db.auditEvents.length, 0);
  } finally {
    global.fetch = originalFetch;
  }
});

test('gumroad webhook returns retryable 503 if D1 mapping write fails', async () => {
  const originalFetch = global.fetch;
  const db = new WebhookD1({ failBatch: true });
  global.fetch = async () =>
    new Response(JSON.stringify(activeSalePayload()), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });

  try {
    const res = await callWebhook({
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: { DB: db, GUMROAD_ACCESS_TOKEN: 'token_123', HASH_PEPPER: 'pepper_123' },
    });

    assert.equal(res.status, 503);
    const json = await res.json();
    assert.equal(json.error.code, 'storage');
    assert.equal(json.error.retryable, true);
    assert.equal(db.idempotency.size, 0);
  } finally {
    global.fetch = originalFetch;
  }
});
