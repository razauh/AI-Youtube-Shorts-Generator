import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('D1 authority inventory matches store.js exports and is complete', () => {
  const storePath = path.resolve('src/store.js');
  const inventoryPath = path.resolve('src/d1_authority_inventory.json');

  const content = fs.readFileSync(storePath, 'utf8');
  const lines = content.split('\n');

  const storeFunctions = new Set();
  for (const line of lines) {
    const match = line.match(/^export\s+async\s+function\s+(\w+)/);
    if (match) {
      storeFunctions.add(match[1]);
    }
  }

  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));
  const validClassifications = ['authority', 'cache', 'mapping', 'audit', 'privacy'];

  // Check that every store function is documented
  for (const func of storeFunctions) {
    const inv = inventory.find(i => i.function_name === func);
    assert.ok(inv, `Function "${func}" in store.js is missing from d1_authority_inventory.json`);
    assert.ok(Array.isArray(inv.tables), `Function "${func}" is missing "tables" array`);
    assert.ok(inv.classification, `Function "${func}" is missing classification`);
    assert.ok(validClassifications.includes(inv.classification), `Function "${func}" has invalid classification "${inv.classification}"`);
    assert.ok(inv.description && inv.description.trim() !== '', `Function "${func}" is missing description`);
    assert.ok(inv.devolens_decoupling_strategy && inv.devolens_decoupling_strategy.trim() !== '', `Function "${func}" is missing devolens_decoupling_strategy`);
  }

  // Check for dead documentation
  for (const inv of inventory) {
    assert.ok(storeFunctions.has(inv.function_name), `Inventory contains function "${inv.function_name}" which is not exported by store.js`);
  }
});
