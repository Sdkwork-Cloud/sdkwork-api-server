import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, utimesSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  ensureFrontendDependenciesReady,
  frontendDistReady,
  frontendDistUpToDate,
  frontendInstallRequired,
  frontendInstallStatus,
  pnpmArgumentPrefix,
  pnpmDisplayCommand,
  pnpmExecutable,
  pnpmProcessSpec,
  pnpmSpawnOptions,
  strictFrontendInstallsEnabled,
  shouldReuseExistingFrontendDist,
} from '../pnpm-launch-lib.mjs';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function withTempApp(callback) {
  const tempRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-pnpm-'));
  const appRoot = path.join(tempRoot, 'app');
  mkdirSync(appRoot, { recursive: true });

  try {
    callback(appRoot);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function setFileTimestamp(filePath, timestampSeconds) {
  utimesSync(filePath, timestampSeconds, timestampSeconds);
}

test('pnpmExecutable uses node.exe on Windows so pnpm can run without cmd.exe shelling', () => {
  assert.equal(pnpmExecutable('win32', 'C:/Tools/node.exe'), 'C:/Tools/node.exe');
  assert.equal(pnpmExecutable('linux'), 'pnpm');
  assert.equal(pnpmExecutable('darwin'), 'pnpm');
});

test('pnpmArgumentPrefix resolves the bundled pnpm.cjs script on Windows', () => {
  assert.deepEqual(
    pnpmArgumentPrefix({
      platform: 'win32',
      execPath: 'C:/Tools/node.exe',
    }),
    [path.join('C:/Tools', 'node_modules', 'pnpm', 'bin', 'pnpm.cjs')],
  );
  assert.deepEqual(
    pnpmArgumentPrefix({
      platform: 'linux',
      execPath: '/usr/bin/node',
    }),
    [],
  );
});

test('pnpmProcessSpec wraps Windows pnpm invocations in a hidden PowerShell command', () => {
  const processSpec = pnpmProcessSpec(['--dir', 'apps/sdkwork-router-admin', 'install'], {
    platform: 'win32',
    execPath: 'C:/Tools/node.exe',
  });

  assert.equal(processSpec.command, 'powershell.exe');
  assert.deepEqual(processSpec.args.slice(0, 4), [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
  ]);
  assert.match(processSpec.args[4], /node\.exe/);
  assert.match(processSpec.args[4], /pnpm\.cjs/);
  assert.match(processSpec.args[4], /sdkwork-router-admin/);
});

test('pnpmDisplayCommand shows the direct node-plus-pnpm command used on Windows', () => {
  const command = pnpmDisplayCommand(['build'], {
    platform: 'win32',
    execPath: 'C:/Tools/node.exe',
  });

  assert.match(command, /^C:\/Tools\/node\.exe /);
  assert.match(command, /node_modules[\\/]+pnpm[\\/]+bin[\\/]+pnpm\.cjs/);
  assert.match(command, / build$/);
});

test('pnpmSpawnOptions keeps Windows launches direct and hidden without opening another shell window', () => {
  const env = { PATH: 'C:/pnpm' };
  const options = pnpmSpawnOptions({
    platform: 'win32',
    env,
    execPath: 'C:/Tools/node.exe',
    cwd: 'D:/workspace/sdkwork-api-router',
  });

  assert.equal(options.cwd, 'D:/workspace/sdkwork-api-router');
  assert.equal(options.shell, false);
  assert.equal(options.stdio, 'inherit');
  assert.equal(options.windowsHide, true);
  assert.match(String(options.env.PATH ?? '').replaceAll('\\', '/'), /^C:\/Tools;C:\/pnpm$/);
  assert.match(options.env.NODE_OPTIONS, /vite-windows-realpath-preload\.mjs/);
  assert.match(options.env.NODE_OPTIONS, /--import=/);
});

test('pnpmSpawnOptions keeps non-Windows launches direct and foreground-safe', () => {
  const env = { PATH: '/usr/bin' };
  const options = pnpmSpawnOptions({
    platform: 'linux',
    env,
  });

  assert.deepEqual(options, {
    env,
    shell: false,
    stdio: 'inherit',
    windowsHide: false,
  });
});

test('pnpmSpawnOptions preserves existing NODE_OPTIONS while adding the Vite preload once on Windows', () => {
  const options = pnpmSpawnOptions({
    platform: 'win32',
    env: {
      NODE_OPTIONS: '--max-old-space-size=4096',
      PATH: 'C:/pnpm',
    },
    execPath: 'C:/Tools/node.exe',
  });

  assert.match(String(options.env.PATH ?? '').replaceAll('\\', '/'), /^C:\/Tools;C:\/pnpm$/);
  assert.match(options.env.NODE_OPTIONS, /--max-old-space-size=4096/);
  assert.match(options.env.NODE_OPTIONS, /vite-windows-realpath-preload\.mjs/);
  assert.equal(
    options.env.NODE_OPTIONS.match(/vite-windows-realpath-preload\.mjs/g)?.length,
    1,
  );
});

test('pnpmSpawnOptions prepends the Node executable directory to PATH on Windows', () => {
  const options = pnpmSpawnOptions({
    platform: 'win32',
    env: {
      PATH: 'C:/pnpm;C:/Windows/System32',
    },
    execPath: 'C:/Tools/node.exe',
  });

  const normalizedPath = String(options.env.PATH ?? '').replaceAll('\\', '/');
  assert.match(normalizedPath, /^C:\/Tools;C:\/pnpm;C:\/Windows\/System32$/);
});

test('dev launchers use the shared pnpm helper for Windows-safe process spawning', () => {
  const scriptPaths = [
    path.join(repoRoot, 'scripts', 'dev', 'start-admin.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-portal.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-web.mjs'),
  ];

  for (const scriptPath of scriptPaths) {
    const script = readFileSync(scriptPath, 'utf8');
    assert.match(script, /pnpm-launch-lib\.mjs/);
    assert.match(script, /pnpmSpawnOptions/);
  }
});

test('frontend launchers repair installs in place without deleting node_modules first', () => {
  const repairScriptPaths = [
    path.join(repoRoot, 'scripts', 'dev', 'start-admin.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-portal.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-web.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-console.mjs'),
  ];

  for (const scriptPath of repairScriptPaths) {
    const script = readFileSync(scriptPath, 'utf8');
    assert.doesNotMatch(script, /removeFrontendNodeModules/);
    assert.match(script, /frontendInstallStatus/);
    assert.match(script, /['"]install['"]/);
  }

  for (const scriptPath of [
    path.join(repoRoot, 'scripts', 'build-router-desktop-assets.mjs'),
    path.join(repoRoot, 'scripts', 'check-router-product.mjs'),
  ]) {
    const script = readFileSync(scriptPath, 'utf8');
    assert.doesNotMatch(script, /removeFrontendNodeModules/);
    assert.match(script, /ensureFrontendDependenciesReady/);
  }
});

test('strictFrontendInstallsEnabled only enables strict mode for explicit truthy env values', () => {
  assert.equal(strictFrontendInstallsEnabled({}), false);
  assert.equal(strictFrontendInstallsEnabled({ SDKWORK_STRICT_FRONTEND_INSTALLS: '' }), false);
  assert.equal(strictFrontendInstallsEnabled({ SDKWORK_STRICT_FRONTEND_INSTALLS: '0' }), false);
  assert.equal(strictFrontendInstallsEnabled({ SDKWORK_STRICT_FRONTEND_INSTALLS: 'false' }), false);
  assert.equal(strictFrontendInstallsEnabled({ SDKWORK_STRICT_FRONTEND_INSTALLS: '1' }), true);
  assert.equal(strictFrontendInstallsEnabled({ SDKWORK_STRICT_FRONTEND_INSTALLS: 'true' }), true);
  assert.equal(strictFrontendInstallsEnabled({ SDKWORK_STRICT_FRONTEND_INSTALLS: 'TRUE' }), true);
});

test('ensureFrontendDependenciesReady refuses to auto-install when strict mode is enabled', () => {
  withTempApp((appRoot) => {
    let installAttempted = false;

    assert.throws(
      () => ensureFrontendDependenciesReady({
        appRoot,
        env: {
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1',
        },
        requiredPackages: ['vite'],
        requiredBinCommands: ['vite'],
        verifyInstalled: () => false,
        spawnInstall: () => {
          installAttempted = true;
          return { status: 0 };
        },
      }),
      /strict frontend install mode requires a prior frozen install/i,
    );

    assert.equal(installAttempted, false);
  });
});

test('ensureFrontendDependenciesReady surfaces unhealthy verification details in strict mode errors', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');
    const typescriptRoot = path.join(nodeModulesRoot, 'typescript');
    const binRoot = path.join(nodeModulesRoot, '.bin');

    mkdirSync(viteRoot, { recursive: true });
    mkdirSync(typescriptRoot, { recursive: true });
    mkdirSync(binRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');
    writeFileSync(path.join(typescriptRoot, 'package.json'), '{"name":"typescript"}\n');
    writeFileSync(path.join(binRoot, 'vite'), '#!/usr/bin/env node\n');
    writeFileSync(path.join(binRoot, 'tsc'), '#!/usr/bin/env node\n');

    assert.throws(
      () => ensureFrontendDependenciesReady({
        appRoot,
        platform: 'linux',
        env: {
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1',
        },
        requiredPackages: ['vite', 'typescript'],
        requiredBinCommands: ['vite', 'tsc'],
        verifyInstalled: () => ({
          ok: false,
          reason: 'vite config failed to load',
          stderr: 'Error: missing aliased module',
        }),
      }),
      /vite config failed to load[\s\S]*missing aliased module/i,
    );
  });
});

test('preview installs can reuse existing dist on Windows spawn EPERM when explicitly allowed', () => {
  assert.equal(
    shouldReuseExistingFrontendDist({
      platform: 'win32',
      stepArgs: ['--dir', 'apps/sdkwork-router-admin', 'install'],
      status: 1,
      errorMessage: 'spawnSync powershell.exe EPERM',
      distReady: true,
      allowInstallReuse: true,
    }),
    true,
  );
  assert.equal(
    shouldReuseExistingFrontendDist({
      platform: 'win32',
      stepArgs: ['--dir', 'apps/sdkwork-router-admin', 'install'],
      status: 1,
      errorMessage: 'spawnSync powershell.exe EPERM',
      distReady: true,
      allowInstallReuse: false,
    }),
    false,
  );
});

test('frontendDistReady only accepts built static sites with an index.html entrypoint', () => {
  withTempApp((appRoot) => {
    const distRoot = path.join(appRoot, 'dist');
    mkdirSync(distRoot, { recursive: true });

    assert.equal(frontendDistReady(distRoot), false);

    writeFileSync(path.join(distRoot, 'index.html'), '<!doctype html>');
    assert.equal(frontendDistReady(distRoot), true);
  });
});

test('frontendDistUpToDate only allows preview build reuse when dist is newer than the tracked frontend inputs', () => {
  withTempApp((appRoot) => {
    const srcRoot = path.join(appRoot, 'src');
    const packagesRoot = path.join(appRoot, 'packages');
    const distRoot = path.join(appRoot, 'dist');
    const sourceFile = path.join(srcRoot, 'main.ts');
    const packageFile = path.join(packagesRoot, 'feature.ts');
    const distEntry = path.join(distRoot, 'index.html');

    mkdirSync(srcRoot, { recursive: true });
    mkdirSync(packagesRoot, { recursive: true });
    mkdirSync(distRoot, { recursive: true });
    writeFileSync(sourceFile, 'export const value = 1;\n');
    writeFileSync(packageFile, 'export const feature = true;\n');
    writeFileSync(distEntry, '<!doctype html>\n');

    setFileTimestamp(sourceFile, 1000);
    setFileTimestamp(packageFile, 1005);
    setFileTimestamp(distEntry, 1010);

    assert.equal(
      frontendDistUpToDate({
        appRoot,
        distDir: distRoot,
        buildInputs: ['src', 'packages'],
      }),
      true,
    );

    setFileTimestamp(sourceFile, 1020);

    assert.equal(
      frontendDistUpToDate({
        appRoot,
        distDir: distRoot,
        buildInputs: ['src', 'packages'],
      }),
      false,
    );
  });
});

test('build fallback refuses to reuse empty dist directories on Windows spawn EPERM', () => {
  assert.equal(
    shouldReuseExistingFrontendDist({
      platform: 'win32',
      stepArgs: ['--dir', 'apps/sdkwork-router-portal', 'build'],
      status: 1,
      errorMessage: 'spawnSync powershell.exe EPERM',
      distReady: false,
    }),
    false,
  );
});

test('frontendInstallRequired returns true when node_modules is missing', () => {
  withTempApp((appRoot) => {
    assert.equal(frontendInstallRequired({ appRoot, requiredPackages: ['vite'] }), true);
  });
});

test('frontendInstallRequired returns true when pnpm metadata is missing even if node_modules exists', () => {
  withTempApp((appRoot) => {
    mkdirSync(path.join(appRoot, 'node_modules'), { recursive: true });

    assert.equal(frontendInstallRequired({ appRoot, requiredPackages: ['vite'] }), true);
  });
});

test('frontendInstallRequired returns true when required frontend packages are missing', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    mkdirSync(nodeModulesRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');

    assert.equal(frontendInstallRequired({ appRoot, requiredPackages: ['vite'] }), true);
  });
});

test('frontendInstallRequired returns false when pnpm metadata and required packages exist', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');
    const tauriCliRoot = path.join(nodeModulesRoot, '@tauri-apps', 'cli');

    mkdirSync(viteRoot, { recursive: true });
    mkdirSync(tauriCliRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');
    writeFileSync(path.join(tauriCliRoot, 'package.json'), '{"name":"@tauri-apps/cli"}\n');

    assert.equal(
      frontendInstallRequired({
        appRoot,
        requiredPackages: ['vite', '@tauri-apps/cli'],
      }),
      false,
    );
  });
});

test('frontendInstallStatus returns unhealthy when required packages exist but the toolchain check fails', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');

    mkdirSync(viteRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');

    assert.equal(
      frontendInstallStatus({
        appRoot,
        requiredPackages: ['vite'],
        verifyInstalled: () => false,
      }),
      'unhealthy',
    );
  });
});

test('frontendInstallStatus returns ready when required packages exist and the toolchain check succeeds', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');
    const binRoot = path.join(nodeModulesRoot, '.bin');

    mkdirSync(viteRoot, { recursive: true });
    mkdirSync(binRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');
    writeFileSync(path.join(binRoot, 'vite'), '#!/usr/bin/env node\n');

    assert.equal(
      frontendInstallStatus({
        appRoot,
        platform: 'linux',
        requiredPackages: ['vite'],
        requiredBinCommands: ['vite'],
        verifyInstalled: () => true,
      }),
      'ready',
    );
  });
});

test('frontendInstallStatus treats unix pnpm bin symlinks as healthy command shims', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');
    const viteBinRoot = path.join(viteRoot, 'bin');
    const viteBinEntry = path.join(viteBinRoot, 'vite.js');
    const binRoot = path.join(nodeModulesRoot, '.bin');
    const viteCommandLink = path.join(binRoot, 'vite');

    mkdirSync(viteBinRoot, { recursive: true });
    mkdirSync(binRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');
    writeFileSync(viteBinEntry, '#!/usr/bin/env node\n');
    symlinkSync(viteBinEntry, viteCommandLink);

    assert.equal(
      frontendInstallStatus({
        appRoot,
        platform: 'linux',
        requiredPackages: ['vite'],
        requiredBinCommands: ['vite'],
        verifyInstalled: () => true,
      }),
      'ready',
    );
  });
});

test('frontendInstallStatus returns unhealthy on Windows when required command shims are missing', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');
    const binRoot = path.join(nodeModulesRoot, '.bin');

    mkdirSync(viteRoot, { recursive: true });
    mkdirSync(binRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');
    writeFileSync(path.join(binRoot, 'vite'), '#!/usr/bin/env node\n');

    assert.equal(
      frontendInstallStatus({
        appRoot,
        platform: 'win32',
        requiredPackages: ['vite'],
        requiredBinCommands: ['vite'],
      }),
      'unhealthy',
    );
  });
});

test('frontendInstallStatus accepts Windows command shims case-insensitively', () => {
  withTempApp((appRoot) => {
    const nodeModulesRoot = path.join(appRoot, 'node_modules');
    const viteRoot = path.join(nodeModulesRoot, 'vite');
    const binRoot = path.join(nodeModulesRoot, '.bin');

    mkdirSync(viteRoot, { recursive: true });
    mkdirSync(binRoot, { recursive: true });
    writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n');
    writeFileSync(path.join(viteRoot, 'package.json'), '{"name":"vite"}\n');
    writeFileSync(path.join(binRoot, 'vite'), '#!/usr/bin/env node\n');
    writeFileSync(path.join(binRoot, 'vite.CMD'), '@echo off\r\n');

    assert.equal(
      frontendInstallStatus({
        appRoot,
        platform: 'win32',
        requiredPackages: ['vite'],
        requiredBinCommands: ['vite'],
      }),
      'ready',
    );
  });
});
