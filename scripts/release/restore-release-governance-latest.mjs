#!/usr/bin/env node

import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  validateReleaseWindowSnapshotArtifact,
} from './compute-release-window-snapshot.mjs';
import {
  validateReleaseTelemetryExportShape,
  validateReleaseTelemetrySnapshotShape,
} from './materialize-release-telemetry-snapshot.mjs';
import {
  validateSloGovernanceEvidenceShape,
} from './materialize-slo-governance-evidence.mjs';
import {
  validateReleaseSyncAuditArtifact,
} from './verify-release-sync.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

function parseJson(text, context) {
  try {
    return JSON.parse(String(text).replace(/^\uFEFF/u, ''));
  } catch (error) {
    throw new Error(
      `invalid ${context}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
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

function canonicalizeJson(value) {
  if (Array.isArray(value)) {
    return value.map((entry) => canonicalizeJson(entry));
  }

  if (!value || typeof value !== 'object') {
    return value;
  }

  return Object.fromEntries(
    Object.keys(value)
      .sort((left, right) => left.localeCompare(right))
      .map((key) => [key, canonicalizeJson(value[key])]),
  );
}

function validateGovernanceArtifact(spec, payload) {
  if (spec.id === 'release-window-snapshot') {
    validateReleaseWindowSnapshotArtifact(payload);
    return;
  }

  if (spec.id === 'release-sync-audit') {
    validateReleaseSyncAuditArtifact(payload);
    return;
  }

  if (spec.id === 'release-telemetry-export') {
    validateReleaseTelemetryExportShape({
      exportBundle: payload,
    });
    return;
  }

  if (spec.id === 'release-telemetry-snapshot') {
    validateReleaseTelemetrySnapshotShape({
      snapshot: payload,
    });
    return;
  }

  if (spec.id === 'release-slo-governance') {
    validateSloGovernanceEvidenceShape({
      evidence: payload,
    });
    return;
  }

  throw new Error(`unsupported release governance artifact spec: ${spec.id}`);
}

export function listReleaseGovernanceLatestArtifactSpecs() {
  return [
    {
      id: 'release-window-snapshot',
      optionKey: 'windowPath',
      relativePath: path.join('docs', 'release', 'release-window-snapshot-latest.json'),
      description: 'governed release-window snapshot',
    },
    {
      id: 'release-sync-audit',
      optionKey: 'syncPath',
      relativePath: path.join('docs', 'release', 'release-sync-audit-latest.json'),
      description: 'governed release-sync audit',
    },
    {
      id: 'release-telemetry-export',
      optionKey: 'telemetryExportPath',
      relativePath: path.join('docs', 'release', 'release-telemetry-export-latest.json'),
      description: 'governed release telemetry export',
    },
    {
      id: 'release-telemetry-snapshot',
      optionKey: 'telemetrySnapshotPath',
      relativePath: path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
      description: 'governed release telemetry snapshot',
    },
    {
      id: 'release-slo-governance',
      optionKey: 'sloPath',
      relativePath: path.join('docs', 'release', 'slo-governance-latest.json'),
      description: 'governed SLO evidence',
    },
  ].map((spec) => ({
    ...spec,
    fileName: path.basename(spec.relativePath),
    portableRelativePath: toPortablePath(spec.relativePath),
  }));
}

function resolveArtifactCandidates({
  spec,
  artifactDir,
  options = {},
  allArtifactFiles = [],
} = {}) {
  const explicitPath = String(options?.[spec.optionKey] ?? '').trim();
  if (explicitPath.length > 0) {
    return [explicitPath];
  }

  if (String(artifactDir ?? '').trim().length === 0) {
    return [];
  }

  const matches = allArtifactFiles.filter((filePath) => {
    const portableFilePath = toPortablePath(filePath);
    return portableFilePath.endsWith(spec.portableRelativePath)
      || path.basename(filePath) === spec.fileName;
  });

  return [...new Set(matches)];
}

function resolveUniqueGovernanceArtifactCandidate({
  spec,
  candidatePaths,
  readFile = readFileSync,
} = {}) {
  if (candidatePaths.length === 0) {
    throw new Error(`missing governance artifact for ${spec.id}: ${spec.fileName}`);
  }

  const parsedCandidates = candidatePaths.map((candidatePath) => ({
    candidatePath,
    payload: parseJson(
      readFile(candidatePath, 'utf8'),
      `${spec.description} file ${candidatePath}`,
    ),
  }));

  const canonicalPayloads = parsedCandidates.map((entry) =>
    JSON.stringify(canonicalizeJson(entry.payload)),
  );
  const distinctPayloads = [...new Set(canonicalPayloads)];

  if (distinctPayloads.length > 1) {
    throw new Error(
      `conflicting duplicate governance artifact for ${spec.id}: ${candidatePaths.join(', ')}`,
    );
  }

  const selected = parsedCandidates[0];
  validateGovernanceArtifact(spec, selected.payload);
  return {
    ...selected,
    duplicateCount: parsedCandidates.length,
  };
}

export function resolveReleaseGovernanceLatestArtifactSources({
  artifactDir = '',
  readFile = readFileSync,
  ...options
} = {}) {
  const specs = listReleaseGovernanceLatestArtifactSpecs();
  const allArtifactFiles = String(artifactDir).trim().length > 0
    ? listFilesRecursively(String(artifactDir).trim())
    : [];

  return specs.map((spec) => {
    const candidatePaths = resolveArtifactCandidates({
      spec,
      artifactDir,
      options,
      allArtifactFiles,
    });
    const selected = resolveUniqueGovernanceArtifactCandidate({
      spec,
      candidatePaths,
      readFile,
    });

    return {
      id: spec.id,
      description: spec.description,
      relativePath: spec.relativePath,
      sourcePath: selected.candidatePath,
      duplicateCount: selected.duplicateCount,
      payload: selected.payload,
    };
  });
}

export function restoreReleaseGovernanceLatestArtifacts({
  artifactDir = '',
  repoRoot = rootDir,
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
  ...options
} = {}) {
  const sources = resolveReleaseGovernanceLatestArtifactSources({
    artifactDir,
    readFile,
    ...options,
  });

  const restored = sources.map((source) => {
    const outputPath = path.join(repoRoot, source.relativePath);
    mkdir(path.dirname(outputPath), { recursive: true });
    writeFile(outputPath, `${JSON.stringify(source.payload, null, 2)}\n`, 'utf8');
    return {
      id: source.id,
      sourcePath: source.sourcePath,
      outputPath,
      duplicateCount: source.duplicateCount,
    };
  });

  return {
    repoRoot,
    restored,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    repoRoot: rootDir,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--artifact-dir') {
      options.artifactDir = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--repo-root') {
      options.repoRoot = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--window') {
      options.windowPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--sync') {
      options.syncPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--telemetry-export') {
      options.telemetryExportPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--telemetry-snapshot') {
      options.telemetrySnapshotPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--slo') {
      options.sloPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  return options;
}

function runCli() {
  const result = restoreReleaseGovernanceLatestArtifacts(parseArgs());
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    runCli();
  } catch (error) {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  }
}
