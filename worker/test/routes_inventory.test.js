import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('Worker routes inventory matches implementation', () => {
  const indexJsPath = path.resolve('src/index.js');
  const inventoryPath = path.resolve('src/routes_inventory.json');

  const indexJsContent = fs.readFileSync(indexJsPath, 'utf8');
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));

  // Parse lines to find exact path matches
  // Pattern: if (method === "METHOD" && path === "PATH")
  const pathEqualsRegex = /method\s*===\s*["'](GET|POST|PUT|DELETE)["']\s*&&\s*path\s*===\s*["']([^"']+)["']/g;
  
  const extractedRoutes = [];
  let match;
  while ((match = pathEqualsRegex.exec(indexJsContent)) !== null) {
    extractedRoutes.push({
      method: match[1],
      path: match[2]
    });
  }

  // Parse lines to find regex matches
  // Pattern: const varName = path.match(/.../)
  // and: if (method === "METHOD" && varName)
  const lines = indexJsContent.split('\n');
  for (const line of lines) {
    const regexMatch = line.match(/const\s+(\w+Match)\s*=\s*path\.match\(\/((?:\\.|\[(?:\\\]|[^\]])+\]|[^/])+)\/\)/);
    if (regexMatch) {
      const varName = regexMatch[1];
      const pattern = regexMatch[2];
      // Find method
      const methodLine = lines.find(l => l.includes('method ===') && l.includes(varName));
      if (methodLine) {
        const methodMatch = methodLine.match(/method\s*===\s*["'](GET|POST|PUT|DELETE)["']/);
        if (methodMatch) {
          extractedRoutes.push({
            method: methodMatch[1],
            pathPattern: pattern
          });
        }
      }
    }
  }

  // Ensure we extracted routes
  assert.ok(extractedRoutes.length > 0, 'Should extract at least one route from index.js');

  // Verify that every extracted route is present in the inventory
  for (const route of extractedRoutes) {
    if (route.path) {
      const found = inventory.find(inv => inv.method === route.method && inv.path === route.path);
      assert.ok(found, `Route ${route.method} ${route.path} was found in index.js but is missing from routes_inventory.json`);
    } else if (route.pathPattern) {
      const found = inventory.find(inv => inv.method === route.method && inv.pathPattern === route.pathPattern);
      assert.ok(found, `Route ${route.method} pattern ${route.pathPattern} was found in index.js but is missing from routes_inventory.json`);
    }
  }

  // Verify that every route in the inventory is valid and fully documented
  const requiredFields = [
    'owner',
    'authority_source',
    'retained_or_deprecated',
    'auth_requirement',
    'caller_surface',
    'response_contract',
    'compatibility_deadline'
  ];

  for (const inv of inventory) {
    const identifier = inv.path ? `${inv.method} ${inv.path}` : `${inv.method} pattern ${inv.pathPattern}`;
    
    // Check that it exists in the codebase (no dead documentation)
    const existsInCode = extractedRoutes.some(route => {
      if (inv.path) {
        return route.method === inv.method && route.path === inv.path;
      } else {
        return route.method === inv.method && route.pathPattern === inv.pathPattern;
      }
    });
    assert.ok(existsInCode, `Inventory contains route ${identifier} which does not exist in index.js`);

    // Verify all required fields
    for (const field of requiredFields) {
      assert.ok(Object.hasOwn(inv, field), `Route ${identifier} is missing required field "${field}"`);
      assert.ok(inv[field] !== null && inv[field] !== undefined && String(inv[field]).trim() !== '', `Route ${identifier} has empty field "${field}"`);
    }

    // Verify specific values
    assert.ok(['retained', 'deprecated'].includes(inv.retained_or_deprecated), `Route ${identifier} field "retained_or_deprecated" must be "retained" or "deprecated"`);

    // Verify exposes_pii_or_secrets consistency
    assert.ok(Object.hasOwn(inv, 'exposes_pii_or_secrets'), `Route ${identifier} is missing "exposes_pii_or_secrets"`);
    assert.ok(typeof inv.exposes_pii_or_secrets === 'boolean', `Route ${identifier} field "exposes_pii_or_secrets" must be a boolean`);
    assert.ok(Array.isArray(inv.exposed_elements), `Route ${identifier} field "exposed_elements" must be an array`);

    if (inv.exposes_pii_or_secrets) {
      assert.ok(inv.exposed_elements.length > 0, `Route ${identifier} is marked as exposing PII/secrets, but "exposed_elements" list is empty`);
    } else {
      assert.equal(inv.exposed_elements.length, 0, `Route ${identifier} is marked as NOT exposing PII/secrets, but "exposed_elements" list is not empty`);
    }
  }
});
