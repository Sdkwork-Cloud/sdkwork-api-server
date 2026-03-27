#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function pnpmCommand(platform = process.platform) {
  return platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function shellForPnpm(platform = process.platform) {
  return platform === 'win32';
}

function toPortablePath(value) {
  return value.replaceAll(path.sep, '/');
}

export function parseRouterProductArgs(argv) {
  const result = {
    mode: 'desktop',
    install: false,
    dryRun: false,
    help: false,
    extraArgs: [],
  };

  let modeSet = false;
  let forwardOnly = false;
  for (const arg of argv) {
    if (forwardOnly) {
      result.extraArgs.push(arg);
      continue;
    }
    if (arg === '--') {
      forwardOnly = true;
      continue;
    }
    if (arg === '--install') {
      result.install = true;
      continue;
    }
    if (arg === '--dry-run') {
      result.dryRun = true;
      continue;
    }
    if (arg === '--help' || arg === '-h') {
      result.help = true;
      continue;
    }
    if (!modeSet && !arg.startsWith('-')) {
      result.mode = arg;
      modeSet = true;
      continue;
    }
    result.extraArgs.push(arg);
  }

  return result;
}

function appendForwardArgs(args, extraArgs) {
  if (!extraArgs.length) {
    return args;
  }

  return [...args, '--', ...extraArgs];
}

export function createRouterProductLaunchPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  mode = 'desktop',
  install = false,
  platform = process.platform,
  env = process.env,
  extraArgs = [],
} = {}) {
  const portalRelativeDir = toPortablePath(path.join('apps', 'sdkwork-router-portal'));
  const portalAbsoluteDir = path.join(workspaceRoot, portalRelativeDir);
  const pnpm = pnpmCommand(platform);
  const shell = shellForPnpm(platform);
  const nodeCommand = process.execPath;
  const plan = [];

  if (install || !existsSync(path.join(portalAbsoluteDir, 'node_modules'))) {
    plan.push({
      label: 'portal install',
      command: pnpm,
      args: ['--dir', portalRelativeDir, 'install'],
      cwd: workspaceRoot,
      env,
      shell,
    });
  }

  let launchArgs;
  let label;
  switch (mode) {
    case 'desktop':
      label = 'portal desktop runtime';
      launchArgs = appendForwardArgs(['--dir', portalRelativeDir, 'tauri:dev'], extraArgs);
      break;
    case 'server':
      label = 'portal product server';
      launchArgs = appendForwardArgs(['--dir', portalRelativeDir, 'server:start'], extraArgs);
      break;
    case 'plan':
      plan.push({
        label: 'portal deployment plan',
        command: nodeCommand,
        args: [
          path.join(workspaceRoot, 'scripts', 'run-router-product-service.mjs'),
          '--dry-run',
          '--plan-format',
          'json',
          ...extraArgs,
        ],
        cwd: portalAbsoluteDir,
        env,
        shell: false,
      });
      return plan;
    case 'check':
      label = 'portal product check';
      launchArgs = ['--dir', portalRelativeDir, 'product:check'];
      break;
    case 'browser':
      label = 'portal browser runtime';
      launchArgs = appendForwardArgs(['--dir', portalRelativeDir, 'dev'], extraArgs);
      break;
    default:
      throw new Error(
        `Unsupported router product mode: ${mode}. Expected one of desktop, server, plan, check, browser.`,
      );
  }

  plan.push({
    label,
    command: pnpm,
    args: launchArgs,
    cwd: workspaceRoot,
    env,
    shell,
  });

  return plan;
}

function printHelp() {
  console.log(`Usage: node scripts/run-router-product.mjs [mode] [options] [mode-args...]

Start the sdkwork-router-portal product as a desktop runtime or server runtime.

Modes:
  desktop  Start the Tauri desktop host and embedded router product runtime (default)
  server   Start router-product-service through the portal product entrypoint
  plan     Print the resolved server deployment plan through the portal entrypoint
  check    Run the integrated product verification flow
  browser  Start the standalone portal browser dev server

Options:
  --install   Run pnpm install for sdkwork-router-portal before starting
  --dry-run   Print the planned commands without running them
  -h, --help  Show this help

Examples:
  node scripts/run-router-product.mjs
  node scripts/run-router-product.mjs server --roles web --gateway-upstream 10.0.0.21:8080
  node scripts/run-router-product.mjs plan --roles web
  node scripts/run-router-product.mjs check
`);
}

async function runStep(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell ?? false,
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
  const settings = parseRouterProductArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    return;
  }

  const plan = createRouterProductLaunchPlan({
    mode: settings.mode,
    install: settings.install,
    extraArgs: settings.extraArgs,
  });

  for (const step of plan) {
    const rendered = `${step.command} ${step.args.join(' ')}`;
    console.error(`[run-router-product] ${rendered}`);
    if (settings.dryRun) {
      continue;
    }
    // eslint-disable-next-line no-await-in-loop
    await runStep(step);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === __filename) {
  main().catch((error) => {
    console.error(`[run-router-product] ${error.message}`);
    process.exit(1);
  });
}
