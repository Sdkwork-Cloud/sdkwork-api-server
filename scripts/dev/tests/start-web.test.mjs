import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';

import {
  parseWebArgs,
  publicEntryUrls,
  webHostEnv,
  webAccessLines,
} from '../web-launch-lib.mjs';

test('parseWebArgs keeps public Pingora bind by default', () => {
  assert.deepEqual(parseWebArgs([]), {
    adminTarget: '127.0.0.1:9981',
    adminSiteTarget: null,
    bind: '0.0.0.0:9983',
    dryRun: false,
    gatewayTarget: '127.0.0.1:9980',
    help: false,
    install: false,
    portalTarget: '127.0.0.1:9982',
    portalSiteTarget: null,
    proxyDev: false,
    preview: false,
    tauri: false,
  });
});

test('parseWebArgs accepts bind override and flags', () => {
  const settings = parseWebArgs([
    '--bind',
    '0.0.0.0:3901',
    '--admin-target',
    '127.0.0.1:18081',
    '--admin-site-target',
    '127.0.0.1:5173',
    '--portal-target',
    '127.0.0.1:18082',
    '--portal-site-target',
    '127.0.0.1:5174',
    '--gateway-target',
    '127.0.0.1:18080',
    '--install',
    '--proxy-dev',
    '--preview',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(settings.bind, '0.0.0.0:3901');
  assert.equal(settings.adminTarget, '127.0.0.1:18081');
  assert.equal(settings.adminSiteTarget, '127.0.0.1:5173');
  assert.equal(settings.portalTarget, '127.0.0.1:18082');
  assert.equal(settings.portalSiteTarget, '127.0.0.1:5174');
  assert.equal(settings.gatewayTarget, '127.0.0.1:18080');
  assert.equal(settings.install, true);
  assert.equal(settings.proxyDev, true);
  assert.equal(settings.preview, true);
  assert.equal(settings.tauri, true);
  assert.equal(settings.dryRun, true);
});

test('publicEntryUrls exposes localhost when Pingora binds all interfaces', () => {
  const urls = publicEntryUrls('0.0.0.0:3901');

  assert.ok(urls.includes('http://127.0.0.1:3901'));
});

test('webAccessLines include admin and portal entrypoints', () => {
  const lines = webAccessLines('0.0.0.0:3901').join('\n');

  assert.match(lines, /SDKWORK_WEB_BIND=0\.0\.0\.0:3901/);
  assert.match(lines, /\/admin\//);
  assert.match(lines, /\/portal\//);
  assert.match(lines, /\/api\/v1\/health/);
});

test('webHostEnv uses bare host:port upstreams and honors overrides', () => {
  const env = webHostEnv('127.0.0.1:13001', {
    adminTarget: '127.0.0.1:18081',
    adminSiteTarget: '127.0.0.1:5173',
    portalTarget: '127.0.0.1:18082',
    portalSiteTarget: '127.0.0.1:5174',
    gatewayTarget: '127.0.0.1:18080',
  });

  assert.equal(env.SDKWORK_WEB_BIND, '127.0.0.1:13001');
  assert.equal(env.SDKWORK_ADMIN_PROXY_TARGET, '127.0.0.1:18081');
  assert.equal(env.SDKWORK_ADMIN_SITE_PROXY_TARGET, '127.0.0.1:5173');
  assert.equal(env.SDKWORK_PORTAL_PROXY_TARGET, '127.0.0.1:18082');
  assert.equal(env.SDKWORK_PORTAL_SITE_PROXY_TARGET, '127.0.0.1:5174');
  assert.equal(env.SDKWORK_GATEWAY_PROXY_TARGET, '127.0.0.1:18080');
  assert.doesNotMatch(env.SDKWORK_ADMIN_PROXY_TARGET, /^http:\/\//);
  assert.doesNotMatch(env.SDKWORK_ADMIN_SITE_PROXY_TARGET, /^http:\/\//);
  assert.doesNotMatch(env.SDKWORK_PORTAL_PROXY_TARGET, /^http:\/\//);
  assert.doesNotMatch(env.SDKWORK_PORTAL_SITE_PROXY_TARGET, /^http:\/\//);
  assert.doesNotMatch(env.SDKWORK_GATEWAY_PROXY_TARGET, /^http:\/\//);
});

test('webAccessLines can describe unified hot-reload proxy mode through Pingora', () => {
  const lines = webAccessLines('0.0.0.0:3901', { proxyDev: true }).join('\n');

  assert.match(lines, /proxy hot reload/i);
  assert.match(lines, /\/admin\//);
  assert.match(lines, /\/portal\//);
});

test('webHostEnv falls back to Visual Studio generator when Windows Ninja is unavailable', () => {
  const env = webHostEnv(
    '127.0.0.1:13001',
    {},
    {
      baseEnv: {
        CMAKE_GENERATOR: 'Ninja',
        HOST_CMAKE_GENERATOR: 'Ninja',
      },
      platform: 'win32',
      hasNinja: false,
    },
  );

  assert.equal(env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
});

test('webHostEnv preserves supported Windows generators when explicitly configured', () => {
  const env = webHostEnv(
    '127.0.0.1:13001',
    {},
    {
      baseEnv: {
        CMAKE_GENERATOR: 'Visual Studio 17 2022',
        HOST_CMAKE_GENERATOR: 'Visual Studio 17 2022',
      },
      platform: 'win32',
      hasNinja: false,
    },
  );

  assert.equal(env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
});

test('preview launchers can reuse existing dist output on Windows spawn EPERM build failures', () => {
  const scriptPaths = [
    path.join(import.meta.dirname, '..', 'start-admin.mjs'),
    path.join(import.meta.dirname, '..', 'start-portal.mjs'),
    path.join(import.meta.dirname, '..', 'start-web.mjs'),
  ];

  for (const scriptPath of scriptPaths) {
    const script = readFileSync(scriptPath, 'utf8');
    assert.match(script, /shouldReuseExistingFrontendDist/);
    assert.match(script, /reusing existing dist/i);
  }
});

test('start-web only emits raw spawn error stacks when the Windows EPERM fallback cannot recover', () => {
  const script = readFileSync(path.join(import.meta.dirname, '..', 'start-web.mjs'), 'utf8');

  assert.match(script, /const reuseExistingDist = shouldReuseExistingFrontendDist/);
  assert.match(script, /if \(result\.error && !reuseExistingDist\)/);
  assert.match(script, /console\.warn\(\`\[start-web\] \$\{label\} failed with Windows spawn EPERM; reusing existing dist at \$\{distDir\}`\)/);
});
