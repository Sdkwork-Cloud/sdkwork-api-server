#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { buildDesktopReleaseEnv } from './desktop-targets.mjs';
import {
  buildDesktopTargetTriple,
  normalizeDesktopPlatform,
} from './desktop-targets.mjs';
import {
  resolveAvailableNativeBuildRoot,
  resolveNativeBuildRootCandidates,
} from './package-release-assets.mjs';
import { withSupportedWindowsCmakeGenerator } from '../run-tauri-cli.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DESKTOP_APP_DIRS = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
  console: path.join(rootDir, 'console'),
};

export function resolveDesktopAppDir(appId) {
  const appDir = DESKTOP_APP_DIRS[appId];
  if (!appDir) {
    throw new Error(`Unsupported desktop application id: ${appId}`);
  }

  return appDir;
}

export function createDesktopReleaseBuildPlan({
  appId,
  appDir = resolveDesktopAppDir(appId),
  platform = process.platform,
  arch = process.arch,
  env = process.env,
  targetTriple = '',
} = {}) {
  const requestedTargetTriple = String(targetTriple ?? '').trim();
  const args = ['tauri:build'];
  const resolvedEnv = requestedTargetTriple
    ? buildDesktopReleaseEnv({
        env,
        targetTriple: requestedTargetTriple,
      })
    : { ...env };

  if (shouldPassExplicitDesktopReleaseTarget({
    targetTriple: requestedTargetTriple,
    platform,
    arch,
  })) {
    args.push('--target', requestedTargetTriple);
  }

  const bundles = resolveDesktopReleaseBundles({ platform });
  if (bundles.length > 0) {
    args.push('--bundles', bundles.join(','));
  }

  if (String(resolvedEnv.GITHUB_ACTIONS ?? '').trim().toLowerCase() === 'true') {
    args.push('--verbose');
  }

  return {
    command: 'pnpm',
    args,
    cwd: appDir,
    env: withSupportedWindowsCmakeGenerator(resolvedEnv, platform),
  };
}

export function resolveDesktopReleaseBundles({
  platform = process.platform,
} = {}) {
  const normalizedPlatform = normalizeDesktopPlatform(platform);

  if (normalizedPlatform === 'windows') {
    return ['nsis'];
  }

  if (normalizedPlatform === 'linux') {
    return ['deb'];
  }

  return ['dmg'];
}

export function shouldPassExplicitDesktopReleaseTarget({
  targetTriple = '',
  platform = process.platform,
  arch = process.arch,
} = {}) {
  const requestedTargetTriple = String(targetTriple ?? '').trim();
  if (!requestedTargetTriple) {
    return false;
  }

  const hostTargetTriple = buildDesktopTargetTriple({
    platform,
    arch,
  });
  return requestedTargetTriple !== hostTargetTriple;
}

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `[truncated]${text.slice(-Math.max(0, maxLength - 11))}`;
}

function escapeGitHubActionsCommandValue(value, { property = false } = {}) {
  let escaped = String(value ?? '');
  escaped = escaped.replaceAll('%', '%25');
  escaped = escaped.replaceAll('\r', '%0D');
  escaped = escaped.replaceAll('\n', '%0A');
  if (property) {
    escaped = escaped.replaceAll(':', '%3A');
    escaped = escaped.replaceAll(',', '%2C');
  }

  return escaped;
}

export function buildDesktopReleaseFailureAnnotation({
  appId = '',
  targetTriple = '',
  error,
} = {}) {
  const scope = [appId, targetTriple].filter(Boolean).join(' ');
  const message = truncateText(
    `${scope ? `[${scope}] ` : ''}${error instanceof Error ? error.message : String(error)}`,
    8000,
  );
  return `::error title=run-desktop-release-build::${escapeGitHubActionsCommandValue(message)}`;
}

function parseCliArgs(argv) {
  const options = {
    appId: '',
    targetTriple: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--app') {
      options.appId = String(next ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--target') {
      options.targetTriple = String(next ?? '').trim();
      index += 1;
    }
  }

  return options;
}

function appendBufferedOutput(buffer, chunk, maxLength = 6000) {
  const text = Buffer.isBuffer(chunk) ? chunk.toString('utf8') : String(chunk ?? '');
  const next = `${buffer}${text}`;
  if (next.length <= maxLength) {
    return next;
  }

  return next.slice(-maxLength);
}

function describeBuildRootState(root) {
  if (!existsSync(root)) {
    return `${root} [missing]`;
  }

  const entries = readdirSync(root, { withFileTypes: true });
  if (entries.length === 0) {
    return `${root} [exists, empty]`;
  }

  const sample = entries.slice(0, 8).map((entry) => entry.name).join(', ');
  const remainingCount = entries.length - Math.min(entries.length, 8);
  const remainingSuffix = remainingCount > 0 ? ` (+${remainingCount} more)` : '';
  return `${root} [exists: ${sample}${remainingSuffix}]`;
}

function verifyDesktopBundleOutput({ appId, targetTriple }) {
  const buildRoots = resolveNativeBuildRootCandidates({
    appId,
    targetTriple,
  });
  const buildRoot = resolveAvailableNativeBuildRoot({
    appId,
    targetTriple,
    buildRoots,
  });
  if (!buildRoot) {
    throw new Error(
      `Tauri build completed without bundle output for ${appId}. candidates: ${buildRoots.map((root) => describeBuildRootState(root)).join(' | ')}`,
    );
  }

  console.error(`[run-desktop-release-build] bundle root: ${buildRoot}`);
}

function buildDesktopReleaseFailure({
  reason,
  stdoutBuffer,
  stderrBuffer,
} = {}) {
  const details = [reason];
  const stderrTail = truncateText(stderrBuffer, 2000);
  const stdoutTail = truncateText(stdoutBuffer, 2000);
  if (stderrTail) {
    details.push(`stderr tail:\n${stderrTail}`);
  }
  if (stdoutTail) {
    details.push(`stdout tail:\n${stdoutTail}`);
  }

  return new Error(details.join('\n'));
}

function reportDesktopReleaseFailure({ appId, targetTriple, error }) {
  if (process.env.GITHUB_ACTIONS === 'true') {
    console.error(buildDesktopReleaseFailureAnnotation({
      appId,
      targetTriple,
      error,
    }));
  }
  console.error(error instanceof Error ? error.stack ?? error.message : String(error));
}

function runCli() {
  const options = parseCliArgs(process.argv.slice(2));
  const plan = createDesktopReleaseBuildPlan({
    appId: options.appId,
    targetTriple: options.targetTriple,
  });
  let stdoutBuffer = '';
  let stderrBuffer = '';
  const child = spawn(plan.command, plan.args, {
    cwd: plan.cwd,
    env: plan.env,
    stdio: ['inherit', 'pipe', 'pipe'],
    shell: process.platform === 'win32',
  });

  child.stdout?.on('data', (chunk) => {
    stdoutBuffer = appendBufferedOutput(stdoutBuffer, chunk);
    process.stdout.write(chunk);
  });

  child.stderr?.on('data', (chunk) => {
    stderrBuffer = appendBufferedOutput(stderrBuffer, chunk);
    process.stderr.write(chunk);
  });

  child.on('error', (error) => {
    reportDesktopReleaseFailure({
      appId: options.appId,
      targetTriple: options.targetTriple,
      error: buildDesktopReleaseFailure({
        reason: `[run-desktop-release-build] ${error.message}`,
        stdoutBuffer,
        stderrBuffer,
      }),
    });
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      reportDesktopReleaseFailure({
        appId: options.appId,
        targetTriple: options.targetTriple,
        error: buildDesktopReleaseFailure({
          reason: `[run-desktop-release-build] build exited with signal ${signal}`,
          stdoutBuffer,
          stderrBuffer,
        }),
      });
      process.exit(1);
      return;
    }

    if ((code ?? 0) !== 0) {
      reportDesktopReleaseFailure({
        appId: options.appId,
        targetTriple: options.targetTriple,
        error: buildDesktopReleaseFailure({
          reason: `[run-desktop-release-build] build exited with code ${code}`,
          stdoutBuffer,
          stderrBuffer,
        }),
      });
      process.exit(code ?? 1);
      return;
    }

    try {
      verifyDesktopBundleOutput({
        appId: options.appId,
        targetTriple: options.targetTriple,
      });
    } catch (error) {
      reportDesktopReleaseFailure({
        appId: options.appId,
        targetTriple: options.targetTriple,
        error,
      });
      process.exit(1);
      return;
    }

    process.exit(0);
  });
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  runCli();
}
