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

  assert.match(usagePage, /MetricCard/);
  assert.match(usagePage, /Total requests/);
  assert.match(usagePage, /Average latency/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.match(usagePage, /Manage keys/);
  assert.match(usagePage, /Review billing/);
  assert.doesNotMatch(usagePage, /ToolbarDisclosure/);
  assert.doesNotMatch(usagePage, /ToolbarSearchField/);
  assert.doesNotMatch(usagePage, /<Tabs/);
  assert.doesNotMatch(usagePage, /AreaChart/);

  assert.match(billingPage, /data-slot="portal-billing-toolbar"/);
  assert.match(billingPage, /<Dialog/);
  assert.match(billingPage, /Open credits/);
  assert.match(billingPage, /Open usage/);
  assert.match(billingPage, /Open account/);
  assert.match(billingPage, /Checkout preview/);
  assert.match(billingPage, /Plan catalog/);
  assert.match(billingPage, /Order workbench/);
  assert.match(billingPage, /Order lane/);
  assert.match(billingPage, /Pending payment queue/);
  assert.doesNotMatch(billingPage, /<Tabs/);

  assert.match(creditsPage, /portal-credits-toolbar/);
  assert.match(creditsPage, /MetricCard/);
  assert.match(creditsPage, /Eligible offers/);
  assert.match(creditsPage, /Potential bonus units/);
  assert.match(creditsPage, /Ledger entries/);
  assert.match(creditsPage, /Quota pressure/);
  assert.match(creditsPage, /<Dialog/);
  assert.match(creditsPage, /Redeem credits/);
  assert.match(creditsPage, /Search offers or ledger/);
  assert.match(creditsPage, /Offer state/);
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
  assert.match(usagePage, /Total spend/);
  assert.match(usagePage, /Actual spend/);
  assert.match(usagePage, /Reference price/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.doesNotMatch(usagePage, /Search usage/);
  assert.doesNotMatch(accountServices, /Date\.now\(\) - index \* 60_000/);
  assert.match(readme, /sdkwork-router-portal-routing/);
  assert.match(readme, /sdkwork-router-portal-user/);
  assert.match(readme, /User[\s\S]*profile and password rotation/);
  assert.match(readme, /Account[\s\S]*cash balance, credits, billing ledger, and runway posture/);
});

test('portal toolbars keep search first and pin actions to one right-aligned row', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const apiKeysToolbar = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx',
  );

  assert.match(commons, /export function ToolbarInline/);

  assert.match(
    usagePage,
    /<ToolbarInline[\s\S]*?data-slot="portal-usage-filter-bar"[\s\S]*?<ToolbarField label=\{t\('API key'\)\}[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap/,
  );
  assert.match(
    accountPage,
    /portal-account-toolbar[\s\S]*?<ToolbarInline[\s\S]*?<ToolbarSearchField[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap[\s\S]*?Open credits/,
  );
  assert.match(
    apiKeysToolbar,
    /<ToolbarInline[\s\S]*?<ToolbarSearchField[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap[\s\S]*?Create API key/,
  );
  assert.match(
    billingPage,
    /portal-billing-toolbar[\s\S]*?<ToolbarInline[\s\S]*?<ToolbarSearchField[\s\S]*?<ToolbarField[\s\S]*?label=\{t\('Order lane'\)\}[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap/,
  );
  assert.match(
    creditsPage,
    /portal-credits-toolbar[\s\S]*?<ToolbarInline[\s\S]*?<ToolbarSearchField[\s\S]*?<ToolbarField[\s\S]*?label=\{t\('View mode'\)\}[\s\S]*?<ToolbarField[\s\S]*?label=\{t\('Offer state'\)\}[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap/,
  );
  assert.match(
    gatewayPage,
    /<ToolbarInline[\s\S]*?data-slot="portal-gateway-filter-bar"[\s\S]*?<ToolbarSearchField[\s\S]*?<ToolbarField label="Workbench lane"[\s\S]*?<ToolbarField label="Operational focus"[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap/,
  );
  assert.match(
    routingPage,
    /<ToolbarInline[\s\S]*?data-slot="portal-routing-filter-bar"[\s\S]*?<ToolbarSearchField[\s\S]*?<ToolbarField label="Workbench lane"[\s\S]*?<ToolbarField[\s\S]*?label="Operational focus"[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap/,
  );
  assert.match(
    routingPage,
    /data-slot="portal-routing-toolbar"[\s\S]*?className="ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap"/,
  );
  assert.doesNotMatch(
    routingPage,
    /data-slot="portal-routing-toolbar" className="flex flex-wrap gap-2"/,
  );
});

test('portal form primitives keep a shadcn-style contract and portal settings flows stay on shared form shells', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');

  assert.match(commons, /file:border-0 file:bg-transparent file:text-sm file:font-medium/);
  assert.match(commons, /disabled:cursor-not-allowed disabled:opacity-50/);
  assert.match(commons, /appearance-none/);
  assert.match(commons, /export function SearchInput/);
  assert.match(commons, /paddingLeft:\s*['"]2\.75rem['"]/);
  assert.match(configCenter, /FormField/);
  assert.match(configCenter, /SearchInput/);
  assert.doesNotMatch(configCenter, /<Search className="absolute left-3 top-1\/2/);
  assert.doesNotMatch(configCenter, /<label className="text-sm font-medium text-zinc-700 dark:text-zinc-300">/);
  assert.match(routingPage, /Label/);
  assert.doesNotMatch(routingPage, /<label className="flex items-center gap-3 text-sm font-medium text-zinc-700 dark:text-zinc-300">/);
});
