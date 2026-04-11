import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';

import {
  buildWorkspaceCommandPlan,
  parseWorkspaceArgs,
  workspaceAccessLines,
} from '../workspace-launch-lib.mjs';

test('parseWorkspaceArgs returns browser-mode defaults', () => {
  const settings = parseWorkspaceArgs([]);

  assert.deepEqual(settings, {
    databaseUrl: null,
    stopFile: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '0.0.0.0:9983',
    install: false,
    preview: false,
    proxyDev: false,
    tauri: false,
    dryRun: false,
    help: false,
  });
});

test('parseWorkspaceArgs forwards install, preview, proxy-dev, tauri, and bind overrides', () => {
  const settings = parseWorkspaceArgs([
    '--database-url',
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    '--stop-file',
    '.tmp/start-workspace.stop',
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
    '--proxy-dev',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(
    settings.databaseUrl,
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
  );
  assert.equal(settings.stopFile, '.tmp/start-workspace.stop');
  assert.equal(settings.gatewayBind, '0.0.0.0:18080');
  assert.equal(settings.adminBind, '0.0.0.0:18081');
  assert.equal(settings.portalBind, '0.0.0.0:18082');
  assert.equal(settings.webBind, '0.0.0.0:13001');
  assert.equal(settings.install, true);
  assert.equal(settings.preview, true);
  assert.equal(settings.proxyDev, true);
  assert.equal(settings.tauri, true);
  assert.equal(settings.dryRun, true);
});

test('buildWorkspaceCommandPlan keeps local config defaults when database override is absent', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '0.0.0.0:9983',
    install: false,
    preview: false,
    proxyDev: false,
    tauri: false,
    dryRun: true,
    help: false,
  });

  assert.equal(plan.backend.scriptPath, 'scripts/dev/start-stack.mjs');
  assert.deepEqual(plan.backend.args, [
    'scripts/dev/start-stack.mjs',
    '--gateway-bind',
    '127.0.0.1:9980',
    '--admin-bind',
    '127.0.0.1:9981',
    '--portal-bind',
    '127.0.0.1:9982',
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
    proxyDev: false,
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
    proxyDev: false,
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

test('buildWorkspaceCommandPlan can proxy admin and portal dev servers through the unified web host without building static assets', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '127.0.0.1:9983',
    install: false,
    preview: false,
    proxyDev: true,
    tauri: false,
    dryRun: true,
    help: false,
  });

  assert.deepEqual(plan.admin.args, ['scripts/dev/start-admin.mjs', '--dry-run']);
  assert.deepEqual(plan.portal.args, ['scripts/dev/start-portal.mjs', '--dry-run']);
  assert.deepEqual(plan.web.args, [
    'scripts/dev/start-web.mjs',
    '--bind',
    '127.0.0.1:9983',
    '--admin-target',
    '127.0.0.1:9981',
    '--portal-target',
    '127.0.0.1:9982',
    '--gateway-target',
    '127.0.0.1:9980',
    '--admin-site-target',
    '127.0.0.1:5173',
    '--portal-site-target',
    '127.0.0.1:5174',
    '--proxy-dev',
    '--dry-run',
  ]);
});

test('buildWorkspaceCommandPlan keeps browser mode on standalone admin and portal apps', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '0.0.0.0:9983',
    install: false,
    preview: false,
    proxyDev: false,
    tauri: false,
    dryRun: true,
    help: false,
  });

  assert.equal(plan.admin.scriptPath, 'scripts/dev/start-admin.mjs');
  assert.deepEqual(plan.admin.args, ['scripts/dev/start-admin.mjs', '--dry-run']);
  assert.equal(plan.portal.scriptPath, 'scripts/dev/start-portal.mjs');
  assert.deepEqual(plan.portal.args, ['scripts/dev/start-portal.mjs', '--dry-run']);
});

test('workspaceAccessLines describe unified access, direct service links, and seeded credentials', () => {
  const previewLines = workspaceAccessLines({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '127.0.0.1:9983',
    install: false,
    preview: true,
    proxyDev: false,
    tauri: false,
    dryRun: false,
    help: false,
  }).join('\n');

  assert.match(previewLines, /Unified Access/);
  assert.match(previewLines, /http:\/\/127\.0\.0\.1:9983\/admin\//);
  assert.match(previewLines, /http:\/\/127\.0\.0\.1:9983\/portal\//);
  assert.match(previewLines, /http:\/\/127\.0\.0\.1:9983\/api\/v1\/health/);
  assert.match(previewLines, /http:\/\/127\.0\.0\.1:9980\/health/);
  assert.match(previewLines, /http:\/\/127\.0\.0\.1:9981\/admin\/health/);
  assert.match(previewLines, /http:\/\/127\.0\.0\.1:9982\/portal\/health/);
  assert.match(previewLines, /admin@sdkwork\.local/);
  assert.match(previewLines, /portal@sdkwork\.local/);

  const browserLines = workspaceAccessLines({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '127.0.0.1:9983',
    install: false,
    preview: false,
    proxyDev: false,
    tauri: false,
    dryRun: false,
    help: false,
  }).join('\n');

  assert.match(browserLines, /Frontend Access/);
  assert.match(browserLines, /http:\/\/127\.0\.0\.1:5173\/admin\//);
  assert.match(browserLines, /http:\/\/127\.0\.0\.1:5174\/portal\//);

  const proxyDevLines = workspaceAccessLines({
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    webBind: '127.0.0.1:9983',
    install: false,
    preview: false,
    proxyDev: true,
    tauri: false,
    dryRun: false,
    help: false,
  }).join('\n');

  assert.match(proxyDevLines, /Unified Access/);
  assert.match(proxyDevLines, /proxy hot reload/i);
  assert.match(proxyDevLines, /http:\/\/127\.0\.0\.1:9983\/admin\//);
  assert.match(proxyDevLines, /http:\/\/127\.0\.0\.1:9983\/portal\//);
});

test('parseWorkspaceArgs rejects missing values and unknown flags', () => {
  assert.throws(() => parseWorkspaceArgs(['--database-url']), {
    message: /requires a value/,
  });
  assert.throws(() => parseWorkspaceArgs(['--stop-file']), {
    message: /requires a value/,
  });
  assert.throws(() => parseWorkspaceArgs(['--unknown-flag']), {
    message: /unknown option/,
  });
});

test('start-workspace monitors an optional cooperative stop file', () => {
  const script = readFileSync(
    path.join(import.meta.dirname, '..', 'start-workspace.mjs'),
    'utf8',
  );

  assert.match(script, /stopFile/);
  assert.match(script, /existsSync\(stopFile\)/);
  assert.match(script, /stop signal file detected/);
  assert.match(script, /controller\.shutdown\('stop-file', 0\)/);
});

test('long-running dev launchers keep their supervisor process alive until shutdown', () => {
  const scriptNames = [
    'start-admin.mjs',
    'start-console.mjs',
    'start-portal.mjs',
    'start-stack.mjs',
    'start-web.mjs',
    'start-workspace.mjs',
  ];

  for (const scriptName of scriptNames) {
    const script = readFileSync(path.join(import.meta.dirname, '..', scriptName), 'utf8');
    assert.match(script, /createSupervisorKeepAlive/);
    assert.match(script, /releaseKeepAlive/);
  }
});
