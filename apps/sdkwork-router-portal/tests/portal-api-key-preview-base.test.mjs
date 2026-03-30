import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadApiKeyServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-api-keys',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('api key usage preview builds its curl example from the resolved gateway base url', () => {
  const services = loadApiKeyServices();

  const preview = services.buildPortalApiKeyUsagePreview(
    {
      tenant_id: 'tenant-1',
      project_id: 'project-1',
      environment: 'live',
      hashed_key: 'hashed-key',
      label: 'Primary live key',
      created_at_ms: 1,
      active: true,
    },
    'plaintext-key',
    'https://router.example.com/api',
  );

  assert.match(preview.curlExample, /https:\/\/router\.example\.com\/api\/v1\/models/);
  assert.doesNotMatch(preview.curlExample, /127\.0\.0\.1:8080\/v1\/models/);
});
