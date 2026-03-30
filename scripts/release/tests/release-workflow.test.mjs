import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('repository exposes a multi-platform GitHub release workflow for tagged and manual product releases', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /windows-2022/);
  assert.match(workflow, /windows-11-arm/);
  assert.match(workflow, /ubuntu-22\.04/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
  assert.match(workflow, /macos-15-intel/);
  assert.match(workflow, /macos-14/);
  assert.match(workflow, /arch:\s*x64/);
  assert.match(workflow, /arch:\s*arm64/);
  assert.match(workflow, /target:\s*x86_64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*aarch64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*x86_64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*aarch64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*x86_64-apple-darwin/);
  assert.match(workflow, /target:\s*aarch64-apple-darwin/);
  assert.match(workflow, /cargo build --release --target \$\{\{ matrix\.target \}\} -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app admin --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /pnpm --dir apps\/sdkwork-router-admin build/);
  assert.match(workflow, /pnpm --dir apps\/sdkwork-router-portal build/);
  assert.match(workflow, /pnpm --dir console build/);
  assert.match(workflow, /pnpm --dir docs build/);
  assert.match(workflow, /release-assets-native-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}/);
  assert.match(workflow, /softprops\/action-gh-release@/);
});

test('portal package exposes unified product launchers and all desktop scripts use the shared tauri runner', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const packageJson = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const consolePackage = JSON.parse(read('console/package.json'));
  const runnerPath = path.join(repoRoot, 'scripts', 'run-tauri-cli.mjs');

  assert.equal(existsSync(runnerPath), true, 'missing shared scripts/run-tauri-cli.mjs');
  assert.match(packageJson.scripts['product:start'], /node \.\.\/\.\.\/scripts\/run-router-product\.mjs/);
  assert.match(packageJson.scripts['product:service'], /node \.\.\/\.\.\/scripts\/run-router-product\.mjs service/);
  assert.match(packageJson.scripts['server:start'], /node \.\.\/\.\.\/scripts\/run-router-product-service\.mjs/);
  assert.match(packageJson.scripts['server:plan'], /node \.\.\/\.\.\/scripts\/run-router-product-service\.mjs --dry-run --plan-format json/);
  assert.match(packageJson.scripts['product:check'], /node \.\.\/\.\.\/scripts\/check-router-product\.mjs/);
  assert.match(adminPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(adminPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(packageJson.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(packageJson.scripts['tauri:dev:service'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev -- --service/);
  assert.match(packageJson.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consolePackage.scripts['tauri:dev'], /node \.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(consolePackage.scripts['tauri:build'], /node \.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.doesNotMatch(packageJson.scripts['tauri:dev'], /powershell/i);
  assert.doesNotMatch(packageJson.scripts['tauri:build'], /powershell/i);
});

test('shared tauri runner only injects the Visual Studio generator on Windows and carries explicit release target metadata', async () => {
  const runner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-tauri-cli.mjs'),
    ).href,
  );

  assert.equal(typeof runner.createTauriCliPlan, 'function');
  assert.equal(typeof runner.withSupportedWindowsCmakeGenerator, 'function');

  const windowsPlan = runner.createTauriCliPlan({
    commandName: 'build',
    args: ['--target', 'aarch64-pc-windows-msvc'],
    platform: 'win32',
    env: {},
  });
  const linuxPlan = runner.createTauriCliPlan({
    commandName: 'build',
    args: ['--target', 'x86_64-unknown-linux-gnu'],
    platform: 'linux',
    env: {},
  });
  const backgroundDevPlan = runner.createTauriCliPlan({
    commandName: 'dev',
    args: ['--', '--service'],
    platform: 'linux',
    env: {},
  });

  assert.equal(windowsPlan.command, 'tauri.cmd');
  assert.deepEqual(windowsPlan.args, ['build', '--target', 'aarch64-pc-windows-msvc']);
  assert.equal(windowsPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(windowsPlan.env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(windowsPlan.env.SDKWORK_DESKTOP_TARGET, 'aarch64-pc-windows-msvc');
  assert.equal(windowsPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'windows');
  assert.equal(windowsPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'arm64');
  assert.equal(windowsPlan.windowsHide, true);

  assert.equal(linuxPlan.command, 'tauri');
  assert.deepEqual(linuxPlan.args, ['build', '--target', 'x86_64-unknown-linux-gnu']);
  assert.equal(linuxPlan.env.SDKWORK_DESKTOP_TARGET, 'x86_64-unknown-linux-gnu');
  assert.equal(linuxPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'linux');
  assert.equal(linuxPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'x64');
  assert.equal(Object.hasOwn(linuxPlan.env, 'CMAKE_GENERATOR'), false);
  assert.equal(backgroundDevPlan.detached, true);
  assert.equal(backgroundDevPlan.windowsHide, false);
});

test('shared tauri runner prepends the local cargo bin directory on Windows', async () => {
  if (process.platform !== 'win32') {
    return;
  }

  const runner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-tauri-cli.mjs'),
    ).href,
  );

  const plan = runner.createTauriCliPlan({
    commandName: 'dev',
    platform: 'win32',
    env: {
      USERPROFILE: process.env.USERPROFILE ?? '',
      PATH: '',
    },
  });

  const expectedCargoBin = path.join(process.env.USERPROFILE ?? '', '.cargo', 'bin').toLowerCase();
  assert.ok(
    String(plan.env.PATH ?? '')
      .toLowerCase()
      .startsWith(expectedCargoBin),
    'cargo bin should be the first PATH entry for tauri commands',
  );
  assert.match(
    String(plan.env.CARGO_TARGET_DIR ?? ''),
    /sdkwork-tauri-target/i,
  );
});
