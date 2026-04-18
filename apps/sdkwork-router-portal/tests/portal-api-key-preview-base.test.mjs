import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

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

test('api key workspace centers credential inventory on a paginated admin table with drawers', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(apiKeysPage, /PortalApiKeyDrawers/);
  assert.match(apiKeysPage, /PortalApiKeyTable/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-toolbar"/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-pagination"/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-table"/);
  assert.doesNotMatch(apiKeysPage, /toolbarSummary/);
  assert.doesNotMatch(apiKeysPage, /Showing \{visible\} of \{total\} keys/);
  assert.doesNotMatch(apiKeysPage, /SectionHeader/);
  assert.doesNotMatch(apiKeysPage, /WorkspacePanel/);
  assert.doesNotMatch(apiKeysPage, /CrudWorkbench/);
  assert.doesNotMatch(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.doesNotMatch(apiKeysPage, /return null;/);
});

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

test('api key services keep group-aware defaults and filters aligned with the newer contract', () => {
  const services = loadApiKeyServices();

  assert.equal(services.createEmptyPortalApiKeyFormState().apiKeyGroupId, 'none');

  const filtered = services.filterPortalApiKeys(
    [
      {
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'live',
        hashed_key: 'hashed-grouped',
        api_key_group_id: 'group-live',
        label: 'Primary live key',
        created_at_ms: 2,
        active: true,
      },
      {
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'live',
        hashed_key: 'hashed-ungrouped',
        api_key_group_id: null,
        label: 'Ungrouped fallback',
        created_at_ms: 1,
        active: true,
      },
    ],
    {
      searchQuery: '',
      environment: 'all',
      groupId: 'group-live',
    },
  );

  assert.deepEqual(
    filtered.map((item) => item.hashed_key),
    ['hashed-grouped'],
  );
});
