import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('api keys page exposes environment strategy and key-handling guidance', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-api-keys/src/repository/index.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(apiKeysPage, /Environment strategy/);
  assert.match(apiKeysPage, /Rotation checklist/);
  assert.match(apiKeysPage, /Key handling guardrails/);
  assert.match(apiKeysPage, /Key label/);
  assert.match(apiKeysPage, /Expires at/);
  assert.match(apiKeysPage, /Last used/);
  assert.match(apiKeysPage, /Revoke/);
  assert.match(apiKeysPage, /Restore/);
  assert.match(apiKeysPage, /Delete/);
  assert.match(portalApi, /label: string/);
  assert.match(portalApi, /expires_at_ms\?: number \| null/);
  assert.match(portalApi, /updatePortalApiKeyStatus/);
  assert.match(portalApi, /deletePortalApiKey/);
  assert.match(repository, /updatePortalApiKeyStatus/);
  assert.match(repository, /deletePortalApiKey/);
  assert.match(portalTypes, /label: string/);
  assert.match(portalTypes, /created_at_ms: number/);
  assert.match(portalTypes, /last_used_at_ms\?: number \| null/);
  assert.match(portalTypes, /expires_at_ms\?: number \| null/);
});

test('usage page exposes traffic and spend diagnosis surfaces', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');

  assert.match(usagePage, /Traffic profile/);
  assert.match(usagePage, /Spend watch/);
  assert.match(usagePage, /Request diagnostics/);
});

test('user page exposes trust, security, and recovery guidance', () => {
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');

  assert.match(userPage, /Profile facts/);
  assert.match(userPage, /Personal security checklist/);
  assert.match(userPage, /Recovery signals/);
});

test('account page exposes financial posture, ledger, and runway guidance', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(accountPage, /Cash balance/);
  assert.match(accountPage, /Ledger evidence/);
  assert.match(accountPage, /Operating guardrails/);
});
