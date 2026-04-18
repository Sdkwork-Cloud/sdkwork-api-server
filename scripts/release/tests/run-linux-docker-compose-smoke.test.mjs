import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('linux Docker Compose smoke script exposes a parseable CLI contract for packaged product bundles', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createLinuxDockerComposeSmokeOptions, 'function');
  assert.equal(typeof module.createLinuxDockerComposeSmokePlan, 'function');
  assert.equal(typeof module.createLinuxDockerComposeSmokeEvidence, 'function');
  assert.equal(typeof module.resolveLinuxDockerComposeSmokeExecutionMode, 'function');
  assert.equal(typeof module.createLinuxDockerRunFallbackResources, 'function');
  assert.equal(typeof module.normalizeSpawnEnvironmentForPlatform, 'function');
  assert.equal(typeof module.normalizeDockerFallbackLogCapture, 'function');
  assert.equal(typeof module.createDockerRunLogEvidence, 'function');
  assert.equal(typeof module.shouldContinueWithoutBrowserSmoke, 'function');
  assert.equal(typeof module.isCliEntrypoint, 'function');
  assert.equal(typeof module.resolveExtractedBundleRoot, 'function');

  const options = module.parseArgs([
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--bundle-path',
    'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
    '--evidence-path',
    'artifacts/release-governance/docker-compose-smoke-linux-x64.json',
  ]);

  assert.deepEqual(options, {
    platform: 'linux',
    arch: 'x64',
    bundlePath: path.resolve(
      repoRoot,
      'artifacts',
      'release',
      'native',
      'linux',
      'x64',
      'bundles',
      'sdkwork-api-router-product-server-linux-x64.tar.gz',
    ),
    evidencePath: path.resolve(
      repoRoot,
      'artifacts',
      'release-governance',
      'docker-compose-smoke-linux-x64.json',
    ),
  });

  const plan = module.createLinuxDockerComposeSmokePlan({
    repoRoot,
    hostPlatform: 'linux',
    ...options,
  });

  assert.equal(plan.bundlePath, options.bundlePath);
  assert.equal(plan.evidencePath, options.evidencePath);
  assert.equal(plan.executionMode, 'docker-compose');
  assert.equal(plan.composeRelativePath, 'deploy/docker/docker-compose.yml');
  assert.equal(plan.envRelativePath, 'deploy/docker/.env');
  assert.equal(plan.overrideRelativePath, 'deploy/docker/docker-compose.smoke.override.yml');
  assert.deepEqual(plan.healthUrls, [
    'http://127.0.0.1:3001/api/v1/health',
    'http://127.0.0.1:3001/api/admin/health',
    'http://127.0.0.1:3001/api/portal/health',
  ]);
  assert.deepEqual(plan.siteUrls, [
    'http://127.0.0.1:3001/admin/',
    'http://127.0.0.1:3001/portal/',
  ]);
  assert.deepEqual(plan.requiredImageRefs, [
    'docker.io/library/postgres:16-alpine',
  ]);
  assert.deepEqual(plan.browserSmokeTargets, [
    {
      label: 'admin',
      url: 'http://127.0.0.1:3001/admin/',
      expectedTexts: [],
      expectedSelectors: [
        'input[type="email"]',
        'input[type="password"]',
        'button[type="submit"]',
      ],
    },
    {
      label: 'portal',
      url: 'http://127.0.0.1:3001/portal/',
      expectedTexts: [
        'Operate routing, credentials, usage, and downloads from one product surface.',
        'Launch sequence',
        'Product pathways',
      ],
      expectedSelectors: [],
    },
  ]);
  assert.match(plan.envContents, /^SDKWORK_BOOTSTRAP_PROFILE=prod$/m);
  assert.match(plan.envContents, /^SDKWORK_ADMIN_JWT_SIGNING_SECRET=/m);
  assert.match(plan.envContents, /^SDKWORK_PORTAL_JWT_SIGNING_SECRET=/m);
  assert.match(plan.envContents, /^SDKWORK_CREDENTIAL_MASTER_KEY=/m);
  assert.match(plan.envContents, /^SDKWORK_METRICS_BEARER_TOKEN=/m);
  assert.match(plan.overrideContents, /3001:3001/);
  assert.deepEqual(
    plan.databaseAssertions.map((entry) => entry.table),
    [
      'ai_channel',
      'ai_proxy_provider',
      'ai_marketing_coupon_template',
      'ai_marketing_campaign',
      'ai_marketing_coupon_code',
    ],
  );
  assert.deepEqual(plan.fallbackResources, {
    networkName: 'sdkwork-release-smoke-linux-x64-default',
    postgresVolumeName: 'sdkwork-release-smoke-linux-x64-sdkwork-postgres',
    postgresContainerName: 'sdkwork-release-smoke-linux-x64-postgres-1',
    routerContainerName: 'sdkwork-release-smoke-linux-x64-router-1',
    routerImageTag: 'sdkwork-release-smoke-linux-x64-router-smoke:latest',
  });

  const successEvidence = module.createLinuxDockerComposeSmokeEvidence({
    repoRoot,
    plan,
    ok: true,
    databaseAssertions: [
      { table: 'ai_channel', count: 19 },
      { table: 'ai_proxy_provider', count: 17 },
    ],
  });
  assert.equal(successEvidence.ok, true);
  assert.equal(successEvidence.platform, 'linux');
  assert.equal(successEvidence.arch, 'x64');
  assert.equal(successEvidence.executionMode, 'docker-compose');
  assert.equal(successEvidence.bundlePath, path.relative(repoRoot, options.bundlePath).replaceAll('\\', '/'));
  assert.equal(successEvidence.evidencePath, path.relative(repoRoot, options.evidencePath).replaceAll('\\', '/'));
  assert.deepEqual(successEvidence.siteUrls, plan.siteUrls);
  assert.deepEqual(successEvidence.browserSmokeTargets, plan.browserSmokeTargets);
  assert.equal(successEvidence.databaseAssertions[0].count, 19);

  const failureEvidence = module.createLinuxDockerComposeSmokeEvidence({
    repoRoot,
    plan,
    ok: false,
    failure: new Error('docker compose smoke failed'),
  });
  assert.equal(failureEvidence.ok, false);
  assert.equal(failureEvidence.failure.message, 'docker compose smoke failed');
});

test('linux Docker Compose smoke options reject unsupported non-linux release lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  assert.throws(
    () => module.createLinuxDockerComposeSmokeOptions({
      repoRoot,
      platform: 'windows',
      arch: 'x64',
    }),
    /only supports linux release lanes/i,
  );
});

test('linux Docker Compose smoke switches to docker-run fallback on Windows hosts', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  assert.equal(
    module.resolveLinuxDockerComposeSmokeExecutionMode({ hostPlatform: 'win32' }),
    'docker-run',
  );
  assert.equal(
    module.resolveLinuxDockerComposeSmokeExecutionMode({ hostPlatform: 'linux' }),
    'docker-compose',
  );

  const plan = module.createLinuxDockerComposeSmokePlan({
    repoRoot,
    hostPlatform: 'win32',
    platform: 'linux',
    arch: 'x64',
    bundlePath: path.resolve(
      repoRoot,
      'artifacts',
      'release',
      'native',
      'linux',
      'x64',
      'bundles',
      'sdkwork-api-router-product-server-linux-x64.tar.gz',
    ),
    evidencePath: path.resolve(
      repoRoot,
      'artifacts',
      'release-governance',
      'docker-compose-smoke-linux-x64.json',
    ),
  });

  assert.equal(plan.executionMode, 'docker-run');
  assert.deepEqual(plan.fallbackResources, {
    networkName: 'sdkwork-release-smoke-linux-x64-default',
    postgresVolumeName: 'sdkwork-release-smoke-linux-x64-sdkwork-postgres',
    postgresContainerName: 'sdkwork-release-smoke-linux-x64-postgres-1',
    routerContainerName: 'sdkwork-release-smoke-linux-x64-router-1',
    routerImageTag: 'sdkwork-release-smoke-linux-x64-router-smoke:latest',
  });
});

test('linux Docker Compose smoke normalizes Windows PATH variants before spawning Docker commands', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  const normalized = module.normalizeSpawnEnvironmentForPlatform({
    platform: 'win32',
    env: {
      PATH: 'C:/docker-bin',
      Path: 'C:/stale-path',
      HOME: 'C:/Users/admin',
    },
  });

  assert.deepEqual(normalized, {
    Path: 'C:/docker-bin',
    HOME: 'C:/Users/admin',
  });
});

test('linux Docker Compose smoke only tolerates missing Chromium on hosted Linux CI lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  const missingBrowserError = new Error(
    'unable to resolve a Chromium-based browser executable for browser runtime smoke on linux',
  );

  assert.equal(
    module.shouldContinueWithoutBrowserSmoke({
      hostPlatform: 'linux',
      env: {
        GITHUB_ACTIONS: 'true',
      },
      error: missingBrowserError,
    }),
    true,
  );

  assert.equal(
    module.shouldContinueWithoutBrowserSmoke({
      hostPlatform: 'linux',
      env: {},
      error: missingBrowserError,
    }),
    false,
  );

  assert.equal(
    module.shouldContinueWithoutBrowserSmoke({
      hostPlatform: 'linux',
      env: {
        GITHUB_ACTIONS: 'true',
      },
      error: new Error('browser runtime smoke did not observe the expected runtime markers before timeout'),
    }),
    false,
  );
});

test('linux Docker Compose smoke resolves the extracted bundle root even when the archive file name changes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  const extractRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-docker-smoke-extract-'));
  const actualBundleRoot = path.join(extractRoot, 'sdkwork-api-router-product-server-linux-x64');
  mkdirSync(actualBundleRoot);

  assert.equal(
    module.resolveExtractedBundleRoot({
      extractRoot,
      bundlePath: path.join(extractRoot, 'sdkwork-api-router-product-server-linux-x64-local.tar.gz'),
    }),
    actualBundleRoot,
  );
});

test('linux Docker Compose smoke suppresses Docker host object lookup anomalies from fallback log evidence', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  const normalized = module.normalizeDockerFallbackLogCapture({
    label: 'router',
    capturedOutput: 'Error response from daemon: {"message":"No such container: sdkwork-release-smoke-linux-x64-router-1"}',
  });

  assert.deepEqual(normalized, {
    content: '',
    diagnostic: 'router logs were unavailable because the Docker host reported an object lookup miss while detached containers were otherwise observable via docker ps',
  });

  const passthrough = module.normalizeDockerFallbackLogCapture({
    label: 'postgres',
    capturedOutput: 'database system is ready to accept connections',
  });
  assert.deepEqual(passthrough, {
    content: 'database system is ready to accept connections',
    diagnostic: '',
  });

  const evidence = module.createDockerRunLogEvidence({
    routerLogOutput: 'Error response from daemon: {"message":"No such container: router"}',
    postgresLogOutput: 'database system is ready to accept connections',
  });
  assert.deepEqual(evidence, {
    logs: {
      router: '',
      postgres: 'database system is ready to accept connections',
    },
    diagnostics: [
      'router logs were unavailable because the Docker host reported an object lookup miss while detached containers were otherwise observable via docker ps',
    ],
  });
});

test('linux Docker Compose smoke CLI entrypoint check is case-insensitive on Windows and strict on Unix', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );

  assert.equal(
    module.isCliEntrypoint({
      argv1: 'd:\\JAVASOURCE\\spring-ai-plus\\spring-ai-plus-business\\apps\\sdkwork-api-router\\scripts\\release\\run-linux-docker-compose-smoke.mjs',
      moduleFile: 'D:\\javasource\\spring-ai-plus\\spring-ai-plus-business\\apps\\sdkwork-api-router\\scripts\\release\\run-linux-docker-compose-smoke.mjs',
      platform: 'win32',
    }),
    true,
  );
  assert.equal(
    module.isCliEntrypoint({
      argv1: '/workspace/scripts/release/run-linux-docker-compose-smoke.mjs',
      moduleFile: '/workspace/scripts/release/run-linux-docker-compose-smoke.mjs',
      platform: 'linux',
    }),
    true,
  );
  assert.equal(
    module.isCliEntrypoint({
      argv1: '/workspace/scripts/release/run-linux-docker-compose-smoke.mjs',
      moduleFile: '/workspace/scripts/release/other-script.mjs',
      platform: 'linux',
    }),
    false,
  );
});
