import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal exposes a dedicated recharge route and package instead of overloading billing for top-up entry', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const routeManifest = read(
    'packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts',
  );
  const routePrefetch = read(
    'packages/sdkwork-router-portal-core/src/application/router/routePrefetch.ts',
  );
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const consoleRoute = read('packages/sdkwork-router-portal-console/src/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const rechargePackageRoot = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-recharge',
  );

  assert.equal(existsSync(path.join(rechargePackageRoot, 'package.json')), true);
  assert.equal(existsSync(path.join(rechargePackageRoot, 'src', 'index.tsx')), true);
  assert.equal(existsSync(path.join(rechargePackageRoot, 'src', 'pages', 'index.tsx')), true);
  assert.equal(existsSync(path.join(rechargePackageRoot, 'src', 'repository', 'index.ts')), true);
  assert.equal(existsSync(path.join(rechargePackageRoot, 'src', 'services', 'index.ts')), true);
  assert.equal(existsSync(path.join(rechargePackageRoot, 'src', 'types', 'index.ts')), true);

  assert.match(portalTypes, /'recharge'/);
  assert.match(portalTypes, /'sdkwork-router-portal-recharge'/);
  assert.match(routes, /key:\s*'recharge'/);
  assert.match(routes, /labelKey:\s*'Recharge'/);
  assert.match(routes, /detailKey:\s*'Top up balance with server-managed recharge options'/);
  assert.match(routes, /group:\s*'revenue'/);
  assert.doesNotMatch(routes, /key:\s*'recharge'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routePaths, /recharge:\s*'\/console\/recharge'/);
  assert.match(routeManifest, /recharge:\s*'sdkwork-router-portal-recharge'/);
  assert.match(routePrefetch, /sdkwork-router-portal-recharge/);
  assert.match(appRoutes, /case 'recharge':/);
  assert.match(appRoutes, /'recharge',/);
  assert.match(consoleRoute, /PortalRechargePage/);
  assert.match(consoleRoute, /case 'recharge':/);
  assert.match(accountPage, /onRecharge=\{\(\) => onNavigate\('recharge'\)\}/);
});

test('recharge page simplifies into a three-section commercial purchase surface', () => {
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');
  const presentation = read('packages/sdkwork-router-portal-recharge/src/pages/presentation.ts');
  const repository = read('packages/sdkwork-router-portal-recharge/src/repository/index.ts');
  const services = read('packages/sdkwork-router-portal-recharge/src/services/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-recharge/src/types/index.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const pageContract = `${page}\n${presentation}`;

  assert.match(portalTypes, /export interface PortalRechargeOption/);
  assert.match(portalTypes, /export interface PortalCustomRechargePolicy/);
  assert.match(portalTypes, /export interface PortalCustomRechargeRule/);
  assert.match(portalTypes, /custom_amount_cents\?: number \| null;/);
  assert.match(portalTypes, /'custom_recharge'/);
  assert.match(portalTypes, /recharge_options: PortalRechargeOption\[];/);
  assert.match(portalTypes, /custom_recharge_policy: PortalCustomRechargePolicy \| null;/);
  assert.match(portalApi, /getPortalCommerceCatalog/);
  assert.match(portalApi, /previewPortalCommerceQuote/);
  assert.match(portalApi, /createPortalCommerceOrder/);
  assert.match(portalApi, /listPortalCommerceOrders/);
  assert.match(repository, /getPortalCommerceCatalog/);
  assert.match(repository, /previewPortalCommerceQuote/);
  assert.match(repository, /createPortalCommerceOrder/);
  assert.match(repository, /listPortalCommerceOrders/);
  assert.doesNotMatch(repository, /getPortalCommerceMembership/);
  assert.doesNotMatch(repository, /getPortalBillingEventSummary/);
  assert.match(repository, /target_kind:\s*'custom_recharge'/);
  assert.match(services, /buildPortalRechargeQuoteSnapshot/);
  assert.match(services, /buildPortalRechargeHistoryRows/);
  assert.doesNotMatch(services, /buildPortalRechargeFinanceProjection/);
  assert.doesNotMatch(services, /buildPortalRechargeSummaryCards/);
  assert.match(pageTypes, /PortalRechargePageProps/);
  assert.doesNotMatch(pageTypes, /membership: PortalCommerceMembership \| null;/);
  assert.doesNotMatch(pageTypes, /billing_event_summary: BillingEventSummary;/);
  assert.doesNotMatch(pageTypes, /PortalRechargeFinanceProjection/);
  assert.match(page, /data-slot="portal-recharge-page"/);
  assert.match(page, /data-slot="portal-recharge-options"/);
  assert.match(page, /data-slot="portal-recharge-selection-hero"/);
  assert.match(page, /data-slot="portal-recharge-posture-strip"/);
  assert.match(page, /data-slot="portal-recharge-guidance-band"/);
  assert.match(page, /data-slot="portal-recharge-custom-form"/);
  assert.match(page, /data-slot="portal-recharge-quote-card"/);
  assert.match(page, /data-slot="portal-recharge-quote-note"/);
  assert.match(page, /data-slot="portal-recharge-selection-story"/);
  assert.match(page, /data-slot="portal-recharge-quote-breakdown"/);
  assert.match(page, /data-slot="portal-recharge-next-step-callout"/);
  assert.match(page, /data-slot="portal-recharge-post-order-handoff"/);
  assert.match(page, /data-slot="portal-recharge-mobile-cta"/);
  assert.match(page, /data-slot="portal-recharge-history-table"/);
  assert.match(page, /data-slot="portal-recharge-history-header"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-summary-grid"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-decision-support"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-multimodal-demand"/);
  assert.match(pageContract, /Recharge options/);
  assert.match(pageContract, /Recommended/);
  assert.match(pageContract, /Custom amount/);
  assert.match(pageContract, /Payment information/);
  assert.match(pageContract, /Checkout summary/);
  assert.match(pageContract, /Selection story/);
  assert.match(pageContract, /Best fit for steady usage/);
  assert.match(pageContract, /Create order in billing/);
  assert.match(pageContract, /Pending settlement queue/);
  assert.match(pageContract, /Latest pending order/);
  assert.match(pageContract, /Open billing to complete payment/);
  assert.match(pageContract, /Order ready for payment/);
  assert.match(pageContract, /Continue in billing/);
  assert.match(pageContract, /Create another order/);
  assert.match(pageContract, /Recharge history/);
  assert.match(pageContract, /Create recharge order/);
  assert.match(pageContract, /Current balance/);
  assert.match(pageContract, /Pending follow-up/);
  assert.match(pageContract, /Checkout stays in billing after order creation\./);
  assert.doesNotMatch(page, /Recharge decision support/);
});
