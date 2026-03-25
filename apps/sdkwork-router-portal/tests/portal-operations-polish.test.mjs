import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('api keys page exposes lifecycle, environment, and key-handling guidance', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const toolbar = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx');
  const dialogs = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx');
  const table = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-api-keys/src/repository/index.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(apiKeysPage, /Credential inventory/);
  assert.match(toolbar, /Open usage/);
  assert.match(dialogs, /Recommended key setup/);
  assert.match(dialogs, /Key label/);
  assert.match(dialogs, /Custom environment/);
  assert.match(dialogs, /Lifecycle policy/);
  assert.match(dialogs, /How to use this key/);
  assert.match(table, /Last authenticated use/);
  assert.match(table, /Usage method/);
  assert.match(table, /Delete/);
  assert.match(table, /Disable|Enable/);
  assert.match(portalApi, /label: string/);
  assert.match(portalApi, /expires_at_ms\?: number \| null/);
  assert.match(portalApi, /updatePortalApiKeyStatus/);
  assert.match(portalApi, /deletePortalApiKey/);
  assert.match(repository, /setPortalApiKeyActive/);
  assert.match(repository, /removePortalApiKey/);
  assert.match(portalTypes, /label: string/);
  assert.match(portalTypes, /created_at_ms: number/);
  assert.match(portalTypes, /last_used_at_ms\?: number \| null/);
  assert.match(portalTypes, /expires_at_ms\?: number \| null/);
});

test('usage page stays focused on a compact request log workbench', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');

  assert.match(usagePage, /Search usage/);
  assert.match(usagePage, /Manage keys/);
  assert.match(usagePage, /Review billing/);
  assert.match(usagePage, /Recorded/);
  assert.doesNotMatch(usagePage, /Request volume/);
  assert.doesNotMatch(usagePage, /Demand mix/);
  assert.doesNotMatch(usagePage, /Request diagnostics/);
});

test('user page exposes trust, security, and recovery guidance', () => {
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');

  assert.match(userPage, /Profile facts/);
  assert.match(userPage, /Personal security checklist/);
  assert.match(userPage, /Recovery signals/);
});

test('account page exposes financial posture, ledger, and runway guidance', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(accountPage, /portal-account-toolbar/);
  assert.match(accountPage, /Search ledger/);
  assert.doesNotMatch(accountPage, /Remaining units:/);
});
