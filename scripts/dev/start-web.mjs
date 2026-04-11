#!/usr/bin/env node

import { existsSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import {
  parseWebArgs,
  webAccessLines,
  webHelpText,
  webHostEnv,
} from './web-launch-lib.mjs';
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

function cargoExecutable() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function runPnpmStep(args, dryRun, label, env, distDir = '', allowInstallReuse = false) {
  const processSpec = pnpmProcessSpec(args);
  console.log(`[start-web] ${label}: ${pnpmDisplayCommand(args)}`);

  if (dryRun) {
    return true;
  }

  const result = spawnSync(processSpec.command, processSpec.args, {
    ...pnpmSpawnOptions({ env, stdio: 'pipe' }),
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });

  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
  const reuseExistingDist = shouldReuseExistingFrontendDist({
    stepArgs: args,
    status: result.status ?? 1,
    stdout: result.stdout,
    stderr: result.stderr,
    errorMessage: result.error?.message ?? '',
    distReady: frontendDistReady(distDir),
    allowInstallReuse,
  });

  if (result.error && !reuseExistingDist) {
    process.stderr.write(`${String(result.error.stack ?? result.error.message ?? result.error)}\n`);
  }

  if (result.status === 0) {
    return true;
  }

  if (reuseExistingDist) {
    console.warn(`[start-web] ${label} failed with Windows spawn EPERM; reusing existing dist at ${distDir}`);
    return true;
  }

  return false;
}

let settings;
try {
  settings = parseWebArgs(process.argv.slice(2));
} catch (error) {
  console.error(`[start-web] ${error.message}`);
  console.error('');
  console.error(webHelpText());
  process.exit(1);
}

if (settings.help) {
  console.log(webHelpText());
  process.exit(0);
}

const env = webHostEnv(settings.bind, {
  adminTarget: settings.adminTarget,
  adminSiteTarget: settings.adminSiteTarget,
  portalTarget: settings.portalTarget,
  portalSiteTarget: settings.portalSiteTarget,
  gatewayTarget: settings.gatewayTarget,
});
for (const line of webAccessLines(settings.bind, { proxyDev: settings.proxyDev })) {
  console.log(line);
}

const appRoots = ['apps/sdkwork-router-admin', 'apps/sdkwork-router-portal'];
for (const appRoot of appRoots) {
  const installStatus = frontendInstallStatus({
    appRoot,
    requiredPackages: ['vite', 'typescript'],
    requiredBinCommands: ['vite', 'tsc'],
    verifyInstalled: () => frontendViteConfigHealthy({
      appRoot,
      command: settings.proxyDev ? 'serve' : 'build',
    }),
  });
  const needInstall = settings.install || installStatus !== 'ready';
  if (needInstall && !runPnpmStep(['--dir', appRoot, 'install'], settings.dryRun, `install ${appRoot}`, env, `${appRoot}/dist`, true)) {
    process.exit(1);
  }
}

if (!settings.proxyDev) {
  for (const appRoot of appRoots) {
    if (!runPnpmStep(['--dir', appRoot, 'build'], settings.dryRun, `build ${appRoot}`, env, `${appRoot}/dist`)) {
      process.exit(1);
    }
  }
}

const webArgs = ['run', '-p', 'router-web-service'];
console.log(`[start-web] ${cargoExecutable()} ${webArgs.join(' ')}`);

if (settings.dryRun) {
  process.exit(0);
}

const child = spawn(cargoExecutable(), webArgs, {
  stdio: 'inherit',
  env,
});
const releaseKeepAlive = createSupervisorKeepAlive();
let shuttingDown = false;
const controller = createSignalController({
  label: 'start-web',
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
