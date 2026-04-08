#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:net';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import {
  applyInstallPlan,
  assertInstallInputsExist,
  createInstallPlan,
  renderRuntimeEnvTemplate,
} from '../../bin/lib/router-runtime-tooling.mjs';
import { resolveDesktopReleaseTarget } from './desktop-targets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_WAIT_SECONDS = 120;
const DEFAULT_HEALTH_ATTEMPTS = 12;
const DEFAULT_HEALTH_DELAY_MS = 1000;

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new Error(`${token} requires a value`);
  }

  return next;
}

function resolveRuntimeHome(repoRoot, runtimeHome, { platform, arch }) {
  if (runtimeHome) {
    return path.isAbsolute(runtimeHome)
      ? runtimeHome
      : path.resolve(repoRoot, runtimeHome);
  }

  return path.resolve(repoRoot, 'artifacts', 'release-smoke', `${platform}-${arch}`);
}

function resolveEvidencePath(repoRoot, evidencePath, { platform, arch }) {
  if (evidencePath) {
    return path.isAbsolute(evidencePath)
      ? evidencePath
      : path.resolve(repoRoot, evidencePath);
  }

  return path.resolve(
    repoRoot,
    'artifacts',
    'release-governance',
    `windows-installed-runtime-smoke-${platform}-${arch}.json`,
  );
}

function assertWindowsRuntimeSmokePorts(ports) {
  for (const key of ['web', 'gateway', 'admin', 'portal']) {
    const value = ports?.[key];
    if (!Number.isInteger(value) || value <= 0) {
      throw new Error(`missing windows runtime smoke port: ${key}`);
    }
  }
}

function renderWindowsInstalledRuntimeSmokeEnvContents({
  runtimeHome,
  ports,
} = {}) {
  assertWindowsRuntimeSmokePorts(ports);

  let contents = renderRuntimeEnvTemplate({
    installRoot: runtimeHome,
    platform: 'win32',
  });

  const replacements = new Map([
    ['SDKWORK_WEB_BIND', `SDKWORK_WEB_BIND="127.0.0.1:${ports.web}"`],
    ['SDKWORK_GATEWAY_BIND', `SDKWORK_GATEWAY_BIND="127.0.0.1:${ports.gateway}"`],
    ['SDKWORK_ADMIN_BIND', `SDKWORK_ADMIN_BIND="127.0.0.1:${ports.admin}"`],
    ['SDKWORK_PORTAL_BIND', `SDKWORK_PORTAL_BIND="127.0.0.1:${ports.portal}"`],
  ]);

  for (const [key, replacement] of replacements.entries()) {
    contents = contents.replace(new RegExp(`^${key}=.*$`, 'm'), replacement);
  }

  return contents;
}

function truncateText(value, maxLength = 1600) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function toPortableRelativePath(repoRoot, targetPath) {
  return (path.relative(repoRoot, targetPath) || '.').replaceAll('\\', '/');
}

function readLogExcerpt(filePath, maxLines = 40) {
  if (!existsSync(filePath)) {
    return '';
  }

  const lines = readFileSync(filePath, 'utf8').trim().split(/\r?\n/u);
  return lines.slice(-maxLines).join('\n').trim();
}

function buildFailureContext(plan) {
  const contexts = [];

  const stdoutExcerpt = readLogExcerpt(plan.stdoutLogPath);
  if (stdoutExcerpt) {
    contexts.push(`stdout log (${plan.stdoutLogPath}):\n${truncateText(stdoutExcerpt)}`);
  }

  const stderrExcerpt = readLogExcerpt(plan.stderrLogPath);
  if (stderrExcerpt) {
    contexts.push(`stderr log (${plan.stderrLogPath}):\n${truncateText(stderrExcerpt)}`);
  }

  return contexts.length > 0 ? `\n${contexts.join('\n\n')}` : '';
}

function buildCommandFailure(label, result, plan) {
  const fragments = [];

  if (result?.error) {
    fragments.push(`error: ${result.error.message}`);
  }

  if (String(result?.stdout ?? '').trim()) {
    fragments.push(`stdout: ${truncateText(result.stdout)}`);
  }

  if (String(result?.stderr ?? '').trim()) {
    fragments.push(`stderr: ${truncateText(result.stderr)}`);
  }

  const exitText = result?.status == null ? 'unknown' : String(result.status);
  return new Error(
    `${label} failed with exit code ${exitText}${fragments.length > 0 ? `\n${fragments.join('\n')}` : ''}${buildFailureContext(plan)}`,
  );
}

function runScriptCommand(command, args, { cwd, env, label, plan } = {}) {
  const result = spawnSync(command, args, {
    cwd,
    env,
    encoding: 'utf8',
    shell: false,
  });

  if (result.error || result.status !== 0) {
    throw buildCommandFailure(label, result, plan);
  }

  return result;
}

async function reserveLoopbackPort() {
  const server = createServer();
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });

  const address = server.address();
  if (!address || typeof address !== 'object') {
    await new Promise((resolve) => server.close(resolve));
    throw new Error('failed to reserve a loopback port');
  }

  const { port } = address;
  await new Promise((resolve, reject) => {
    server.close((error) => {
      if (error) {
        reject(error);
        return;
      }

      resolve();
    });
  });

  return port;
}

async function allocateLoopbackPorts() {
  return {
    web: await reserveLoopbackPort(),
    gateway: await reserveLoopbackPort(),
    admin: await reserveLoopbackPort(),
    portal: await reserveLoopbackPort(),
  };
}

async function assertHealthyResponse(url) {
  const response = await fetch(url, {
    signal: AbortSignal.timeout(5000),
  });
  const body = String(await response.text()).trim();

  if (!response.ok) {
    throw new Error(`${url} returned HTTP ${response.status}: ${truncateText(body, 400)}`);
  }

  if (body.length > 0 && body.toLowerCase() !== 'ok') {
    throw new Error(`${url} returned unexpected body: ${truncateText(body, 400)}`);
  }
}

async function waitForHealthUrls(urls) {
  let lastError = null;

  for (let attempt = 0; attempt < DEFAULT_HEALTH_ATTEMPTS; attempt += 1) {
    try {
      for (const url of urls) {
        // eslint-disable-next-line no-await-in-loop
        await assertHealthyResponse(url);
      }

      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (attempt + 1 >= DEFAULT_HEALTH_ATTEMPTS) {
        break;
      }

      // eslint-disable-next-line no-await-in-loop
      await delay(DEFAULT_HEALTH_DELAY_MS);
    }
  }

  throw new Error(
    `installed runtime health checks did not stabilize after ${DEFAULT_HEALTH_ATTEMPTS} attempts: ${lastError?.message ?? 'unknown error'}`,
  );
}

export function createWindowsInstalledRuntimeSmokeOptions({
  repoRoot = rootDir,
  platform = process.platform,
  arch = process.arch,
  target = '',
  runtimeHome = '',
  evidencePath = '',
} = {}) {
  const resolvedTarget = resolveDesktopReleaseTarget({
    targetTriple: target,
    platform,
    arch,
  });

  if (resolvedTarget.platform !== 'windows') {
    throw new Error('run-windows-installed-runtime-smoke only supports windows release lanes');
  }

  return {
    platform: resolvedTarget.platform,
    arch: resolvedTarget.arch,
    target: resolvedTarget.targetTriple,
    runtimeHome: resolveRuntimeHome(repoRoot, runtimeHome, resolvedTarget),
    evidencePath: resolveEvidencePath(repoRoot, evidencePath, resolvedTarget),
  };
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    platform: '',
    arch: '',
    target: '',
    runtimeHome: '',
    evidencePath: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--platform') {
      options.platform = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--arch') {
      options.arch = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--target') {
      options.target = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--runtime-home') {
      options.runtimeHome = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--evidence-path') {
      options.evidencePath = readOptionValue(token, next);
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!options.platform) {
    throw new Error('--platform is required');
  }
  if (!options.arch) {
    throw new Error('--arch is required');
  }
  if (!options.target) {
    throw new Error('--target is required');
  }

  return createWindowsInstalledRuntimeSmokeOptions({
    repoRoot: rootDir,
    ...options,
  });
}

export function createWindowsInstalledRuntimeSmokePlan({
  repoRoot = rootDir,
  platform,
  arch,
  target,
  runtimeHome,
  evidencePath,
  env = process.env,
  ports = {
    web: 9983,
    gateway: 9980,
    admin: 9981,
    portal: 9982,
  },
} = {}) {
  const options = createWindowsInstalledRuntimeSmokeOptions({
    repoRoot,
    platform,
    arch,
    target,
    runtimeHome,
    evidencePath,
  });

  assertWindowsRuntimeSmokePorts(ports);

  const installPlan = createInstallPlan({
    repoRoot,
    installRoot: options.runtimeHome,
    platform: options.platform,
    arch: options.arch,
    env: {
      ...env,
      SDKWORK_DESKTOP_TARGET: options.target,
    },
  });

  return {
    ...options,
    installPlan,
    routerEnvPath: path.join(options.runtimeHome, 'config', 'router.env'),
    routerEnvContents: renderWindowsInstalledRuntimeSmokeEnvContents({
      runtimeHome: options.runtimeHome,
      ports,
    }),
    startCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(options.runtimeHome, 'bin', 'start.ps1'),
        '-Home',
        options.runtimeHome,
        '-WaitSeconds',
        String(DEFAULT_WAIT_SECONDS),
      ],
    },
    stopCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(options.runtimeHome, 'bin', 'stop.ps1'),
        '-Home',
        options.runtimeHome,
        '-WaitSeconds',
        String(DEFAULT_WAIT_SECONDS),
      ],
    },
    pidFilePath: path.join(options.runtimeHome, 'var', 'run', 'router-product-service.pid'),
    stdoutLogPath: path.join(options.runtimeHome, 'var', 'log', 'router-product-service.stdout.log'),
    stderrLogPath: path.join(options.runtimeHome, 'var', 'log', 'router-product-service.stderr.log'),
    healthUrls: [
      `http://127.0.0.1:${ports.web}/api/v1/health`,
      `http://127.0.0.1:${ports.web}/api/admin/health`,
      `http://127.0.0.1:${ports.web}/api/portal/health`,
    ],
  };
}

export function createWindowsInstalledRuntimeSmokeEvidence({
  repoRoot = rootDir,
  plan,
  ok,
  failure = null,
} = {}) {
  const stdoutLogExcerpt = readLogExcerpt(plan.stdoutLogPath);
  const stderrLogExcerpt = readLogExcerpt(plan.stderrLogPath);

  const evidence = {
    generatedAt: new Date().toISOString(),
    ok,
    platform: plan.platform,
    arch: plan.arch,
    target: plan.target,
    runtimeHome: toPortableRelativePath(repoRoot, plan.runtimeHome),
    evidencePath: toPortableRelativePath(repoRoot, plan.evidencePath),
    healthUrls: plan.healthUrls,
  };

  if (stdoutLogExcerpt || stderrLogExcerpt) {
    evidence.logs = {};
    if (stdoutLogExcerpt) {
      evidence.logs.stdout = stdoutLogExcerpt;
    }
    if (stderrLogExcerpt) {
      evidence.logs.stderr = stderrLogExcerpt;
    }
  }

  if (!ok) {
    evidence.failure = {
      message: failure instanceof Error ? failure.message : String(failure ?? 'unknown error'),
    };
  }

  return evidence;
}

function writeWindowsInstalledRuntimeSmokeEvidence({
  evidencePath,
  evidence,
} = {}) {
  mkdirSync(path.dirname(evidencePath), { recursive: true });
  writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
}

export async function runWindowsInstalledRuntimeSmoke({
  repoRoot = rootDir,
  platform,
  arch,
  target,
  runtimeHome,
  evidencePath,
  env = process.env,
} = {}) {
  const ports = await allocateLoopbackPorts();
  const plan = createWindowsInstalledRuntimeSmokePlan({
    repoRoot,
    platform,
    arch,
    target,
    runtimeHome,
    evidencePath,
    env,
    ports,
  });

  let failure = null;

  try {
    assertInstallInputsExist(plan.installPlan);
    applyInstallPlan(plan.installPlan, {
      force: true,
    });
    writeFileSync(plan.routerEnvPath, plan.routerEnvContents, 'utf8');

    runScriptCommand(plan.startCommand.command, plan.startCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime start.ps1',
      plan,
    });

    await waitForHealthUrls(plan.healthUrls);

    runScriptCommand(plan.stopCommand.command, plan.stopCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime stop.ps1',
      plan,
    });

    const evidence = createWindowsInstalledRuntimeSmokeEvidence({
      repoRoot,
      plan,
      ok: true,
    });
    writeWindowsInstalledRuntimeSmokeEvidence({
      evidencePath: plan.evidencePath,
      evidence,
    });
    return evidence;
  } catch (error) {
    failure = error instanceof Error ? error : new Error(String(error));

    try {
      if (existsSync(plan.pidFilePath)) {
        runScriptCommand(plan.stopCommand.command, plan.stopCommand.args, {
          cwd: repoRoot,
          env,
          label: 'installed runtime stop.ps1',
          plan,
        });
      }
    } catch {
      // Preserve the original smoke failure; stop cleanup is best-effort.
    }

    const evidence = createWindowsInstalledRuntimeSmokeEvidence({
      repoRoot,
      plan,
      ok: false,
      failure,
    });
    writeWindowsInstalledRuntimeSmokeEvidence({
      evidencePath: plan.evidencePath,
      evidence,
    });
    throw failure;
  }
}

async function main() {
  const options = parseArgs();
  const evidence = await runWindowsInstalledRuntimeSmoke(options);
  console.log(JSON.stringify(evidence, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
