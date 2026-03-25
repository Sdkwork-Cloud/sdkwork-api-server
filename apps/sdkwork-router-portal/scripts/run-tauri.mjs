#!/usr/bin/env node

import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const WINDOWS_CMAKE_GENERATOR = 'Visual Studio 17 2022';
const VALID_MODES = new Set(['dev', 'build']);

export function createTauriCommandPlan({
  mode,
  platform = process.platform,
  env = process.env,
} = {}) {
  if (!VALID_MODES.has(mode)) {
    throw new Error(
      `Unsupported tauri mode "${mode}". Expected one of: ${Array.from(VALID_MODES).join(', ')}.`,
    );
  }

  const nextEnv = { ...env };
  if (platform === 'win32' && !nextEnv.CMAKE_GENERATOR) {
    nextEnv.CMAKE_GENERATOR = WINDOWS_CMAKE_GENERATOR;
  }

  return {
    command: 'tauri',
    args: [mode],
    env: nextEnv,
  };
}

function printUsage() {
  console.error('Usage: node ./scripts/run-tauri.mjs <dev|build>');
}

function runCli() {
  const mode = process.argv[2];
  if (!mode) {
    printUsage();
    process.exit(1);
  }

  let plan;
  try {
    plan = createTauriCommandPlan({ mode });
  } catch (error) {
    console.error(`[run-tauri] ${error.message}`);
    printUsage();
    process.exit(1);
  }

  const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
  const appRoot = path.resolve(scriptDirectory, '..');
  const child = spawn(plan.command, plan.args, {
    cwd: appRoot,
    env: plan.env,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  child.on('error', (error) => {
    console.error(`[run-tauri] ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      console.error(`[run-tauri] tauri exited with signal ${signal}`);
      process.exit(1);
    }
    process.exit(code ?? 0);
  });
}

const entryFile = fileURLToPath(import.meta.url);
if (process.argv[1] && path.resolve(process.argv[1]) === entryFile) {
  runCli();
}
