#!/usr/bin/env node

import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { assertFrontendBudgets } from './check-router-frontend-budgets.mjs';
import {
  checkFrontendViteConfig,
  ensureFrontendDependenciesReady,
} from './dev/pnpm-launch-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

export function createDesktopAssetBuildPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
} = {}) {
  const nodeCommand = process.execPath;
  const appRoots = [
    path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  ];

  return appRoots.map((cwd) => ({
    cwd,
    command: nodeCommand,
    args: [
      path.join(workspaceRoot, 'scripts', 'dev', 'run-vite-cli.mjs'),
      'build',
    ],
    shell: false,
    windowsHide: platform === 'win32',
  }));
}

async function runBuild(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      stdio: 'inherit',
      shell: step.shell,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`build in ${step.cwd} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`build in ${step.cwd} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

export async function runPostBuildChecks({
  workspaceRoot = path.resolve(__dirname, '..'),
} = {}) {
  return assertFrontendBudgets({
    workspaceRoot,
  });
}

async function main() {
  const plan = createDesktopAssetBuildPlan();
  for (const step of plan) {
    ensureFrontendDependenciesReady({
      appRoot: step.cwd,
      requiredPackages: ['vite', 'typescript'],
      requiredBinCommands: ['vite', 'tsc'],
      verifyInstalled: () => checkFrontendViteConfig({
        appRoot: step.cwd,
        command: 'build',
      }),
    });
  }

  for (const step of plan) {
    // eslint-disable-next-line no-await-in-loop
    await runBuild(step);
  }

  await runPostBuildChecks();
}

if (__filename === process.argv[1]) {
  main().catch((error) => {
    console.error(`[build-router-desktop-assets] ${error.message}`);
    process.exit(1);
  });
}
