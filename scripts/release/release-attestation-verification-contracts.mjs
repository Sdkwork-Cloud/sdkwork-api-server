import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export async function assertReleaseAttestationVerificationContracts({
  repoRoot,
} = {}) {
  const scriptPath = path.join(repoRoot, 'scripts', 'release', 'verify-release-attestations.mjs');
  assert.equal(existsSync(scriptPath), true, 'missing scripts/release/verify-release-attestations.mjs');

  const module = await import(pathToFileURL(scriptPath).href);

  assert.equal(typeof module.listReleaseAttestationSubjectSpecs, 'function');
  assert.equal(typeof module.resolveReleaseAttestationRepositorySlug, 'function');
  assert.equal(typeof module.resolveReleaseAttestationVerificationSubjects, 'function');
  assert.equal(typeof module.resolveGhRunner, 'function');
  assert.equal(typeof module.createReleaseAttestationVerificationPlan, 'function');
  assert.equal(typeof module.verifyReleaseAttestations, 'function');

  const subjectSpecs = module.listReleaseAttestationSubjectSpecs();
  assert.deepEqual(
    subjectSpecs.map((spec) => spec.id),
    [
      'release-window-snapshot',
      'release-sync-audit',
      'release-telemetry-export',
      'release-telemetry-snapshot',
      'release-slo-governance',
      'unix-installed-runtime-smoke',
      'windows-installed-runtime-smoke',
      'release-assets',
    ],
  );
  assert.equal(module.resolveReleaseAttestationRepositorySlug({ env: {} }), 'Sdkwork-Cloud/sdkwork-api-router');
}
