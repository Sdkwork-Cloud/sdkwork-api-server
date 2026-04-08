#!/usr/bin/env node

import { existsSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');
const defaultReleaseWindowSnapshotPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-window-snapshot-latest.json',
);

function parseJson(text, context) {
  try {
    return JSON.parse(String(text).replace(/^\uFEFF/u, ''));
  } catch (error) {
    throw new Error(
      `invalid ${context}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function isObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function isNonNegativeInteger(value) {
  return Number.isInteger(value) && value >= 0;
}

export function validateReleaseWindowSnapshot(snapshot) {
  if (!isObject(snapshot)) {
    throw new Error('release window snapshot must be a JSON object');
  }

  if (typeof snapshot.latestReleaseTag !== 'string') {
    throw new Error('release window snapshot must include string latestReleaseTag');
  }

  if (snapshot.commitsSinceLatestRelease !== null && !isNonNegativeInteger(snapshot.commitsSinceLatestRelease)) {
    throw new Error('release window snapshot must include numeric commitsSinceLatestRelease or null');
  }

  if (!isNonNegativeInteger(snapshot.workingTreeEntryCount)) {
    throw new Error('release window snapshot must include numeric workingTreeEntryCount');
  }

  if (typeof snapshot.hasReleaseBaseline !== 'boolean') {
    throw new Error('release window snapshot must include boolean hasReleaseBaseline');
  }

  if (!snapshot.hasReleaseBaseline && snapshot.latestReleaseTag.length > 0) {
    throw new Error('release window snapshot cannot carry latestReleaseTag when hasReleaseBaseline is false');
  }

  if (!snapshot.hasReleaseBaseline && snapshot.commitsSinceLatestRelease !== null) {
    throw new Error('release window snapshot cannot carry commitsSinceLatestRelease when hasReleaseBaseline is false');
  }

  return {
    hasReleaseBaseline: snapshot.hasReleaseBaseline,
  };
}

export function validateReleaseWindowSnapshotArtifact(artifact) {
  if (!isObject(artifact)) {
    throw new Error('release window snapshot artifact must be a JSON object');
  }

  if (String(artifact.generatedAt ?? '').trim().length === 0) {
    throw new Error('release window snapshot artifact must include generatedAt');
  }

  if (!isObject(artifact.source)) {
    throw new Error('release window snapshot artifact must include a source object');
  }

  if (String(artifact.source.kind ?? '').trim().length === 0) {
    throw new Error('release window snapshot artifact source must include kind');
  }

  validateReleaseWindowSnapshot(artifact.snapshot);
  return {
    sourceKind: String(artifact.source.kind),
  };
}

function normalizeReleaseWindowSnapshotInputPayload(payload) {
  if (isObject(payload) && Object.hasOwn(payload, 'snapshot')) {
    validateReleaseWindowSnapshotArtifact(payload);
    return payload.snapshot;
  }

  validateReleaseWindowSnapshot(payload);
  return payload;
}

export function resolveReleaseWindowSnapshotInput({
  snapshotPath,
  snapshotJson,
  env = process.env,
  readFile = readFileSync,
} = {}) {
  const resolvedSnapshotPath = String(
    snapshotPath ?? env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH ?? '',
  ).trim();
  const resolvedSnapshotJson = String(
    snapshotJson ?? env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON ?? '',
  ).trim();

  if (resolvedSnapshotJson.length > 0) {
    return {
      source: 'json',
      snapshot: normalizeReleaseWindowSnapshotInputPayload(
        parseJson(resolvedSnapshotJson, 'release window snapshot JSON'),
      ),
    };
  }

  if (resolvedSnapshotPath.length > 0) {
    return {
      source: 'file',
      snapshot: normalizeReleaseWindowSnapshotInputPayload(
        parseJson(
          readFile(resolvedSnapshotPath, 'utf8'),
          `release window snapshot file ${resolvedSnapshotPath}`,
        ),
      ),
    };
  }

  if (existsSync(defaultReleaseWindowSnapshotPath)) {
    return {
      source: 'default-file',
      snapshot: normalizeReleaseWindowSnapshotInputPayload(
        parseJson(
          readFile(defaultReleaseWindowSnapshotPath, 'utf8'),
          `release window snapshot file ${defaultReleaseWindowSnapshotPath}`,
        ),
      ),
    };
  }

  return null;
}

export function findLatestReleaseTag(tagListOutput = '') {
  return String(tagListOutput ?? '')
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .find((line) => line.length > 0) ?? '';
}

export function countWorkingTreeEntries(statusText = '') {
  return String(statusText ?? '')
    .split(/\r?\n/u)
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0)
    .length;
}

export function resolveGitRunner({
  platform = process.platform,
} = {}) {
  if (platform === 'win32') {
    return {
      command: 'git.exe',
      shell: false,
    };
  }

  return {
    command: 'git',
    shell: false,
  };
}

function runGitCommand({
  args,
  spawnSyncImpl = spawnSync,
} = {}) {
  const runner = resolveGitRunner();
  const result = spawnSyncImpl(runner.command, args, {
    cwd: rootDir,
    encoding: 'utf8',
    shell: runner.shell,
    stdio: 'pipe',
  });

  if (result.error) {
    throw new Error(`git ${args.join(' ')} failed: ${result.error.message}`);
  }

  if ((result.status ?? 0) !== 0) {
    const stdout = String(result.stdout ?? '').trim();
    const stderr = String(result.stderr ?? '').trim();
    const details = [stdout, stderr].filter(Boolean).join('\n');
    throw new Error(
      `git ${args.join(' ')} exited with code ${result.status ?? 'unknown'}${details ? `\n${details}` : ''}`,
    );
  }

  return String(result.stdout ?? '');
}

export function isGitCommandExecutionBlockedError(error) {
  return /(eperm|eacces)/i.test(String(error instanceof Error ? error.message : error ?? ''));
}

export function collectReleaseWindowSnapshot({
  spawnSyncImpl = spawnSync,
} = {}) {
  const tagListOutput = runGitCommand({
    args: ['tag', '--list', 'release-*', '--sort=-creatordate'],
    spawnSyncImpl,
  });
  const latestReleaseTag = findLatestReleaseTag(tagListOutput);

  const statusOutput = runGitCommand({
    args: ['status', '--short'],
    spawnSyncImpl,
  });
  const workingTreeEntryCount = countWorkingTreeEntries(statusOutput);

  let commitsSinceLatestRelease = null;
  if (latestReleaseTag) {
    const revListOutput = runGitCommand({
      args: ['rev-list', '--count', `${latestReleaseTag}..HEAD`],
      spawnSyncImpl,
    });
    commitsSinceLatestRelease = Number.parseInt(revListOutput.trim(), 10);
  }

  return {
    latestReleaseTag,
    commitsSinceLatestRelease: Number.isFinite(commitsSinceLatestRelease)
      ? commitsSinceLatestRelease
      : null,
    workingTreeEntryCount,
    hasReleaseBaseline: latestReleaseTag.length > 0,
  };
}

export function collectReleaseWindowSnapshotResult({
  snapshotPath,
  snapshotJson,
  env = process.env,
  readFile = readFileSync,
  spawnSyncImpl = spawnSync,
} = {}) {
  try {
    const resolvedInput = resolveReleaseWindowSnapshotInput({
      snapshotPath,
      snapshotJson,
      env,
      readFile,
    });
    if (resolvedInput) {
      return {
        ok: true,
        blocked: false,
        reason: '',
        errorMessage: '',
        snapshot: resolvedInput.snapshot,
      };
    }

    return {
      ok: true,
      blocked: false,
      reason: '',
      errorMessage: '',
      snapshot: collectReleaseWindowSnapshot({
        spawnSyncImpl,
      }),
    };
  } catch (error) {
    if (isGitCommandExecutionBlockedError(error)) {
      return {
        ok: false,
        blocked: true,
        reason: 'command-exec-blocked',
        errorMessage: error instanceof Error ? error.message : String(error),
        snapshot: null,
      };
    }

    throw error;
  }
}

function parseArgs(argv = process.argv.slice(2)) {
  let format = 'text';
  let snapshotPath = '';
  let snapshotJson = '';

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--format') {
      format = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--snapshot') {
      snapshotPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--snapshot-json') {
      snapshotJson = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!['text', 'json'].includes(format)) {
    throw new Error(`unsupported format: ${format}`);
  }

  return {
    format,
    snapshotPath,
    snapshotJson,
  };
}

function formatReleaseWindowSnapshotText(snapshot) {
  const baseline = snapshot.hasReleaseBaseline
    ? snapshot.latestReleaseTag
    : 'missing';
  const commits = snapshot.commitsSinceLatestRelease ?? 'unknown';
  return [
    `[release-window-snapshot] latest_release_tag=${baseline}`,
    `[release-window-snapshot] commits_since_latest_release=${commits}`,
    `[release-window-snapshot] working_tree_entry_count=${snapshot.workingTreeEntryCount}`,
  ].join('\n');
}

function formatReleaseWindowSnapshotResultText(result) {
  if (result.ok) {
    return formatReleaseWindowSnapshotText(result.snapshot);
  }

  return [
    `[release-window-snapshot] blocked=${result.blocked}`,
    `[release-window-snapshot] reason=${result.reason || 'unknown'}`,
    `[release-window-snapshot] error=${result.errorMessage || 'unknown'}`,
  ].join('\n');
}

function main() {
  const { format, snapshotPath, snapshotJson } = parseArgs();
  const result = collectReleaseWindowSnapshotResult({
    snapshotPath,
    snapshotJson,
  });

  if (format === 'json') {
    console.log(JSON.stringify(result, null, 2));
  } else {
    console.log(formatReleaseWindowSnapshotResultText(result));
  }

  if (!result.ok) {
    process.exit(1);
  }
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  }
}
