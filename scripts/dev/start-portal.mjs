#!/usr/bin/env node

import { existsSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import {
  createSupervisorKeepAlive,
  createSignalController,
  didChildExitFail,
} from './process-supervision.mjs';
import {
  frontendDistReady,
  frontendInstallStatus,
  frontendViteConfigHealthy,
  pnpmDisplayCommand,
  pnpmProcessSpec,
  pnpmSpawnOptions,
  shouldReuseExistingFrontendDist,
} from './pnpm-launch-lib.mjs';

function parseArgs(argv) {
  const result = {
    dryRun: false,
    help: false,
    install: false,
    preview: false,
  };

  for (const arg of argv) {
    if (arg === '--dry-run') {
      result.dryRun = true;
    } else if (arg === '--install') {
      result.install = true;
    } else if (arg === '--preview') {
      result.preview = true;
    } else if (arg === '--help' || arg === '-h') {
      result.help = true;
    }
  }

  return result;
}

function printHelp() {
  console.log(`Usage: node scripts/dev/start-portal.mjs [options]

Starts the standalone sdkwork-router-portal app.

Options:
  --install   Run pnpm install before starting
  --preview   Build and preview the portal instead of dev mode
  --dry-run   Print the commands without running them
  -h, --help  Show this help
`);
}

function runStep(args, dryRun, distDir = '', allowInstallReuse = false) {
  const processSpec = pnpmProcessSpec(args);
  const command = pnpmDisplayCommand(args);
  console.log(`[start-portal] ${command}`);

  if (dryRun) {
    return true;
  }

  const result = spawnSync(processSpec.command, processSpec.args, {
    ...pnpmSpawnOptions({ stdio: 'pipe' }),
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });

  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
  if (result.error) {
    process.stderr.write(`${String(result.error.stack ?? result.error.message ?? result.error)}\n`);
  }

  if (result.status === 0) {
    return true;
  }

  if (shouldReuseExistingFrontendDist({
    stepArgs: args,
    status: result.status ?? 1,
    stdout: result.stdout,
    stderr: result.stderr,
    errorMessage: result.error?.message ?? '',
    distReady: frontendDistReady(distDir),
    allowInstallReuse,
  })) {
    console.warn(`[start-portal] ${command} failed with Windows spawn EPERM; reusing existing dist at ${distDir}`);
    return true;
  }

  return false;
}

const settings = parseArgs(process.argv.slice(2));
if (settings.help) {
  printHelp();
  process.exit(0);
}

const portalRoot = 'apps/sdkwork-router-portal';
const installStatus = frontendInstallStatus({
  appRoot: portalRoot,
  requiredPackages: ['vite', 'typescript'],
  requiredBinCommands: ['vite', 'tsc'],
  verifyInstalled: () => frontendViteConfigHealthy({
    appRoot: portalRoot,
    command: settings.preview ? 'build' : 'serve',
  }),
});
const needInstall = settings.install || installStatus !== 'ready';
if (needInstall && !runStep(['--dir', portalRoot, 'install'], settings.dryRun, `${portalRoot}/dist`, settings.preview)) {
  process.exit(1);
}

if (settings.preview) {
  if (!runStep(['--dir', portalRoot, 'build'], settings.dryRun, `${portalRoot}/dist`)) {
    process.exit(1);
  }
  if (!runStep(['--dir', portalRoot, 'preview'], settings.dryRun)) {
    process.exit(1);
  }
  process.exit(0);
}

const longRunningArgs = ['--dir', portalRoot, 'dev'];
const longRunningProcessSpec = pnpmProcessSpec(longRunningArgs);
const command = pnpmDisplayCommand(longRunningArgs);
console.log(`[start-portal] ${command}`);

if (settings.dryRun) {
  process.exit(0);
}

const child = spawn(longRunningProcessSpec.command, longRunningProcessSpec.args, {
  ...pnpmSpawnOptions(),
});
const releaseKeepAlive = createSupervisorKeepAlive();
let shuttingDown = false;
const controller = createSignalController({
  label: 'start-portal',
  children: [child],
  onShutdownStart: () => {
    shuttingDown = true;
    releaseKeepAlive();
  },
});
controller.register();

child.on('exit', (code, signal) => {
  releaseKeepAlive();
  if (shuttingDown) {
    return;
  }

  process.exit(didChildExitFail(code, signal) ? code ?? 1 : 0);
});
