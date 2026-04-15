import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const defaultReleaseTelemetrySnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const defaultReleaseTelemetryExportPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
);

function withTemporarilyMissingFiles(filePaths, callback) {
  const originals = filePaths.map((filePath) => ({
    filePath,
    hadOriginalFile: existsSync(filePath),
    originalContent: existsSync(filePath) ? readFileSync(filePath, 'utf8') : null,
  }));

  for (const entry of originals) {
    if (entry.hadOriginalFile) {
      rmSync(entry.filePath, { force: true });
    }
  }

  try {
    return callback();
  } finally {
    for (const entry of originals) {
      if (entry.hadOriginalFile) {
        writeFileSync(entry.filePath, entry.originalContent, 'utf8');
      }
    }
  }
}

function createTelemetrySnapshotPayload() {
  return {
    generatedAt: '2026-04-08T10:00:00Z',
    source: {
      kind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
    },
    targets: {
      'gateway-availability': { ratio: 0.9997, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
      'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
      'admin-api-availability': { ratio: 0.9994, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'portal-api-availability': { ratio: 0.9993, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'routing-simulation-p95-latency': { value: 420, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'api-key-issuance-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'runtime-rollout-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'billing-event-write-success-rate': { ratio: 0.9995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'account-hold-creation-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'request-settlement-finalize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'pricing-lifecycle-synchronize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    },
  };
}

function createTelemetryExportPayload() {
  const snapshot = createTelemetrySnapshotPayload();
  return {
    generatedAt: snapshot.generatedAt,
    source: snapshot.source,
    prometheus: {
      gateway: [
        '# HELP sdkwork_http_requests_total Total HTTP requests observed',
        '# TYPE sdkwork_http_requests_total counter',
        'sdkwork_http_requests_total{service="gateway-service",method="GET",route="/health",status="200"} 9997',
        'sdkwork_http_requests_total{service="gateway-service",method="GET",route="/health",status="503"} 3',
      ].join('\n'),
      admin: [
        '# HELP sdkwork_http_requests_total Total HTTP requests observed',
        '# TYPE sdkwork_http_requests_total counter',
        'sdkwork_http_requests_total{service="admin-api-service",method="GET",route="/health",status="200"} 4997',
        'sdkwork_http_requests_total{service="admin-api-service",method="GET",route="/health",status="503"} 3',
      ].join('\n'),
      portal: [
        '# HELP sdkwork_http_requests_total Total HTTP requests observed',
        '# TYPE sdkwork_http_requests_total counter',
        'sdkwork_http_requests_total{service="portal-api-service",method="GET",route="/health",status="200"} 9993',
        'sdkwork_http_requests_total{service="portal-api-service",method="GET",route="/health",status="503"} 7',
      ].join('\n'),
    },
    supplemental: {
      targets: {
        'gateway-non-streaming-success-rate': snapshot.targets['gateway-non-streaming-success-rate'],
        'gateway-streaming-completion-success-rate': snapshot.targets['gateway-streaming-completion-success-rate'],
        'gateway-fallback-success-rate': snapshot.targets['gateway-fallback-success-rate'],
        'gateway-provider-timeout-budget': snapshot.targets['gateway-provider-timeout-budget'],
        'routing-simulation-p95-latency': snapshot.targets['routing-simulation-p95-latency'],
        'api-key-issuance-success-rate': snapshot.targets['api-key-issuance-success-rate'],
        'runtime-rollout-success-rate': snapshot.targets['runtime-rollout-success-rate'],
        'billing-event-write-success-rate': snapshot.targets['billing-event-write-success-rate'],
        'account-hold-creation-success-rate': snapshot.targets['account-hold-creation-success-rate'],
        'request-settlement-finalize-success-rate': snapshot.targets['request-settlement-finalize-success-rate'],
        'pricing-lifecycle-synchronize-success-rate': snapshot.targets['pricing-lifecycle-synchronize-success-rate'],
      },
    },
  };
}

test('slo governance exposes a machine-readable quantitative baseline', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );

  assert.equal(typeof module.listSloGovernanceTargets, 'function');
  assert.equal(typeof module.evaluateSloGovernanceEvidence, 'function');
  assert.equal(typeof module.collectSloGovernanceResult, 'function');

  const targets = module.listSloGovernanceTargets();
  assert.equal(targets.length, 14);
  assert.deepEqual(
    targets.map((target) => target.id),
    [
      'gateway-availability',
      'gateway-non-streaming-success-rate',
      'gateway-streaming-completion-success-rate',
      'gateway-fallback-success-rate',
      'gateway-provider-timeout-budget',
      'admin-api-availability',
      'portal-api-availability',
      'routing-simulation-p95-latency',
      'api-key-issuance-success-rate',
      'runtime-rollout-success-rate',
      'billing-event-write-success-rate',
      'account-hold-creation-success-rate',
      'request-settlement-finalize-success-rate',
      'pricing-lifecycle-synchronize-success-rate',
    ],
  );
  assert.ok(
    targets.every((target) => Array.isArray(target.burnRateWindows) && target.burnRateWindows.length === 2),
    'every governed SLO target should expose fast and slow burn-rate windows',
  );
});

test('slo governance passes when evidence satisfies objectives and burn-rate ceilings', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );

  const evidence = {
    generatedAt: '2026-04-08T10:00:00Z',
    targets: {
      'gateway-availability': { ratio: 0.9997, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
      'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
      'admin-api-availability': { ratio: 0.9994, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'portal-api-availability': { ratio: 0.9993, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'routing-simulation-p95-latency': { value: 420, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'api-key-issuance-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'runtime-rollout-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'billing-event-write-success-rate': { ratio: 0.9995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'account-hold-creation-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'request-settlement-finalize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'pricing-lifecycle-synchronize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    },
  };

  const result = module.evaluateSloGovernanceEvidence({ evidence });
  assert.equal(result.ok, true);
  assert.equal(result.blocked, false);
  assert.deepEqual(result.failingTargetIds, []);
  assert.deepEqual(result.missingTargetIds, []);
});

test('slo governance fails when evidence breaches a quantitative objective or burn-rate ceiling', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );

  const evidence = {
    generatedAt: '2026-04-08T10:00:00Z',
    targets: {
      'gateway-availability': { ratio: 0.9997, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
      'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
      'admin-api-availability': { ratio: 0.9994, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'portal-api-availability': { ratio: 0.9993, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'routing-simulation-p95-latency': { value: 950, burnRates: { '1h': 18, '6h': 0.5 } },
      'api-key-issuance-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'runtime-rollout-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'billing-event-write-success-rate': { ratio: 0.9995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'account-hold-creation-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'request-settlement-finalize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'pricing-lifecycle-synchronize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    },
  };

  const result = module.evaluateSloGovernanceEvidence({ evidence });
  assert.equal(result.ok, false);
  assert.equal(result.blocked, false);
  assert.deepEqual(result.missingTargetIds, []);
  assert.ok(result.failingTargetIds.includes('routing-simulation-p95-latency'));
  assert.ok(
    result.failingTargets.some((target) =>
      target.id === 'routing-simulation-p95-latency'
      && target.reasons.some((reason) => reason.includes('burn rate')),
    ),
  );
});

test('slo governance reports evidence-missing as a blocked release-governance result', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-governance-missing-'));
  const missingEvidencePath = path.join(fixtureRoot, 'missing-slo-evidence.json');

  const result = withTemporarilyMissingFiles(
    [
      defaultReleaseTelemetrySnapshotPath,
      defaultReleaseTelemetryExportPath,
    ],
    () => module.collectSloGovernanceResult({
      evidencePath: missingEvidencePath,
    }),
  );

  assert.equal(result.ok, false);
  assert.equal(result.blocked, true);
  assert.equal(result.reason, 'evidence-missing');
  assert.equal(result.summary, null);
});

test('slo governance collection mirrors the evaluation result when a quantitative evidence file exists', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-governance-'));
  const evidencePath = path.join(fixtureRoot, 'slo-evidence.json');
  writeFileSync(evidencePath, JSON.stringify({
    generatedAt: '2026-04-08T10:00:00Z',
    targets: {
      'gateway-availability': { ratio: 0.9997, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
      'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
      'admin-api-availability': { ratio: 0.9994, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'portal-api-availability': { ratio: 0.9993, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'routing-simulation-p95-latency': { value: 420, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'api-key-issuance-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'runtime-rollout-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'billing-event-write-success-rate': { ratio: 0.9995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'account-hold-creation-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'request-settlement-finalize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'pricing-lifecycle-synchronize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    },
  }, null, 2), 'utf8');

  const result = module.collectSloGovernanceResult({
    evidencePath,
  });

  assert.equal(result.ok, true);
  assert.equal(result.blocked, false);
  assert.equal(result.reason, '');
  assert.ok(result.summary);
  assert.equal(result.summary.ok, true);
});

test('slo governance materializes governed evidence from a telemetry snapshot when the evidence artifact is missing', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-governance-'));
  const evidencePath = path.join(fixtureRoot, 'slo-governance-latest.json');
  const telemetrySnapshotPath = path.join(fixtureRoot, 'release-telemetry-snapshot-latest.json');
  writeFileSync(
    telemetrySnapshotPath,
    `${JSON.stringify(createTelemetrySnapshotPayload(), null, 2)}\n`,
    'utf8',
  );

  const result = module.collectSloGovernanceResult({
    evidencePath,
    telemetrySnapshotPath,
  });

  assert.equal(result.ok, true);
  assert.equal(result.blocked, false);
  assert.equal(result.reason, '');
  assert.ok(result.summary);
  assert.equal(result.summary.ok, true);
  assert.equal(existsSync(evidencePath), true);
  assert.equal(
    JSON.parse(readFileSync(evidencePath, 'utf8')).baselineId,
    'release-slo-governance-baseline-2026-04-08',
  );
});

test('slo governance materializes governed evidence from a telemetry export when both evidence and snapshot artifacts are missing', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-governance-'));
  const evidencePath = path.join(fixtureRoot, 'slo-governance-latest.json');
  const telemetrySnapshotPath = path.join(fixtureRoot, 'release-telemetry-snapshot-latest.json');
  const telemetryExportPath = path.join(fixtureRoot, 'release-telemetry-export-latest.json');
  writeFileSync(
    telemetryExportPath,
    `${JSON.stringify(createTelemetryExportPayload(), null, 2)}\n`,
    'utf8',
  );

  const result = withTemporarilyMissingFiles(
    [
      defaultReleaseTelemetrySnapshotPath,
      defaultReleaseTelemetryExportPath,
    ],
    () => module.collectSloGovernanceResult({
      evidencePath,
      telemetrySnapshotPath,
      telemetryExportPath,
    }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.blocked, false);
  assert.equal(result.reason, '');
  assert.ok(result.summary);
  assert.equal(result.summary.ok, true);
  assert.equal(existsSync(telemetrySnapshotPath), true);
  assert.equal(existsSync(evidencePath), true);
  assert.equal(
    JSON.parse(readFileSync(evidencePath, 'utf8')).baselineId,
    'release-slo-governance-baseline-2026-04-08',
  );
});
