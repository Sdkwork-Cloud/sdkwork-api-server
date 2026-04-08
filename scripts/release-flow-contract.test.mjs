import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
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
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs web/);
  assert.match(workflow, /release-assets-native-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}/);
  assert.match(workflow, /softprops\/action-gh-release@/);
});

test('tauri package scripts stay portable across admin, portal, and console apps', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const portalPackage = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const consolePackage = JSON.parse(read('console/package.json'));
  const consoleTauriCargo = read('console/src-tauri/Cargo.toml');

  assert.match(adminPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(adminPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(portalPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(portalPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consolePackage.scripts['tauri:dev'], /node \.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(consolePackage.scripts['tauri:build'], /node \.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consoleTauriCargo, /^\[workspace\]$/m);
  assert.doesNotMatch(portalPackage.scripts['tauri:dev'], /powershell/i);
  assert.doesNotMatch(portalPackage.scripts['tauri:build'], /powershell/i);
});

test('web workspace scripts stay portable across Windows, native Unix, and WSL-mounted worktrees', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const portalPackage = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const pnpmLaunchLib = read('scripts/dev/pnpm-launch-lib.mjs');
  const runTscCli = read('scripts/dev/run-tsc-cli.mjs');

  assert.match(adminPackage.scripts.dev, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(adminPackage.scripts.build, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(adminPackage.scripts.typecheck, /^node \.\.\/\.\.\/scripts\/dev\/run-tsc-cli\.mjs(?:\s|$)/);
  assert.match(adminPackage.scripts.preview, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(portalPackage.scripts.dev, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(portalPackage.scripts.build, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(portalPackage.scripts.typecheck, /^tsc(?:\s|$)/);
  assert.match(portalPackage.scripts.preview, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.doesNotMatch(adminPackage.scripts.dev, /run-frontend-tool/);
  assert.doesNotMatch(adminPackage.scripts.build, /run-frontend-tool/);
  assert.doesNotMatch(adminPackage.scripts.typecheck, /run-frontend-tool/);
  assert.doesNotMatch(adminPackage.scripts.preview, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.dev, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.build, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.typecheck, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.preview, /run-frontend-tool/);

  assert.match(pnpmLaunchLib, /requiredBinCommands/);
  assert.match(pnpmLaunchLib, /node_modules', '\.bin'/);
  assert.match(pnpmLaunchLib, /platform === 'win32'/);
  assert.match(pnpmLaunchLib, /\.cmd/);
  assert.match(runTscCli, /resolveReadablePackageEntry/);
  assert.match(runTscCli, /typescript/);
  assert.match(runTscCli, /lib', 'tsc\.js'/);
});

test('desktop tauri configs enable bundling for native release packaging', () => {
  const adminTauriConfig = JSON.parse(read('apps/sdkwork-router-admin/src-tauri/tauri.conf.json'));
  const portalTauriConfig = JSON.parse(read('apps/sdkwork-router-portal/src-tauri/tauri.conf.json'));

  assert.equal(adminTauriConfig.bundle?.active, true);
  assert.equal(portalTauriConfig.bundle?.active, true);
  assert.deepEqual(
    adminTauriConfig.bundle?.icon,
    [
      'icons/32x32.png',
      'icons/128x128.png',
      'icons/128x128@2x.png',
      'icons/icon.icns',
      'icons/icon.ico',
    ],
  );
  assert.deepEqual(
    portalTauriConfig.bundle?.icon,
    [
      'icons/32x32.png',
      'icons/128x128.png',
      'icons/128x128@2x.png',
      'icons/icon.icns',
      'icons/icon.ico',
    ],
  );
  assert.deepEqual(
    adminTauriConfig.bundle?.resources,
    {
      '../dist/': 'embedded-sites/admin/',
      '../../sdkwork-router-portal/dist/': 'embedded-sites/portal/',
    },
  );
  assert.deepEqual(
    portalTauriConfig.bundle?.resources,
    {
      '../dist/': 'embedded-sites/portal/',
      '../../sdkwork-router-admin/dist/': 'embedded-sites/admin/',
    },
  );
});

test('release target helpers and desktop release runner resolve explicit target triples', async () => {
  const helperPath = path.join(rootDir, 'scripts', 'release', 'desktop-targets.mjs');
  const runnerPath = path.join(rootDir, 'scripts', 'release', 'run-desktop-release-build.mjs');
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const workspaceTargetDirPath = path.join(rootDir, 'scripts', 'workspace-target-dir.mjs');

  assert.equal(existsSync(helperPath), true, 'missing scripts/release/desktop-targets.mjs');
  assert.equal(existsSync(runnerPath), true, 'missing scripts/release/run-desktop-release-build.mjs');
  assert.equal(existsSync(packagerPath), true, 'missing scripts/release/package-release-assets.mjs');

  const helper = await import(pathToFileURL(helperPath).href);
  const runner = await import(pathToFileURL(runnerPath).href);
  const packager = await import(pathToFileURL(packagerPath).href);
  const workspaceTargetDir = await import(pathToFileURL(workspaceTargetDirPath).href);

  assert.equal(typeof helper.parseDesktopTargetTriple, 'function');
  assert.equal(typeof helper.resolveDesktopReleaseTarget, 'function');
  assert.equal(typeof runner.createDesktopReleaseBuildPlan, 'function');
  assert.equal(typeof runner.buildDesktopReleaseFailureAnnotation, 'function');
  assert.equal(typeof runner.resolveDesktopReleaseBundles, 'function');
  assert.equal(typeof runner.shouldPassExplicitDesktopReleaseTarget, 'function');
  assert.equal(typeof packager.resolveNativeBuildRoot, 'function');
  assert.equal(typeof packager.resolveNativeBuildRootCandidates, 'function');
  assert.equal(typeof packager.listNativeServiceBinaryNames, 'function');
  assert.equal(typeof packager.listNativeDesktopAppIds, 'function');
  assert.equal(typeof packager.buildNativeProductServerArchiveBaseName, 'function');
  assert.equal(typeof packager.createTarCommandPlan, 'function');
  assert.equal(typeof workspaceTargetDir.resolveWorkspaceTargetDir, 'function');

  const expectedWorkspaceTargetDir = workspaceTargetDir.resolveWorkspaceTargetDir({
    workspaceRoot: rootDir,
    env: process.env,
    platform: process.platform,
  });

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
    arch: 'x64',
    env: {},
    targetTriple: 'aarch64-pc-windows-msvc',
  });

  assert.equal(portalBuildPlan.command, 'pnpm');
  assert.deepEqual(portalBuildPlan.args, [
    'tauri:build',
    '--target',
    'aarch64-pc-windows-msvc',
    '--bundles',
    'nsis',
  ]);
  assert.equal(portalBuildPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET, 'aarch64-pc-windows-msvc');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'windows');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'arm64');
  assert.deepEqual(
    runner.resolveDesktopReleaseBundles({
      platform: 'darwin',
    }),
    ['dmg'],
  );
  assert.equal(
    runner.shouldPassExplicitDesktopReleaseTarget({
      targetTriple: 'x86_64-unknown-linux-gnu',
      platform: 'linux',
      arch: 'x64',
    }),
    false,
  );
  assert.deepEqual(
    runner.createDesktopReleaseBuildPlan({
      appId: 'admin',
      appDir: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
      platform: 'linux',
      arch: 'x64',
      env: {
        GITHUB_ACTIONS: 'true',
      },
      targetTriple: 'x86_64-unknown-linux-gnu',
    }).args,
    [
      'tauri:build',
      '--bundles',
      'deb',
      '--verbose',
    ],
  );

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
  assert.deepEqual(
    packager.resolveNativeBuildRootCandidates({
      appId: 'admin',
      targetTriple: 'x86_64-pc-windows-msvc',
    }).map((entry) => entry.replaceAll('\\', '/')),
    [
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'target',
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'target',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        expectedWorkspaceTargetDir,
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        expectedWorkspaceTargetDir,
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'target',
        'sdkwork-router-admin-tauri',
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'target',
        'sdkwork-router-admin-tauri',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
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
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'src-tauri',
        'target',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
    ],
  );
  assert.match(
    packager.listNativeServiceBinaryNames().join(','),
    /router-product-service/,
  );
  assert.deepEqual(packager.listNativeDesktopAppIds(), ['admin', 'portal']);
  assert.equal(
    packager.buildNativeProductServerArchiveBaseName({
      platformId: 'linux',
      archId: 'arm64',
    }),
    'sdkwork-api-router-product-server-linux-arm64',
  );
  assert.deepEqual(
    packager.createTarCommandPlan({
      archivePath: 'C:\\release\\bundle.tar.gz',
      workingDirectory: 'C:\\release',
      entryName: 'bundle',
      platform: 'win32',
    }),
    {
      command: 'tar',
      args: ['--force-local', '-czf', 'C:\\release\\bundle.tar.gz', '-C', 'C:\\release', 'bundle'],
      shell: true,
    },
  );
  assert.equal(
    runner.buildDesktopReleaseFailureAnnotation({
      appId: 'admin',
      targetTriple: 'x86_64-unknown-linux-gnu',
      error: new Error('bundle missing 50%\nnext line'),
    }),
    '::error title=run-desktop-release-build::[admin x86_64-unknown-linux-gnu] bundle missing 50%25%0Anext line',
  );
  assert.match(
    runner.buildDesktopReleaseFailureAnnotation({
      appId: 'admin',
      targetTriple: 'x86_64-unknown-linux-gnu',
      error: new Error(`${'prefix-'.repeat(2000)}actual-final-error`),
    }),
    /actual-final-error/,
  );
});

test('native desktop packager skips empty bundle roots and selects the first root that contains artifacts', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.resolveAvailableNativeBuildRoot, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const emptyRoot = path.join(stagingRoot, 'candidate-empty');
    const populatedRoot = path.join(stagingRoot, 'candidate-populated');
    mkdirSync(emptyRoot, { recursive: true });
    mkdirSync(path.join(populatedRoot, 'nsis'), { recursive: true });
    writeFileSync(path.join(populatedRoot, 'nsis', 'sdkwork-router-portal.exe'), 'artifact', 'utf8');

    assert.equal(
      packager.resolveAvailableNativeBuildRoot({
        buildRoots: [emptyRoot, populatedRoot],
      }).replaceAll('\\', '/'),
      populatedRoot.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native desktop packager also accepts generic repository target bundle roots used by some CI layouts', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.resolveAvailableNativeBuildRoot, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const genericTargetRoot = path.join(stagingRoot, 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');
    const appLocalRoot = path.join(stagingRoot, 'apps', 'sdkwork-router-admin', 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');

    mkdirSync(path.join(genericTargetRoot, 'nsis'), { recursive: true });
    mkdirSync(appLocalRoot, { recursive: true });
    writeFileSync(path.join(genericTargetRoot, 'nsis', 'sdkwork-router-admin.exe'), 'artifact', 'utf8');

    assert.equal(
      packager.resolveAvailableNativeBuildRoot({
        buildRoots: [appLocalRoot, genericTargetRoot],
      }).replaceAll('\\', '/'),
      genericTargetRoot.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native desktop packager also accepts app-root target bundle roots used by tauri v2 project layouts', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.resolveAvailableNativeBuildRoot, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const appRootTarget = path.join(stagingRoot, 'apps', 'sdkwork-router-admin', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');
    const srcTauriTarget = path.join(stagingRoot, 'apps', 'sdkwork-router-admin', 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');

    mkdirSync(path.join(appRootTarget, 'nsis'), { recursive: true });
    mkdirSync(srcTauriTarget, { recursive: true });
    writeFileSync(path.join(appRootTarget, 'nsis', 'sdkwork-router-admin.exe'), 'artifact', 'utf8');

    assert.equal(
      packager.resolveAvailableNativeBuildRoot({
        buildRoots: [srcTauriTarget, appRootTarget],
      }).replaceAll('\\', '/'),
      appRootTarget.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native release packager exposes GitHub annotation-safe error formatting for CI failures', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.buildGitHubActionsErrorAnnotation, 'function');
  assert.equal(
    packager.buildGitHubActionsErrorAnnotation({
      title: 'package:release,assets',
      error: new Error('bundle missing 50%\nnext line\rfinal line'),
    }),
    '::error title=package%3Arelease%2Cassets::bundle missing 50%25%0Anext line%0Dfinal line',
  );
});
