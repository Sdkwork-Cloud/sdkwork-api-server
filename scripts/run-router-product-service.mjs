#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { withSupportedWindowsCmakeGenerator } from './run-tauri-cli.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..');

function resolveRustRunner(platform = process.platform, env = process.env) {
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

export function normalizeRouterProductServiceArgs(argv = []) {
  return argv.filter((arg) => arg !== '--');
}

export function createRouterProductServiceRunPlan({
  argv = process.argv.slice(2),
  platform = process.platform,
  env = process.env,
} = {}) {
  const runner = resolveRustRunner(platform, env);
  const cargoArgs = [...runner.args, 'run'];

  if (env.SDKWORK_ROUTER_VERBOSE_CARGO !== '1') {
    cargoArgs.push('--quiet');
  }

  cargoArgs.push(
    '-p',
    'router-product-service',
    '--',
    ...normalizeRouterProductServiceArgs(argv),
  );

  return {
    command: runner.command,
    args: cargoArgs,
    shell: runner.shell,
  };
}

function runCli() {
  const plan = createRouterProductServiceRunPlan();
  const child = spawn(plan.command, plan.args, {
    cwd: workspaceRoot,
    env: withSupportedWindowsCmakeGenerator(process.env, process.platform),
    stdio: 'inherit',
    shell: plan.shell,
  });

  child.on('error', (error) => {
    console.error(`[run-router-product-service] ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      console.error(`[run-router-product-service] command exited with signal ${signal}`);
      process.exit(1);
    }

    process.exit(code ?? 0);
  });
}

if (__filename === process.argv[1]) {
  runCli();
}
