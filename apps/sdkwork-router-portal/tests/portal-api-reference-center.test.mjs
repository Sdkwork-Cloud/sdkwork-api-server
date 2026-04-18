import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');
const workspaceRoot = path.resolve(appRoot, '..', '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readWorkspace(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

function loadApiReferenceTransport() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-api-reference',
      'src',
      'openapiTransport.ts',
    ),
  );
}

test('public site exposes a dedicated API reference center after models and before docs', () => {
  const topNavigation = read('packages/sdkwork-router-portal-core/src/components/PortalTopNavigation.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const apiReferencePage = read('packages/sdkwork-router-portal-api-reference/src/index.tsx');

  assert.match(topNavigation, /key:\s*'api-reference'/);
  assert.match(topNavigation, /labelKey:\s*'API Reference'/);
  assert.match(topNavigation, /href:\s*'\/api-reference'/);
  assert.match(topNavigation, /key:\s*'models'[\s\S]*key:\s*'api-reference'[\s\S]*key:\s*'docs'/);

  assert.match(portalTypes, /'api-reference'/);
  assert.match(routePaths, /api-reference:\s*'\/api-reference'/);

  assert.match(appRoutes, /PortalApiReferencePage/);
  assert.match(appRoutes, /import\('sdkwork-router-portal-api-reference'\)/);
  assert.match(appRoutes, /PORTAL_ROUTE_PATHS\[['"]api-reference['"]\]|PORTAL_ROUTE_PATHS/);

  assert.match(apiReferencePage, /OpenAPI 3\.1/);
  assert.match(apiReferencePage, /Gateway API/);
  assert.match(apiReferencePage, /Portal API/);
  assert.match(apiReferencePage, /\/openapi\.json/);
  assert.match(apiReferencePage, /\/api\/portal\/openapi\.json|\/portal\/openapi\.json/);
});

test('API reference center derives route coverage and schema facts from live OpenAPI endpoints', () => {
  const apiReferencePage = read('packages/sdkwork-router-portal-api-reference/src/index.tsx');

  assert.match(apiReferencePage, /fetch\(/);
  assert.match(apiReferencePage, /Promise\.all|Promise\.allSettled/);
  assert.match(apiReferencePage, /operationCount/);
  assert.match(apiReferencePage, /schemaVersion/);
  assert.match(apiReferencePage, /tagCount|routeFamilyCount/);
  assert.match(apiReferencePage, /specEndpoint:\s*'\/openapi\.json'/);
  assert.match(apiReferencePage, /specEndpoint:\s*'\/api\/portal\/openapi\.json'/);
  assert.match(apiReferencePage, /case 'conversations':/);
  assert.match(apiReferencePage, /case 'files':/);
  assert.match(apiReferencePage, /case 'uploads':/);
  assert.match(apiReferencePage, /case 'batches':/);
  assert.match(apiReferencePage, /case 'vector-stores':/);
  assert.match(apiReferencePage, /case 'threads':/);
  assert.match(apiReferencePage, /case 'runs':/);
  assert.match(apiReferencePage, /case 'market':/);
  assert.match(apiReferencePage, /case 'marketing':/);
  assert.match(apiReferencePage, /case 'commercial':/);
  assert.doesNotMatch(apiReferencePage, /routeFamilies:\s*\[/);
});

test('gateway API reference documents coupon-first market and commercial public routes for downstream consumers', () => {
  const apiReferencePage = read('packages/sdkwork-router-portal-api-reference/src/index.tsx');
  const gatewayApiDoc = readWorkspace('docs/api-reference/gateway-api.md');

  assert.match(
    apiReferencePage,
    /market, coupon, and commercial account workflows/i,
  );
  assert.match(gatewayApiDoc, /\| `market` \| `GET \/market\/products`, `GET \/market\/offers`, `POST \/market\/quotes` \|/);
  assert.match(
    gatewayApiDoc,
    /\| `marketing` \| `POST \/marketing\/coupons\/validate`, `POST \/marketing\/coupons\/reserve`, `POST \/marketing\/coupons\/confirm`, `POST \/marketing\/coupons\/rollback` \|/,
  );
  assert.match(
    gatewayApiDoc,
    /\| `commercial` \| `GET \/commercial\/account`, `GET \/commercial\/account\/benefit-lots` \|/,
  );
  assert.match(gatewayApiDoc, /after_lot_id/);
  assert.match(gatewayApiDoc, /next_after_lot_id/);
  assert.match(gatewayApiDoc, /scope_order_id/);
});

test('API reference center preserves unsafe integers when reading live OpenAPI documents', async () => {
  const { readOpenApiDocument } = loadApiReferenceTransport();

  const document = await readOpenApiDocument(
    new Response(
      '{"openapi":"3.1.0","paths":{},"unsafe_marker":9007199254740993}',
      {
        status: 200,
        headers: {
          'content-type': 'application/json',
        },
      },
    ),
  );

  assert.equal(document.unsafe_marker, '9007199254740993');
});
