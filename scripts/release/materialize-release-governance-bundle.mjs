#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  listReleaseGovernanceLatestArtifactSpecs,
  resolveReleaseGovernanceLatestArtifactSources,
} from './restore-release-governance-latest.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

function resolveRepoRoot(repoRoot = rootDir) {
  return path.resolve(String(repoRoot ?? '').trim() || rootDir);
}

function resolveOutputDir({
  repoRoot = rootDir,
  outputDir,
} = {}) {
  const resolvedRepoRoot = resolveRepoRoot(repoRoot);
  const normalizedOutputDir = String(outputDir ?? '').trim();
  if (normalizedOutputDir.length > 0) {
    return path.resolve(normalizedOutputDir);
  }

  return path.join(resolvedRepoRoot, 'artifacts', 'release-governance-bundle');
}

export function listReleaseGovernanceBundleArtifactSpecs({
  repoRoot = rootDir,
} = {}) {
  const resolvedRepoRoot = resolveRepoRoot(repoRoot);
  return listReleaseGovernanceLatestArtifactSpecs().map((spec) => ({
    ...spec,
    sourcePath: path.join(resolvedRepoRoot, spec.relativePath),
    bundleRelativePath: toPortablePath(spec.relativePath),
  }));
}

export function createReleaseGovernanceBundleManifest({
  generatedAt = new Date().toISOString(),
  artifacts = [],
} = {}) {
  return {
    version: 1,
    generatedAt: String(generatedAt),
    bundleEntryCount: artifacts.length,
    artifacts: artifacts.map((artifact) => ({
      id: artifact.id,
      relativePath: toPortablePath(artifact.relativePath),
      sourceRelativePath: toPortablePath(artifact.sourceRelativePath),
    })),
    restore: {
      command: 'node scripts/release/restore-release-governance-latest.mjs --artifact-dir <downloaded-dir>',
    },
  };
}

export function materializeReleaseGovernanceBundle({
  repoRoot = rootDir,
  outputDir,
  generatedAt = new Date().toISOString(),
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
} = {}) {
  const resolvedRepoRoot = resolveRepoRoot(repoRoot);
  const resolvedOutputDir = resolveOutputDir({
    repoRoot: resolvedRepoRoot,
    outputDir,
  });
  const bundleSpecs = listReleaseGovernanceBundleArtifactSpecs({
    repoRoot: resolvedRepoRoot,
  });

  const sourceOptions = Object.fromEntries(
    bundleSpecs.map((spec) => [spec.optionKey, spec.sourcePath]),
  );
  const sources = resolveReleaseGovernanceLatestArtifactSources({
    readFile,
    ...sourceOptions,
  });

  const artifacts = sources.map((source) => {
    const outputPath = path.join(resolvedOutputDir, source.relativePath);
    mkdir(path.dirname(outputPath), { recursive: true });
    writeFile(outputPath, `${JSON.stringify(source.payload, null, 2)}\n`, 'utf8');

    return {
      id: source.id,
      sourceRelativePath: toPortablePath(path.relative(resolvedRepoRoot, source.sourcePath)),
      relativePath: toPortablePath(source.relativePath),
      outputPath,
    };
  });

  const manifest = createReleaseGovernanceBundleManifest({
    generatedAt,
    artifacts,
  });
  const manifestPath = path.join(
    resolvedOutputDir,
    'release-governance-bundle-manifest.json',
  );
  mkdir(path.dirname(manifestPath), { recursive: true });
  writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');

  return {
    repoRoot: resolvedRepoRoot,
    outputDir: resolvedOutputDir,
    generatedAt: manifest.generatedAt,
    bundleEntryCount: artifacts.length,
    manifestPath,
    artifacts,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--repo-root') {
      options.repoRoot = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--output-dir') {
      options.outputDir = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--generated-at') {
      options.generatedAt = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  return options;
}

function runCli() {
  const result = materializeReleaseGovernanceBundle(parseArgs());
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
