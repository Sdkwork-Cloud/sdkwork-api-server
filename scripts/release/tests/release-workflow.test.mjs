import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('repository exposes a cross-platform GitHub release workflow for tagged and manual releases', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /windows-latest/);
  assert.match(workflow, /ubuntu-latest/);
  assert.match(workflow, /macos-latest/);
  assert.match(workflow, /cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service/);
  assert.match(workflow, /pnpm --dir apps\/sdkwork-router-admin build/);
  assert.match(workflow, /pnpm --dir apps\/sdkwork-router-portal build/);
  assert.match(workflow, /pnpm --dir docs build/);
  assert.match(workflow, /softprops\/action-gh-release@/);
});

test('portal tauri scripts are cross-platform and no longer depend on a powershell-only wrapper', () => {
  const packageJson = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const runnerPath = path.join(
    repoRoot,
    'apps',
    'sdkwork-router-portal',
    'scripts',
    'run-tauri.mjs',
  );

  assert.equal(existsSync(runnerPath), true, 'missing portal scripts/run-tauri.mjs');
  assert.match(packageJson.scripts['tauri:dev'], /node \.\/scripts\/run-tauri\.mjs dev/);
  assert.match(packageJson.scripts['tauri:build'], /node \.\/scripts\/run-tauri\.mjs build/);
  assert.doesNotMatch(packageJson.scripts['tauri:dev'], /powershell/i);
  assert.doesNotMatch(packageJson.scripts['tauri:build'], /powershell/i);
});

test('portal tauri runner only injects the Visual Studio generator on Windows and keeps unix builds clean', async () => {
  const runner = await import(
    pathToFileURL(
      path.join(repoRoot, 'apps', 'sdkwork-router-portal', 'scripts', 'run-tauri.mjs'),
    ).href,
  );

  assert.equal(typeof runner.createTauriCommandPlan, 'function');

  const windowsPlan = runner.createTauriCommandPlan({
    mode: 'build',
    platform: 'win32',
    env: {},
  });
  const linuxPlan = runner.createTauriCommandPlan({
    mode: 'build',
    platform: 'linux',
    env: {},
  });

  assert.equal(windowsPlan.command, 'tauri');
  assert.deepEqual(windowsPlan.args, ['build']);
  assert.equal(windowsPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');

  assert.equal(linuxPlan.command, 'tauri');
  assert.deepEqual(linuxPlan.args, ['build']);
  assert.equal(Object.hasOwn(linuxPlan.env, 'CMAKE_GENERATOR'), false);
});
