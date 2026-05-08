import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';

test('frontend shell boot entry exists', () => {
  assert.equal(existsSync('src/main.js'), true);

  const html = readFileSync('index.html', 'utf8');
  assert.match(html, /<div id="app"><\/div>/);
});
