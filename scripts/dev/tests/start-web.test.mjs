import test from 'node:test';
import assert from 'node:assert/strict';

import {
  parseWebArgs,
  publicEntryUrls,
  webHostEnv,
  webAccessLines,
} from '../web-launch-lib.mjs';

test('parseWebArgs keeps public Pingora bind by default', () => {
  assert.deepEqual(parseWebArgs([]), {
    adminTarget: '127.0.0.1:8081',
    bind: '0.0.0.0:3001',
    dryRun: false,
    gatewayTarget: '127.0.0.1:8080',
    help: false,
    install: false,
    portalTarget: '127.0.0.1:8082',
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
    '--portal-target',
    '127.0.0.1:18082',
    '--gateway-target',
    '127.0.0.1:18080',
    '--install',
    '--preview',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(settings.bind, '0.0.0.0:3901');
  assert.equal(settings.adminTarget, '127.0.0.1:18081');
  assert.equal(settings.portalTarget, '127.0.0.1:18082');
  assert.equal(settings.gatewayTarget, '127.0.0.1:18080');
  assert.equal(settings.install, true);
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
});

test('webHostEnv uses bare host:port upstreams and honors overrides', () => {
  const env = webHostEnv('127.0.0.1:13001', {
    adminTarget: '127.0.0.1:18081',
    portalTarget: '127.0.0.1:18082',
    gatewayTarget: '127.0.0.1:18080',
  });

  assert.equal(env.SDKWORK_WEB_BIND, '127.0.0.1:13001');
  assert.equal(env.SDKWORK_ADMIN_PROXY_TARGET, '127.0.0.1:18081');
  assert.equal(env.SDKWORK_PORTAL_PROXY_TARGET, '127.0.0.1:18082');
  assert.equal(env.SDKWORK_GATEWAY_PROXY_TARGET, '127.0.0.1:18080');
  assert.doesNotMatch(env.SDKWORK_ADMIN_PROXY_TARGET, /^http:\/\//);
  assert.doesNotMatch(env.SDKWORK_PORTAL_PROXY_TARGET, /^http:\/\//);
  assert.doesNotMatch(env.SDKWORK_GATEWAY_PROXY_TARGET, /^http:\/\//);
});
