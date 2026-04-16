#!/usr/bin/env node

import { existsSync } from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import {
  databaseDisplayValue,
  parseStackArgs,
  serviceEnv,
  stackHelpText,
} from './backend-launch-lib.mjs';
import {
  createSupervisorKeepAlive,
  createSignalController,
  didChildExitFail,
} from './process-supervision.mjs';

function cargoExecutable() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function resolveBackendLaunchSpec(packageName, env) {
  const usePrebuiltBinaries =
    process.platform === 'win32' && env.SDKWORK_ROUTER_USE_PREBUILT_BACKEND_BINARIES === '1';
  if (usePrebuiltBinaries && env.CARGO_TARGET_DIR) {
    const binaryPath = path.resolve(env.CARGO_TARGET_DIR, 'debug', `${packageName}.exe`);
    if (existsSync(binaryPath)) {
      return {
        command: binaryPath,
        args: [],
      };
    }
  }

  return {
    command: cargoExecutable(),
    args: ['run', '-p', packageName],
  };
}

function startService(packageName, settings, children, onFailure) {
  const env = serviceEnv(settings);
  const launchSpec = resolveBackendLaunchSpec(packageName, env);
  const command = [launchSpec.command, ...launchSpec.args].join(' ');
  console.log(`[start-stack] ${command}`);

  if (settings.dryRun) {
    return;
  }

  const child = spawn(launchSpec.command, launchSpec.args, {
    env,
    stdio: 'inherit',
  });
  children.push(child);

  child.on('exit', (code, signal) => {
    if (signal) {
      console.log(`[start-stack] ${packageName} exited with signal ${signal}`);
    } else {
      console.log(`[start-stack] ${packageName} exited with code ${code ?? 0}`);
    }

    onFailure(`${packageName} exit`, didChildExitFail(code, signal) ? code ?? 1 : 0);
  });
}

const settings = parseStackArgs(process.argv.slice(2));
if (settings.help) {
  console.log(stackHelpText());
  process.exit(0);
}

console.log('[start-stack] shared configuration');
console.log(`  SDKWORK_DATABASE_URL=${databaseDisplayValue(settings)}`);
console.log(`  SDKWORK_ADMIN_BIND=${settings.adminBind}`);
console.log(`  SDKWORK_GATEWAY_BIND=${settings.gatewayBind}`);
console.log(`  SDKWORK_PORTAL_BIND=${settings.portalBind}`);

const children = [];
let exited = false;
const releaseKeepAlive = createSupervisorKeepAlive();
const controller = createSignalController({
  label: 'start-stack',
  children,
  onShutdownStart: () => {
    exited = true;
    releaseKeepAlive();
  },
});
controller.register();

function stopOnFailure(reason, exitCode) {
  if (exited) {
    return;
  }

  exited = true;
  releaseKeepAlive();
  void controller.shutdown(reason, exitCode);
}

startService('admin-api-service', settings, children, stopOnFailure);
startService('gateway-service', settings, children, stopOnFailure);
startService('portal-api-service', settings, children, stopOnFailure);

if (settings.dryRun) {
  process.exit(0);
}
