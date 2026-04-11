import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const workspaceRoot = path.resolve(appRoot, '..', '..');

function readWorkspace(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

test('main coupon runtime paths no longer depend on legacy coupon fallback sources', () => {
  for (const relativePath of [
    'crates/sdkwork-api-interface-http/src/gateway_market.rs',
    'crates/sdkwork-api-interface-portal/src/marketing.rs',
    'crates/sdkwork-api-interface-portal/src/marketing_handlers.rs',
    'crates/sdkwork-api-app-commerce/src/coupon_catalog.rs',
    'crates/sdkwork-api-app-commerce/src/coupon_state.rs',
  ]) {
    const source = readWorkspace(relativePath);
    assert.doesNotMatch(
      source,
      /project_legacy_coupon_campaign|list_active_coupons\(|compatibility_source|load_compatibility_marketing_coupon_context/,
      `expected ${relativePath} to stop depending on legacy coupon fallback`,
    );
  }
});

test('coupon-facing runtime crates stop declaring legacy coupon app dependencies on main paths', () => {
  const workspaceCargo = readWorkspace('Cargo.toml');
  const commerceCargo = readWorkspace('crates/sdkwork-api-app-commerce/Cargo.toml');
  const marketingCargo = readWorkspace('crates/sdkwork-api-app-marketing/Cargo.toml');
  const portalCargo = readWorkspace('crates/sdkwork-api-interface-portal/Cargo.toml');

  assert.doesNotMatch(workspaceCargo, /sdkwork-api-app-coupon/);
  assert.doesNotMatch(workspaceCargo, /sdkwork-api-domain-coupon/);
  assert.doesNotMatch(commerceCargo, /sdkwork-api-app-coupon/);
  assert.doesNotMatch(commerceCargo, /sdkwork-api-domain-coupon/);
  assert.doesNotMatch(marketingCargo, /sdkwork-api-domain-coupon/);
  assert.doesNotMatch(portalCargo, /sdkwork-api-app-coupon/);
});

test('admin coupon detail no longer exposes legacy compatibility language in the primary workbench', () => {
  const detailPanel = readWorkspace(
    'apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsDetailPanel.tsx',
  );
  const i18n = readWorkspace(
    'apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx',
  );

  assert.doesNotMatch(detailPanel, /Legacy coupon compatibility/);
  assert.doesNotMatch(
    detailPanel,
    /This record remains available for the compatibility layer/,
  );
  assert.doesNotMatch(
    detailPanel,
    /Use this panel to review the historical coupon posture/,
  );
  assert.doesNotMatch(i18n, /Legacy coupon compatibility/);
});

test('admin control plane removes legacy coupon compatibility routes and client mutations', () => {
  const routes = readWorkspace('crates/sdkwork-api-interface-admin/src/routes.rs');
  const marketing = readWorkspace('crates/sdkwork-api-interface-admin/src/marketing.rs');
  const adminLib = readWorkspace('crates/sdkwork-api-interface-admin/src/lib.rs');
  const adminCargo = readWorkspace('crates/sdkwork-api-interface-admin/Cargo.toml');
  const adminApi = readWorkspace(
    'apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts',
  );
  const workbench = readWorkspace(
    'apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbench.tsx',
  );
  const workbenchActions = readWorkspace(
    'apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbenchActions.ts',
  );
  const appRoutes = readWorkspace(
    'apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx',
  );

  assert.doesNotMatch(routes, /"\/admin\/coupons"|\/admin\/coupons/);
  assert.doesNotMatch(
    marketing,
    /list_coupons_handler|create_coupon_handler|delete_coupon_handler|list_legacy_coupon_projections_from_marketing|project_marketing_coupon_to_legacy|sync_legacy_coupon_marketing_projection|CouponCampaign::new|persist_coupon\(|delete_coupon\(/,
  );
  assert.doesNotMatch(
    adminLib,
    /sdkwork_api_app_coupon|sdkwork_api_domain_coupon|project_legacy_coupon_campaign|CouponCampaign/,
  );
  assert.doesNotMatch(adminCargo, /sdkwork-api-app-coupon|sdkwork-api-domain-coupon/);
  assert.doesNotMatch(adminApi, /listCoupons|saveCoupon|deleteCoupon/);
  assert.doesNotMatch(workbench, /listCoupons/);
  assert.doesNotMatch(
    workbenchActions,
    /handleSaveCoupon|handleToggleCoupon|handleDeleteCoupon|saveCoupon|deleteCoupon/,
  );
  assert.doesNotMatch(appRoutes, /onSaveCoupon|onToggleCoupon|onDeleteCoupon/);
});

test('workspace root no longer ships legacy coupon model, bootstrap, or storage contracts', () => {
  const appMarketing = readWorkspace('crates/sdkwork-api-app-marketing/src/lib.rs');
  const storageCore = readWorkspace('crates/sdkwork-api-storage-core/src/admin_store.rs');
  const storageFacets = readWorkspace('crates/sdkwork-api-storage-core/src/admin_facets.rs');
  const runtimeManifest = readWorkspace(
    'crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs',
  );
  const runtimeRegistry = readWorkspace(
    'crates/sdkwork-api-app-runtime/src/bootstrap_data/registry.rs',
  );

  assert.doesNotMatch(
    appMarketing,
    /sdkwork_api_domain_coupon|CouponCampaign|project_legacy_coupon_campaign/,
  );
  assert.doesNotMatch(
    storageCore,
    /insert_coupon\(|list_coupons\(|list_active_coupons\(|find_coupon\(|delete_coupon\(|CouponCampaign/,
  );
  assert.doesNotMatch(
    storageFacets,
    /insert_coupon\(|list_coupons\(|find_coupon\(|delete_coupon\(|CouponCampaign/,
  );
  assert.doesNotMatch(runtimeManifest, /\bcoupons:\s*Vec<CouponCampaign>|CouponCampaign/);
  assert.doesNotMatch(runtimeManifest, /coupons\.id|coupons\.code|data\.coupons|merged\.coupons/);
  assert.doesNotMatch(
    runtimeRegistry,
    /pack\.data\.coupons|insert_coupon\(|CouponCampaign|LegacyCoupon/,
  );
});

test('admin api reference stops documenting legacy /admin/coupons compatibility', () => {
  const adminApiReference = readWorkspace('docs/api-reference/admin-api.md');
  assert.doesNotMatch(
    adminApiReference,
    /GET\/POST \/coupons|DELETE \/coupons\/\{coupon_id\}|legacy `\/admin\/coupons` compatibility|GET\/POST \/admin\/coupons remains a compatibility surface/,
  );
});
