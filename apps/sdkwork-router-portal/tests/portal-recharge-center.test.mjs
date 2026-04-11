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
  const repository = read('packages/sdkwork-router-portal-recharge/src/repository/index.ts');
  const services = read('packages/sdkwork-router-portal-recharge/src/services/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-recharge/src/types/index.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

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
  assert.match(page, /data-slot="portal-recharge-custom-form"/);
  assert.match(page, /data-slot="portal-recharge-quote-card"/);
  assert.match(page, /data-slot="portal-recharge-history-table"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-summary-grid"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-decision-support"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-multimodal-demand"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-balance-pill"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-trust-strip"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-cta-note"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-balance-compare"/);
  assert.doesNotMatch(page, /data-slot="portal-recharge-payment-assurance"/);
  assert.match(page, /Recharge options/);
  assert.match(page, /Recommended/);
  assert.match(page, /Custom amount/);
  assert.match(page, /Payment information/);
  assert.match(page, /Recharge history/);
  assert.match(page, /Create recharge order/);
  assert.match(page, /Preview amount/);
  assert.doesNotMatch(page, /Commercial recharge/);
  assert.doesNotMatch(page, /Current runway/);
  assert.doesNotMatch(page, /Most teams start here/);
  assert.doesNotMatch(page, /Server-managed pricing/);
  assert.doesNotMatch(page, /Billing settlement/);
  assert.doesNotMatch(page, /Before settlement/);
  assert.doesNotMatch(page, /After settlement/);
  assert.doesNotMatch(page, /Order creation only/);
  assert.doesNotMatch(page, /Settle later in billing/);
  assert.doesNotMatch(page, /Secure billing handoff/);
  assert.doesNotMatch(page, /Balance updates after payment settles/);
  assert.doesNotMatch(page, /Secondary operations/);
  assert.doesNotMatch(page, /Recharge decision support/);
});
