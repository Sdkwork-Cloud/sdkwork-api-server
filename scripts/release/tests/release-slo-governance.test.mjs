import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

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

  const result = module.collectSloGovernanceResult({
    evidencePath: path.join(repoRoot, 'scripts', 'release', 'fixtures', 'missing-slo-evidence.json'),
  });

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
