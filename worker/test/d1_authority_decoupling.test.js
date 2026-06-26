import test from 'node:test';
import assert from 'node:assert/strict';
import { MockD1Database, call } from './contract.test.js';

test('activation decisions cannot be made solely from D1 state and require Devolens key creation', async () => {
  const originalFetch = global.fetch;
  const db = new MockD1Database();
  global.fetch = async (url) => {
    if (url.includes('api.gumroad.com')) {
      return new Response(
        JSON.stringify({
          success: true,
          sale: {
            id: 'sale_9xy123',
            product_id: 'prod_123',
            email: 'buyer@example.com',
            license_key: 'GUMROAD-VAL-KEY',
          },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      );
    }
    if (url.includes('/api/key/CreateKey')) {
      return new Response(JSON.stringify({ result: 1, message: 'devolens rejected' }), { status: 200 });
    }
    return new Response(JSON.stringify({}), { status: 200 });
  };

  try {
    const res = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: { sale_id: 'sale_9xy123', product_id: 'prod_123', email: 'buyer@example.com' },
      env: {
        DB: db,
        GUMROAD_ACCESS_TOKEN: 'token_123',
        HASH_PEPPER: 'pepper_123',
        DEVOLENS_WEBHOOK_TOKEN: 'devolens_tok',
        DEVOLENS_PRODUCT_ID: 'dev_prod',
      },
    });

    assert.equal(res.status, 502);
    assert.equal(db.licenses.size, 0);
    assert.equal(db.idempotency.size, 0);
  } finally {
    global.fetch = originalFetch;
  }
});

test('validation decisions cannot be made solely from D1 state and require Devolens session verification', async () => {
  const db = new MockD1Database();
  db.licenses.set('d1-only-active-license', {
    license_key_hash: 'd1-only-active-license',
    purchaser_email: 'buyer@example.com',
    entitlement_status: 'active',
    provider: 'gumroad',
    provider_sale_id: 'sale_1',
    updated_at_ms: 1,
  });

  const res = await call('/v1/license/validate', {
    body: { license_key: 'D1-ONLY-ACTIVE-LICENSE' },
    env: { DB: db },
  });

  assert.equal(res.status, 404);
  assert.equal((await res.json()).error.code, 'route_not_found');
});

test('disable decisions (admin disable) cannot be made solely from D1 state and direct D1-only mutations are disabled', async () => {
  // Target behavior: Direct disabling via D1 is disabled/deprecated (returns 410 and does not mutate D1).
  const db = new MockD1Database();
  const licenseHash = 'test-lic-001';
  db.licenses.set(licenseHash, {
    license_key_hash: licenseHash,
    purchaser_email: 'buyer@example.com',
    entitlement_status: 'active',
    provider: 'gumroad',
    provider_sale_id: 'sale_1',
    updated_at_ms: 1,
  });

  const res = await call('/v1/admin/licenses/disable', {
    headers: { authorization: 'Bearer admin-secret', 'x-idempotency-key': 'disable-test-1' },
    body: {
      license_hash_prefix: 'test-lic',
      reason: 'test',
      deactivate_bindings: true,
    },
    env: { DB: db, ADMIN_API_TOKEN: 'admin-secret' },
  });

  assert.equal(res.status, 410);
  const json = await res.json();
  assert.equal(json.error.code, 'gone');
  assert.equal(db.licenses.get(licenseHash).entitlement_status, 'active');
});


test('reset decisions (admin reset approve) cannot be made solely from D1 state and require Devolens deactivation', async () => {
  const db = new MockD1Database();
  db.deviceBindings.set('dev-1', {
    device_id: 'dev-1',
    license_key_hash: 'license-hash',
    public_key: 'pub',
    fingerprint_json: '{}',
    status: 'active',
    updated_at_ms: 1,
  });

  const res = await call('/v1/admin/reset/approve', {
    headers: { authorization: 'Bearer admin-secret', 'x-idempotency-key': 'reset-approve-1' },
    body: { request_id: 'reset-1', reason: 'support review' },
    env: { DB: db, ADMIN_API_TOKEN: 'admin-secret' },
  });

  assert.equal(res.status, 410);
  assert.equal((await res.json()).error.code, 'gone');
  assert.equal(db.deviceBindings.get('dev-1').status, 'active');
});

test('refund decisions (gumroad webhook) cannot be made solely from D1 state and require Devolens BlockKey', async () => {
  const originalFetch = global.fetch;
  const db = new MockD1Database();
  const fetchRequests = [];
  global.fetch = async (url, options) => {
    fetchRequests.push({ url, options });
    if (url.includes('api.gumroad.com')) {
      return new Response(
        JSON.stringify({
          success: true,
          sale: {
            id: 'sale_9xy123',
            product_id: 'prod_123',
            email: 'buyer@example.com',
            license_key: 'GUMROAD-VAL-KEY',
            refunded: true,
          },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      );
    }
    if (url.includes('/api/key/BlockKey')) {
      return new Response(JSON.stringify({ result: 0 }), { status: 200 });
    }
    return new Response(JSON.stringify({}), { status: 200 });
  };

  try {
    const res = await call('/v1/license/webhooks/gumroad', {
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
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
    const blockKeyRequest = fetchRequests.find((request) => request.url.includes('/api/key/BlockKey'));
    assert.ok(blockKeyRequest);
    assert.ok(blockKeyRequest.options.body.includes('Key=GUMROAD-VAL-KEY'));
  } finally {
    global.fetch = originalFetch;
  }
});
