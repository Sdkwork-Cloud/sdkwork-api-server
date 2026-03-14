#!/usr/bin/env node

import { spawn } from 'node:child_process';

function parseArgs(argv) {
  const result = {
    databaseUrl: 'sqlite://sdkwork-api-server.db',
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    dryRun: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--dry-run') {
      result.dryRun = true;
      continue;
    }
    if (arg === '--help' || arg === '-h') {
      result.help = true;
      continue;
    }
    if (arg === '--database-url') {
      result.databaseUrl = argv[index + 1] ?? result.databaseUrl;
      index += 1;
      continue;
    }
    if (arg === '--gateway-bind') {
      result.gatewayBind = argv[index + 1] ?? result.gatewayBind;
      index += 1;
      continue;
    }
    if (arg === '--admin-bind') {
      result.adminBind = argv[index + 1] ?? result.adminBind;
      index += 1;
      continue;
    }
    if (arg === '--portal-bind') {
      result.portalBind = argv[index + 1] ?? result.portalBind;
      index += 1;
    }
  }

  return result;
}

function printHelp() {
  console.log(`Usage: node scripts/dev/start-stack.mjs [options]

Starts admin, gateway, and portal services in the current terminal.

Options:
  --database-url <url>   Shared SDKWORK_DATABASE_URL value
  --gateway-bind <bind>  SDKWORK_GATEWAY_BIND override
  --admin-bind <bind>    SDKWORK_ADMIN_BIND override
  --portal-bind <bind>   SDKWORK_PORTAL_BIND override
  --dry-run              Print commands without starting processes
  -h, --help             Show this help
`);
}

function serviceEnv(settings) {
  return {
    ...process.env,
    SDKWORK_DATABASE_URL: settings.databaseUrl,
    SDKWORK_GATEWAY_BIND: settings.gatewayBind,
    SDKWORK_ADMIN_BIND: settings.adminBind,
    SDKWORK_PORTAL_BIND: settings.portalBind,
  };
}

function cargoExecutable() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function startService(packageName, settings, children) {
  const env = serviceEnv(settings);
  const command = `${cargoExecutable()} run -p ${packageName}`;
  console.log(`[start-stack] ${command}`);

  if (settings.dryRun) {
    return;
  }

  const child = spawn(cargoExecutable(), ['run', '-p', packageName], {
    env,
    stdio: 'inherit',
  });
  children.push(child);

  child.on('exit', (code, signal) => {
    if (signal) {
      console.log(`[start-stack] ${packageName} exited with signal ${signal}`);
      return;
    }
    console.log(`[start-stack] ${packageName} exited with code ${code ?? 0}`);
  });
}

function installSignalHandlers(children) {
  let stopping = false;

  function shutdown(signal) {
    if (stopping) {
      return;
    }
    stopping = true;
    console.log(`[start-stack] received ${signal}, stopping child processes`);
    for (const child of children) {
      if (!child.killed) {
        child.kill('SIGTERM');
      }
    }
    setTimeout(() => process.exit(0), 150);
  }

  process.on('SIGINT', () => shutdown('SIGINT'));
  process.on('SIGTERM', () => shutdown('SIGTERM'));
}

const settings = parseArgs(process.argv.slice(2));
if (settings.help) {
  printHelp();
  process.exit(0);
}

console.log('[start-stack] shared configuration');
console.log(`  SDKWORK_DATABASE_URL=${settings.databaseUrl}`);
console.log(`  SDKWORK_ADMIN_BIND=${settings.adminBind}`);
console.log(`  SDKWORK_GATEWAY_BIND=${settings.gatewayBind}`);
console.log(`  SDKWORK_PORTAL_BIND=${settings.portalBind}`);

const children = [];
installSignalHandlers(children);
startService('admin-api-service', settings, children);
startService('gateway-service', settings, children);
startService('portal-api-service', settings, children);

if (settings.dryRun) {
  process.exit(0);
}
