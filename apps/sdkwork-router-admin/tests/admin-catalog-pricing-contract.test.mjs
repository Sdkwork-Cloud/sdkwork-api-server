import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('catalog pricing contract exposes source metadata and tiered pricing across types, api, and ui', () => {
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const detailPanel = read('packages/sdkwork-router-admin-catalog/src/page/CatalogDetailPanel.tsx');
  const dialog = read('packages/sdkwork-router-admin-catalog/src/page/CatalogModelPriceDialog.tsx');

  assert.match(types, /export interface ModelPriceTier/);
  assert.match(types, /price_source_kind:\s*string;/);
  assert.match(types, /billing_notes\?:\s*string\s*\|\s*null;/);
  assert.match(types, /pricing_tiers:\s*ModelPriceTier\[\];/);

  assert.match(adminApi, /price_source_kind:\s*string;/);
  assert.match(adminApi, /billing_notes\?:\s*string\s*\|\s*null;/);
  assert.match(adminApi, /pricing_tiers:\s*ModelPriceTier\[\];/);

  assert.match(detailPanel, /Price source/);
  assert.match(detailPanel, /Billing notes/);
  assert.match(detailPanel, /Tiered pricing/);

  assert.match(dialog, /Price source/);
  assert.match(dialog, /Billing notes/);
  assert.match(dialog, /Pricing tiers JSON/);
});

test('catalog provider management contract exposes provider-model editing and pricing coverage affordances', () => {
  const providerCatalog = read('packages/sdkwork-router-admin-core/src/providerCatalog.ts');
  const providerDialog = read('packages/sdkwork-router-admin-catalog/src/page/CatalogProviderDialog.tsx');
  const detailPanel = read('packages/sdkwork-router-admin-catalog/src/page/CatalogDetailPanel.tsx');
  const registry = read('packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx');

  assert.match(providerCatalog, /recommendedModelPriceSourceKind/);
  assert.match(providerCatalog, /summarizeProviderPricingCoverage/);

  assert.match(providerDialog, /Provider model id/);
  assert.match(providerDialog, /Provider model family/);
  assert.match(providerDialog, /Prompt caching/);
  assert.match(providerDialog, /Reasoning usage/);
  assert.match(providerDialog, /Default route/);

  assert.match(detailPanel, /Pricing coverage/);
  assert.match(detailPanel, /Missing pricing/);
  assert.match(detailPanel, /Provider model family/);
  assert.match(detailPanel, /Add pricing/);

  assert.match(registry, /Pricing coverage/);
});
