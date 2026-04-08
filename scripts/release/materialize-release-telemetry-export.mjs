#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  resolveReleaseTelemetryExportInput,
  validateReleaseTelemetryExportShape,
} from './materialize-release-telemetry-snapshot.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_OUTPUT_PATH = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
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

function hasConfiguredInput(...values) {
  return values.some((value) => String(value ?? '').trim().length > 0);
}

function readTextInput({
  filePath,
  directText,
  envFilePath,
  envText,
  context,
  readFile = readFileSync,
} = {}) {
  const resolvedText = String(directText ?? envText ?? '').trim();
  if (resolvedText.length > 0) {
    return resolvedText;
  }

  const resolvedPath = String(filePath ?? envFilePath ?? '').trim();
  if (resolvedPath.length > 0) {
    return String(readFile(resolvedPath, 'utf8')).replace(/^\uFEFF/u, '');
  }

  return '';
}

function readJsonInput({
  filePath,
  jsonText,
  envFilePath,
  envJsonText,
  context,
  readFile = readFileSync,
} = {}) {
  const resolvedJson = String(jsonText ?? envJsonText ?? '').trim();
  if (resolvedJson.length > 0) {
    return parseJson(resolvedJson, context);
  }

  const resolvedPath = String(filePath ?? envFilePath ?? '').trim();
  if (resolvedPath.length > 0) {
    return parseJson(
      readFile(resolvedPath, 'utf8'),
      `${context} file ${resolvedPath}`,
    );
  }

  return null;
}

function normalizeSupplementalTargetsPayload(payload) {
  if (payload === null) {
    return {};
  }

  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    throw new Error('release telemetry supplemental targets must be a JSON object');
  }

  const targetPayload = payload.targets ?? payload;
  if (!targetPayload || typeof targetPayload !== 'object' || Array.isArray(targetPayload)) {
    throw new Error('release telemetry supplemental targets must resolve to a JSON object');
  }

  return targetPayload;
}

function resolveFreshnessMinutes(value) {
  const normalized = String(value ?? '').trim();
  if (normalized.length === 0) {
    return null;
  }

  const parsed = Number(normalized);
  if (!Number.isFinite(parsed)) {
    throw new Error(`invalid release telemetry freshnessMinutes: ${normalized}`);
  }

  return parsed;
}

export function resolveReleaseTelemetryExportProducerInput({
  exportPath,
  exportJson,
  gatewayPrometheusPath,
  gatewayPrometheusText,
  adminPrometheusPath,
  adminPrometheusText,
  portalPrometheusPath,
  portalPrometheusText,
  supplementalTargetsPath,
  supplementalTargetsJson,
  generatedAt,
  sourceKind,
  sourceProvenance,
  freshnessMinutes,
  env = process.env,
  readFile = readFileSync,
} = {}) {
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
      payload: exportInput.payload,
    };
  }

  const gatewayPrometheus = readTextInput({
    filePath: gatewayPrometheusPath,
    directText: gatewayPrometheusText,
    envFilePath: env.SDKWORK_RELEASE_TELEMETRY_GATEWAY_PROMETHEUS_PATH,
    envText: env.SDKWORK_RELEASE_TELEMETRY_GATEWAY_PROMETHEUS_TEXT,
    context: 'gateway Prometheus',
    readFile,
  });
  const adminPrometheus = readTextInput({
    filePath: adminPrometheusPath,
    directText: adminPrometheusText,
    envFilePath: env.SDKWORK_RELEASE_TELEMETRY_ADMIN_PROMETHEUS_PATH,
    envText: env.SDKWORK_RELEASE_TELEMETRY_ADMIN_PROMETHEUS_TEXT,
    context: 'admin Prometheus',
    readFile,
  });
  const portalPrometheus = readTextInput({
    filePath: portalPrometheusPath,
    directText: portalPrometheusText,
    envFilePath: env.SDKWORK_RELEASE_TELEMETRY_PORTAL_PROMETHEUS_PATH,
    envText: env.SDKWORK_RELEASE_TELEMETRY_PORTAL_PROMETHEUS_TEXT,
    context: 'portal Prometheus',
    readFile,
  });
  const supplementalTargets = normalizeSupplementalTargetsPayload(
    readJsonInput({
      filePath: supplementalTargetsPath,
      jsonText: supplementalTargetsJson,
      envFilePath: env.SDKWORK_RELEASE_TELEMETRY_SUPPLEMENTAL_TARGETS_PATH,
      envJsonText: env.SDKWORK_RELEASE_TELEMETRY_SUPPLEMENTAL_TARGETS_JSON,
      context: 'release telemetry supplemental targets',
      readFile,
    }),
  );

  if (!hasConfiguredInput(gatewayPrometheus, adminPrometheus, portalPrometheus)) {
    throw new Error(
      'missing release telemetry export input; set exportPath, exportJson, gatewayPrometheusPath, gatewayPrometheusText, adminPrometheusPath, adminPrometheusText, portalPrometheusPath, portalPrometheusText, supplementalTargetsPath, supplementalTargetsJson, SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH, SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON, SDKWORK_RELEASE_TELEMETRY_GATEWAY_PROMETHEUS_PATH, SDKWORK_RELEASE_TELEMETRY_GATEWAY_PROMETHEUS_TEXT, SDKWORK_RELEASE_TELEMETRY_ADMIN_PROMETHEUS_PATH, SDKWORK_RELEASE_TELEMETRY_ADMIN_PROMETHEUS_TEXT, SDKWORK_RELEASE_TELEMETRY_PORTAL_PROMETHEUS_PATH, SDKWORK_RELEASE_TELEMETRY_PORTAL_PROMETHEUS_TEXT, SDKWORK_RELEASE_TELEMETRY_SUPPLEMENTAL_TARGETS_PATH, or SDKWORK_RELEASE_TELEMETRY_SUPPLEMENTAL_TARGETS_JSON',
    );
  }

  if (!gatewayPrometheus || !adminPrometheus || !portalPrometheus) {
    throw new Error(
      'release telemetry export producer requires gateway, admin, and portal Prometheus inputs',
    );
  }

  const resolvedGeneratedAt = String(
    generatedAt
    ?? env.SDKWORK_RELEASE_TELEMETRY_GENERATED_AT
    ?? '',
  ).trim() || new Date().toISOString();
  const resolvedSourceKind = String(
    sourceKind
    ?? env.SDKWORK_RELEASE_TELEMETRY_SOURCE_KIND
    ?? '',
  ).trim() || 'observability-control-plane';
  const resolvedSourceProvenance = String(
    sourceProvenance
    ?? env.SDKWORK_RELEASE_TELEMETRY_SOURCE_PROVENANCE
    ?? '',
  ).trim();
  const resolvedFreshnessMinutes = resolveFreshnessMinutes(
    freshnessMinutes ?? env.SDKWORK_RELEASE_TELEMETRY_SOURCE_FRESHNESS_MINUTES,
  );

  const payload = {
    generatedAt: resolvedGeneratedAt,
    source: {
      kind: resolvedSourceKind,
    },
    prometheus: {
      gateway: gatewayPrometheus,
      admin: adminPrometheus,
      portal: portalPrometheus,
    },
    supplemental: {
      targets: supplementalTargets,
    },
  };

  if (resolvedSourceProvenance.length > 0) {
    payload.source.provenance = resolvedSourceProvenance;
  }

  if (resolvedFreshnessMinutes !== null) {
    payload.source.freshnessMinutes = resolvedFreshnessMinutes;
  }

  return {
    source: 'prometheus-handoff',
    payload,
  };
}

export function materializeReleaseTelemetryExport({
  exportPath,
  exportJson,
  gatewayPrometheusPath,
  gatewayPrometheusText,
  adminPrometheusPath,
  adminPrometheusText,
  portalPrometheusPath,
  portalPrometheusText,
  supplementalTargetsPath,
  supplementalTargetsJson,
  generatedAt,
  sourceKind,
  sourceProvenance,
  freshnessMinutes,
  env = process.env,
  outputPath = DEFAULT_OUTPUT_PATH,
  readFile = readFileSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
} = {}) {
  const input = resolveReleaseTelemetryExportProducerInput({
    exportPath,
    exportJson,
    gatewayPrometheusPath,
    gatewayPrometheusText,
    adminPrometheusPath,
    adminPrometheusText,
    portalPrometheusPath,
    portalPrometheusText,
    supplementalTargetsPath,
    supplementalTargetsJson,
    generatedAt,
    sourceKind,
    sourceProvenance,
    freshnessMinutes,
    env,
    readFile,
  });

  validateReleaseTelemetryExportShape({
    exportBundle: input.payload,
  });

  const artifact = {
    version: 1,
    generatedAt: String(input.payload.generatedAt),
    source: input.payload.source,
    prometheus: input.payload.prometheus,
    supplemental: input.payload.supplemental ?? { targets: {} },
  };

  mkdir(path.dirname(outputPath), { recursive: true });
  writeFile(outputPath, `${JSON.stringify(artifact, null, 2)}\n`, 'utf8');

  return {
    outputPath,
    source: input.source,
    generatedAt: artifact.generatedAt,
    kind: String(artifact.source.kind),
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--export') {
      options.exportPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--export-json') {
      options.exportJson = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--gateway-prometheus') {
      options.gatewayPrometheusPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--gateway-prometheus-text') {
      options.gatewayPrometheusText = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--admin-prometheus') {
      options.adminPrometheusPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--admin-prometheus-text') {
      options.adminPrometheusText = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--portal-prometheus') {
      options.portalPrometheusPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--portal-prometheus-text') {
      options.portalPrometheusText = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--supplemental-targets') {
      options.supplementalTargetsPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--supplemental-targets-json') {
      options.supplementalTargetsJson = String(argv[index + 1] ?? '').trim();
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
    if (token === '--freshness-minutes') {
      options.freshnessMinutes = String(argv[index + 1] ?? '').trim();
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
  const result = materializeReleaseTelemetryExport(parseArgs());
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
