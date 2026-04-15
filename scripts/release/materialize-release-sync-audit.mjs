#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  auditReleaseSyncRepositories,
  validateReleaseSyncAuditArtifact,
  validateReleaseSyncAuditSummary,
} from './verify-release-sync.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_OUTPUT_PATH = path.join(
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

function resolveSource(source = {}, fallbackKind) {
  const resolvedSource = {
    kind: String(source.kind ?? '').trim() || fallbackKind,
  };

  const provenance = String(source.provenance ?? '').trim();
  if (provenance.length > 0) {
    resolvedSource.provenance = provenance;
  }

  return resolvedSource;
}

function createReleaseSyncAuditArtifact({
  summary,
  generatedAt = new Date().toISOString(),
  source = {},
  sourceKind = 'release-sync-audit',
  sourceProvenance = '',
} = {}) {
  validateReleaseSyncAuditSummary(summary);

  return {
    version: 1,
    generatedAt: String(generatedAt),
    source: resolveSource(
      {
        ...source,
        kind: String(source.kind ?? '').trim() || sourceKind,
        provenance: String(source.provenance ?? '').trim() || String(sourceProvenance ?? '').trim(),
      },
      sourceKind,
    ),
    summary,
  };
}

export function resolveReleaseSyncAuditProducerInput({
  auditPath,
  auditJson,
  generatedAt,
  sourceKind,
  sourceProvenance,
  specs,
  env = process.env,
  readFile = readFileSync,
  spawnSyncImpl,
} = {}) {
  const resolvedAuditPath = String(
    auditPath ?? env.SDKWORK_RELEASE_SYNC_AUDIT_PATH ?? '',
  ).trim();
  const resolvedAuditJson = String(
    auditJson ?? env.SDKWORK_RELEASE_SYNC_AUDIT_JSON ?? '',
  ).trim();

  if (resolvedAuditJson.length > 0 || resolvedAuditPath.length > 0) {
    const payload = resolvedAuditJson.length > 0
      ? parseJson(resolvedAuditJson, 'release sync audit JSON')
      : parseJson(
        readFile(resolvedAuditPath, 'utf8'),
        `release sync audit file ${resolvedAuditPath}`,
      );

    if (payload && typeof payload === 'object' && Object.hasOwn(payload, 'summary')) {
      validateReleaseSyncAuditArtifact(payload);
      return {
        source: resolvedAuditJson.length > 0 ? 'audit-json' : 'audit-file',
        artifact: createReleaseSyncAuditArtifact({
          summary: payload.summary,
          generatedAt: payload.generatedAt,
          source: payload.source,
          sourceKind: String(sourceKind ?? '').trim() || 'release-sync-audit',
          sourceProvenance,
        }),
      };
    }

    validateReleaseSyncAuditSummary(payload);
    return {
      source: resolvedAuditJson.length > 0 ? 'audit-json' : 'audit-file',
      artifact: createReleaseSyncAuditArtifact({
        summary: payload,
        generatedAt: String(generatedAt ?? '').trim() || new Date().toISOString(),
        sourceKind: String(sourceKind ?? '').trim() || 'release-sync-audit',
        sourceProvenance,
      }),
    };
  }

  const summary = auditReleaseSyncRepositories({
    specs,
    preferDefaultArtifact: false,
    env,
    readFile,
    spawnSyncImpl,
  });
  if (!summary.releasable && summary.reports.some((report) =>
    report.reasons.includes('command-exec-blocked'),
  )) {
    throw new Error('[release-sync-audit] command-exec-blocked: live git execution denied');
  }

  return {
    source: 'live-git',
    artifact: createReleaseSyncAuditArtifact({
      summary,
      generatedAt: String(generatedAt ?? '').trim() || new Date().toISOString(),
      sourceKind: String(sourceKind ?? '').trim() || 'release-sync-audit-live-git',
      sourceProvenance,
    }),
  };
}

export function materializeReleaseSyncAudit({
  auditPath,
  auditJson,
  generatedAt,
  sourceKind,
  sourceProvenance,
  specs,
  env = process.env,
  outputPath = DEFAULT_OUTPUT_PATH,
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
  spawnSyncImpl,
} = {}) {
  const input = resolveReleaseSyncAuditProducerInput({
    auditPath,
    auditJson,
    generatedAt,
    sourceKind,
    sourceProvenance,
    specs,
    env,
    readFile,
    spawnSyncImpl,
  });

  validateReleaseSyncAuditArtifact(input.artifact);
  mkdir(path.dirname(outputPath), { recursive: true });
  writeFile(outputPath, `${JSON.stringify(input.artifact, null, 2)}\n`, 'utf8');

  return {
    outputPath,
    source: input.source,
    generatedAt: input.artifact.generatedAt,
    kind: String(input.artifact.source.kind),
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--audit') {
      options.auditPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--audit-json') {
      options.auditJson = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--generated-at') {
      options.generatedAt = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--source-kind') {
      options.sourceKind = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--source-provenance') {
      options.sourceProvenance = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--output') {
      options.outputPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  return options;
}

function runCli() {
  const result = materializeReleaseSyncAudit(parseArgs());
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
