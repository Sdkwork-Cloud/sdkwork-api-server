import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  databaseDisplayValue,
  parseStackArgs,
  serviceEnv,
} from '../backend-launch-lib.mjs';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('parseStackArgs keeps local sqlite default when no database override is provided', () => {
  const settings = parseStackArgs([]);

  assert.equal(settings.databaseUrl, null);
  assert.equal(settings.gatewayBind, '127.0.0.1:9980');
  assert.equal(settings.adminBind, '127.0.0.1:9981');
  assert.equal(settings.portalBind, '127.0.0.1:9982');
  assert.equal(databaseDisplayValue(settings), '(local default via config loader)');
});

test('serviceEnv omits SDKWORK_DATABASE_URL when local config defaults should apply', () => {
  const env = serviceEnv(
    {
      databaseUrl: null,
      gatewayBind: '127.0.0.1:9980',
      adminBind: '127.0.0.1:9981',
      portalBind: '127.0.0.1:9982',
    },
    {
      SDKWORK_DATABASE_URL: 'postgres://should-be-removed',
    },
  );

  assert.equal(env.SDKWORK_DATABASE_URL, undefined);
  assert.equal(env.SDKWORK_GATEWAY_BIND, '127.0.0.1:9980');
  assert.equal(env.SDKWORK_ADMIN_BIND, '127.0.0.1:9981');
  assert.equal(env.SDKWORK_PORTAL_BIND, '127.0.0.1:9982');
});

test('start-stack reuses prebuilt Windows backend binaries after warm-up instead of spawning competing cargo runs', {
  skip: process.platform !== 'win32',
}, () => {
  const targetDir = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-start-stack-target-'));
  const debugDir = path.join(targetDir, 'debug');
  mkdirSync(debugDir, { recursive: true });

  for (const packageName of ['admin-api-service', 'gateway-service', 'portal-api-service']) {
    writeFileSync(path.join(debugDir, `${packageName}.exe`), '');
  }

  try {
    const result = spawnSync(
      process.execPath,
      ['scripts/dev/start-stack.mjs', '--dry-run'],
      {
        cwd: repoRoot,
        env: {
          ...process.env,
          SDKWORK_ROUTER_USE_PREBUILT_BACKEND_BINARIES: '1',
          CARGO_TARGET_DIR: targetDir,
        },
        encoding: 'utf8',
      },
    );

    assert.equal(result.status, 0, result.stderr || result.stdout);
    const normalizedOutput = result.stdout.replaceAll('\\', '/');
    const normalizedTargetDir = targetDir.replaceAll('\\', '/').replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    assert.match(normalizedOutput, new RegExp(`${normalizedTargetDir}/debug/admin-api-service\\.exe`));
    assert.match(normalizedOutput, new RegExp(`${normalizedTargetDir}/debug/gateway-service\\.exe`));
    assert.match(normalizedOutput, new RegExp(`${normalizedTargetDir}/debug/portal-api-service\\.exe`));
    assert.doesNotMatch(normalizedOutput, /cargo(?:\.exe)? run -p admin-api-service/);
    assert.doesNotMatch(normalizedOutput, /cargo(?:\.exe)? run -p gateway-service/);
    assert.doesNotMatch(normalizedOutput, /cargo(?:\.exe)? run -p portal-api-service/);
  } finally {
    rmSync(targetDir, { recursive: true, force: true });
  }
});
