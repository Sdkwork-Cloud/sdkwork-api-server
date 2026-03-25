import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('remaining portal workspaces keep compact controls while preserving focused dialog flows', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(usagePage, /portal-usage-toolbar/);
  assert.match(usagePage, /Search usage/);
  assert.match(usagePage, /Manage keys/);
  assert.doesNotMatch(usagePage, /<Tabs/);
  assert.doesNotMatch(usagePage, /AreaChart/);

  assert.match(billingPage, /<Tabs/);
  assert.match(billingPage, /<Dialog/);
  assert.match(billingPage, /Checkout preview/);
  assert.match(billingPage, /Plan catalog/);

  assert.match(creditsPage, /portal-credits-toolbar/);
  assert.match(creditsPage, /<Dialog/);
  assert.match(creditsPage, /Redeem credits/);
  assert.match(creditsPage, /Search offers or ledger/);
  assert.doesNotMatch(creditsPage, /<Tabs/);

  assert.match(accountPage, /portal-account-toolbar/);
  assert.match(accountPage, /Search ledger/);
  assert.doesNotMatch(accountPage, /<Tabs/);
});

test('usage contracts and financial evidence stay aligned with real server data', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const accountServices = read('packages/sdkwork-router-portal-account/src/services/index.ts');
  const readme = read('README.md');

  assert.match(portalTypes, /input_tokens: number/);
  assert.match(portalTypes, /output_tokens: number/);
  assert.match(portalTypes, /total_tokens: number/);
  assert.match(usagePage, /Input tokens/);
  assert.match(usagePage, /Output tokens/);
  assert.match(usagePage, /Total tokens/);
  assert.match(usagePage, /Search usage/);
  assert.doesNotMatch(accountServices, /Date\.now\(\) - index \* 60_000/);
  assert.match(readme, /sdkwork-router-portal-routing/);
  assert.match(readme, /sdkwork-router-portal-user/);
  assert.match(readme, /User[\s\S]*profile and password rotation/);
  assert.match(readme, /Account[\s\S]*cash balance, credits, billing ledger, and runway posture/);
});
