import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('slo governance contracts require the governed architecture baselines and machine-readable targets', () => {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'slo-governance-contracts.mjs'),
    ).href,
  ).then((module) => module.assertSloGovernanceContracts({
    repoRoot,
  }));
});
