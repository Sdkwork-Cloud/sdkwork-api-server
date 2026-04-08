import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createEvidencePayload() {
  return {
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
}

function createTelemetrySnapshotPayload() {
  return {
    generatedAt: '2026-04-08T10:00:00Z',
    source: {
      kind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
    },
    targets: createEvidencePayload().targets,
  };
}

test('slo evidence materializer exports helpers and writes a governed release artifact from a source file', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveSloGovernanceEvidenceInput, 'function');
  assert.equal(typeof module.validateSloGovernanceEvidenceShape, 'function');
  assert.equal(typeof module.materializeSloGovernanceEvidence, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-evidence-'));
  const sourcePath = path.join(fixtureRoot, 'source.json');
  const outputPath = path.join(fixtureRoot, 'slo-governance-latest.json');

  writeFileSync(sourcePath, JSON.stringify(createEvidencePayload(), null, 2), 'utf8');

  const result = module.materializeSloGovernanceEvidence({
    evidencePath: sourcePath,
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  assert.equal(existsSync(outputPath), true);

  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.baselineId, 'release-slo-governance-baseline-2026-04-08');
  assert.equal(written.generatedAt, '2026-04-08T10:00:00Z');
  assert.equal(written.targets['routing-simulation-p95-latency'].value, 420);
});

test('slo evidence materializer also accepts direct JSON input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-evidence-'));
  const outputPath = path.join(fixtureRoot, 'slo-governance-latest.json');

  const result = module.materializeSloGovernanceEvidence({
    evidenceJson: JSON.stringify(createEvidencePayload()),
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.targets['gateway-availability'].ratio, 0.9997);
});

test('slo evidence materializer derives governed release evidence from a telemetry snapshot file', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-evidence-'));
  const snapshotPath = path.join(fixtureRoot, 'release-telemetry-snapshot.json');
  const outputPath = path.join(fixtureRoot, 'slo-governance-latest.json');

  writeFileSync(snapshotPath, JSON.stringify(createTelemetrySnapshotPayload(), null, 2), 'utf8');

  const result = module.materializeSloGovernanceEvidence({
    telemetrySnapshotPath: snapshotPath,
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  assert.equal(result.source, 'telemetry-snapshot-file');

  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.baselineId, 'release-slo-governance-baseline-2026-04-08');
  assert.equal(written.targets['routing-simulation-p95-latency'].value, 420);
});

test('slo evidence materializer rejects missing evidence input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );

  assert.throws(
    () => module.materializeSloGovernanceEvidence({
      env: {},
      outputPath: path.join(os.tmpdir(), 'unused.json'),
    }),
    /missing SLO governance evidence input/i,
  );
});

test('slo evidence materializer accepts UTF-8 BOM encoded source files for Windows interoperability', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-slo-evidence-'));
  const sourcePath = path.join(fixtureRoot, 'source-with-bom.json');
  const outputPath = path.join(fixtureRoot, 'slo-governance-latest.json');

  writeFileSync(
    sourcePath,
    `\uFEFF${JSON.stringify(createEvidencePayload(), null, 2)}`,
    'utf8',
  );

  const result = module.materializeSloGovernanceEvidence({
    evidencePath: sourcePath,
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.generatedAt, '2026-04-08T10:00:00Z');
});
