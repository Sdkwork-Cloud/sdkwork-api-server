import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createTelemetrySnapshotPayload() {
  return {
    generatedAt: '2026-04-08T10:00:00Z',
    targets: {
      'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
      'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
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

function createSupplementalTargetsPayload() {
  return {
    targets: createTelemetrySnapshotPayload().targets,
  };
}

test('release telemetry export materializer builds a governed export artifact from control-plane handoff inputs', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-export.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveReleaseTelemetryExportProducerInput, 'function');
  assert.equal(typeof module.materializeReleaseTelemetryExport, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-telemetry-export-'));
  const supplementalTargetsPath = path.join(fixtureRoot, 'supplemental-targets.json');
  const outputPath = path.join(fixtureRoot, 'release-telemetry-export-latest.json');

  writeFileSync(
    supplementalTargetsPath,
    JSON.stringify(createSupplementalTargetsPayload(), null, 2),
    'utf8',
  );

  const result = module.materializeReleaseTelemetryExport({
    generatedAt: '2026-04-08T10:00:00Z',
    sourceKind: 'observability-control-plane',
    sourceProvenance: 'synthetic-test',
    freshnessMinutes: 5,
    gatewayPrometheusText: createPrometheusHttpCounterSamples({
      service: 'gateway-service',
      healthyCount: 9997,
      unhealthyCount: 3,
    }),
    adminPrometheusText: createPrometheusHttpCounterSamples({
      service: 'admin-api-service',
      healthyCount: 4997,
      unhealthyCount: 3,
    }),
    portalPrometheusText: createPrometheusHttpCounterSamples({
      service: 'portal-api-service',
      healthyCount: 9993,
      unhealthyCount: 7,
    }),
    supplementalTargetsPath,
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  assert.equal(existsSync(outputPath), true);

  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.version, 1);
  assert.equal(written.generatedAt, '2026-04-08T10:00:00Z');
  assert.equal(written.source.kind, 'observability-control-plane');
  assert.equal(written.source.provenance, 'synthetic-test');
  assert.equal(written.source.freshnessMinutes, 5);
  assert.match(written.prometheus.gateway, /gateway-service/);
  assert.equal(written.supplemental.targets['routing-simulation-p95-latency'].value, 420);
});

test('release telemetry export materializer also accepts a governed export bundle directly', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-export.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-telemetry-export-'));
  const outputPath = path.join(fixtureRoot, 'release-telemetry-export-latest.json');

  const result = module.materializeReleaseTelemetryExport({
    exportJson: JSON.stringify({
      generatedAt: '2026-04-08T10:00:00Z',
      source: {
        kind: 'observability-control-plane',
        provenance: 'synthetic-test',
        freshnessMinutes: 5,
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
      supplemental: createSupplementalTargetsPayload(),
    }),
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.source.kind, 'observability-control-plane');
  assert.equal(written.supplemental.targets['gateway-fallback-success-rate'].ratio, 0.985);
});

test('release telemetry export materializer rejects missing producer input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-export.mjs'),
    ).href,
  );

  assert.throws(
    () => module.materializeReleaseTelemetryExport({
      env: {},
      outputPath: path.join(os.tmpdir(), 'unused-release-telemetry-export.json'),
    }),
    /missing release telemetry export input/i,
  );
});
