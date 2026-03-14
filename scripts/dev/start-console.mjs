#!/usr/bin/env node

import { existsSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';

function parseArgs(argv) {
  const result = {
    dryRun: false,
    help: false,
    install: false,
    preview: false,
    tauri: false,
  };

  for (const arg of argv) {
    if (arg === '--dry-run') {
      result.dryRun = true;
    } else if (arg === '--install') {
      result.install = true;
    } else if (arg === '--preview') {
      result.preview = true;
    } else if (arg === '--tauri') {
      result.tauri = true;
    } else if (arg === '--help' || arg === '-h') {
      result.help = true;
    }
  }

  return result;
}

function printHelp() {
  console.log(`Usage: node scripts/dev/start-console.mjs [options]

Starts the shared browser console or the Tauri desktop host.

Options:
  --install   Run pnpm install before starting
  --preview   Build and preview the console instead of dev mode
  --tauri     Start the Tauri desktop shell; browser remains available through the Vite dev server
  --dry-run   Print the commands without running them
  -h, --help  Show this help
`);
}

function pnpmExecutable() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function runStep(args, dryRun) {
  const command = `${pnpmExecutable()} ${args.join(' ')}`;
  console.log(`[start-console] ${command}`);

  if (dryRun) {
    return true;
  }

  const result = spawnSync(pnpmExecutable(), args, {
    stdio: 'inherit',
  });
  return result.status === 0;
}

const settings = parseArgs(process.argv.slice(2));
if (settings.help) {
  printHelp();
  process.exit(0);
}

const needInstall = settings.install || !existsSync('console/node_modules');
if (needInstall && !runStep(['--dir', 'console', 'install'], settings.dryRun)) {
  process.exit(1);
}

if (settings.preview) {
  if (!runStep(['--dir', 'console', 'build'], settings.dryRun)) {
    process.exit(1);
  }
  if (!runStep(['--dir', 'console', 'preview'], settings.dryRun)) {
    process.exit(1);
  }
  process.exit(0);
}

const longRunningArgs = settings.tauri
  ? ['--dir', 'console', 'tauri:dev']
  : ['--dir', 'console', 'dev'];
const command = `${pnpmExecutable()} ${longRunningArgs.join(' ')}`;
console.log(`[start-console] ${command}`);

if (settings.tauri) {
  console.log('[start-console] Tauri dev mode still exposes the Vite browser UI on http://127.0.0.1:5173');
}

if (settings.dryRun) {
  process.exit(0);
}

const child = spawn(pnpmExecutable(), longRunningArgs, {
  stdio: 'inherit',
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
