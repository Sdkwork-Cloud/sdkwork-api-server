import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const DIRECTLY_DERIVED_AVAILABILITY_TARGET_IDS = new Set([
  'gateway-availability',
  'admin-api-availability',
  'portal-api-availability',
]);

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

function createPrometheusHttpCounterSamples({
  service,
  healthyCount,
  unhealthyCount,
} = {}) {
  return [
    '# HELP sdkwork_http_requests_total Total HTTP requests observed',
    '# TYPE sdkwork_http_requests_total counter',
    `sdkwork_http_requests_total{service="${service}",method="GET",route="/health",status="200"} ${healthyCount}`,
    `sdkwork_http_requests_total{service="${service}",method="GET",route="/health",status="503"} ${unhealthyCount}`,
  ].join('\n');
}

function createTelemetryExportPayload() {
  const snapshotPayload = createTelemetrySnapshotPayload();
  const supplementalTargets = Object.fromEntries(
    Object.entries(snapshotPayload.targets).filter(
      ([targetId]) => !DIRECTLY_DERIVED_AVAILABILITY_TARGET_IDS.has(targetId),
    ),
  );

  return {
    generatedAt: snapshotPayload.generatedAt,
    source: {
      kind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
    },
    prometheus: {
      gateway: createPrometheusHttpCounterSamples({
        service: 'gateway-service',
        healthyCount: 9997,
        unhealthyCount: 3,
      }),
      admin: createPrometheusHttpCounterSamples({
        service: 'admin-api-service',
        healthyCount: 4997,
        unhealthyCount: 3,
      }),
      portal: createPrometheusHttpCounterSamples({
        service: 'portal-api-service',
        healthyCount: 9993,
        unhealthyCount: 7,
      }),
    },
    supplemental: {
      targets: supplementalTargets,
    },
  };
}

test('release telemetry snapshot materializer derives a governed snapshot from a release telemetry export bundle', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveReleaseTelemetryExportInput, 'function');
  assert.equal(typeof module.resolveReleaseTelemetrySnapshotInput, 'function');
  assert.equal(typeof module.deriveReleaseTelemetrySnapshotFromExport, 'function');
  assert.equal(typeof module.validateReleaseTelemetrySnapshotShape, 'function');
  assert.equal(typeof module.materializeReleaseTelemetrySnapshot, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-telemetry-'));
  const exportPath = path.join(fixtureRoot, 'release-telemetry-export.json');
  const outputPath = path.join(fixtureRoot, 'release-telemetry-snapshot-latest.json');
  const exportPayload = createTelemetryExportPayload();

  writeFileSync(exportPath, JSON.stringify(exportPayload, null, 2), 'utf8');

  const resolved = module.resolveReleaseTelemetryExportInput({
    exportPath,
  });
  assert.equal(resolved.source, 'file');

  const derived = module.deriveReleaseTelemetrySnapshotFromExport({
    exportBundle: resolved.payload,
  });
  assert.equal(derived.generatedAt, exportPayload.generatedAt);
  assert.equal(derived.source.kind, 'release-telemetry-export');
  assert.equal(derived.source.exportKind, 'observability-control-plane');
  assert.equal(derived.targets['gateway-availability'].ratio, 0.9997);
  assert.equal(derived.targets['admin-api-availability'].ratio, 0.9994);
  assert.equal(derived.targets['portal-api-availability'].ratio, 0.9993);
  assert.equal(derived.targets['routing-simulation-p95-latency'].value, 420);

  const result = module.materializeReleaseTelemetrySnapshot({
    exportPath,
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  assert.equal(existsSync(outputPath), true);

  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.snapshotId, 'release-telemetry-snapshot-v1');
  assert.equal(written.generatedAt, '2026-04-08T10:00:00Z');
  assert.equal(written.source.kind, 'release-telemetry-export');
  assert.equal(written.source.exportKind, 'observability-control-plane');
  assert.equal(written.targets['gateway-availability'].ratio, 0.9997);
  assert.equal(written.targets['routing-simulation-p95-latency'].value, 420);
});

test('release telemetry snapshot materializer also accepts direct JSON input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-telemetry-'));
  const outputPath = path.join(fixtureRoot, 'release-telemetry-snapshot-latest.json');

  const result = module.materializeReleaseTelemetrySnapshot({
    snapshotJson: JSON.stringify(createTelemetrySnapshotPayload()),
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.targets['gateway-availability'].ratio, 0.9997);
});

test('release telemetry snapshot materializer rejects missing snapshot input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    ).href,
  );

  assert.throws(
    () => module.materializeReleaseTelemetrySnapshot({
      env: {},
      outputPath: path.join(os.tmpdir(), 'unused.json'),
    }),
    /missing release telemetry input/i,
  );
});

test('release telemetry snapshot materializer accepts UTF-8 BOM encoded source files for Windows interoperability', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-telemetry-'));
  const sourcePath = path.join(fixtureRoot, 'release-telemetry-source-with-bom.json');
  const outputPath = path.join(fixtureRoot, 'release-telemetry-snapshot-latest.json');

  writeFileSync(
    sourcePath,
    `\uFEFF${JSON.stringify(createTelemetrySnapshotPayload(), null, 2)}`,
    'utf8',
  );

  const result = module.materializeReleaseTelemetrySnapshot({
    snapshotPath: sourcePath,
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.generatedAt, '2026-04-08T10:00:00Z');
});
