import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('Devolens calls and token usage inventory is complete and secure', () => {
  const indexJsPath = path.resolve('src/index.js');
  const authWorkerRsPath = path.resolve('../app/src-tauri/src/auth_worker.rs');
  const privacyRsPath = path.resolve('../app/src-tauri/src/commands/privacy.rs');
  const inventoryPath = path.resolve('src/devolens_inventory.json');

  const filesToScan = [
    { name: 'worker/src/index.js', path: indexJsPath },
    { name: 'app/src-tauri/src/auth_worker.rs', path: authWorkerRsPath },
    { name: 'app/src-tauri/src/commands/privacy.rs', path: privacyRsPath }
  ];

  const detectedCalls = [];

  for (const file of filesToScan) {
    if (!fs.existsSync(file.path)) {
      continue;
    }
    const content = fs.readFileSync(file.path, 'utf8');
    
    // Scan for pattern like api/key/... or /api/key/...
    const regex = /(?:api\/key\/[A-Za-z]+)/g;
    let match;
    while ((match = regex.exec(content)) !== null) {
      const endpoint = '/' + match[0];
      if (!detectedCalls.some(c => c.endpoint === endpoint && c.caller === file.name)) {
        detectedCalls.push({ endpoint, caller: file.name });
      }
    }
  }

  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));

  // 1. Check that all detected calls are documented in the inventory
  for (const call of detectedCalls) {
    const inv = inventory.find(i => i.endpoint === call.endpoint && i.caller === call.caller);
    assert.ok(
      inv,
      `Detected Devolens call to "${call.endpoint}" in "${call.caller}" is missing from devolens_inventory.json`
    );

    // Validate fields presence
    assert.ok(inv.required_scope, `Entry for "${call.endpoint}" in "${call.caller}" is missing required_scope`);
    assert.ok(
      ['Client', 'Management'].includes(inv.token_security_category),
      `Entry for "${call.endpoint}" in "${call.caller}" has invalid token_security_category "${inv.token_security_category}"`
    );
    assert.ok(Array.isArray(inv.parameters), `Entry for "${call.endpoint}" in "${call.caller}" is missing parameters array`);
    assert.ok(inv.error_mapping, `Entry for "${call.endpoint}" in "${call.caller}" is missing error_mapping`);
    assert.ok(inv.retry_policy, `Entry for "${call.endpoint}" in "${call.caller}" is missing retry_policy`);
    assert.ok(typeof inv.migration_required === 'boolean', `Entry for "${call.endpoint}" in "${call.caller}" is missing migration_required boolean`);
  }

  // 2. Check for dead entries in the inventory
  for (const inv of inventory) {
    const isDetected = detectedCalls.some(c => c.endpoint === inv.endpoint && c.caller === inv.caller);
    assert.ok(
      isDetected,
      `Inventory documents a call to "${inv.endpoint}" from "${inv.caller}" which does not exist in the source files`
    );
  }

  // 3. ENFORCE SECURITY RULE: Never expose management-capable tokens/endpoints to desktop clients without flagging them for migration.
  for (const inv of inventory) {
    if (inv.caller.startsWith('app/src-tauri/') && inv.token_security_category === 'Management') {
      assert.equal(
        inv.migration_required,
        true,
        `SECURITY VIOLATION: Endpoint "${inv.endpoint}" is called by desktop client "${inv.caller}" using a Management token scope, but is not marked for migration!`
      );
    }
  }
});
