import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('remaining portal workspaces use admin-style tabs and dialogs instead of long inline flows', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(usagePage, /<Tabs/);
  assert.match(usagePage, /Request log/);
  assert.match(usagePage, /Demand mix/);
  assert.match(usagePage, /AreaChart/);

  assert.match(billingPage, /<Tabs/);
  assert.match(billingPage, /<Dialog/);
  assert.match(billingPage, /Checkout preview/);
  assert.match(billingPage, /Plan catalog/);

  assert.match(creditsPage, /<Tabs/);
  assert.match(creditsPage, /<Dialog/);
  assert.match(creditsPage, /Redeem credits/);
  assert.match(creditsPage, /Offer catalog/);

  assert.match(accountPage, /<Tabs/);
  assert.match(accountPage, /Balance summary/);
  assert.match(accountPage, /Ledger table/);
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
  assert.doesNotMatch(accountServices, /Date\.now\(\) - index \* 60_000/);
  assert.match(readme, /sdkwork-router-portal-routing/);
  assert.match(readme, /sdkwork-router-portal-user/);
  assert.match(readme, /User[\s\S]*profile and password rotation/);
  assert.match(readme, /Account[\s\S]*cash balance, credits, billing ledger, and runway posture/);
});
