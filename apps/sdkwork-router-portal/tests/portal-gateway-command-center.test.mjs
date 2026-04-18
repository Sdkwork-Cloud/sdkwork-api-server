import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadGatewayServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-gateway',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('gateway page centers the command center on the shared management workbench', () => {
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const gatewayComponents = read('packages/sdkwork-router-portal-gateway/src/components/index.tsx');

  assert.match(gatewayPage, /ManagementWorkbench/);
  assert.match(gatewayPage, /title=\{t\('Command workbench'\)\}/);
  assert.match(gatewayPage, /data-slot="portal-gateway-filter-bar"/);
  assert.match(gatewayPage, /Refresh command center/);
  assert.match(gatewayPage, /Refresh service health/);
  assert.doesNotMatch(gatewayPage, /<Surface/);
  assert.doesNotMatch(gatewayComponents, /export \{ Surface \}/);
});

test('gateway command center derives launch readiness blockers and desktop runtime controls', () => {
  const { buildGatewayCommandCenterSnapshot } = loadGatewayServices();

  const snapshot = buildGatewayCommandCenterSnapshot({
    dashboard: {
      workspace: {
        user: {
          id: 'portal-user',
          email: 'portal@example.com',
          display_name: 'Portal User',
          workspace_tenant_id: 'tenant-demo',
          workspace_project_id: 'project-demo',
          active: true,
          created_at_ms: 1710000000,
        },
        tenant: {
          id: 'tenant-demo',
          name: 'Tenant Demo',
        },
        project: {
          tenant_id: 'tenant-demo',
          id: 'project-demo',
          name: 'Project Demo',
        },
      },
      usage_summary: {
        total_requests: 24,
        project_count: 1,
        model_count: 2,
        provider_count: 2,
        projects: [{ project_id: 'project-demo', request_count: 24 }],
        providers: [{ provider: 'openai', request_count: 24, project_count: 1 }],
        models: [{ model: 'gpt-4.1', request_count: 24, provider_count: 1 }],
      },
      billing_summary: {
        project_id: 'project-demo',
        entry_count: 3,
        used_units: 15000,
        booked_amount: 79,
        quota_policy_id: 'policy-demo',
        quota_limit_units: 15000,
        remaining_units: 0,
        exhausted: true,
      },
      recent_requests: [],
      api_key_count: 0,
    },
    commerceCatalog: {
      plans: [
        {
          id: 'growth',
          name: 'Growth',
          price_label: '$79.00',
          cadence: '/month',
          included_units: 100000,
          highlight: 'Best for launch teams',
          features: ['100k units'],
          cta: 'Choose growth',
          source: 'live',
        },
      ],
      packs: [
        {
          id: 'pack-100k',
          label: 'Boost 100k',
          points: 100000,
          price_label: '$40.00',
          note: 'Emergency top-up',
          source: 'live',
        },
      ],
      coupons: [],
    },
    membership: null,
    desktopRuntime: {
      mode: 'desktop',
      roles: ['web', 'gateway', 'admin', 'portal'],
      publicBaseUrl: 'http://127.0.0.1:48123',
      publicBindAddr: '127.0.0.1:48123',
      gatewayBindAddr: '127.0.0.1:8080',
      adminBindAddr: '127.0.0.1:8081',
      portalBindAddr: '127.0.0.1:8082',
    },
    gatewayBaseUrl: 'http://127.0.0.1:48123/api',
    runtimeHealth: {
      mode: 'desktop',
      checkedAtMs: 1710000000,
      services: [
        {
          id: 'web',
          label: 'Web entrypoint',
          status: 'healthy',
          healthUrl: 'http://127.0.0.1:48123/',
          detail: 'ok',
          httpStatus: 200,
          responseTimeMs: 20,
        },
        {
          id: 'gateway',
          label: 'Gateway',
          status: 'unreachable',
          healthUrl: 'http://127.0.0.1:8080/health',
          detail: 'Gateway is unreachable from the current session.',
          httpStatus: null,
          responseTimeMs: null,
        },
        {
          id: 'admin',
          label: 'Admin control plane',
          status: 'healthy',
          healthUrl: 'http://127.0.0.1:8081/admin/health',
          detail: 'ok',
          httpStatus: 200,
          responseTimeMs: 18,
        },
        {
          id: 'portal',
          label: 'Portal API',
          status: 'healthy',
          healthUrl: 'http://127.0.0.1:8082/portal/health',
          detail: 'ok',
          httpStatus: 200,
          responseTimeMs: 21,
        },
      ],
    },
  });

  assert.equal(snapshot.launchReadiness.status, 'blocked');
  assert.ok(snapshot.launchReadiness.score < 50);
  assert.ok(
    snapshot.launchReadiness.blockers.some((entry) => /API key/i.test(entry)),
    'missing API key blocker',
  );
  assert.ok(
    snapshot.launchReadiness.blockers.some((entry) => /runway exhausted/i.test(entry)),
    'missing billing blocker',
  );
  assert.ok(
    snapshot.launchReadiness.blockers.some((entry) => /gateway/i.test(entry)),
    'missing runtime blocker',
  );
  assert.ok(
    snapshot.runtimeControls.some(
      (control) => control.action === 'restart-desktop-runtime' && control.enabled,
    ),
    'desktop runtime restart control should be enabled for desktop sessions',
  );
});
