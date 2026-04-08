import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('observability contract helper exposes the governed release assertions', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'observability-contracts.mjs'),
    ).href,
  );

  assert.equal(typeof module.assertObservabilityContracts, 'function');

  await assert.doesNotReject(
    module.assertObservabilityContracts({
      repoRoot,
    }),
  );
});
