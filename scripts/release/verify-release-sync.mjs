#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');
const defaultReleaseSyncAuditPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-sync-audit-latest.json',
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

const RELEASE_SYNC_REPOSITORY_SPECS = Object.freeze([
  Object.freeze({
    id: 'sdkwork-api-router',
    targetDir: rootDir,
    expectedGitRoot: rootDir,
    expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
    envRefKey: 'SDKWORK_API_ROUTER_GIT_REF',
    defaultRef: 'main',
  }),
  Object.freeze({
    id: 'sdkwork-core',
    targetDir: path.resolve(rootDir, '..', 'sdkwork-core'),
    expectedGitRoot: path.resolve(rootDir, '..', 'sdkwork-core'),
    expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-core.git',
    envRefKey: 'SDKWORK_CORE_GIT_REF',
    defaultRef: 'main',
  }),
  Object.freeze({
    id: 'sdkwork-ui',
    targetDir: path.resolve(rootDir, '..', 'sdkwork-ui'),
    expectedGitRoot: path.resolve(rootDir, '..', 'sdkwork-ui'),
    expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-ui.git',
    envRefKey: 'SDKWORK_UI_GIT_REF',
    defaultRef: 'main',
  }),
  Object.freeze({
    id: 'sdkwork-appbase',
    targetDir: path.resolve(rootDir, '..', 'sdkwork-appbase'),
    expectedGitRoot: path.resolve(rootDir, '..', 'sdkwork-appbase'),
    expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-appbase.git',
    envRefKey: 'SDKWORK_APPBASE_GIT_REF',
    defaultRef: 'main',
  }),
  Object.freeze({
    id: 'sdkwork-craw-chat-sdk',
    targetDir: path.resolve(
      rootDir,
      '..',
      'craw-chat',
      'sdks',
      'sdkwork-craw-chat-sdk',
      'sdkwork-craw-chat-sdk-typescript',
    ),
    expectedGitRoot: path.resolve(rootDir, '..', 'craw-chat'),
    expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/craw-chat.git',
    envRefKey: 'SDKWORK_CRAW_CHAT_SDK_GIT_REF',
    defaultRef: 'main',
  }),
]);

export function listReleaseSyncRepositorySpecs() {
  return RELEASE_SYNC_REPOSITORY_SPECS.map((spec) => ({
    ...spec,
  }));
}

export function validateReleaseSyncAuditReport(report) {
  if (!isObject(report)) {
    throw new Error('release sync audit report must be a JSON object');
  }

  const requiredStringFields = [
    'id',
    'targetDir',
    'expectedGitRoot',
    'topLevel',
    'remoteUrl',
    'localHead',
    'remoteHead',
    'branch',
    'upstream',
  ];
  for (const field of requiredStringFields) {
    if (typeof report[field] !== 'string') {
      throw new Error(`release sync audit report must include string ${field}`);
    }
  }

  if (report.expectedRef !== undefined && typeof report.expectedRef !== 'string') {
    throw new Error('release sync audit report expectedRef must be a string when present');
  }

  if (!isNonNegativeInteger(report.ahead)) {
    throw new Error('release sync audit report must include numeric ahead');
  }

  if (!isNonNegativeInteger(report.behind)) {
    throw new Error('release sync audit report must include numeric behind');
  }

  if (typeof report.isDirty !== 'boolean') {
    throw new Error('release sync audit report must include boolean isDirty');
  }

  if (!Array.isArray(report.reasons) || report.reasons.some((reason) => typeof reason !== 'string')) {
    throw new Error('release sync audit report must include string-array reasons');
  }

  if (typeof report.releasable !== 'boolean') {
    throw new Error('release sync audit report must include boolean releasable');
  }

  if (report.releasable !== (report.reasons.length === 0)) {
    throw new Error('release sync audit report releasable must match reasons emptiness');
  }
}

export function validateReleaseSyncAuditSummary(summary) {
  if (!isObject(summary)) {
    throw new Error('release sync audit summary must be a JSON object');
  }

  if (typeof summary.releasable !== 'boolean') {
    throw new Error('release sync audit summary must include boolean releasable');
  }

  if (!Array.isArray(summary.reports)) {
    throw new Error('release sync audit summary must include array reports');
  }

  for (const report of summary.reports) {
    validateReleaseSyncAuditReport(report);
  }

  if (summary.releasable !== summary.reports.every((report) => report.releasable === true)) {
    throw new Error('release sync audit summary releasable must match report releasability');
  }
}

export function validateReleaseSyncAuditArtifact(artifact) {
  if (!isObject(artifact)) {
    throw new Error('release sync audit artifact must be a JSON object');
  }

  if (String(artifact.generatedAt ?? '').trim().length === 0) {
    throw new Error('release sync audit artifact must include generatedAt');
  }

  if (!isObject(artifact.source)) {
    throw new Error('release sync audit artifact must include a source object');
  }

  if (String(artifact.source.kind ?? '').trim().length === 0) {
    throw new Error('release sync audit artifact source must include kind');
  }

  validateReleaseSyncAuditSummary(artifact.summary);
}

function normalizeReleaseSyncAuditInputPayload(payload) {
  if (isObject(payload) && Object.hasOwn(payload, 'summary')) {
    validateReleaseSyncAuditArtifact(payload);
    return payload.summary;
  }

  validateReleaseSyncAuditSummary(payload);
  return payload;
}

export function resolveReleaseSyncAuditInput({
  auditPath,
  auditJson,
  preferDefaultArtifact = true,
  env = process.env,
  readFile = readFileSync,
} = {}) {
  const resolvedAuditPath = String(
    auditPath ?? env.SDKWORK_RELEASE_SYNC_AUDIT_PATH ?? '',
  ).trim();
  const resolvedAuditJson = String(
    auditJson ?? env.SDKWORK_RELEASE_SYNC_AUDIT_JSON ?? '',
  ).trim();

  if (resolvedAuditJson.length > 0) {
    return {
      source: 'json',
      summary: normalizeReleaseSyncAuditInputPayload(
        parseJson(resolvedAuditJson, 'release sync audit JSON'),
      ),
    };
  }

  if (resolvedAuditPath.length > 0) {
    return {
      source: 'file',
      summary: normalizeReleaseSyncAuditInputPayload(
        parseJson(
          readFile(resolvedAuditPath, 'utf8'),
          `release sync audit file ${resolvedAuditPath}`,
        ),
      ),
    };
  }

  if (preferDefaultArtifact && existsSync(defaultReleaseSyncAuditPath)) {
    return {
      source: 'default-file',
      summary: normalizeReleaseSyncAuditInputPayload(
        parseJson(
          readFile(defaultReleaseSyncAuditPath, 'utf8'),
          `release sync audit file ${defaultReleaseSyncAuditPath}`,
        ),
      ),
    };
  }

  return null;
}

export function resolveReleaseSyncRepositoryRef({
  spec,
  env = process.env,
} = {}) {
  const configuredRef = String(env?.[spec?.envRefKey] ?? '').trim();
  return configuredRef.length > 0 ? configuredRef : String(spec?.defaultRef ?? 'main');
}

function normalizePathForCompare(value) {
  return path.resolve(String(value ?? '')).replaceAll('\\', '/').toLowerCase();
}

function normalizeGitHubRemoteUrlForCompare(value = '') {
  const normalized = String(value ?? '').trim();
  if (normalized.length === 0) {
    return '';
  }

  const httpsMatch = normalized.match(/^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/iu);
  if (httpsMatch) {
    return `github.com/${httpsMatch[1].toLowerCase()}/${httpsMatch[2].toLowerCase()}`;
  }

  const sshMatch = normalized.match(/^git@github\.com:([^/]+)\/([^/]+?)(?:\.git)?$/iu);
  if (sshMatch) {
    return `github.com/${sshMatch[1].toLowerCase()}/${sshMatch[2].toLowerCase()}`;
  }

  const sshUrlMatch = normalized.match(/^ssh:\/\/git@github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/iu);
  if (sshUrlMatch) {
    return `github.com/${sshUrlMatch[1].toLowerCase()}/${sshUrlMatch[2].toLowerCase()}`;
  }

  return normalized;
}

export function parseGitStatusBranchSummary(statusText = '') {
  const lines = String(statusText ?? '')
    .split(/\r?\n/u)
    .filter((line) => line.length > 0);
  const branchLine = lines.find((line) => line.startsWith('## ')) ?? '';
  const changeLines = lines.filter((line) => line !== branchLine);

  let branch = '';
  let upstream = '';
  let ahead = 0;
  let behind = 0;

  if (branchLine) {
    let summary = branchLine.slice(3).trim();
    let trackingSummary = '';
    const trackingMatch = summary.match(/ \[(.+)\]$/u);
    if (trackingMatch) {
      trackingSummary = trackingMatch[1];
      summary = summary.slice(0, -trackingMatch[0].length);
    }

    const relationParts = summary.split('...');
    branch = relationParts[0]?.trim() ?? '';
    upstream = relationParts[1]?.trim() ?? '';

    for (const rawPart of trackingSummary.split(',')) {
      const part = rawPart.trim();
      const aheadMatch = part.match(/^ahead (\d+)$/u);
      if (aheadMatch) {
        ahead = Number.parseInt(aheadMatch[1], 10);
        continue;
      }

      const behindMatch = part.match(/^behind (\d+)$/u);
      if (behindMatch) {
        behind = Number.parseInt(behindMatch[1], 10);
      }
    }
  }

  return {
    branchLine,
    branch,
    upstream,
    ahead,
    behind,
    isDirty: changeLines.length > 0,
    changeLines,
    hasTrackingDivergence: ahead > 0 || behind > 0,
  };
}

function isTagLikeRef(ref = '') {
  const normalizedRef = String(ref ?? '').trim();
  return normalizedRef.startsWith('refs/tags/')
    || /^release-/u.test(normalizedRef);
}

export function parseRemoteHeadStdout(stdout = '') {
  const lines = String(stdout ?? '')
    .trim()
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

  const lastLine = lines.at(-1) ?? '';
  return lastLine.split(/\s+/u)[0] ?? '';
}

export function evaluateReleaseSyncRepositoryAudit({
  spec,
  expectedRef = 'main',
  topLevel = '',
  statusText = '',
  remoteUrl = '',
  localHead = '',
  remoteHeadResult = {
    ok: false,
    stdout: '',
    stderr: '',
  },
} = {}) {
  const statusSummary = parseGitStatusBranchSummary(statusText);
  const reasons = [];
  const requireTrackingBranch = !isTagLikeRef(expectedRef);

  if (!topLevel || normalizePathForCompare(topLevel) !== normalizePathForCompare(spec.expectedGitRoot)) {
    reasons.push('not-standalone-root');
  }

  if (statusSummary.isDirty) {
    reasons.push('dirty-working-tree');
  }

  if (
    (requireTrackingBranch && (!statusSummary.upstream || statusSummary.ahead > 0 || statusSummary.behind > 0))
    || (!requireTrackingBranch && (statusSummary.ahead > 0 || statusSummary.behind > 0))
  ) {
    reasons.push('branch-not-synced');
  }

  if (
    remoteUrl
    && spec.expectedRemoteUrl
    && normalizeGitHubRemoteUrlForCompare(remoteUrl) !== normalizeGitHubRemoteUrlForCompare(spec.expectedRemoteUrl)
  ) {
    reasons.push('remote-url-mismatch');
  }

  const remoteHead = remoteHeadResult.ok ? parseRemoteHeadStdout(remoteHeadResult.stdout) : '';
  if (!remoteHeadResult.ok) {
    reasons.push('remote-unverifiable');
  } else if (localHead && remoteHead && localHead.trim() !== remoteHead.trim()) {
    reasons.push('head-mismatch');
  }

  return {
    id: spec.id,
    targetDir: spec.targetDir,
    expectedGitRoot: spec.expectedGitRoot,
    topLevel,
    remoteUrl,
    localHead,
    remoteHead,
    expectedRef,
    branch: statusSummary.branch,
    upstream: statusSummary.upstream,
    ahead: statusSummary.ahead,
    behind: statusSummary.behind,
    isDirty: statusSummary.isDirty,
    reasons,
    releasable: reasons.length === 0,
  };
}

export function isReleaseSyncAuditPassing(reports = []) {
  return reports.every((report) => report.releasable === true);
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

export function isGitCommandExecutionBlocked(results = []) {
  return results.some((result) =>
    /(eperm|eacces)/i.test(String(result?.errorMessage ?? '')),
  );
}

function runGitCommand({
  cwd,
  args,
  spawnSyncImpl = spawnSync,
} = {}) {
  const runner = resolveGitRunner();
  const result = spawnSyncImpl(runner.command, args, {
    cwd,
    encoding: 'utf8',
    shell: runner.shell,
    stdio: 'pipe',
  });

  return {
    ok: !result.error && (result.status ?? 1) === 0,
    status: result.status ?? 1,
    stdout: String(result.stdout ?? ''),
    stderr: result.error ? String(result.error.message ?? '') : String(result.stderr ?? ''),
    errorMessage: result.error ? String(result.error.message ?? '') : '',
  };
}

function auditRepositorySpec(spec, {
  spawnSyncImpl = spawnSync,
} = {}) {
  if (!existsSync(spec.targetDir)) {
    return {
      id: spec.id,
      targetDir: spec.targetDir,
      expectedGitRoot: spec.expectedGitRoot,
      topLevel: '',
      remoteUrl: '',
      localHead: '',
      remoteHead: '',
      branch: '',
      upstream: '',
      ahead: 0,
      behind: 0,
      isDirty: false,
      reasons: ['missing-path'],
      releasable: false,
    };
  }

  const topLevelResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['rev-parse', '--show-toplevel'],
    spawnSyncImpl,
  });
  const statusResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['status', '--short', '--branch'],
    spawnSyncImpl,
  });
  const remoteUrlResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['remote', 'get-url', 'origin'],
    spawnSyncImpl,
  });
  const expectedRef = resolveReleaseSyncRepositoryRef({ spec });
  const localHeadResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['rev-parse', 'HEAD'],
    spawnSyncImpl,
  });
  const remoteHeadResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['ls-remote', 'origin', expectedRef],
    spawnSyncImpl,
  });

  if (isGitCommandExecutionBlocked([
    topLevelResult,
    statusResult,
    remoteUrlResult,
    localHeadResult,
    remoteHeadResult,
  ])) {
    return {
      id: spec.id,
      targetDir: spec.targetDir,
      expectedGitRoot: spec.expectedGitRoot,
      topLevel: '',
      remoteUrl: '',
      localHead: '',
      remoteHead: '',
      branch: '',
      upstream: '',
      ahead: 0,
      behind: 0,
      isDirty: false,
      reasons: ['command-exec-blocked'],
      releasable: false,
    };
  }

  return evaluateReleaseSyncRepositoryAudit({
    spec,
    expectedRef,
    topLevel: topLevelResult.ok ? topLevelResult.stdout.trim() : '',
    statusText: statusResult.stdout,
    remoteUrl: remoteUrlResult.ok ? remoteUrlResult.stdout.trim() : '',
    localHead: localHeadResult.ok ? localHeadResult.stdout.trim() : '',
    remoteHeadResult,
  });
}

export function auditReleaseSyncRepositories({
  specs = listReleaseSyncRepositorySpecs(),
  auditPath,
  auditJson,
  preferDefaultArtifact = true,
  env = process.env,
  readFile = readFileSync,
  spawnSyncImpl = spawnSync,
} = {}) {
  const resolvedInput = resolveReleaseSyncAuditInput({
    auditPath,
    auditJson,
    preferDefaultArtifact,
    env,
    readFile,
  });
  if (resolvedInput) {
    return resolvedInput.summary;
  }

  const reports = specs.map((spec) => auditRepositorySpec(spec, {
    spawnSyncImpl,
  }));
  return {
    releasable: isReleaseSyncAuditPassing(reports),
    reports,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  let format = 'text';
  let auditPath = '';
  let auditJson = '';
  let live = false;

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--format') {
      format = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--audit') {
      auditPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--audit-json') {
      auditJson = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--live') {
      live = true;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!['text', 'json'].includes(format)) {
    throw new Error(`unsupported format: ${format}`);
  }

  return {
    format,
    auditPath,
    auditJson,
    live,
  };
}

export function formatReleaseSyncTextReport(reports = []) {
  return reports.map((report) => {
    const state = report.releasable ? 'PASS' : 'BLOCK';
    const reasons = report.reasons.length > 0 ? report.reasons.join(', ') : 'none';
    return `[verify-release-sync] ${state} ${report.id} branch=${report.branch || 'unknown'} upstream=${report.upstream || 'missing'} dirty=${report.isDirty} reasons=${reasons}`;
  }).join('\n');
}

function main() {
  const { format, auditPath, auditJson, live } = parseArgs();
  const summary = auditReleaseSyncRepositories({
    auditPath,
    auditJson,
    preferDefaultArtifact: !live,
  });

  if (format === 'json') {
    console.log(JSON.stringify(summary, null, 2));
  } else {
    const reportText = formatReleaseSyncTextReport(summary.reports);
    if (reportText) {
      console.error(reportText);
    }
  }

  if (!summary.releasable) {
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
