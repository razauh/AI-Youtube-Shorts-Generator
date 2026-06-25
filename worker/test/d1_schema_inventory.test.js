import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('D1 schema inventory matches migrations and is complete', () => {
  const migrationsDir = path.resolve('worker/migrations');
  const inventoryPath = path.resolve('worker/src/d1_schema_inventory.json');

  const files = fs.readdirSync(migrationsDir).sort();
  const schema = {};

  // Parse SQL migrations
  for (const file of files) {
    if (!file.endsWith('.sql')) continue;
    const content = fs.readFileSync(path.join(migrationsDir, file), 'utf8');
    const lines = content.split('\n');

    let currentTable = null;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      const createTableMatch = trimmed.match(/CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\w+)/i);
      if (createTableMatch) {
        currentTable = createTableMatch[1];
        schema[currentTable] = schema[currentTable] || new Set();
        continue;
      }

      if (currentTable && trimmed.startsWith(');')) {
        currentTable = null;
        continue;
      }

      if (currentTable) {
        if (/^(?:PRIMARY\s+KEY|UNIQUE|FOREIGN\s+KEY|CHECK|CONSTRAINT|INDEX)/i.test(trimmed)) {
          continue;
        }
        const colMatch = trimmed.match(/^(\w+)\s+(\w+)/);
        if (colMatch) {
          schema[currentTable].add(colMatch[1]);
        }
      } else {
        const alterTableMatch = trimmed.match(/ALTER\s+TABLE\s+(\w+)\s+ADD\s+(?:COLUMN\s+)?(\w+)/i);
        if (alterTableMatch) {
          const table = alterTableMatch[1];
          const col = alterTableMatch[2];
          schema[table] = schema[table] || new Set();
          schema[table].add(col);
        }
      }
    }
  }

  // Load inventory
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));

  const validClassifications = ['keep', 'deprecate', 'migrate', 'delete_later'];

  // Check that every table/column from migrations is documented in the inventory
  for (const table of Object.keys(schema)) {
    const tableInv = inventory.find(t => t.table === table);
    assert.ok(tableInv, `Table "${table}" found in migrations is missing from d1_schema_inventory.json`);
    
    assert.ok(tableInv.classification, `Table "${table}" is missing classification`);
    assert.ok(validClassifications.includes(tableInv.classification), `Table "${table}" has invalid classification "${tableInv.classification}"`);
    assert.ok(tableInv.retention_rule && tableInv.retention_rule.trim() !== '', `Table "${table}" is missing retention_rule`);
    assert.ok(tableInv.compatibility_rule && tableInv.compatibility_rule.trim() !== '', `Table "${table}" is missing compatibility_rule`);

    for (const column of schema[table]) {
      const colInv = tableInv.columns?.find(c => c.name === column);
      assert.ok(colInv, `Column "${table}.${column}" found in migrations is missing from d1_schema_inventory.json`);
      
      assert.ok(colInv.classification, `Column "${table}.${column}" is missing classification`);
      assert.ok(validClassifications.includes(colInv.classification), `Column "${table}.${column}" has invalid classification "${colInv.classification}"`);
      assert.ok(colInv.retention_rule && colInv.retention_rule.trim() !== '', `Column "${table}.${column}" is missing retention_rule`);
      assert.ok(colInv.compatibility_rule && colInv.compatibility_rule.trim() !== '', `Column "${table}.${column}" is missing compatibility_rule`);
      assert.ok(Array.isArray(colInv.caller_mapping), `Column "${table}.${column}" field "caller_mapping" must be an array`);
      for (const caller of colInv.caller_mapping) {
        assert.ok(typeof caller === 'string' && caller.trim() !== '', `Column "${table}.${column}" has empty caller reference`);
      }
    }
  }

  // Check for dead documentation (entries in inventory that do not exist in migrations)
  for (const t of inventory) {
    assert.ok(Object.hasOwn(schema, t.table), `Inventory contains table "${t.table}" which is not defined in any migrations`);
    for (const c of t.columns || []) {
      assert.ok(schema[t.table].has(c.name), `Inventory contains column "${t.table}.${c.name}" which is not defined in any migrations`);
    }
  }
});
