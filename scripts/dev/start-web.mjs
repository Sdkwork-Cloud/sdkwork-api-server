#!/usr/bin/env node

import { existsSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import {
  parseWebArgs,
  webAccessLines,
  webHelpText,
  webHostEnv,
} from './web-launch-lib.mjs';

function pnpmExecutable() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function cargoExecutable() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function runStep(command, args, dryRun, label, env) {
  console.log(`[start-web] ${label}: ${command} ${args.join(' ')}`);

  if (dryRun) {
    return true;
  }

  const result = spawnSync(command, args, {
    stdio: 'inherit',
    env,
  });
  return result.status === 0;
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
  portalTarget: settings.portalTarget,
  gatewayTarget: settings.gatewayTarget,
});
for (const line of webAccessLines(settings.bind)) {
  console.log(line);
}

const appRoots = ['apps/sdkwork-router-admin', 'apps/sdkwork-router-portal'];
for (const appRoot of appRoots) {
  const needInstall = settings.install || !existsSync(`${appRoot}/node_modules`);
  if (needInstall && !runStep(pnpmExecutable(), ['--dir', appRoot, 'install'], settings.dryRun, `install ${appRoot}`, env)) {
    process.exit(1);
  }
}

for (const appRoot of appRoots) {
  if (!runStep(pnpmExecutable(), ['--dir', appRoot, 'build'], settings.dryRun, `build ${appRoot}`, env)) {
    process.exit(1);
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

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
