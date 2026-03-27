#!/usr/bin/env node

import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

function pnpmCommand(platform = process.platform) {
  return platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

export function createDesktopAssetBuildPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
} = {}) {
  const appRoots = [
    path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  ];

  return appRoots.map((cwd) => ({
    cwd,
    command: pnpmCommand(platform),
    args: ['build'],
    shell: platform === 'win32',
  }));
}

async function runBuild(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      stdio: 'inherit',
      shell: step.shell,
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

async function main() {
  for (const step of createDesktopAssetBuildPlan()) {
    // eslint-disable-next-line no-await-in-loop
    await runBuild(step);
  }
}

if (__filename === process.argv[1]) {
  main().catch((error) => {
    console.error(`[build-router-desktop-assets] ${error.message}`);
    process.exit(1);
  });
}
