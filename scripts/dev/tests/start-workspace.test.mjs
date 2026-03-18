import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildWorkspaceCommandPlan,
  parseWorkspaceArgs,
} from '../workspace-launch-lib.mjs';

test('parseWorkspaceArgs returns browser-mode defaults', () => {
  const settings = parseWorkspaceArgs([]);

  assert.deepEqual(settings, {
    databaseUrl: null,
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    webBind: '0.0.0.0:3001',
    install: false,
    preview: false,
    tauri: false,
    dryRun: false,
    help: false,
  });
});

test('parseWorkspaceArgs forwards install, preview, tauri, and bind overrides', () => {
  const settings = parseWorkspaceArgs([
    '--database-url',
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    '--gateway-bind',
    '0.0.0.0:18080',
    '--admin-bind',
    '0.0.0.0:18081',
    '--portal-bind',
    '0.0.0.0:18082',
    '--web-bind',
    '0.0.0.0:13001',
    '--install',
    '--preview',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(
    settings.databaseUrl,
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
  );
  assert.equal(settings.gatewayBind, '0.0.0.0:18080');
  assert.equal(settings.adminBind, '0.0.0.0:18081');
  assert.equal(settings.portalBind, '0.0.0.0:18082');
  assert.equal(settings.webBind, '0.0.0.0:13001');
  assert.equal(settings.install, true);
  assert.equal(settings.preview, true);
  assert.equal(settings.tauri, true);
  assert.equal(settings.dryRun, true);
});

test('buildWorkspaceCommandPlan keeps local config defaults when database override is absent', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    webBind: '0.0.0.0:3001',
    install: false,
    preview: false,
    tauri: false,
    dryRun: true,
    help: false,
  });

  assert.equal(plan.backend.scriptPath, 'scripts/dev/start-stack.mjs');
  assert.deepEqual(plan.backend.args, [
    'scripts/dev/start-stack.mjs',
    '--gateway-bind',
    '127.0.0.1:8080',
    '--admin-bind',
    '127.0.0.1:8081',
    '--portal-bind',
    '127.0.0.1:8082',
    '--dry-run',
  ]);
});

test('buildWorkspaceCommandPlan forwards backend and console flags to child scripts', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: 'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    gatewayBind: '0.0.0.0:18080',
    adminBind: '0.0.0.0:18081',
    portalBind: '0.0.0.0:18082',
    webBind: '0.0.0.0:13001',
    install: true,
    preview: false,
    tauri: true,
    dryRun: true,
    help: false,
  });

  assert.equal(plan.backend.scriptPath, 'scripts/dev/start-stack.mjs');
  assert.deepEqual(plan.backend.args, [
    'scripts/dev/start-stack.mjs',
    '--database-url',
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    '--gateway-bind',
    '0.0.0.0:18080',
    '--admin-bind',
    '0.0.0.0:18081',
    '--portal-bind',
    '0.0.0.0:18082',
    '--dry-run',
  ]);

  assert.equal(plan.admin.scriptPath, 'scripts/dev/start-admin.mjs');
  assert.deepEqual(plan.admin.args, [
    'scripts/dev/start-admin.mjs',
    '--install',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(plan.web.scriptPath, 'scripts/dev/start-web.mjs');
  assert.deepEqual(plan.web.args, [
    'scripts/dev/start-web.mjs',
    '--bind',
    '0.0.0.0:13001',
    '--admin-target',
    '0.0.0.0:18081',
    '--portal-target',
    '0.0.0.0:18082',
    '--gateway-target',
    '0.0.0.0:18080',
    '--install',
    '--tauri',
    '--dry-run',
  ]);
});

test('buildWorkspaceCommandPlan forwards backend binds to preview web host', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: 'sqlite:///tmp/sdkwork-router-e2e/sdkwork-api-server.db',
    gatewayBind: '127.0.0.1:18080',
    adminBind: '127.0.0.1:18081',
    portalBind: '127.0.0.1:18082',
    webBind: '127.0.0.1:13001',
    install: false,
    preview: true,
    tauri: false,
    dryRun: true,
    help: false,
  });

  assert.deepEqual(plan.web.args, [
    'scripts/dev/start-web.mjs',
    '--bind',
    '127.0.0.1:13001',
    '--admin-target',
    '127.0.0.1:18081',
    '--portal-target',
    '127.0.0.1:18082',
    '--gateway-target',
    '127.0.0.1:18080',
    '--preview',
    '--dry-run',
  ]);
});

test('buildWorkspaceCommandPlan keeps browser mode on standalone admin and portal apps', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    webBind: '0.0.0.0:3001',
    install: false,
    preview: false,
    tauri: false,
    dryRun: true,
    help: false,
  });

  assert.equal(plan.admin.scriptPath, 'scripts/dev/start-admin.mjs');
  assert.deepEqual(plan.admin.args, ['scripts/dev/start-admin.mjs', '--dry-run']);
  assert.equal(plan.portal.scriptPath, 'scripts/dev/start-portal.mjs');
  assert.deepEqual(plan.portal.args, ['scripts/dev/start-portal.mjs', '--dry-run']);
});

test('parseWorkspaceArgs rejects missing values and unknown flags', () => {
  assert.throws(() => parseWorkspaceArgs(['--database-url']), {
    message: /requires a value/,
  });
  assert.throws(() => parseWorkspaceArgs(['--unknown-flag']), {
    message: /unknown option/,
  });
});
