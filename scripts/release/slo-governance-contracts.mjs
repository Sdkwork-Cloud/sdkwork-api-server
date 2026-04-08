import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export async function assertSloGovernanceContracts({
  repoRoot,
} = {}) {
  const modulePath = path.join(repoRoot, 'scripts', 'release', 'slo-governance.mjs');
  const testPath = path.join(repoRoot, 'scripts', 'release', 'tests', 'release-slo-governance.test.mjs');

  assert.equal(existsSync(modulePath), true, 'missing scripts/release/slo-governance.mjs');
  assert.equal(existsSync(testPath), true, 'missing scripts/release/tests/release-slo-governance.test.mjs');
  assert.equal(
    existsSync(path.join(repoRoot, 'docs', '架构', '135-可观测性与SLO治理设计-2026-04-07.md')),
    true,
    'missing docs/架构/135-可观测性与SLO治理设计-2026-04-07.md',
  );
  assert.equal(
    existsSync(path.join(repoRoot, 'docs', '架构', '143-全局架构对齐与收口计划-2026-04-08.md')),
    true,
    'missing docs/架构/143-全局架构对齐与收口计划-2026-04-08.md',
  );

  const module = await import(pathToFileURL(modulePath).href);
  assert.equal(typeof module.listSloGovernanceTargets, 'function');
  assert.equal(typeof module.evaluateSloGovernanceEvidence, 'function');
  assert.equal(typeof module.collectSloGovernanceResult, 'function');

  const targets = module.listSloGovernanceTargets();
  assert.equal(targets.length, 14);
  assert.deepEqual(
    [...new Set(targets.map((target) => target.plane))].sort(),
    ['commercial-plane', 'control-plane', 'data-plane'],
  );
  assert.ok(
    targets.every((target) => Array.isArray(target.evidenceSources) && target.evidenceSources.length > 0),
    'every governed SLO target should cite at least one evidence source',
  );
  assert.ok(
    targets.every((target) => Array.isArray(target.burnRateWindows) && target.burnRateWindows.length === 2),
    'every governed SLO target should expose fast and slow burn-rate windows',
  );
}
