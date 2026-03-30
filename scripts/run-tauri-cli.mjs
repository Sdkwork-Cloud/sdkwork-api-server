#!/usr/bin/env node

import fs from 'node:fs';
import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  buildDesktopReleaseEnv,
  DESKTOP_TARGET_ENV_VAR,
} from './release/desktop-targets.mjs';

const __filename = fileURLToPath(import.meta.url);
const BACKGROUND_LAUNCH_ENV = 'SDKWORK_ROUTER_BACKGROUND';

function normalizeCliArgs(args = []) {
  return args.filter((arg) => arg !== '--');
}

function shouldLaunchInBackground(commandName, args = [], env = process.env) {
  if (commandName !== 'dev') {
    return false;
  }

  if (String(env[BACKGROUND_LAUNCH_ENV] ?? '').trim() === '1') {
    return true;
  }

  return normalizeCliArgs(args).some((arg) => arg === '--service' || arg === '--start-hidden');
}

export function withSupportedWindowsCmakeGenerator(
  baseEnv = process.env,
  platform = process.platform,
) {
  const env = { ...baseEnv };
  if (platform !== 'win32') {
    return env;
  }

  const requestedGenerator = String(env.CMAKE_GENERATOR ?? '').trim();
  if (requestedGenerator.length > 0 && !requestedGenerator.includes('2026')) {
    return env;
  }

  env.CMAKE_GENERATOR = 'Visual Studio 17 2022';
  env.HOST_CMAKE_GENERATOR = 'Visual Studio 17 2022';
  return env;
}

function resolveCargoBinDir(baseEnv = process.env, platform = process.platform) {
  if (platform === 'win32') {
    const cargoHome = String(baseEnv.CARGO_HOME ?? '').trim()
      || (baseEnv.USERPROFILE ? path.join(baseEnv.USERPROFILE, '.cargo') : '');
    return cargoHome ? path.join(cargoHome, 'bin') : null;
  }

  const home = String(baseEnv.HOME ?? '').trim();
  return home ? path.join(home, '.cargo', 'bin') : null;
}

function withCargoToolchainOnPath(baseEnv = process.env, platform = process.platform) {
  const env = { ...baseEnv };
  const cargoBinDir = resolveCargoBinDir(baseEnv, platform);
  if (!cargoBinDir || !fs.existsSync(cargoBinDir)) {
    return env;
  }

  const currentPath = String(env.PATH ?? env.Path ?? '').trim();
  const pathEntries = currentPath ? currentPath.split(path.delimiter) : [];
  if (!pathEntries.some((entry) => entry.toLowerCase() === cargoBinDir.toLowerCase())) {
    const joinedPath = [cargoBinDir, ...pathEntries].filter(Boolean).join(path.delimiter);
    env.PATH = joinedPath;
    env.Path = joinedPath;
  }

  return env;
}

function resolveWindowsCargoTargetDir(baseEnv = process.env, cwd = process.cwd()) {
  const existingTargetDir = String(baseEnv.CARGO_TARGET_DIR ?? '').trim();
  if (existingTargetDir) {
    return null;
  }

  const tempRoot = String(baseEnv.TEMP ?? baseEnv.TMP ?? '').trim()
    || (baseEnv.USERPROFILE ? path.join(baseEnv.USERPROFILE, 'AppData', 'Local', 'Temp') : '');
  if (!tempRoot || !fs.existsSync(tempRoot)) {
    return null;
  }

  const appName = path.basename(cwd).trim().toLowerCase();
  return path.join(tempRoot, 'sdkwork-tauri-target', appName || 'app');
}

function extractTargetTriple(args, env = process.env) {
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === '--target') {
      return String(args[index + 1] ?? '').trim();
    }
  }

  return String(env?.[DESKTOP_TARGET_ENV_VAR] ?? '').trim();
}

export function createTauriCliPlan({
  commandName,
  args = [],
  cwd = process.cwd(),
  env = process.env,
  platform = process.platform,
} = {}) {
  if (typeof commandName !== 'string' || commandName.trim().length === 0) {
    throw new Error('commandName is required.');
  }

  const background = shouldLaunchInBackground(commandName, args, env);
  const requestedTargetTriple = extractTargetTriple(args, env);
  const resolvedEnv = requestedTargetTriple
    ? buildDesktopReleaseEnv({
        env,
        targetTriple: requestedTargetTriple,
      })
    : { ...env };
  if (platform === 'win32') {
    const shortTargetDir = resolveWindowsCargoTargetDir(resolvedEnv, cwd);
    if (shortTargetDir) {
      resolvedEnv.CARGO_TARGET_DIR = shortTargetDir;
    }
  }

  return {
    command: platform === 'win32' ? 'tauri.cmd' : 'tauri',
    args: [commandName, ...args],
    cwd,
    env: withSupportedWindowsCmakeGenerator(
      withCargoToolchainOnPath(resolvedEnv, platform),
      platform,
    ),
    shell: platform === 'win32',
    detached: background,
    windowsHide: platform === 'win32',
  };
}

function runCli() {
  const [commandName, ...args] = process.argv.slice(2);
  const plan = createTauriCliPlan({
    commandName,
    args,
  });
  const child = spawn(plan.command, plan.args, {
    cwd: plan.cwd,
    env: plan.env,
    stdio: plan.detached ? 'ignore' : 'inherit',
    detached: plan.detached ?? false,
    shell: plan.shell,
    windowsHide: plan.windowsHide ?? process.platform === 'win32',
  });

  child.on('error', (error) => {
    console.error(`[run-tauri-cli] ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      console.error(`[run-tauri-cli] command exited with signal ${signal}`);
      process.exit(1);
    }

    process.exit(code ?? 0);
  });

  if (plan.detached) {
    child.unref();
    process.exit(0);
  }
}

if (__filename === process.argv[1]) {
  runCli();
}
