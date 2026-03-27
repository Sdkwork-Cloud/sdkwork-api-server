#!/usr/bin/env node

import { spawn } from 'node:child_process';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  buildDesktopReleaseEnv,
  DESKTOP_TARGET_ENV_VAR,
} from './release/desktop-targets.mjs';

const __filename = fileURLToPath(import.meta.url);

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

  const requestedTargetTriple = extractTargetTriple(args, env);
  const resolvedEnv = requestedTargetTriple
    ? buildDesktopReleaseEnv({
        env,
        targetTriple: requestedTargetTriple,
      })
    : { ...env };

  return {
    command: platform === 'win32' ? 'tauri.cmd' : 'tauri',
    args: [commandName, ...args],
    cwd,
    env: withSupportedWindowsCmakeGenerator(resolvedEnv, platform),
    shell: platform === 'win32',
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
    stdio: 'inherit',
    shell: plan.shell,
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
}

if (__filename === process.argv[1]) {
  runCli();
}
