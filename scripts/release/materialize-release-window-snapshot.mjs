#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  collectReleaseWindowSnapshotResult,
  validateReleaseWindowSnapshot,
  validateReleaseWindowSnapshotArtifact,
} from './compute-release-window-snapshot.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_OUTPUT_PATH = path.join(
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

function createReleaseWindowSnapshotArtifact({
  snapshot,
  generatedAt = new Date().toISOString(),
  source = {},
  sourceKind = 'release-window-snapshot',
  sourceProvenance = '',
} = {}) {
  validateReleaseWindowSnapshot(snapshot);

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
    snapshot,
  };
}

export function resolveReleaseWindowSnapshotProducerInput({
  snapshotPath,
  snapshotJson,
  generatedAt,
  sourceKind,
  sourceProvenance,
  env = process.env,
  readFile = readFileSync,
  spawnSyncImpl,
} = {}) {
  const resolvedSnapshotPath = String(
    snapshotPath ?? env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH ?? '',
  ).trim();
  const resolvedSnapshotJson = String(
    snapshotJson ?? env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON ?? '',
  ).trim();

  if (resolvedSnapshotJson.length > 0 || resolvedSnapshotPath.length > 0) {
    const payload = resolvedSnapshotJson.length > 0
      ? parseJson(resolvedSnapshotJson, 'release window snapshot JSON')
      : parseJson(
        readFile(resolvedSnapshotPath, 'utf8'),
        `release window snapshot file ${resolvedSnapshotPath}`,
      );

    if (payload && typeof payload === 'object' && Object.hasOwn(payload, 'snapshot')) {
      validateReleaseWindowSnapshotArtifact(payload);
      return {
        source: resolvedSnapshotJson.length > 0 ? 'snapshot-json' : 'snapshot-file',
        artifact: createReleaseWindowSnapshotArtifact({
          snapshot: payload.snapshot,
          generatedAt: payload.generatedAt,
          source: payload.source,
          sourceKind: String(sourceKind ?? '').trim() || 'release-window-snapshot',
          sourceProvenance,
        }),
      };
    }

    validateReleaseWindowSnapshot(payload);
    return {
      source: resolvedSnapshotJson.length > 0 ? 'snapshot-json' : 'snapshot-file',
      artifact: createReleaseWindowSnapshotArtifact({
        snapshot: payload,
        generatedAt: String(generatedAt ?? '').trim() || new Date().toISOString(),
        sourceKind: String(sourceKind ?? '').trim() || 'release-window-snapshot',
        sourceProvenance,
      }),
    };
  }

  const result = collectReleaseWindowSnapshotResult({
    env,
    spawnSyncImpl,
  });
  if (!result.ok) {
    throw new Error(`[release-window-snapshot] ${result.reason || 'unknown'}: ${result.errorMessage || 'unknown'}`);
  }

  return {
    source: 'live-git',
    artifact: createReleaseWindowSnapshotArtifact({
      snapshot: result.snapshot,
      generatedAt: String(generatedAt ?? '').trim() || new Date().toISOString(),
      sourceKind: String(sourceKind ?? '').trim() || 'release-window-live-git',
      sourceProvenance,
    }),
  };
}

export function materializeReleaseWindowSnapshot({
  snapshotPath,
  snapshotJson,
  generatedAt,
  sourceKind,
  sourceProvenance,
  env = process.env,
  outputPath = DEFAULT_OUTPUT_PATH,
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
  spawnSyncImpl,
} = {}) {
  const input = resolveReleaseWindowSnapshotProducerInput({
    snapshotPath,
    snapshotJson,
    generatedAt,
    sourceKind,
    sourceProvenance,
    env,
    readFile,
    spawnSyncImpl,
  });

  validateReleaseWindowSnapshotArtifact(input.artifact);
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
    if (token === '--snapshot') {
      options.snapshotPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--snapshot-json') {
      options.snapshotJson = String(argv[index + 1] ?? '').trim();
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
  const result = materializeReleaseWindowSnapshot(parseArgs());
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
