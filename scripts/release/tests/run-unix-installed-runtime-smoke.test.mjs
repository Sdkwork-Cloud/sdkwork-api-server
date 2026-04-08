import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('unix installed runtime smoke script exposes a parseable CLI contract for release workflows', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-unix-installed-runtime-smoke.mjs'),
    ).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createUnixInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof module.createUnixInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof module.createUnixInstalledRuntimeSmokeEvidence, 'function');

  const options = module.parseArgs([
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--target',
    'x86_64-unknown-linux-gnu',
    '--runtime-home',
    'artifacts/release-smoke/linux-x64',
    '--evidence-path',
    'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
  ]);

  assert.deepEqual(options, {
    platform: 'linux',
    arch: 'x64',
    target: 'x86_64-unknown-linux-gnu',
    runtimeHome: path.resolve(repoRoot, 'artifacts', 'release-smoke', 'linux-x64'),
    evidencePath: path.resolve(repoRoot, 'artifacts', 'release-governance', 'unix-installed-runtime-smoke-linux-x64.json'),
  });

  const plan = module.createUnixInstalledRuntimeSmokePlan({
    repoRoot,
    ...options,
    ports: {
      web: 19483,
      gateway: 19480,
      admin: 19481,
      portal: 19482,
    },
  });

  assert.equal(plan.runtimeHome, options.runtimeHome);
  assert.equal(plan.evidencePath, options.evidencePath);
  assert.equal(plan.installPlan.directories[0], options.runtimeHome);
  assert.deepEqual(plan.healthUrls, [
    'http://127.0.0.1:19483/api/v1/health',
    'http://127.0.0.1:19483/api/admin/health',
    'http://127.0.0.1:19483/api/portal/health',
  ]);
  assert.deepEqual(plan.startCommand.args, ['--home', options.runtimeHome, '--wait-seconds', '120']);
  assert.deepEqual(plan.stopCommand.args, ['--home', options.runtimeHome, '--wait-seconds', '120']);
  assert.match(plan.routerEnvContents, /SDKWORK_WEB_BIND="127\.0\.0\.1:19483"/);
  assert.match(plan.routerEnvContents, /SDKWORK_GATEWAY_BIND="127\.0\.0\.1:19480"/);
  assert.match(plan.routerEnvContents, /SDKWORK_ADMIN_BIND="127\.0\.0\.1:19481"/);
  assert.match(plan.routerEnvContents, /SDKWORK_PORTAL_BIND="127\.0\.0\.1:19482"/);

  const successEvidence = module.createUnixInstalledRuntimeSmokeEvidence({
    plan,
    ok: true,
  });
  assert.equal(successEvidence.ok, true);
  assert.equal(successEvidence.platform, 'linux');
  assert.equal(successEvidence.arch, 'x64');
  assert.equal(successEvidence.target, 'x86_64-unknown-linux-gnu');
  assert.deepEqual(successEvidence.healthUrls, plan.healthUrls);
  assert.equal(successEvidence.runtimeHome, path.relative(repoRoot, options.runtimeHome).replaceAll('\\', '/'));
  assert.equal(successEvidence.evidencePath, path.relative(repoRoot, options.evidencePath).replaceAll('\\', '/'));

  const failureEvidence = module.createUnixInstalledRuntimeSmokeEvidence({
    plan,
    ok: false,
    failure: new Error('health probe failed'),
  });
  assert.equal(failureEvidence.ok, false);
  assert.equal(failureEvidence.failure.message, 'health probe failed');
});

test('unix installed runtime smoke options reject unsupported Windows release lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-unix-installed-runtime-smoke.mjs'),
    ).href,
  );

  assert.throws(
    () => module.createUnixInstalledRuntimeSmokeOptions({
      repoRoot,
      platform: 'windows',
      arch: 'x64',
      target: 'x86_64-pc-windows-msvc',
    }),
    /only supports linux and macos release lanes/i,
  );
});
