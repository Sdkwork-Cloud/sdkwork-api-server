#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  resolveReleaseTelemetrySnapshotInput,
  validateReleaseTelemetrySnapshotShape,
} from './materialize-release-telemetry-snapshot.mjs';
import { SLO_GOVERNANCE_BASELINE } from './slo-governance.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_OUTPUT_PATH = path.join(rootDir, 'docs', 'release', 'slo-governance-latest.json');

function parseJson(text, context) {
  try {
    return JSON.parse(String(text).replace(/^\uFEFF/u, ''));
  } catch (error) {
    throw new Error(
      `invalid ${context}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function isFiniteNumber(value) {
  return typeof value === 'number' && Number.isFinite(value);
}

export function resolveSloGovernanceEvidenceInput({
  evidencePath,
  evidenceJson,
  telemetrySnapshotPath,
  telemetrySnapshotJson,
  baseline = SLO_GOVERNANCE_BASELINE,
  env = process.env,
  readFile = readFileSync,
} = {}) {
  const resolvedEvidencePath = String(
    evidencePath ?? env.SDKWORK_SLO_GOVERNANCE_EVIDENCE_PATH ?? '',
  ).trim();
  const resolvedEvidenceJson = String(
    evidenceJson ?? env.SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON ?? '',
  ).trim();
  const resolvedTelemetrySnapshotPath = String(
    telemetrySnapshotPath ?? env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH ?? '',
  ).trim();
  const resolvedTelemetrySnapshotJson = String(
    telemetrySnapshotJson ?? env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON ?? '',
  ).trim();

  if (resolvedEvidenceJson.length > 0) {
    return {
      source: 'json',
      payload: parseJson(resolvedEvidenceJson, 'SLO governance evidence JSON'),
    };
  }

  if (resolvedEvidencePath.length > 0) {
    return {
      source: 'file',
      evidencePath: resolvedEvidencePath,
      payload: parseJson(
        readFile(resolvedEvidencePath, 'utf8'),
        `SLO governance evidence file ${resolvedEvidencePath}`,
      ),
    };
  }

  if (
    resolvedTelemetrySnapshotJson.length > 0
    || resolvedTelemetrySnapshotPath.length > 0
  ) {
    const snapshotInput = resolveReleaseTelemetrySnapshotInput({
      snapshotPath: resolvedTelemetrySnapshotPath,
      snapshotJson: resolvedTelemetrySnapshotJson,
      env: {},
      readFile,
    });
    const payload = deriveSloGovernanceEvidenceFromReleaseTelemetrySnapshot({
      snapshot: snapshotInput.payload,
      baseline,
    });
    return {
      source: snapshotInput.source === 'json'
        ? 'telemetry-snapshot-json'
        : 'telemetry-snapshot-file',
      payload,
    };
  }

  throw new Error(
    'missing SLO governance evidence input; set evidencePath, evidenceJson, telemetrySnapshotPath, telemetrySnapshotJson, SDKWORK_SLO_GOVERNANCE_EVIDENCE_PATH, SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON, SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH, or SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON',
  );
}

export function deriveSloGovernanceEvidenceFromReleaseTelemetrySnapshot({
  snapshot,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  validateReleaseTelemetrySnapshotShape({
    snapshot,
    baseline,
  });

  return {
    generatedAt: String(snapshot.generatedAt),
    targets: snapshot.targets,
  };
}

export function validateSloGovernanceEvidenceShape({
  evidence,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  if (!evidence || typeof evidence !== 'object') {
    throw new Error('SLO governance evidence must be a JSON object');
  }

  if (String(evidence.generatedAt ?? '').trim().length === 0) {
    throw new Error('SLO governance evidence must include generatedAt');
  }

  if (!evidence.targets || typeof evidence.targets !== 'object') {
    throw new Error('SLO governance evidence must include a targets object');
  }

  for (const target of baseline.targets) {
    const targetEvidence = evidence.targets[target.id];
    if (!targetEvidence || typeof targetEvidence !== 'object') {
      throw new Error(`SLO governance evidence is missing target ${target.id}`);
    }

    if (target.indicatorType === 'value_max') {
      if (!isFiniteNumber(targetEvidence.value)) {
        throw new Error(`SLO governance target ${target.id} must include numeric value`);
      }
    } else if (!isFiniteNumber(targetEvidence.ratio)) {
      throw new Error(`SLO governance target ${target.id} must include numeric ratio`);
    }

    if (!targetEvidence.burnRates || typeof targetEvidence.burnRates !== 'object') {
      throw new Error(`SLO governance target ${target.id} must include burnRates`);
    }

    for (const window of target.burnRateWindows) {
      if (!isFiniteNumber(targetEvidence.burnRates[window.window])) {
        throw new Error(
          `SLO governance target ${target.id} must include numeric burn rate for ${window.window}`,
        );
      }
    }
  }

  return {
    baselineId: baseline.baselineId,
    targetCount: baseline.targets.length,
  };
}

export function materializeSloGovernanceEvidence({
  evidencePath,
  evidenceJson,
  telemetrySnapshotPath,
  telemetrySnapshotJson,
  env = process.env,
  outputPath = DEFAULT_OUTPUT_PATH,
  baseline = SLO_GOVERNANCE_BASELINE,
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
} = {}) {
  const input = resolveSloGovernanceEvidenceInput({
    evidencePath,
    evidenceJson,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    baseline,
    env,
    readFile,
  });

  validateSloGovernanceEvidenceShape({
    evidence: input.payload,
    baseline,
  });

  const artifact = {
    version: baseline.version,
    baselineId: baseline.baselineId,
    baselineDate: baseline.baselineDate,
    generatedAt: String(input.payload.generatedAt),
    targets: input.payload.targets,
  };

  mkdir(path.dirname(outputPath), { recursive: true });
  writeFile(outputPath, `${JSON.stringify(artifact, null, 2)}\n`, 'utf8');

  return {
    outputPath,
    source: input.source,
    baselineId: artifact.baselineId,
    generatedAt: artifact.generatedAt,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  let evidencePath;
  let evidenceJson;
  let telemetrySnapshotPath;
  let telemetrySnapshotJson;
  let outputPath;

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--evidence') {
      evidencePath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--evidence-json') {
      evidenceJson = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--snapshot') {
      telemetrySnapshotPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--snapshot-json') {
      telemetrySnapshotJson = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--output') {
      outputPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  return {
    evidencePath,
    evidenceJson,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    outputPath,
  };
}

function runCli() {
  const {
    evidencePath,
    evidenceJson,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    outputPath,
  } = parseArgs();
  const result = materializeSloGovernanceEvidence({
    evidencePath,
    evidenceJson,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    outputPath,
  });

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
