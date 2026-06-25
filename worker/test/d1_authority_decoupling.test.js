import test from 'node:test';
import assert from 'node:assert/strict';
import { MockD1Database, call } from './contract.test.js';

test('activation decisions cannot be made solely from D1 state and require Devolens key creation', { todo: true }, () => {
  // Target behavior: Gumroad webhook must successfully call Devolens api/key/CreateKey.
  // If Devolens call fails or token is missing, the activation must fail.
  assert.fail('Devolens integration for key creation must be enforced.');
});

test('validation decisions cannot be made solely from D1 state and require Devolens session verification', { todo: true }, () => {
  // Target behavior: Client license validation must go directly to Devolens or use a verified local cache signature.
  // It cannot trust local D1 rows blindly as the authority source.
  assert.fail('Devolens validation contract must be enforced.');
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


test('reset decisions (admin reset approve) cannot be made solely from D1 state and require Devolens deactivation', { todo: true }, () => {
  // Target behavior: Approving a device reset must deactivate the binding on Devolens.
  // It cannot solely update D1 device_bindings state to inactive.
  assert.fail('Devolens deactivation request must be invoked.');
});

test('refund decisions (gumroad webhook) cannot be made solely from D1 state and require Devolens BlockKey', { todo: true }, () => {
  // Target behavior: Refunding a purchase via Gumroad webhook must block the key on Devolens.
  // Webhook must abort/fail if Devolens block fails or is bypassed.
  assert.fail('Devolens BlockKey request must be enforced.');
});
