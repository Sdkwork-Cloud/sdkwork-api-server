#!/usr/bin/env node

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { materializeReleaseTelemetrySnapshot } from './materialize-release-telemetry-snapshot.mjs';
import { materializeSloGovernanceEvidence } from './materialize-slo-governance-evidence.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');
const DEFAULT_SLO_GOVERNANCE_EVIDENCE_PATH = path.join(
  rootDir,
  'docs',
  'release',
  'slo-governance-latest.json',
);
const DEFAULT_RELEASE_TELEMETRY_SNAPSHOT_PATH = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const DEFAULT_RELEASE_TELEMETRY_EXPORT_PATH = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
);

const DEFAULT_BURN_RATE_WINDOWS = Object.freeze([
  Object.freeze({
    window: '1h',
    maxBurnRate: 14.4,
    severity: 'release-blocker',
  }),
  Object.freeze({
    window: '6h',
    maxBurnRate: 6,
    severity: 'release-blocker',
  }),
]);

function createRatioTarget({
  id,
  plane,
  objective,
  evidenceSources,
  description,
}) {
  return Object.freeze({
    id,
    plane,
    indicatorType: 'ratio_min',
    objective,
    unit: 'ratio',
    description,
    evidenceSources: Object.freeze([...evidenceSources]),
    burnRateWindows: DEFAULT_BURN_RATE_WINDOWS,
  });
}

function createBudgetTarget({
  id,
  plane,
  objective,
  evidenceSources,
  description,
}) {
  return Object.freeze({
    id,
    plane,
    indicatorType: 'ratio_max',
    objective,
    unit: 'ratio',
    description,
    evidenceSources: Object.freeze([...evidenceSources]),
    burnRateWindows: DEFAULT_BURN_RATE_WINDOWS,
  });
}

function createLatencyTarget({
  id,
  plane,
  objective,
  evidenceSources,
  description,
}) {
  return Object.freeze({
    id,
    plane,
    indicatorType: 'value_max',
    objective,
    unit: 'ms',
    description,
    evidenceSources: Object.freeze([...evidenceSources]),
    burnRateWindows: DEFAULT_BURN_RATE_WINDOWS,
  });
}

export const SLO_GOVERNANCE_BASELINE = Object.freeze({
  version: 1,
  baselineId: 'release-slo-governance-baseline-2026-04-08',
  baselineDate: '2026-04-08',
  sources: Object.freeze([
    'docs/架构/135-可观测性与SLO治理设计-2026-04-07.md',
    'docs/架构/143-全局架构对齐与收口计划-2026-04-08.md',
    'docs/架构/145-可观测性发布门禁收口-2026-04-08.md',
  ]),
  targets: Object.freeze([
    createRatioTarget({
      id: 'gateway-availability',
      plane: 'data-plane',
      objective: 0.9995,
      description: 'Gateway release gate requires platform ingress availability evidence.',
      evidenceSources: ['sdkwork_http_requests_total', '/health', '/metrics'],
    }),
    createRatioTarget({
      id: 'gateway-non-streaming-success-rate',
      plane: 'data-plane',
      objective: 0.995,
      description: 'Non-streaming gateway completions must remain inside the release success budget.',
      evidenceSources: ['sdkwork_http_requests_total', 'RoutingDecisionLog'],
    }),
    createRatioTarget({
      id: 'gateway-streaming-completion-success-rate',
      plane: 'data-plane',
      objective: 0.99,
      description: 'Streaming responses must complete inside the governed success envelope.',
      evidenceSources: ['sdkwork_http_requests_total', 'RoutingDecisionLog'],
    }),
    createRatioTarget({
      id: 'gateway-fallback-success-rate',
      plane: 'data-plane',
      objective: 0.97,
      description: 'Fallback should stay viable enough to justify release continuation.',
      evidenceSources: ['RoutingDecisionLog', 'ProviderHealthSnapshot'],
    }),
    createBudgetTarget({
      id: 'gateway-provider-timeout-budget',
      plane: 'data-plane',
      objective: 0.01,
      description: 'Provider timeout share must stay below the governed budget.',
      evidenceSources: ['ProviderHealthSnapshot', 'sdkwork_http_requests_total'],
    }),
    createRatioTarget({
      id: 'admin-api-availability',
      plane: 'control-plane',
      objective: 0.999,
      description: 'Admin control-plane availability remains a release prerequisite.',
      evidenceSources: ['sdkwork_http_requests_total', '/metrics'],
    }),
    createRatioTarget({
      id: 'portal-api-availability',
      plane: 'control-plane',
      objective: 0.999,
      description: 'Portal control-plane availability remains a release prerequisite.',
      evidenceSources: ['sdkwork_http_requests_total', '/metrics'],
    }),
    createLatencyTarget({
      id: 'routing-simulation-p95-latency',
      plane: 'control-plane',
      objective: 600,
      description: 'Routing simulation p95 must stay below the governed operator threshold.',
      evidenceSources: ['/admin/routing/simulations', 'RoutingDecisionLog'],
    }),
    createRatioTarget({
      id: 'api-key-issuance-success-rate',
      plane: 'control-plane',
      objective: 0.99,
      description: 'API key issuance must stay within the governed success envelope.',
      evidenceSources: ['/admin/auth', '/portal/api-keys'],
    }),
    createRatioTarget({
      id: 'runtime-rollout-success-rate',
      plane: 'control-plane',
      objective: 0.99,
      description: 'Runtime reload and rollout must stay inside the governed change budget.',
      evidenceSources: ['runtime-config/rollouts', 'extensions/runtime-rollouts'],
    }),
    createRatioTarget({
      id: 'billing-event-write-success-rate',
      plane: 'commercial-plane',
      objective: 0.999,
      description: 'Billing event persistence remains a release-blocking commercial invariant.',
      evidenceSources: ['billing/events', 'billing/events/summary'],
    }),
    createRatioTarget({
      id: 'account-hold-creation-success-rate',
      plane: 'commercial-plane',
      objective: 0.99,
      description: 'Account hold creation must stay inside the governed commercial success envelope.',
      evidenceSources: ['billing/account-holds'],
    }),
    createRatioTarget({
      id: 'request-settlement-finalize-success-rate',
      plane: 'commercial-plane',
      objective: 0.99,
      description: 'Settlement finalization must stay inside the governed commercial success envelope.',
      evidenceSources: ['billing/request-settlements'],
    }),
    createRatioTarget({
      id: 'pricing-lifecycle-synchronize-success-rate',
      plane: 'commercial-plane',
      objective: 0.99,
      description: 'Pricing lifecycle synchronization must stay inside the governed commercial success envelope.',
      evidenceSources: ['billing/accounts', 'pricing lifecycle'],
    }),
  ]),
});

export function listSloGovernanceTargets({
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  return baseline.targets.map((target) => ({
    ...target,
    burnRateWindows: target.burnRateWindows.map((window) => ({ ...window })),
    evidenceSources: [...target.evidenceSources],
  }));
}

function readJsonFile(filePath) {
  return JSON.parse(readFileSync(filePath, 'utf8'));
}

function validateNumber(value) {
  return typeof value === 'number' && Number.isFinite(value);
}

function evaluateTarget({
  target,
  evidence,
} = {}) {
  const reasons = [];

  if (!evidence || typeof evidence !== 'object') {
    reasons.push('missing evidence payload');
    return {
      id: target.id,
      reasons,
    };
  }

  if (target.indicatorType === 'ratio_min') {
    if (!validateNumber(evidence.ratio)) {
      reasons.push('missing ratio evidence');
    } else if (evidence.ratio < target.objective) {
      reasons.push(`ratio ${evidence.ratio} is below objective ${target.objective}`);
    }
  } else if (target.indicatorType === 'ratio_max') {
    if (!validateNumber(evidence.ratio)) {
      reasons.push('missing ratio evidence');
    } else if (evidence.ratio > target.objective) {
      reasons.push(`ratio ${evidence.ratio} exceeds budget ${target.objective}`);
    }
  } else if (target.indicatorType === 'value_max') {
    if (!validateNumber(evidence.value)) {
      reasons.push('missing value evidence');
    } else if (evidence.value > target.objective) {
      reasons.push(`value ${evidence.value} exceeds max ${target.objective}${target.unit}`);
    }
  } else {
    reasons.push(`unsupported indicator type ${target.indicatorType}`);
  }

  const burnRates = evidence.burnRates;
  for (const window of target.burnRateWindows) {
    if (!burnRates || !validateNumber(burnRates[window.window])) {
      reasons.push(`missing burn rate for ${window.window}`);
      continue;
    }

    if (burnRates[window.window] > window.maxBurnRate) {
      reasons.push(
        `burn rate ${burnRates[window.window]} exceeds ${window.window} max ${window.maxBurnRate}`,
      );
    }
  }

  return {
    id: target.id,
    reasons,
  };
}

export function evaluateSloGovernanceEvidence({
  evidence,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  const evidenceTargets = evidence?.targets && typeof evidence.targets === 'object'
    ? evidence.targets
    : {};

  const failingTargets = [];
  const missingTargetIds = [];
  const passingTargetIds = [];

  for (const target of baseline.targets) {
    const targetEvidence = evidenceTargets[target.id];
    if (!targetEvidence) {
      missingTargetIds.push(target.id);
      continue;
    }

    const evaluation = evaluateTarget({
      target,
      evidence: targetEvidence,
    });

    if (evaluation.reasons.length > 0) {
      failingTargets.push(evaluation);
      continue;
    }

    passingTargetIds.push(target.id);
  }

  return {
    ok: missingTargetIds.length === 0 && failingTargets.length === 0,
    blocked: false,
    reason: '',
    generatedAt: String(evidence?.generatedAt ?? ''),
    baselineId: baseline.baselineId,
    passingTargetIds,
    missingTargetIds,
    failingTargetIds: failingTargets.map((target) => target.id),
    failingTargets,
  };
}

export function collectSloGovernanceResult({
  evidencePath = DEFAULT_SLO_GOVERNANCE_EVIDENCE_PATH,
  telemetrySnapshotPath,
  telemetrySnapshotJson,
  telemetryExportPath,
  telemetryExportJson,
  env = process.env,
  baseline = SLO_GOVERNANCE_BASELINE,
} = {}) {
  if (!existsSync(evidencePath)) {
    try {
      const explicitSnapshotPath = String(telemetrySnapshotPath ?? '').trim();
      const explicitSnapshotJson = String(telemetrySnapshotJson ?? '').trim();
      const explicitExportPath = String(telemetryExportPath ?? '').trim();
      const explicitExportJson = String(telemetryExportJson ?? '').trim();
      const envSnapshotPath = String(env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH ?? '').trim();
      const envSnapshotJson = String(env.SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON ?? '').trim();
      const envExportPath = String(env.SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH ?? '').trim();
      const envExportJson = String(env.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON ?? '').trim();

      const hasExplicitSnapshotFile = explicitSnapshotPath.length > 0 && existsSync(explicitSnapshotPath);
      let resolvedTelemetrySnapshotPath = hasExplicitSnapshotFile
        ? explicitSnapshotPath
        : '';
      if (!resolvedTelemetrySnapshotPath && existsSync(DEFAULT_RELEASE_TELEMETRY_SNAPSHOT_PATH)) {
        resolvedTelemetrySnapshotPath = DEFAULT_RELEASE_TELEMETRY_SNAPSHOT_PATH;
      }

      const hasSnapshotInput = resolvedTelemetrySnapshotPath.length > 0
        || explicitSnapshotJson.length > 0
        || envSnapshotPath.length > 0
        || envSnapshotJson.length > 0;
      const hasExportInput = explicitExportPath.length > 0
        || explicitExportJson.length > 0
        || envExportPath.length > 0
        || envExportJson.length > 0
        || existsSync(DEFAULT_RELEASE_TELEMETRY_EXPORT_PATH);

      if (!hasSnapshotInput && !hasExportInput) {
        return {
          ok: false,
          blocked: true,
          reason: 'evidence-missing',
          errorMessage: `missing ${path.relative(rootDir, evidencePath) || evidencePath}`,
          summary: null,
        };
      }

      if (!resolvedTelemetrySnapshotPath || !existsSync(resolvedTelemetrySnapshotPath)) {
        const snapshotResult = materializeReleaseTelemetrySnapshot({
          snapshotPath: hasExplicitSnapshotFile ? explicitSnapshotPath : undefined,
          snapshotJson: explicitSnapshotJson || undefined,
          exportPath: explicitExportPath || undefined,
          exportJson: explicitExportJson || undefined,
          env,
          outputPath: !hasExplicitSnapshotFile && explicitSnapshotPath
            ? explicitSnapshotPath
            : undefined,
        });
        resolvedTelemetrySnapshotPath = snapshotResult.outputPath;
      }

      materializeSloGovernanceEvidence({
        telemetrySnapshotPath: resolvedTelemetrySnapshotPath,
        outputPath: evidencePath,
        env,
      });
    } catch (error) {
      return {
        ok: false,
        blocked: false,
        reason: 'invalid-evidence',
        errorMessage: error instanceof Error ? error.message : String(error),
        summary: null,
      };
    }

    if (!existsSync(evidencePath)) {
      return {
        ok: false,
        blocked: true,
        reason: 'evidence-missing',
        errorMessage: `missing ${path.relative(rootDir, evidencePath) || evidencePath}`,
        summary: null,
      };
    }
  }

  try {
    const evidence = readJsonFile(evidencePath);
    const summary = evaluateSloGovernanceEvidence({
      evidence,
      baseline,
    });
    return {
      ok: summary.ok,
      blocked: summary.blocked,
      reason: summary.reason,
      errorMessage: '',
      summary,
    };
  } catch (error) {
    return {
      ok: false,
      blocked: false,
      reason: 'invalid-evidence',
      errorMessage: error instanceof Error ? error.message : String(error),
      summary: null,
    };
  }
}

function parseArgs(argv = process.argv.slice(2)) {
  let format = 'text';
  let evidencePath;
  let telemetrySnapshotPath;
  let telemetrySnapshotJson;
  let telemetryExportPath;
  let telemetryExportJson;

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--format') {
      format = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--evidence') {
      evidencePath = String(argv[index + 1] ?? '').trim();
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
    if (token === '--export') {
      telemetryExportPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--export-json') {
      telemetryExportJson = String(argv[index + 1] ?? '').trim();
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
    evidencePath,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    telemetryExportPath,
    telemetryExportJson,
  };
}

function formatText(result) {
  if (result.summary) {
    const summary = result.summary;
    return [
      `[release-slo-governance] ok=${summary.ok}`,
      `[release-slo-governance] baseline=${summary.baselineId}`,
      `[release-slo-governance] passing=${summary.passingTargetIds.length}`,
      `[release-slo-governance] missing=${summary.missingTargetIds.length}`,
      `[release-slo-governance] failing=${summary.failingTargetIds.length}`,
      ...summary.failingTargets.map((target) =>
        `[release-slo-governance] fail ${target.id}: ${target.reasons.join('; ')}`),
    ].join('\n');
  }

  return [
    `[release-slo-governance] blocked=${result.blocked}`,
    `[release-slo-governance] reason=${result.reason || 'unknown'}`,
    `[release-slo-governance] error=${result.errorMessage || 'unknown'}`,
  ].join('\n');
}

function main() {
  const {
    format,
    evidencePath,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    telemetryExportPath,
    telemetryExportJson,
  } = parseArgs();
  const result = collectSloGovernanceResult({
    evidencePath,
    telemetrySnapshotPath,
    telemetrySnapshotJson,
    telemetryExportPath,
    telemetryExportJson,
  });
  const payload = result.summary
    ? {
      ok: result.summary.ok,
      blocked: result.summary.blocked,
      reason: result.summary.reason,
      summary: result.summary,
    }
    : result;

  if (format === 'json') {
    console.log(JSON.stringify(payload, null, 2));
  } else {
    console.log(formatText(result));
  }

  if (!result.summary || !result.summary.ok) {
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
