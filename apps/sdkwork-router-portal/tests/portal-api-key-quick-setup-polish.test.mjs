import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadQuickSetupServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-api-keys',
      'src',
      'services',
      'quickSetup.ts',
    ),
  );
}

test('api key quick setup never falls back to an internal fake plaintext placeholder', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const drawers = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.doesNotMatch(apiKeysPage, /api-key-not-visible-on-this-device/);
  assert.match(drawers, /selectedPlan\s*&&\s*selectedPlan\.available/);
  assert.match(drawers, /plan\.availabilityDetail/);
  assert.doesNotMatch(drawers, /`\s*璺痋s*\$\{instance\.detail\}/);

  assert.match(
    commons,
    /'Rotate this key to reveal a new one-time secret before applying \{label\} setup or copying local snippets\.'/,
  );
});

test('api key quick setup plans become explicitly unavailable when plaintext is not present', () => {
  const quickSetup = loadQuickSetupServices();

  const unavailablePlans = quickSetup.buildApiKeyQuickSetupPlans({
    hashedKey: 'hashed-key-demo',
    label: 'Primary live key',
    plaintextKey: null,
    gatewayBaseUrl: 'https://router.example.com/api',
  });

  assert.ok(unavailablePlans.length > 0);
  assert.ok(unavailablePlans.every((plan) => plan.available === false));
  assert.ok(unavailablePlans.every((plan) => plan.snippets.length === 0));
  assert.ok(
    unavailablePlans.every((plan) =>
      /Rotate this key to reveal a new one-time secret/.test(plan.availabilityDetail ?? ''),
    ),
  );

  const availablePlans = quickSetup.buildApiKeyQuickSetupPlans({
    hashedKey: 'hashed-key-demo',
    label: 'Primary live key',
    plaintextKey: 'plaintext-live-secret',
    gatewayBaseUrl: 'https://router.example.com/api',
  });

  assert.ok(availablePlans.every((plan) => plan.available === true));
  assert.ok(availablePlans.some((plan) => plan.snippets.length > 0));
});
