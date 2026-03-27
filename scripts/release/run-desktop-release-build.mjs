#!/usr/bin/env node

import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { buildDesktopReleaseEnv } from './desktop-targets.mjs';
import { withSupportedWindowsCmakeGenerator } from '../run-tauri-cli.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DESKTOP_APP_DIRS = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
  console: path.join(rootDir, 'console'),
};

export function resolveDesktopAppDir(appId) {
  const appDir = DESKTOP_APP_DIRS[appId];
  if (!appDir) {
    throw new Error(`Unsupported desktop application id: ${appId}`);
  }

  return appDir;
}

export function createDesktopReleaseBuildPlan({
  appId,
  appDir = resolveDesktopAppDir(appId),
  platform = process.platform,
  env = process.env,
  targetTriple = '',
} = {}) {
  const requestedTargetTriple = String(targetTriple ?? '').trim();
  const args = ['tauri:build'];
  const resolvedEnv = requestedTargetTriple
    ? buildDesktopReleaseEnv({
        env,
        targetTriple: requestedTargetTriple,
      })
    : { ...env };

  if (requestedTargetTriple) {
    args.push('--', '--target', requestedTargetTriple);
  }

  return {
    command: 'pnpm',
    args,
    cwd: appDir,
    env: withSupportedWindowsCmakeGenerator(resolvedEnv, platform),
  };
}

function parseCliArgs(argv) {
  const options = {
    appId: '',
    targetTriple: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--app') {
      options.appId = String(next ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--target') {
      options.targetTriple = String(next ?? '').trim();
      index += 1;
    }
  }

  return options;
}

function runCli() {
  const options = parseCliArgs(process.argv.slice(2));
  const plan = createDesktopReleaseBuildPlan({
    appId: options.appId,
    targetTriple: options.targetTriple,
  });
  const child = spawn(plan.command, plan.args, {
    cwd: plan.cwd,
    env: plan.env,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  child.on('error', (error) => {
    console.error(`[run-desktop-release-build] ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      console.error(`[run-desktop-release-build] build exited with signal ${signal}`);
      process.exit(1);
    }

    process.exit(code ?? 0);
  });
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  runCli();
}
