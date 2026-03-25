import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal route pages expose contextual handoffs to the next user action', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const apiKeysToolbar = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.match(apiKeysToolbar, /Open usage/);
  assert.match(usagePage, /Manage keys/);
  assert.match(usagePage, /Review billing/);
  assert.match(creditsPage, /Search offers or ledger/);
  assert.match(billingPage, /Activation path/);
  assert.match(userPage, /Return to command center/);
  assert.match(accountPage, /Search ledger/);
});
