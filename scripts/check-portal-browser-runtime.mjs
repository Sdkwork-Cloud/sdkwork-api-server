#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import net from 'node:net';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import { runBrowserRuntimeSmoke } from './browser-runtime-smoke.mjs';
import {
  ensureFrontendDependenciesReady,
  frontendViteConfigHealthy,
  pnpmProcessSpec,
  pnpmSpawnOptions,
} from './dev/pnpm-launch-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEFAULT_TIMEOUT_MS = 45_000;
const PORTAL_EXPECTED_TEXTS = [
  'Unified AI gateway workspace',
  'Operate routing, credentials, usage, and downloads from one product surface.',
];
const PORTAL_EXPECTED_SELECTORS = [
  '[data-slot="portal-home-page"]',
  '[data-slot="portal-home-metrics"]',
];

function truncateText(value, maxLength = 400) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function runForegroundStep(step, {
  env = process.env,
  platform = process.platform,
} = {}) {
  const result = spawnSync(step.command, step.args, {
    ...pnpmSpawnOptions({
      platform,
      env,
      cwd: step.cwd,
      stdio: 'pipe',
    }),
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });

  if ((result.status ?? 1) !== 0) {
    throw new Error(
      `${step.label} failed with exit code ${result.status ?? 'unknown'}: ${truncateText(`${result.stdout ?? ''}\n${result.stderr ?? ''}`, 1200)}`,
    );
  }
}

function killProcessTree(child, platform = process.platform) {
  if (!child?.pid) {
    return;
  }

  if (platform === 'win32') {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    });
    return;
  }

  child.kill('SIGTERM');
}

async function waitForHttpOk(url, timeoutMs = DEFAULT_TIMEOUT_MS) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, {
        signal: AbortSignal.timeout(3000),
      });
      if (!response.ok) {
        throw new Error(`${url} returned HTTP ${response.status}`);
      }

      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await delay(250);
    }
  }

  throw new Error(
    `${url} did not become reachable within ${timeoutMs}ms: ${lastError?.message ?? 'unknown error'}`,
  );
}

async function findAvailablePort() {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve(port);
      });
    });
  });
}

export function createPortalBrowserRuntimeSmokePlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  portalAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  platform = process.platform,
  env = process.env,
  previewPort = 4174,
} = {}) {
  return {
    portalAppDir,
    previewUrl: `http://127.0.0.1:${previewPort}/portal/`,
    expectedTexts: PORTAL_EXPECTED_TEXTS,
    expectedSelectors: PORTAL_EXPECTED_SELECTORS,
    buildStep: {
      ...pnpmProcessSpec(['build'], { platform }),
      label: 'portal production build',
      cwd: portalAppDir,
    },
    previewStep: {
      command: process.execPath,
      args: [
        path.join(workspaceRoot, 'scripts', 'dev', 'run-vite-cli.mjs'),
        'preview',
        '--host',
        '127.0.0.1',
        '--port',
        String(previewPort),
        '--strictPort',
      ],
      label: 'portal preview server',
      cwd: portalAppDir,
      env,
    },
  };
}

export async function runPortalBrowserRuntimeSmoke({
  workspaceRoot = path.resolve(__dirname, '..'),
  portalAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  platform = process.platform,
  env = process.env,
} = {}) {
  ensureFrontendDependenciesReady({
    appRoot: portalAppDir,
    requiredPackages: ['vite', 'typescript'],
    requiredBinCommands: ['vite', 'tsc'],
    verifyInstalled: () => frontendViteConfigHealthy({
      appRoot: portalAppDir,
      command: 'build',
    }),
    platform,
    env,
  });

  const previewPort = await findAvailablePort();
  const plan = createPortalBrowserRuntimeSmokePlan({
    workspaceRoot,
    portalAppDir,
    platform,
    env,
    previewPort,
  });

  runForegroundStep(plan.buildStep, { env, platform });

  const previewProcess = spawn(plan.previewStep.command, plan.previewStep.args, {
    ...pnpmSpawnOptions({
      platform,
      env,
      cwd: plan.previewStep.cwd,
      stdio: 'pipe',
    }),
  });
  let previewStdout = '';
  let previewStderr = '';

  previewProcess.stdout?.on('data', (chunk) => {
    previewStdout += String(chunk);
  });
  previewProcess.stderr?.on('data', (chunk) => {
    previewStderr += String(chunk);
  });

  try {
    await waitForHttpOk(plan.previewUrl, DEFAULT_TIMEOUT_MS);
    return await runBrowserRuntimeSmoke({
      url: plan.previewUrl,
      expectedSelectors: plan.expectedSelectors,
      timeoutMs: DEFAULT_TIMEOUT_MS,
      platform,
      env,
    });
  } finally {
    killProcessTree(previewProcess, platform);
    await delay(250).catch(() => {});
  }
}

async function main() {
  const result = await runPortalBrowserRuntimeSmoke();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
