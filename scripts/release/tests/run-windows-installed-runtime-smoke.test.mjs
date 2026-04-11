import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('windows installed runtime smoke script exposes a parseable CLI contract for release workflows', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createWindowsInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof module.createWindowsInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof module.createWindowsInstalledRuntimeSmokeEvidence, 'function');

  const options = module.parseArgs([
    '--platform',
    'windows',
    '--arch',
    'x64',
    '--target',
    'x86_64-pc-windows-msvc',
    '--runtime-home',
    'artifacts/release-smoke/windows-x64',
    '--evidence-path',
    'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
  ]);

  assert.deepEqual(options, {
    platform: 'windows',
    arch: 'x64',
    target: 'x86_64-pc-windows-msvc',
    runtimeHome: path.resolve(repoRoot, 'artifacts', 'release-smoke', 'windows-x64'),
    evidencePath: path.resolve(repoRoot, 'artifacts', 'release-governance', 'windows-installed-runtime-smoke-windows-x64.json'),
  });

  const plan = module.createWindowsInstalledRuntimeSmokePlan({
    repoRoot,
    ...options,
    ports: {
      web: 29483,
      gateway: 29480,
      admin: 29481,
      portal: 29482,
    },
  });

  assert.equal(plan.runtimeHome, options.runtimeHome);
  assert.equal(plan.evidencePath, options.evidencePath);
  assert.equal(plan.installPlan.directories[0], options.runtimeHome);
  assert.deepEqual(plan.healthUrls, [
    'http://127.0.0.1:29483/api/v1/health',
    'http://127.0.0.1:29483/api/admin/health',
    'http://127.0.0.1:29483/api/portal/health',
  ]);
  assert.deepEqual(plan.startCommand.args, [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-File',
    path.join(options.runtimeHome, 'bin', 'start.ps1'),
    '-Home',
    options.runtimeHome,
    '-WaitSeconds',
    '120',
  ]);
  assert.equal(plan.startCommand.stdio, 'ignore');
  assert.deepEqual(plan.stopCommand.args, [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-File',
    path.join(options.runtimeHome, 'bin', 'stop.ps1'),
    '-Home',
    options.runtimeHome,
    '-WaitSeconds',
    '120',
  ]);
  assert.match(plan.routerEnvContents, /SDKWORK_WEB_BIND="127\.0\.0\.1:29483"/);
  assert.match(plan.routerEnvContents, /SDKWORK_GATEWAY_BIND="127\.0\.0\.1:29480"/);
  assert.match(plan.routerEnvContents, /SDKWORK_ADMIN_BIND="127\.0\.0\.1:29481"/);
  assert.match(plan.routerEnvContents, /SDKWORK_PORTAL_BIND="127\.0\.0\.1:29482"/);

  const successEvidence = module.createWindowsInstalledRuntimeSmokeEvidence({
    plan,
    ok: true,
  });
  assert.equal(successEvidence.ok, true);
  assert.equal(successEvidence.platform, 'windows');
  assert.equal(successEvidence.arch, 'x64');
  assert.equal(successEvidence.target, 'x86_64-pc-windows-msvc');
  assert.deepEqual(successEvidence.healthUrls, plan.healthUrls);
  assert.equal(successEvidence.runtimeHome, path.relative(repoRoot, options.runtimeHome).replaceAll('\\', '/'));
  assert.equal(successEvidence.evidencePath, path.relative(repoRoot, options.evidencePath).replaceAll('\\', '/'));

  const failureEvidence = module.createWindowsInstalledRuntimeSmokeEvidence({
    plan,
    ok: false,
    failure: new Error('powershell smoke failed'),
  });
  assert.equal(failureEvidence.ok, false);
  assert.equal(failureEvidence.failure.message, 'powershell smoke failed');
});

test('windows installed runtime smoke options reject unsupported Unix release lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );

  assert.throws(
    () => module.createWindowsInstalledRuntimeSmokeOptions({
      repoRoot,
      platform: 'linux',
      arch: 'x64',
      target: 'x86_64-unknown-linux-gnu',
    }),
    /only supports windows release lanes/i,
  );
});
