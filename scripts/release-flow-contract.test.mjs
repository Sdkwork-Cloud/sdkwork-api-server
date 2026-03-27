import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const rootDir = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(rootDir, relativePath), 'utf8');
}

test('repository exposes a native platform and architecture release workflow', () => {
  const workflowPath = path.join(rootDir, '.github', 'workflows', 'release.yml');
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /windows-2022/);
  assert.match(workflow, /windows-11-arm/);
  assert.match(workflow, /ubuntu-22\.04/);
  assert.match(workflow, /macos-14/);
  assert.match(workflow, /arch:\s*x64/);
  assert.match(workflow, /arch:\s*arm64/);
  assert.match(workflow, /target:\s*x86_64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*aarch64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*x86_64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*aarch64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*x86_64-apple-darwin/);
  assert.match(workflow, /target:\s*aarch64-apple-darwin/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
  assert.match(workflow, /cargo build --release --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /router-product-service/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app admin --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app console --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs web/);
  assert.match(workflow, /release-assets-native-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}/);
  assert.match(workflow, /softprops\/action-gh-release@/);
});

test('tauri package scripts stay portable across admin, portal, and console apps', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const portalPackage = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const consolePackage = JSON.parse(read('console/package.json'));

  assert.match(adminPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(adminPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(portalPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(portalPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consolePackage.scripts['tauri:dev'], /node \.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(consolePackage.scripts['tauri:build'], /node \.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.doesNotMatch(portalPackage.scripts['tauri:dev'], /powershell/i);
  assert.doesNotMatch(portalPackage.scripts['tauri:build'], /powershell/i);
});

test('release target helpers and desktop release runner resolve explicit target triples', async () => {
  const helperPath = path.join(rootDir, 'scripts', 'release', 'desktop-targets.mjs');
  const runnerPath = path.join(rootDir, 'scripts', 'release', 'run-desktop-release-build.mjs');
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');

  assert.equal(existsSync(helperPath), true, 'missing scripts/release/desktop-targets.mjs');
  assert.equal(existsSync(runnerPath), true, 'missing scripts/release/run-desktop-release-build.mjs');
  assert.equal(existsSync(packagerPath), true, 'missing scripts/release/package-release-assets.mjs');

  const helper = await import(pathToFileURL(helperPath).href);
  const runner = await import(pathToFileURL(runnerPath).href);
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof helper.parseDesktopTargetTriple, 'function');
  assert.equal(typeof helper.resolveDesktopReleaseTarget, 'function');
  assert.equal(typeof runner.createDesktopReleaseBuildPlan, 'function');
  assert.equal(typeof packager.resolveNativeBuildRoot, 'function');
  assert.equal(typeof packager.listNativeServiceBinaryNames, 'function');
  assert.equal(typeof packager.buildNativeProductServerArchiveBaseName, 'function');

  assert.deepEqual(
    helper.parseDesktopTargetTriple('aarch64-pc-windows-msvc'),
    {
      platform: 'windows',
      arch: 'arm64',
      targetTriple: 'aarch64-pc-windows-msvc',
    },
  );
  assert.deepEqual(
    helper.resolveDesktopReleaseTarget({
      env: { SDKWORK_DESKTOP_TARGET: 'x86_64-apple-darwin' },
    }),
    {
      platform: 'macos',
      arch: 'x64',
      targetTriple: 'x86_64-apple-darwin',
    },
  );

  const portalBuildPlan = runner.createDesktopReleaseBuildPlan({
    appId: 'portal',
    appDir: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
    platform: 'win32',
    env: {},
    targetTriple: 'aarch64-pc-windows-msvc',
  });

  assert.equal(portalBuildPlan.command, 'pnpm');
  assert.deepEqual(portalBuildPlan.args, [
    'tauri:build',
    '--',
    '--target',
    'aarch64-pc-windows-msvc',
  ]);
  assert.equal(portalBuildPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET, 'aarch64-pc-windows-msvc');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'windows');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'arm64');

  assert.equal(
    packager.resolveNativeBuildRoot({
      appId: 'admin',
      targetTriple: 'x86_64-pc-windows-msvc',
    }).replaceAll('\\', '/'),
    path.join(
      rootDir,
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'target',
      'x86_64-pc-windows-msvc',
      'release',
      'bundle',
    ).replaceAll('\\', '/'),
  );
  assert.match(
    packager.listNativeServiceBinaryNames().join(','),
    /router-product-service/,
  );
  assert.equal(
    packager.buildNativeProductServerArchiveBaseName({
      platformId: 'linux',
      archId: 'arm64',
    }),
    'sdkwork-api-router-product-server-linux-arm64',
  );
});
