#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { withSupportedWindowsCmakeGenerator } from './run-tauri-cli.mjs';
import { withManagedWorkspaceTargetDir, withManagedWorkspaceTempDir } from './workspace-target-dir.mjs';
import {
  checkFrontendViteConfig,
  ensureFrontendDependenciesReady,
  pnpmExecutable,
  pnpmProcessSpec,
} from './dev/pnpm-launch-lib.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export function pnpmCommand(platform = process.platform) {
  return pnpmExecutable(platform);
}

export function resolveRustRunner(platform = process.platform, env = process.env) {
  if (platform === 'win32') {
    const rustupPath = path.join(env.USERPROFILE ?? '', '.cargo', 'bin', 'rustup.exe');
    if (existsSync(rustupPath)) {
      return {
        command: rustupPath,
        args: ['run', 'stable', 'cargo'],
        shell: false,
      };
    }

    return {
      command: 'rustup.exe',
      args: ['run', 'stable', 'cargo'],
      shell: true,
    };
  }

  return {
    command: 'rustup',
    args: ['run', 'stable', 'cargo'],
    shell: false,
  };
}

export function productCheckBaseEnv({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  env = process.env,
} = {}) {
  const managedEnv = withManagedWorkspaceTargetDir({
    workspaceRoot,
    env: withManagedWorkspaceTempDir({
      workspaceRoot,
      env: withSupportedWindowsCmakeGenerator(env, platform),
      platform,
    }),
    platform,
  });

  return {
    ...managedEnv,
    CARGO_TARGET_DIR: managedEnv.CARGO_TARGET_DIR ?? path.join(workspaceRoot, 'target'),
  };
}

export function createProductCheckPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  portalAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  adminAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
  docsDir = path.join(workspaceRoot, 'docs'),
  platform = process.platform,
  env = process.env,
} = {}) {
  const nodeCommand = process.execPath;
  const baseEnv = productCheckBaseEnv({
    workspaceRoot,
    platform,
    env,
  });
  const rustRunner = resolveRustRunner(platform, baseEnv);
  const docsBuildProcess = pnpmProcessSpec(['--dir', 'docs', 'build'], {
    platform,
    execPath: nodeCommand,
  });
  const cargoArgs = [...rustRunner.args, 'check'];

  if (env.SDKWORK_ROUTER_VERBOSE_CARGO !== '1') {
    cargoArgs.push('--quiet');
  }

  cargoArgs.push('-p', 'router-product-service');
  const tscCliScript = path.join(workspaceRoot, 'scripts', 'dev', 'run-tsc-cli.mjs');

  return [
    {
      label: 'portal typecheck',
      command: nodeCommand,
      args: [tscCliScript, '--noEmit'],
      cwd: portalAppDir,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'portal regression tests',
      command: nodeCommand,
      args: ['--test', 'tests/*.mjs'],
      cwd: portalAppDir,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'portal browser runtime smoke',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'check-portal-browser-runtime.mjs')],
      cwd: workspaceRoot,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'admin typecheck',
      command: nodeCommand,
      args: [tscCliScript, '--noEmit'],
      cwd: adminAppDir,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'admin regression tests',
      command: nodeCommand,
      args: ['--test', 'tests/*.mjs'],
      cwd: adminAppDir,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'admin browser runtime smoke',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'check-admin-browser-runtime.mjs')],
      cwd: workspaceRoot,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'docs bootstrap safety',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'check-router-docs-safety.mjs')],
      cwd: workspaceRoot,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'docs site build',
      command: docsBuildProcess.command,
      args: docsBuildProcess.args,
      cwd: workspaceRoot,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'workspace dependency audit',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'check-rust-dependency-audit.mjs')],
      cwd: workspaceRoot,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'portal desktop runtime payload',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'prepare-router-portal-desktop-runtime.mjs')],
      cwd: workspaceRoot,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'server cargo check',
      command: rustRunner.command,
      args: cargoArgs,
      cwd: workspaceRoot,
      env: baseEnv,
      shell: rustRunner.shell,
      windowsHide: platform === 'win32',
    },
    {
      label: 'server deployment plan',
      command: nodeCommand,
      args: [
        path.join(workspaceRoot, 'scripts', 'run-router-product-service.mjs'),
        '--dry-run',
        '--plan-format',
        'json',
        '--bind',
        '127.0.0.1:3001',
      ],
      cwd: portalAppDir,
      env: baseEnv,
      shell: false,
      windowsHide: platform === 'win32',
    },
  ];
}

async function runStep(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell ?? false,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`${step.label} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`${step.label} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

async function main() {
  const workspaceRoot = path.resolve(__dirname, '..');
  const baseEnv = productCheckBaseEnv({
    workspaceRoot,
    platform: process.platform,
    env: process.env,
  });
  const plan = createProductCheckPlan({
    workspaceRoot,
    platform: process.platform,
    env: baseEnv,
  });
  for (const appRoot of [
    path.join(__dirname, '..', 'apps', 'sdkwork-router-portal'),
    path.join(__dirname, '..', 'apps', 'sdkwork-router-admin'),
  ]) {
    ensureFrontendDependenciesReady({
      appRoot,
      requiredPackages: ['vite', 'typescript'],
      requiredBinCommands: ['vite', 'tsc'],
      verifyInstalled: () => checkFrontendViteConfig({
        appRoot,
        command: 'build',
      }),
      platform: process.platform,
      env: baseEnv,
    });
  }
  ensureFrontendDependenciesReady({
    appRoot: path.join(__dirname, '..', 'docs'),
    requiredPackages: ['vitepress'],
    requiredBinCommands: ['vitepress'],
    platform: process.platform,
    env: baseEnv,
  });

  for (const step of plan) {
    console.error(`[check-router-product] ${step.label}`);
    // eslint-disable-next-line no-await-in-loop
    await runStep(step);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(`[check-router-product] ${error.message}`);
    process.exit(1);
  });
}
