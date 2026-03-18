import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('form-heavy portal pages use tabs and dialogs instead of always-expanded forms', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');

  assert.match(apiKeysPage, /<Dialog/);
  assert.match(apiKeysPage, /<Tabs/);
  assert.match(apiKeysPage, /Credential inventory/);

  assert.match(routingPage, /<Tabs/);
  assert.match(routingPage, /Policy editor/);
  assert.match(routingPage, /Evidence stream/);

  assert.match(userPage, /<Tabs/);
  assert.match(userPage, /<Dialog/);
  assert.match(userPage, /Security center/);
});
