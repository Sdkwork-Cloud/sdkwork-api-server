import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin shared table exposes slot structure and sticky commercial header styling', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(commons, /data-slot="table-container"/);
  assert.match(commons, /data-slot="table-header"/);
  assert.match(commons, /data-slot="table-empty"/);
  assert.match(theme, /position:\s*sticky;/);
  assert.match(theme, /border-spacing:\s*0;/);
});
