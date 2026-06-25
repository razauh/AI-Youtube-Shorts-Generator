import test from 'node:test';
import assert from 'node:assert/strict';
import { createDevolensKey, blockDevolensKey, deactivateDevolensKey } from '../src/devolensBridge.js';

test('createDevolensKey constructs correct URL, uses DEVOLENS_WEBHOOK_TOKEN', async () => {
  const originalFetch = global.fetch;
  let callArgs = null;
  global.fetch = async (url, options) => {
    callArgs = { url, options };
    return new Response(JSON.stringify({ result: 0, message: 'success' }));
  };

  const env = {
    DEVOLENS_WEBHOOK_TOKEN: 'webhook_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987',
    DEVOLENS_BASE_URL: 'https://devolens.test'
  };

  try {
    const res = await createDevolensKey(env, 'test-key-123');
    assert.equal(res.ok, true);
    assert.equal(callArgs.url, 'https://devolens.test/api/key/CreateKey');
    const params = new URLSearchParams(callArgs.options.body);
    assert.equal(params.get('token'), 'webhook_token_123');
  } finally {
    global.fetch = originalFetch;
  }
});

test('blockDevolensKey constructs correct URL, uses DEVOLENS_WEBHOOK_TOKEN', async () => {
  const originalFetch = global.fetch;
  let callArgs = null;
  global.fetch = async (url, options) => {
    callArgs = { url, options };
    return new Response(JSON.stringify({ result: 0 }));
  };

  const env = {
    DEVOLENS_WEBHOOK_TOKEN: 'webhook_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987',
    DEVOLENS_BASE_URL: 'https://devolens.test'
  };

  try {
    const res = await blockDevolensKey(env, 'test-key-123');
    assert.equal(res.ok, true);
    assert.equal(callArgs.url, 'https://devolens.test/api/key/BlockKey');
    const params = new URLSearchParams(callArgs.options.body);
    assert.equal(params.get('token'), 'webhook_token_123');
  } finally {
    global.fetch = originalFetch;
  }
});

test('deactivateDevolensKey constructs correct URL, uses DEVOLENS_CLIENT_TOKEN', async () => {
  const originalFetch = global.fetch;
  let callArgs = null;
  global.fetch = async (url, options) => {
    callArgs = { url, options };
    return new Response(JSON.stringify({ result: 0 }));
  };

  const env = {
    DEVOLENS_CLIENT_TOKEN: 'client_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987',
    DEVOLENS_BASE_URL: 'https://devolens.test'
  };

  try {
    const res = await deactivateDevolensKey(env, 'test-key-123', 'mac_abc');
    assert.equal(res.ok, true);
    assert.equal(callArgs.url, 'https://devolens.test/api/key/Deactivate');
    const params = new URLSearchParams(callArgs.options.body);
    assert.equal(params.get('token'), 'client_token_123');
  } finally {
    global.fetch = originalFetch;
  }
});

test('bridge falls back to DEVOLENS_ACCESS_TOKEN and warns when specific token is missing', async () => {
  const originalFetch = global.fetch;
  const originalWarn = console.warn;
  let warnings = [];
  console.warn = (...args) => {
    warnings.push(args.join(' '));
  };

  global.fetch = async () => new Response(JSON.stringify({ result: 0 }));

  const env = {
    DEVOLENS_ACCESS_TOKEN: 'legacy_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987',
    DEVOLENS_BASE_URL: 'https://devolens.test'
  };

  try {
    const res = await deactivateDevolensKey(env, 'test-key-123', 'mac_abc');
    assert.equal(res.ok, true);
    assert.equal(warnings.length, 1);
    assert.ok(warnings[0].includes('DEVOLENS_ACCESS_TOKEN is deprecated'));
    assert.ok(warnings[0].includes('DEVOLENS_CLIENT_TOKEN'));
  } finally {
    global.fetch = originalFetch;
    console.warn = originalWarn;
  }
});

test('bridge returns configuration missing error with the correct token variable name', async () => {
  const env = {
    DEVOLENS_PRODUCT_ID: 'prod_987'
  };

  const res = await createDevolensKey(env, 'test-key-123');
  assert.equal(res.ok, false);
  assert.equal(res.code, 'devolens_error');
  assert.ok(res.message.includes('DEVOLENS_WEBHOOK_TOKEN'));
});

test('bridge handles timeout via AbortController and classifies it as retryable', async () => {
  const originalFetch = global.fetch;
  global.fetch = async (url, options) => {
    assert.ok(options.signal instanceof AbortSignal);
    throw new DOMException('The user aborted a request.', 'AbortError');
  };

  const env = {
    DEVOLENS_WEBHOOK_TOKEN: 'webhook_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987'
  };

  try {
    const res = await createDevolensKey(env, 'test-key-123');
    assert.equal(res.ok, false);
    assert.equal(res.code, 'worker_unreachable');
    assert.equal(res.retryable, true);
  } finally {
    global.fetch = originalFetch;
  }
});

test('bridge maps non-2xx HTTP responses to retryable or terminal codes safely', async () => {
  const originalFetch = global.fetch;
  global.fetch = async () => new Response('Internal Server Error', { status: 500 });

  const env = {
    DEVOLENS_WEBHOOK_TOKEN: 'webhook_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987'
  };

  try {
    const res = await createDevolensKey(env, 'test-key-123');
    assert.equal(res.ok, false);
    assert.equal(res.code, 'worker_unreachable');
    assert.equal(res.status, 503);
    assert.equal(res.retryable, true);
  } finally {
    global.fetch = originalFetch;
  }
});

test('bridge maps Devolens result non-zero response to terminal failure', async () => {
  const originalFetch = global.fetch;
  global.fetch = async () => new Response(JSON.stringify({ result: 1, message: 'Key already exists' }));

  const env = {
    DEVOLENS_WEBHOOK_TOKEN: 'webhook_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987'
  };

  try {
    const res = await createDevolensKey(env, 'test-key-123');
    assert.equal(res.ok, false);
    assert.equal(res.code, 'devolens_error');
    assert.equal(res.status, 502);
    assert.equal(res.retryable, false);
    assert.equal(res.message, 'Key already exists');
  } finally {
    global.fetch = originalFetch;
  }
});

test('bridge redacts token and raw license key from error messages and logs', async () => {
  const originalFetch = global.fetch;
  global.fetch = async () => {
    throw new Error('Failed to send token webhook_token_123 for key test-key-123');
  };

  const env = {
    DEVOLENS_WEBHOOK_TOKEN: 'webhook_token_123',
    DEVOLENS_PRODUCT_ID: 'prod_987'
  };

  try {
    const res = await createDevolensKey(env, 'test-key-123');
    assert.equal(res.ok, false);
    assert.ok(!res.message.includes('webhook_token_123'));
    assert.ok(!res.message.includes('test-key-123'));
    assert.ok(res.message.includes('[redacted-token]'));
    assert.ok(res.message.includes('[redacted-key]'));
  } finally {
    global.fetch = originalFetch;
  }
});
