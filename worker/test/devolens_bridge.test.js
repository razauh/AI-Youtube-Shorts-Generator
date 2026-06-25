import test from 'node:test';
import assert from 'node:assert/strict';
import { createDevolensKey, blockDevolensKey, deactivateDevolensKey } from '../src/devolensBridge.js';

const mockEnv = {
  DEVOLENS_ACCESS_TOKEN: 'management_token_123',
  DEVOLENS_PRODUCT_ID: 'prod_987',
  DEVOLENS_BASE_URL: 'https://devolens.test'
};

test('createDevolensKey constructs correct URL, uses POST form urlencoded, and sends token', async () => {
  const originalFetch = global.fetch;
  let callArgs = null;
  global.fetch = async (url, options) => {
    callArgs = { url, options };
    return new Response(JSON.stringify({ result: 0, message: 'success' }));
  };

  try {
    const res = await createDevolensKey(mockEnv, 'test-key-123');
    assert.equal(res.ok, true);
    assert.equal(callArgs.url, 'https://devolens.test/api/key/CreateKey');
    assert.equal(callArgs.options.method, 'POST');
    assert.equal(callArgs.options.headers['content-type'], 'application/x-www-form-urlencoded');

    const params = new URLSearchParams(callArgs.options.body);
    assert.equal(params.get('token'), 'management_token_123');
    assert.equal(params.get('ProductId'), 'prod_987');
    assert.equal(params.get('Key'), 'test-key-123');
  } finally {
    global.fetch = originalFetch;
  }
});

test('blockDevolensKey constructs correct URL, uses BlockKey endpoint', async () => {
  const originalFetch = global.fetch;
  let callArgs = null;
  global.fetch = async (url, options) => {
    callArgs = { url, options };
    return new Response(JSON.stringify({ result: 0 }));
  };

  try {
    const res = await blockDevolensKey(mockEnv, 'test-key-123');
    assert.equal(res.ok, true);
    assert.equal(callArgs.url, 'https://devolens.test/api/key/BlockKey');
    const params = new URLSearchParams(callArgs.options.body);
    assert.equal(params.get('Key'), 'test-key-123');
  } finally {
    global.fetch = originalFetch;
  }
});

test('deactivateDevolensKey constructs correct URL, uses Deactivate endpoint', async () => {
  const originalFetch = global.fetch;
  let callArgs = null;
  global.fetch = async (url, options) => {
    callArgs = { url, options };
    return new Response(JSON.stringify({ result: 0 }));
  };

  try {
    const res = await deactivateDevolensKey(mockEnv, 'test-key-123', 'mac_abc');
    assert.equal(res.ok, true);
    assert.equal(callArgs.url, 'https://devolens.test/api/key/Deactivate');
    const params = new URLSearchParams(callArgs.options.body);
    assert.equal(params.get('Key'), 'test-key-123');
    assert.equal(params.get('MachineCode'), 'mac_abc');
  } finally {
    global.fetch = originalFetch;
  }
});

test('bridge handles timeout via AbortController and classifies it as retryable', async () => {
  const originalFetch = global.fetch;
  global.fetch = async (url, options) => {
    assert.ok(options.signal instanceof AbortSignal);
    throw new DOMException('The user aborted a request.', 'AbortError');
  };

  try {
    const res = await createDevolensKey(mockEnv, 'test-key-123');
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

  try {
    const res = await createDevolensKey(mockEnv, 'test-key-123');
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

  try {
    const res = await createDevolensKey(mockEnv, 'test-key-123');
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
    throw new Error(`Failed to send token ${mockEnv.DEVOLENS_ACCESS_TOKEN} for key test-key-123`);
  };

  try {
    const res = await createDevolensKey(mockEnv, 'test-key-123');
    assert.equal(res.ok, false);
    assert.ok(!res.message.includes('management_token_123'));
    assert.ok(!res.message.includes('test-key-123'));
    assert.ok(res.message.includes('[redacted-token]'));
    assert.ok(res.message.includes('[redacted-key]'));
  } finally {
    global.fetch = originalFetch;
  }
});
