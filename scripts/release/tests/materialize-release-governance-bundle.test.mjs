import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function writeJson(root, relativePath, payload) {
  const targetPath = path.join(root, relativePath);
  mkdirSync(path.dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
  return targetPath;
}

function createGovernanceTargets() {
  return {
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
  };
}

function createSupplementalTargets() {
  const targets = createGovernanceTargets();
  return Object.fromEntries(
    Object.entries(targets).filter(([targetId]) => ![
      'gateway-availability',
      'admin-api-availability',
      'portal-api-availability',
    ].includes(targetId)),
  );
}

function createGovernanceArtifacts(root) {
  writeJson(root, 'docs/release/release-window-snapshot-latest.json', {
    version: 1,
    generatedAt: '2026-04-09T09:00:00Z',
    source: { kind: 'release-window-snapshot-fixture', provenance: 'synthetic-test' },
    snapshot: {
      latestReleaseTag: 'release-2026-03-28-8',
      commitsSinceLatestRelease: 16,
      workingTreeEntryCount: 627,
      hasReleaseBaseline: true,
    },
  });
  writeJson(root, 'docs/release/release-sync-audit-latest.json', {
    version: 1,
    generatedAt: '2026-04-09T09:01:00Z',
    source: { kind: 'release-sync-audit-fixture', provenance: 'synthetic-test' },
    summary: {
      releasable: true,
      reports: [
        {
          id: 'sdkwork-api-router',
          targetDir: root,
          expectedGitRoot: root,
          topLevel: root,
          remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
          localHead: 'abc123',
          remoteHead: 'abc123',
          expectedRef: 'main',
          branch: 'main',
          upstream: 'origin/main',
          ahead: 0,
          behind: 0,
          isDirty: false,
          reasons: [],
          releasable: true,
        },
      ],
    },
  });
  writeJson(root, 'docs/release/release-telemetry-export-latest.json', {
    version: 1,
    generatedAt: '2026-04-09T09:02:00Z',
    source: { kind: 'observability-control-plane', provenance: 'synthetic-test', freshnessMinutes: 5 },
    prometheus: {
      gateway: '# HELP sdkwork_http_requests_total Total HTTP requests observed',
      admin: '# HELP sdkwork_http_requests_total Total HTTP requests observed',
      portal: '# HELP sdkwork_http_requests_total Total HTTP requests observed',
    },
    supplemental: {
      targets: createSupplementalTargets(),
    },
  });
  writeJson(root, 'docs/release/release-telemetry-snapshot-latest.json', {
    version: 1,
    snapshotId: 'release-telemetry-snapshot-v1',
    generatedAt: '2026-04-09T09:03:00Z',
    source: {
      kind: 'release-telemetry-export',
      exportKind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
      directTargetIds: ['gateway-availability', 'admin-api-availability', 'portal-api-availability'],
      supplementalTargetIds: Object.keys(createSupplementalTargets()),
    },
    targets: createGovernanceTargets(),
  });
  writeJson(root, 'docs/release/slo-governance-latest.json', {
    version: 1,
    baselineId: 'release-slo-governance-baseline-2026-04-08',
    baselineDate: '2026-04-08',
    generatedAt: '2026-04-09T09:04:00Z',
    targets: createGovernanceTargets(),
  });
}

test('release governance bundle materializer writes a single bundle directory plus manifest for restore operators', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-governance-bundle.mjs'),
    ).href,
  );

  assert.equal(typeof module.listReleaseGovernanceBundleArtifactSpecs, 'function');
  assert.equal(typeof module.createReleaseGovernanceBundleManifest, 'function');
  assert.equal(typeof module.materializeReleaseGovernanceBundle, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-bundle-'));
  createGovernanceArtifacts(fixtureRoot);

  try {
    const result = module.materializeReleaseGovernanceBundle({
      repoRoot: fixtureRoot,
      outputDir: path.join(fixtureRoot, 'artifacts', 'release-governance-bundle'),
      generatedAt: '2026-04-09T09:05:00Z',
    });

    assert.equal(result.bundleEntryCount, 5);
    assert.equal(existsSync(path.join(result.outputDir, 'docs', 'release', 'release-window-snapshot-latest.json')), true);
    assert.equal(existsSync(path.join(result.outputDir, 'release-governance-bundle-manifest.json')), true);

    const manifest = JSON.parse(
      readFileSync(path.join(result.outputDir, 'release-governance-bundle-manifest.json'), 'utf8'),
    );
    assert.equal(manifest.version, 1);
    assert.equal(manifest.bundleEntryCount, 5);
    assert.deepEqual(
      manifest.artifacts.map((artifact) => artifact.id),
      [
        'release-window-snapshot',
        'release-sync-audit',
        'release-telemetry-export',
        'release-telemetry-snapshot',
        'release-slo-governance',
      ],
    );
    assert.match(
      manifest.restore.command,
      /node scripts\/release\/restore-release-governance-latest\.mjs --artifact-dir/,
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
