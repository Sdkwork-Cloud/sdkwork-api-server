#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { withSupportedWindowsCmakeGenerator } from './run-tauri-cli.mjs';
import {
  ensureFrontendDependenciesReady,
  frontendViteConfigHealthy,
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

export function createProductCheckPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  portalAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  adminAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
  platform = process.platform,
  env = process.env,
} = {}) {
  const nodeCommand = process.execPath;
  const baseEnv = withSupportedWindowsCmakeGenerator(env, platform);
  const rustRunner = resolveRustRunner(platform, env);
  const cargoArgs = [...rustRunner.args, 'check'];

  if (env.SDKWORK_ROUTER_VERBOSE_CARGO !== '1') {
    cargoArgs.push('--quiet');
  }

  cargoArgs.push('-p', 'router-product-service');

  return [
    {
      ...pnpmProcessSpec(['typecheck'], { platform }),
      label: 'portal typecheck',
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
      ...pnpmProcessSpec(['typecheck'], { platform }),
      label: 'admin typecheck',
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
      label: 'docs bootstrap safety',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'check-router-docs-safety.mjs')],
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
      label: 'desktop assets build',
      command: nodeCommand,
      args: [path.join(workspaceRoot, 'scripts', 'build-router-desktop-assets.mjs')],
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
  const plan = createProductCheckPlan();
  for (const appRoot of [
    path.join(__dirname, '..', 'apps', 'sdkwork-router-portal'),
    path.join(__dirname, '..', 'apps', 'sdkwork-router-admin'),
  ]) {
    ensureFrontendDependenciesReady({
      appRoot,
      requiredPackages: ['vite', 'typescript'],
      requiredBinCommands: ['vite', 'tsc'],
      verifyInstalled: () => frontendViteConfigHealthy({
        appRoot,
        command: 'build',
      }),
    });
  }

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
