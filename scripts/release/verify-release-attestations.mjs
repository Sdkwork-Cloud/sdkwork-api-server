#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_REPOSITORY_SLUG = 'Sdkwork-Cloud/sdkwork-api-router';

const RELEASE_ATTESTATION_SUBJECT_SPECS = Object.freeze([
  Object.freeze({
    id: 'release-window-snapshot',
    type: 'file',
    relativePath: path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    description: 'governed release-window snapshot',
  }),
  Object.freeze({
    id: 'release-sync-audit',
    type: 'file',
    relativePath: path.join('docs', 'release', 'release-sync-audit-latest.json'),
    description: 'governed release-sync audit',
  }),
  Object.freeze({
    id: 'release-telemetry-export',
    type: 'file',
    relativePath: path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    description: 'governed release telemetry export',
  }),
  Object.freeze({
    id: 'release-telemetry-snapshot',
    type: 'file',
    relativePath: path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    description: 'governed release telemetry snapshot',
  }),
  Object.freeze({
    id: 'release-slo-governance',
    type: 'file',
    relativePath: path.join('docs', 'release', 'slo-governance-latest.json'),
    description: 'governed SLO evidence',
  }),
  Object.freeze({
    id: 'unix-installed-runtime-smoke',
    type: 'pattern',
    relativeDirectory: path.join('artifacts', 'release-governance'),
    fileNamePattern: /^unix-installed-runtime-smoke-.*\.json$/u,
    description: 'Unix installed runtime smoke evidence',
  }),
  Object.freeze({
    id: 'windows-installed-runtime-smoke',
    type: 'pattern',
    relativeDirectory: path.join('artifacts', 'release-governance'),
    fileNamePattern: /^windows-installed-runtime-smoke-.*\.json$/u,
    description: 'Windows installed runtime smoke evidence',
  }),
  Object.freeze({
    id: 'release-assets',
    type: 'tree',
    relativeDirectory: path.join('artifacts', 'release'),
    description: 'packaged release assets',
  }),
]);

function toPortableRelativePath(repoRoot, targetPath) {
  return (path.relative(repoRoot, targetPath) || '.').replaceAll('\\', '/');
}

function listFilesRecursively(directoryPath) {
  if (!existsSync(directoryPath)) {
    return [];
  }

  const entries = readdirSync(directoryPath, {
    withFileTypes: true,
  });

  const files = [];
  for (const entry of entries) {
    const entryPath = path.join(directoryPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursively(entryPath));
      continue;
    }

    if (entry.isFile()) {
      files.push(entryPath);
    }
  }

  return files.sort((left, right) => left.localeCompare(right));
}

function createMissingSubjectSpec(spec, repoRoot) {
  const expectedPath = spec.type === 'file'
    ? path.resolve(repoRoot, spec.relativePath)
    : path.resolve(repoRoot, spec.relativeDirectory);

  return {
    ...spec,
    expectedPath,
    expectedRelativePath: toPortableRelativePath(repoRoot, expectedPath),
  };
}

function createSubject(spec, repoRoot, subjectPath) {
  const relativeSubjectPath = toPortableRelativePath(repoRoot, subjectPath);
  return {
    id: spec.type === 'file'
      ? spec.id
      : `${spec.id}:${relativeSubjectPath}`,
    specId: spec.id,
    description: spec.description,
    subjectPath,
    relativeSubjectPath,
  };
}

function createMissingSubjectReport(spec) {
  return {
    id: spec.id,
    specId: spec.id,
    description: spec.description,
    ok: false,
    blocked: true,
    reason: 'subject-path-missing',
    subjectPath: '',
    relativeSubjectPath: '',
    expectedPath: spec.expectedPath,
    expectedRelativePath: spec.expectedRelativePath,
    stdout: '',
    stderr: '',
    errorMessage: '',
  };
}

function createBlockedExecutionReport(subject, reason, errorMessage = '') {
  return {
    id: subject.id,
    specId: subject.specId,
    description: subject.description,
    ok: false,
    blocked: true,
    reason,
    subjectPath: subject.subjectPath,
    relativeSubjectPath: subject.relativeSubjectPath,
    expectedPath: '',
    expectedRelativePath: '',
    stdout: '',
    stderr: '',
    errorMessage,
  };
}

function createVerificationReport(subject, result) {
  const stdout = String(result?.stdout ?? '');
  const stderr = String(result?.stderr ?? '');
  const errorMessage = result?.error ? String(result.error.message ?? '') : '';
  const ok = !result?.error && (result?.status ?? 1) === 0;

  return {
    id: subject.id,
    specId: subject.specId,
    description: subject.description,
    ok,
    blocked: false,
    reason: ok ? '' : 'attestation-verify-failed',
    subjectPath: subject.subjectPath,
    relativeSubjectPath: subject.relativeSubjectPath,
    expectedPath: '',
    expectedRelativePath: '',
    stdout,
    stderr,
    errorMessage,
  };
}

function summarizeReleaseAttestationReports({
  repoSlug,
  reports,
} = {}) {
  const verifiedIds = [];
  const blockedIds = [];
  const failingIds = [];

  for (const report of reports) {
    if (report.ok) {
      verifiedIds.push(report.id);
      continue;
    }

    if (report.blocked) {
      blockedIds.push(report.id);
      continue;
    }

    failingIds.push(report.id);
  }

  const blocked = failingIds.length === 0 && blockedIds.length > 0;
  const ok = failingIds.length === 0 && blockedIds.length === 0;
  const reason = ok
    ? ''
    : failingIds.length > 0
      ? 'attestation-verify-failed'
      : reports.find((report) => report.blocked)?.reason ?? 'subject-path-missing';

  return {
    ok,
    blocked,
    reason,
    repoSlug,
    verifiedCount: verifiedIds.length,
    blockedCount: blockedIds.length,
    failedCount: failingIds.length,
    verifiedIds,
    blockedIds,
    failingIds,
    reports,
  };
}

function classifyGhExecutionBlock(errorMessage = '') {
  const normalized = String(errorMessage ?? '').trim();
  if (/enoent/i.test(normalized)) {
    return 'gh-cli-missing';
  }
  if (/(eperm|eacces)/i.test(normalized)) {
    return 'command-exec-blocked';
  }
  return '';
}

export function listReleaseAttestationSubjectSpecs() {
  return RELEASE_ATTESTATION_SUBJECT_SPECS.map((spec) => ({ ...spec }));
}

export function resolveReleaseAttestationRepositorySlug({
  env = process.env,
  repoSlug = '',
} = {}) {
  const configured = String(
    repoSlug
    || env?.SDKWORK_RELEASE_ATTESTATION_REPOSITORY
    || env?.GITHUB_REPOSITORY
    || '',
  ).trim();

  return configured.length > 0 ? configured : DEFAULT_REPOSITORY_SLUG;
}

export function resolveReleaseAttestationVerificationSubjects({
  repoRoot = rootDir,
} = {}) {
  const subjects = [];
  const missingSubjectSpecs = [];

  for (const spec of RELEASE_ATTESTATION_SUBJECT_SPECS) {
    if (spec.type === 'file') {
      const subjectPath = path.resolve(repoRoot, spec.relativePath);
      if (!existsSync(subjectPath)) {
        missingSubjectSpecs.push(createMissingSubjectSpec(spec, repoRoot));
        continue;
      }

      subjects.push(createSubject(spec, repoRoot, subjectPath));
      continue;
    }

    if (spec.type === 'pattern') {
      const directoryPath = path.resolve(repoRoot, spec.relativeDirectory);
      const matches = listFilesRecursively(directoryPath)
        .filter((subjectPath) => spec.fileNamePattern.test(path.basename(subjectPath)));
      if (matches.length === 0) {
        missingSubjectSpecs.push(createMissingSubjectSpec(spec, repoRoot));
        continue;
      }

      subjects.push(...matches.map((subjectPath) => createSubject(spec, repoRoot, subjectPath)));
      continue;
    }

    if (spec.type === 'tree') {
      const directoryPath = path.resolve(repoRoot, spec.relativeDirectory);
      const matches = listFilesRecursively(directoryPath);
      if (matches.length === 0) {
        missingSubjectSpecs.push(createMissingSubjectSpec(spec, repoRoot));
        continue;
      }

      subjects.push(...matches.map((subjectPath) => createSubject(spec, repoRoot, subjectPath)));
    }
  }

  return {
    subjects,
    missingSubjectSpecs,
  };
}

export function resolveGhRunner({
  platform = process.platform,
} = {}) {
  if (platform === 'win32') {
    return {
      command: 'gh.exe',
      shell: true,
    };
  }

  return {
    command: 'gh',
    shell: false,
  };
}

export function createReleaseAttestationVerificationPlan({
  repoRoot = rootDir,
  env = process.env,
  repoSlug = '',
} = {}) {
  const resolvedRepoSlug = resolveReleaseAttestationRepositorySlug({
    env,
    repoSlug,
  });
  const runner = resolveGhRunner();
  const discovery = resolveReleaseAttestationVerificationSubjects({
    repoRoot,
  });

  return {
    repoSlug: resolvedRepoSlug,
    subjects: discovery.subjects,
    missingSubjectSpecs: discovery.missingSubjectSpecs,
    commands: discovery.subjects.map((subject) => ({
      id: subject.id,
      specId: subject.specId,
      description: subject.description,
      subjectPath: subject.subjectPath,
      relativeSubjectPath: subject.relativeSubjectPath,
      command: runner.command,
      args: [
        'attestation',
        'verify',
        subject.subjectPath,
        '--repo',
        resolvedRepoSlug,
      ],
      shell: runner.shell,
    })),
  };
}

export function verifyReleaseAttestations({
  repoRoot = rootDir,
  env = process.env,
  repoSlug = '',
  spawnSyncImpl = spawnSync,
} = {}) {
  const plan = createReleaseAttestationVerificationPlan({
    repoRoot,
    env,
    repoSlug,
  });
  const reports = plan.missingSubjectSpecs.map((spec) => createMissingSubjectReport(spec));

  if (plan.commands.length === 0) {
    return summarizeReleaseAttestationReports({
      repoSlug: plan.repoSlug,
      reports,
    });
  }

  for (let index = 0; index < plan.commands.length; index += 1) {
    const commandPlan = plan.commands[index];
    const result = spawnSyncImpl(commandPlan.command, commandPlan.args, {
      cwd: repoRoot,
      encoding: 'utf8',
      shell: commandPlan.shell,
      stdio: 'pipe',
      env,
    });

    const errorMessage = result?.error ? String(result.error.message ?? '') : '';
    const blockedReason = classifyGhExecutionBlock(errorMessage);
    if (blockedReason) {
      const remainingCommands = plan.commands.slice(index);
      reports.push(
        ...remainingCommands.map((remainingCommand) =>
          createBlockedExecutionReport(remainingCommand, blockedReason, errorMessage)),
      );
      return summarizeReleaseAttestationReports({
        repoSlug: plan.repoSlug,
        reports,
      });
    }

    reports.push(createVerificationReport(commandPlan, result));
  }

  return summarizeReleaseAttestationReports({
    repoSlug: plan.repoSlug,
    reports,
  });
}

function parseArgs(argv = process.argv.slice(2)) {
  let format = 'text';
  let repoSlug = '';

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--format') {
      format = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--repo') {
      repoSlug = String(argv[index + 1] ?? '').trim();
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
    repoSlug,
  };
}

function formatReleaseAttestationVerificationResultText(result) {
  const lines = [
    `[release-attestations] repo=${result.repoSlug}`,
    `[release-attestations] verified=${result.verifiedCount}`,
    `[release-attestations] blocked=${result.blockedCount}`,
    `[release-attestations] failed=${result.failedCount}`,
  ];

  for (const report of result.reports) {
    const state = report.ok ? 'VERIFIED' : report.blocked ? 'BLOCK' : 'FAIL';
    const location = report.relativeSubjectPath
      || report.expectedRelativePath
      || report.subjectPath
      || report.expectedPath
      || 'unknown';
    lines.push(`[release-attestations] ${state} ${report.specId} ${location}`);
    if (!report.ok) {
      lines.push(`[release-attestations] reason=${report.reason}`);
    }
  }

  return lines.join('\n');
}

function main() {
  const { format, repoSlug } = parseArgs();
  const result = verifyReleaseAttestations({
    repoSlug,
  });

  if (format === 'json') {
    console.log(JSON.stringify(result, null, 2));
  } else {
    console.log(formatReleaseAttestationVerificationResultText(result));
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
