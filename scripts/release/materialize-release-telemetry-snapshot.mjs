#!/usr/bin/env node

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { SLO_GOVERNANCE_BASELINE } from './slo-governance.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const RELEASE_TELEMETRY_SNAPSHOT_ID = 'release-telemetry-snapshot-v1';
const DEFAULT_OUTPUT_PATH = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const DEFAULT_EXPORT_INPUT_PATH = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
);
const PROMETHEUS_SAMPLE_PATTERN = /^([A-Za-z_:][A-Za-z0-9_:]*)(?:\{([^}]*)\})?\s+([-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?)$/;
const HTTP_REQUEST_TOTAL_METRIC = 'sdkwork_http_requests_total';
const DIRECT_PROMETHEUS_TARGET_SPECS = Object.freeze([
  Object.freeze({ targetId: 'gateway-availability', prometheusKey: 'gateway' }),
  Object.freeze({ targetId: 'admin-api-availability', prometheusKey: 'admin' }),
  Object.freeze({ targetId: 'portal-api-availability', prometheusKey: 'portal' }),
]);

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

function normalizeNumber(value) {
  return Number.parseFloat(value.toFixed(6));
}

function hasConfiguredInput(...values) {
  return values.some((value) => String(value ?? '').trim().length > 0);
}

function unescapePrometheusLabelValue(value) {
  return value.replace(/\\([\\n"])/g, (_match, token) => {
    if (token === 'n') {
      return '\n';
    }
    return token;
  });
}

function parsePrometheusLabels(text) {
  const labels = {};
  const labelText = String(text ?? '').trim();

  if (labelText.length === 0) {
    return labels;
  }

  const pattern = /([A-Za-z_][A-Za-z0-9_]*)="((?:\\.|[^"])*)"(?:,|$)/g;
  let lastIndex = 0;
  let match;
  while ((match = pattern.exec(labelText)) !== null) {
    if (match.index !== lastIndex) {
      throw new Error(`invalid Prometheus labels: ${labelText}`);
    }
    labels[match[1]] = unescapePrometheusLabelValue(match[2]);
    lastIndex = pattern.lastIndex;
  }

  if (lastIndex !== labelText.length) {
    throw new Error(`invalid Prometheus labels: ${labelText}`);
  }

  return labels;
}

function parsePrometheusSamples(text, context) {
  const samples = [];
  const lines = String(text ?? '').split(/\r?\n/u);

  for (const line of lines) {
    const trimmedLine = line.trim();
    if (trimmedLine.length === 0 || trimmedLine.startsWith('#')) {
      continue;
    }

    const match = PROMETHEUS_SAMPLE_PATTERN.exec(trimmedLine);
    if (!match) {
      throw new Error(`invalid ${context} Prometheus sample line: ${trimmedLine}`);
    }

    const value = Number(match[3]);
    if (!isFiniteNumber(value)) {
      throw new Error(`invalid ${context} Prometheus sample value: ${trimmedLine}`);
    }

    samples.push({
      name: match[1],
      labels: parsePrometheusLabels(match[2] ?? ''),
      value,
    });
  }

  return samples;
}

function computeTargetBurnRate({
  target,
  evidence,
} = {}) {
  if (target.indicatorType === 'ratio_min') {
    const errorBudget = 1 - target.objective;
    return errorBudget <= 0 ? 0 : normalizeNumber((1 - evidence.ratio) / errorBudget);
  }

  if (target.indicatorType === 'ratio_max') {
    return target.objective <= 0 ? 0 : normalizeNumber(evidence.ratio / target.objective);
  }

  if (target.indicatorType === 'value_max') {
    return target.objective <= 0 ? 0 : normalizeNumber(evidence.value / target.objective);
  }

  throw new Error(`unsupported release telemetry target type ${target.indicatorType}`);
}

function deriveBurnRatesForTarget({
  target,
  evidence,
} = {}) {
  const burnRate = computeTargetBurnRate({
    target,
    evidence,
  });

  return Object.fromEntries(
    target.burnRateWindows.map((window) => [window.window, burnRate]),
  );
}

function deriveAvailabilityRatioFromPrometheusText({
  prometheusText,
  prometheusKey,
  targetId,
} = {}) {
  const samples = parsePrometheusSamples(
    prometheusText,
    `release telemetry export ${prometheusKey}`,
  );

  let totalRequests = 0;
  let availableRequests = 0;

  for (const sample of samples) {
    if (sample.name !== HTTP_REQUEST_TOTAL_METRIC) {
      continue;
    }

    const status = Number.parseInt(String(sample.labels.status ?? ''), 10);
    if (!Number.isInteger(status)) {
      continue;
    }

    totalRequests += sample.value;
    if (status < 500) {
      availableRequests += sample.value;
    }
  }

  if (totalRequests <= 0) {
    throw new Error(
      `release telemetry export ${prometheusKey} must include ${HTTP_REQUEST_TOTAL_METRIC} samples for ${targetId}`,
    );
  }

  return normalizeNumber(availableRequests / totalRequests);
}

export function resolveReleaseTelemetrySnapshotInput({
  snapshotPath,
  snapshotJson,
  env = process.env,
  readFile = readFileSync,
} = {}) {
  const resolvedSnapshotPath = String(
    snapshotPath ?? env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH ?? '',
  ).trim();
  const resolvedSnapshotJson = String(
    snapshotJson ?? env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON ?? '',
  ).trim();

  if (resolvedSnapshotJson.length > 0) {
    return {
      source: 'json',
      payload: parseJson(resolvedSnapshotJson, 'release telemetry snapshot JSON'),
    };
  }

  if (resolvedSnapshotPath.length > 0) {
    return {
      source: 'file',
      snapshotPath: resolvedSnapshotPath,
      payload: parseJson(
        readFile(resolvedSnapshotPath, 'utf8'),
        `release telemetry snapshot file ${resolvedSnapshotPath}`,
      ),
    };
  }

  throw new Error(
    'missing release telemetry snapshot input; set snapshotPath, snapshotJson, SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH, or SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON',
  );
}

export function resolveReleaseTelemetryExportInput({
  exportPath,
  exportJson,
  env = process.env,
  readFile = readFileSync,
} = {}) {
  const resolvedExportPath = String(
    exportPath ?? env.SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH ?? '',
  ).trim();
  const resolvedExportJson = String(
    exportJson ?? env.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON ?? '',
  ).trim();

  if (resolvedExportJson.length > 0) {
    return {
      source: 'json',
      payload: parseJson(resolvedExportJson, 'release telemetry export JSON'),
    };
  }

  if (resolvedExportPath.length > 0) {
    return {
      source: 'file',
      exportPath: resolvedExportPath,
      payload: parseJson(
        readFile(resolvedExportPath, 'utf8'),
        `release telemetry export file ${resolvedExportPath}`,
      ),
    };
  }

  throw new Error(
    'missing release telemetry export input; set exportPath, exportJson, SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH, or SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON',
  );
}

export function validateReleaseTelemetryExportShape({
  exportBundle,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  if (!exportBundle || typeof exportBundle !== 'object') {
    throw new Error('release telemetry export must be a JSON object');
  }

  if (String(exportBundle.generatedAt ?? '').trim().length === 0) {
    throw new Error('release telemetry export must include generatedAt');
  }

  if (!exportBundle.source || typeof exportBundle.source !== 'object') {
    throw new Error('release telemetry export must include a source object');
  }

  if (String(exportBundle.source.kind ?? '').trim().length === 0) {
    throw new Error('release telemetry export source must include kind');
  }

  if (!exportBundle.prometheus || typeof exportBundle.prometheus !== 'object') {
    throw new Error('release telemetry export must include a prometheus object');
  }

  const directTargetIds = new Set(
    DIRECT_PROMETHEUS_TARGET_SPECS
      .map((spec) => spec.targetId)
      .filter((targetId) => baseline.targets.some((target) => target.id === targetId)),
  );

  for (const spec of DIRECT_PROMETHEUS_TARGET_SPECS) {
    if (!directTargetIds.has(spec.targetId)) {
      continue;
    }

    if (String(exportBundle.prometheus[spec.prometheusKey] ?? '').trim().length === 0) {
      throw new Error(
        `release telemetry export must include prometheus.${spec.prometheusKey} for ${spec.targetId}`,
      );
    }
  }

  if (
    exportBundle.supplemental !== undefined
    && (
      !exportBundle.supplemental
      || typeof exportBundle.supplemental !== 'object'
      || Array.isArray(exportBundle.supplemental)
    )
  ) {
    throw new Error('release telemetry export supplemental must be a JSON object');
  }

  if (
    exportBundle.supplemental?.targets !== undefined
    && (
      !exportBundle.supplemental.targets
      || typeof exportBundle.supplemental.targets !== 'object'
      || Array.isArray(exportBundle.supplemental.targets)
    )
  ) {
    throw new Error('release telemetry export supplemental.targets must be a JSON object');
  }
}

export function validateReleaseTelemetrySnapshotShape({
  snapshot,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  if (!snapshot || typeof snapshot !== 'object') {
    throw new Error('release telemetry snapshot must be a JSON object');
  }

  if (String(snapshot.generatedAt ?? '').trim().length === 0) {
    throw new Error('release telemetry snapshot must include generatedAt');
  }

  if (!snapshot.source || typeof snapshot.source !== 'object') {
    throw new Error('release telemetry snapshot must include a source object');
  }

  if (String(snapshot.source.kind ?? '').trim().length === 0) {
    throw new Error('release telemetry snapshot source must include kind');
  }

  if (!snapshot.targets || typeof snapshot.targets !== 'object') {
    throw new Error('release telemetry snapshot must include a targets object');
  }

  for (const target of baseline.targets) {
    const targetSnapshot = snapshot.targets[target.id];
    if (!targetSnapshot || typeof targetSnapshot !== 'object') {
      throw new Error(`release telemetry snapshot is missing target ${target.id}`);
    }

    if (target.indicatorType === 'value_max') {
      if (!isFiniteNumber(targetSnapshot.value)) {
        throw new Error(`release telemetry target ${target.id} must include numeric value`);
      }
    } else if (!isFiniteNumber(targetSnapshot.ratio)) {
      throw new Error(`release telemetry target ${target.id} must include numeric ratio`);
    }

    if (!targetSnapshot.burnRates || typeof targetSnapshot.burnRates !== 'object') {
      throw new Error(`release telemetry target ${target.id} must include burnRates`);
    }

    for (const window of target.burnRateWindows) {
      if (!isFiniteNumber(targetSnapshot.burnRates[window.window])) {
        throw new Error(
          `release telemetry target ${target.id} must include numeric burn rate for ${window.window}`,
        );
      }
    }
  }

  return {
    snapshotId: RELEASE_TELEMETRY_SNAPSHOT_ID,
    targetCount: baseline.targets.length,
  };
}

export function deriveReleaseTelemetrySnapshotFromExport({
  exportBundle,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  validateReleaseTelemetryExportShape({
    exportBundle,
    baseline,
  });

  const targetMap = new Map(baseline.targets.map((target) => [target.id, target]));
  const directlyDerivedTargets = {};

  for (const spec of DIRECT_PROMETHEUS_TARGET_SPECS) {
    const target = targetMap.get(spec.targetId);
    if (!target) {
      continue;
    }

    const ratio = deriveAvailabilityRatioFromPrometheusText({
      prometheusText: exportBundle.prometheus?.[spec.prometheusKey],
      prometheusKey: spec.prometheusKey,
      targetId: spec.targetId,
    });

    const evidence = { ratio };
    directlyDerivedTargets[spec.targetId] = {
      ...evidence,
      burnRates: deriveBurnRatesForTarget({
        target,
        evidence,
      }),
    };
  }

  const supplementalTargets = exportBundle.supplemental?.targets ?? {};
  for (const targetId of Object.keys(supplementalTargets)) {
    if (Object.hasOwn(directlyDerivedTargets, targetId)) {
      throw new Error(
        `release telemetry export supplemental.targets must not redefine directly derived target ${targetId}`,
      );
    }
  }

  const source = {
    kind: 'release-telemetry-export',
    exportKind: String(exportBundle.source.kind),
    directTargetIds: Object.keys(directlyDerivedTargets).sort(),
    supplementalTargetIds: Object.keys(supplementalTargets).sort(),
  };

  if (isFiniteNumber(exportBundle.source.freshnessMinutes)) {
    source.freshnessMinutes = exportBundle.source.freshnessMinutes;
  }

  if (String(exportBundle.source.provenance ?? '').trim().length > 0) {
    source.provenance = String(exportBundle.source.provenance);
  }

  const snapshot = {
    generatedAt: String(exportBundle.generatedAt),
    source,
    targets: {
      ...supplementalTargets,
      ...directlyDerivedTargets,
    },
  };

  validateReleaseTelemetrySnapshotShape({
    snapshot,
    baseline,
  });

  return snapshot;
}

function resolveReleaseTelemetryMaterializationInput({
  snapshotPath,
  snapshotJson,
  exportPath,
  exportJson,
  env = process.env,
  baseline = SLO_GOVERNANCE_BASELINE,
  readFile = readFileSync,
} = {}) {
  if (
    hasConfiguredInput(
      snapshotPath,
      snapshotJson,
      env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH,
      env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON,
    )
  ) {
    const snapshotInput = resolveReleaseTelemetrySnapshotInput({
      snapshotPath,
      snapshotJson,
      env,
      readFile,
    });

    return {
      source: snapshotInput.source === 'json' ? 'snapshot-json' : 'snapshot-file',
      payload: snapshotInput.payload,
    };
  }

  if (
    hasConfiguredInput(
      exportPath,
      exportJson,
      env.SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH,
      env.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON,
    )
  ) {
    const exportInput = resolveReleaseTelemetryExportInput({
      exportPath,
      exportJson,
      env,
      readFile,
    });

    return {
      source: exportInput.source === 'json' ? 'export-json' : 'export-file',
      payload: deriveReleaseTelemetrySnapshotFromExport({
        exportBundle: exportInput.payload,
        baseline,
      }),
    };
  }

  if (existsSync(DEFAULT_EXPORT_INPUT_PATH)) {
    const exportInput = resolveReleaseTelemetryExportInput({
      exportPath: DEFAULT_EXPORT_INPUT_PATH,
      env: {},
      readFile,
    });

    return {
      source: 'export-file',
      payload: deriveReleaseTelemetrySnapshotFromExport({
        exportBundle: exportInput.payload,
        baseline,
      }),
    };
  }

  throw new Error(
    'missing release telemetry input; set snapshotPath, snapshotJson, exportPath, exportJson, SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH, SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON, SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH, or SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON',
  );
}

export function materializeReleaseTelemetrySnapshot({
  snapshotPath,
  snapshotJson,
  exportPath,
  exportJson,
  env = process.env,
  outputPath = DEFAULT_OUTPUT_PATH,
  baseline = SLO_GOVERNANCE_BASELINE,
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
} = {}) {
  const input = resolveReleaseTelemetryMaterializationInput({
    snapshotPath,
    snapshotJson,
    exportPath,
    exportJson,
    env,
    baseline,
    readFile,
  });

  validateReleaseTelemetrySnapshotShape({
    snapshot: input.payload,
    baseline,
  });

  const artifact = {
    version: 1,
    snapshotId: RELEASE_TELEMETRY_SNAPSHOT_ID,
    generatedAt: String(input.payload.generatedAt),
    source: input.payload.source,
    targets: input.payload.targets,
  };

  mkdir(path.dirname(outputPath), { recursive: true });
  writeFile(outputPath, `${JSON.stringify(artifact, null, 2)}\n`, 'utf8');

  return {
    outputPath,
    source: input.source,
    snapshotId: artifact.snapshotId,
    generatedAt: artifact.generatedAt,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  let snapshotPath;
  let snapshotJson;
  let exportPath;
  let exportJson;
  let outputPath;

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
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
    if (token === '--export') {
      exportPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--export-json') {
      exportJson = String(argv[index + 1] ?? '').trim();
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
    snapshotPath,
    snapshotJson,
    exportPath,
    exportJson,
    outputPath,
  };
}

function runCli() {
  const {
    snapshotPath,
    snapshotJson,
    exportPath,
    exportJson,
    outputPath,
  } = parseArgs();
  const result = materializeReleaseTelemetrySnapshot({
    snapshotPath,
    snapshotJson,
    exportPath,
    exportJson,
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
