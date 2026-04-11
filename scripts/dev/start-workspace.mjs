#!/usr/bin/env node

import path from 'node:path';
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import {
  buildWorkspaceCommandPlan,
  parseWorkspaceArgs,
  workspaceAccessLines,
  workspaceHelpText,
} from './workspace-launch-lib.mjs';
import {
  createSupervisorKeepAlive,
  createSignalController,
  didChildExitFail,
} from './process-supervision.mjs';

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, '..', '..');
process.chdir(repositoryRoot);

function formatCommand(command, args) {
  return [command, ...args].join(' ');
}

function spawnStep(step, nodeExecutable, children) {
  const command = formatCommand(nodeExecutable, step.args);
  console.log(`[start-workspace] ${step.name}: ${command}`);

  const child = spawn(nodeExecutable, step.args, {
    cwd: repositoryRoot,
    stdio: 'inherit',
  });
  children.push(child);

  child.on('exit', (code, signal) => {
    if (signal) {
      console.log(`[start-workspace] ${step.name} exited with signal ${signal}`);
      return;
    }
    console.log(`[start-workspace] ${step.name} exited with code ${code ?? 0}`);
  });

  return child;
}

function watchStopFile(stopFile, onStopRequested) {
  if (!stopFile) {
    return () => {};
  }

  let stopped = false;
  const timer = setInterval(() => {
    if (stopped || !existsSync(stopFile)) {
      return;
    }

    stopped = true;
    clearInterval(timer);
    console.log(`[start-workspace] stop signal file detected: ${stopFile}`);
    onStopRequested();
  }, 1000);

  if (typeof timer.unref === 'function') {
    timer.unref();
  }

  return () => {
    stopped = true;
    clearInterval(timer);
  };
}

function main() {
  let settings;
  try {
    settings = parseWorkspaceArgs(process.argv.slice(2));
  } catch (error) {
    console.error(`[start-workspace] ${error.message}`);
    console.error('');
    console.error(workspaceHelpText());
    process.exit(1);
  }

  if (settings.help) {
    console.log(workspaceHelpText());
    process.exit(0);
  }

  const plan = buildWorkspaceCommandPlan(settings);

  console.log('[start-workspace] unified launch settings');
  console.log(
    `  SDKWORK_DATABASE_URL=${settings.databaseUrl ?? '(local default via config loader)'}`,
  );
  console.log(`  SDKWORK_GATEWAY_BIND=${settings.gatewayBind}`);
  console.log(`  SDKWORK_ADMIN_BIND=${settings.adminBind}`);
  console.log(`  SDKWORK_PORTAL_BIND=${settings.portalBind}`);
  console.log(`  SDKWORK_WEB_BIND=${settings.webBind}`);
  console.log(`  frontend_mode=${settings.preview ? 'preview' : settings.proxyDev ? 'proxy-dev' : settings.tauri ? 'tauri' : 'browser'}`);
  for (const line of workspaceAccessLines(settings)) {
    console.log(line);
  }

  if (settings.dryRun) {
    console.log(`[start-workspace] ${plan.backend.name}: ${formatCommand(plan.nodeExecutable, plan.backend.args)}`);
    if (settings.preview) {
      console.log(`[start-workspace] ${plan.web.name}: ${formatCommand(plan.nodeExecutable, plan.web.args)}`);
    } else if (settings.proxyDev) {
      console.log(`[start-workspace] ${plan.admin.name}: ${formatCommand(plan.nodeExecutable, plan.admin.args)}`);
      console.log(`[start-workspace] ${plan.portal.name}: ${formatCommand(plan.nodeExecutable, plan.portal.args)}`);
      console.log(`[start-workspace] ${plan.web.name}: ${formatCommand(plan.nodeExecutable, plan.web.args)}`);
    } else if (settings.tauri) {
      console.log(`[start-workspace] ${plan.admin.name}: ${formatCommand(plan.nodeExecutable, plan.admin.args)}`);
      console.log(`[start-workspace] ${plan.web.name}: ${formatCommand(plan.nodeExecutable, plan.web.args)}`);
    } else {
      console.log(`[start-workspace] ${plan.admin.name}: ${formatCommand(plan.nodeExecutable, plan.admin.args)}`);
      console.log(`[start-workspace] ${plan.portal.name}: ${formatCommand(plan.nodeExecutable, plan.portal.args)}`);
    }
    process.exit(0);
  }

  const children = [];
  let exited = false;
  let stopFileWatcher = () => {};
  const releaseKeepAlive = createSupervisorKeepAlive();
  const controller = createSignalController({
    label: 'start-workspace',
    children,
    onShutdownStart: () => {
      exited = true;
      stopFileWatcher();
      releaseKeepAlive();
    },
  });
  controller.register();
  stopFileWatcher = watchStopFile(settings.stopFile, () => {
    if (exited) {
      return;
    }
    exited = true;
    void controller.shutdown('stop-file', 0);
  });

  const steps = settings.preview
    ? [plan.backend, plan.web]
    : settings.proxyDev
      ? [plan.backend, plan.admin, plan.portal, plan.web]
    : settings.tauri
      ? [plan.backend, plan.admin, plan.web]
      : [plan.backend, plan.admin, plan.portal];

  for (const step of steps) {
    const child = spawnStep(step, plan.nodeExecutable, children);
    child.on('exit', (code, signal) => {
      if (exited) {
        return;
      }

      exited = true;
      releaseKeepAlive();
      void controller.shutdown(
        `${step.name} exit`,
        didChildExitFail(code, signal) ? code ?? 1 : 0,
      );
    });
  }
}

const currentEntry = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (currentEntry === fileURLToPath(import.meta.url)) {
  main();
}
