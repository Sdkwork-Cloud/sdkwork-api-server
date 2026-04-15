import assert from 'node:assert/strict';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const releaseWindowSnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-window-snapshot-latest.json',
);
const releaseSyncAuditPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-sync-audit-latest.json',
);
const releaseTelemetryExportPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
);
const releaseTelemetrySnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const sloGovernanceEvidencePath = path.join(
  repoRoot,
  'docs',
  'release',
  'slo-governance-latest.json',
);

const DIRECTLY_DERIVED_AVAILABILITY_TARGET_IDS = new Set([
  'gateway-availability',
  'admin-api-availability',
  'portal-api-availability',
]);

function createReleaseWindowSnapshotArtifactPayload() {
  return {
    version: 1,
    generatedAt: '2026-04-08T12:00:00Z',
    source: {
      kind: 'release-window-snapshot-fixture',
      provenance: 'synthetic-test',
    },
    snapshot: {
      latestReleaseTag: 'release-2026-03-28-8',
      commitsSinceLatestRelease: 16,
      workingTreeEntryCount: 627,
      hasReleaseBaseline: true,
    },
  };
}

function createReleaseSyncAuditArtifactPayload(head = 'abc123') {
  return {
    version: 1,
    generatedAt: '2026-04-08T13:00:00Z',
    source: {
      kind: 'release-sync-audit-fixture',
      provenance: 'synthetic-test',
    },
    summary: {
      releasable: true,
      reports: [
        {
          id: 'sdkwork-api-router',
          targetDir: repoRoot,
          expectedGitRoot: repoRoot,
          topLevel: repoRoot,
          remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
          localHead: head,
          remoteHead: head,
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
  };
}

function createTelemetrySnapshotPayload() {
  return {
    version: 1,
    snapshotId: 'release-telemetry-snapshot-v1',
    generatedAt: '2026-04-08T10:00:00Z',
    source: {
      kind: 'release-telemetry-export',
      exportKind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
      directTargetIds: [
        'admin-api-availability',
        'gateway-availability',
        'portal-api-availability',
      ],
      supplementalTargetIds: [
        'account-hold-creation-success-rate',
        'api-key-issuance-success-rate',
        'billing-event-write-success-rate',
        'gateway-fallback-success-rate',
        'gateway-non-streaming-success-rate',
        'gateway-provider-timeout-budget',
        'gateway-streaming-completion-success-rate',
        'pricing-lifecycle-synchronize-success-rate',
        'request-settlement-finalize-success-rate',
        'routing-simulation-p95-latency',
        'runtime-rollout-success-rate',
      ],
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
    version: 1,
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

function createSloGovernancePayload() {
  return {
    version: 1,
    baselineId: 'release-slo-governance-baseline-2026-04-08',
    baselineDate: '2026-04-08',
    generatedAt: '2026-04-08T10:00:00Z',
    targets: createTelemetrySnapshotPayload().targets,
  };
}

function cleanupGovernedReleaseArtifacts() {
  for (const artifactPath of [
    releaseWindowSnapshotPath,
    releaseSyncAuditPath,
    releaseTelemetryExportPath,
    releaseTelemetrySnapshotPath,
    sloGovernanceEvidencePath,
  ]) {
    if (existsSync(artifactPath)) {
      rmSync(artifactPath, { force: true });
    }
  }
}

async function withCleanedGovernedReleaseArtifacts(callback) {
  const originals = [
    releaseWindowSnapshotPath,
    releaseSyncAuditPath,
    releaseTelemetryExportPath,
    releaseTelemetrySnapshotPath,
    sloGovernanceEvidencePath,
  ].map((filePath) => ({
    filePath,
    hadOriginalFile: existsSync(filePath),
    originalContent: existsSync(filePath) ? readFileSync(filePath, 'utf8') : null,
  }));

  cleanupGovernedReleaseArtifacts();

  try {
    return await callback();
  } finally {
    cleanupGovernedReleaseArtifacts();
    for (const entry of originals) {
      if (entry.hadOriginalFile) {
        writeFileSync(entry.filePath, entry.originalContent, 'utf8');
      }
    }
  }
}

function writeArtifact(root, directoryName, relativePath, payload) {
  const targetPath = path.join(root, directoryName, relativePath);
  mkdirSync(path.dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
}

test('restore release governance latest materializer restores required governance artifacts from a downloaded artifact directory', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
    ).href,
  );

  assert.equal(typeof module.listReleaseGovernanceLatestArtifactSpecs, 'function');
  assert.equal(typeof module.resolveReleaseGovernanceLatestArtifactSources, 'function');
  assert.equal(typeof module.restoreReleaseGovernanceLatestArtifacts, 'function');

  const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));
  const targetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-target-'));

  writeArtifact(
    artifactRoot,
    'release-governance-window-snapshot-web',
    path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    createReleaseWindowSnapshotArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-sync-audit-web',
    path.join('docs', 'release', 'release-sync-audit-latest.json'),
    createReleaseSyncAuditArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-export-web',
    path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    createTelemetryExportPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-snapshot-web',
    path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    createTelemetrySnapshotPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-slo-evidence-web',
    path.join('docs', 'release', 'slo-governance-latest.json'),
    createSloGovernancePayload(),
  );

  const result = module.restoreReleaseGovernanceLatestArtifacts({
    artifactDir: artifactRoot,
    repoRoot: targetRoot,
  });

  assert.equal(result.restored.length, 5);
  assert.equal(
    existsSync(path.join(targetRoot, 'docs', 'release', 'release-window-snapshot-latest.json')),
    true,
  );
  assert.equal(
    JSON.parse(
      readFileSync(
        path.join(targetRoot, 'docs', 'release', 'release-sync-audit-latest.json'),
        'utf8',
      ),
    ).summary.releasable,
    true,
  );
});

test('restore release governance latest materializer tolerates duplicate identical artifacts across lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
    ).href,
  );

  const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));
  const targetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-target-'));
  const syncAuditPayload = createReleaseSyncAuditArtifactPayload();

  for (const directoryName of ['release-governance-sync-audit-web', 'release-governance-sync-audit-linux-x64']) {
    writeArtifact(
      artifactRoot,
      directoryName,
      path.join('docs', 'release', 'release-sync-audit-latest.json'),
      syncAuditPayload,
    );
  }

  writeArtifact(
    artifactRoot,
    'release-governance-window-snapshot-web',
    path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    createReleaseWindowSnapshotArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-export-web',
    path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    createTelemetryExportPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-snapshot-web',
    path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    createTelemetrySnapshotPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-slo-evidence-web',
    path.join('docs', 'release', 'slo-governance-latest.json'),
    createSloGovernancePayload(),
  );

  const result = module.restoreReleaseGovernanceLatestArtifacts({
    artifactDir: artifactRoot,
    repoRoot: targetRoot,
  });

  assert.equal(result.restored.length, 5);
  assert.match(
    result.restored.find((item) => item.id === 'release-sync-audit')?.sourcePath ?? '',
    /release-governance-sync-audit-/,
  );
});

test('restore release governance latest materializer rejects conflicting duplicate artifacts', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
    ).href,
  );

  const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));
  const targetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-target-'));

  writeArtifact(
    artifactRoot,
    'release-governance-sync-audit-web',
    path.join('docs', 'release', 'release-sync-audit-latest.json'),
    createReleaseSyncAuditArtifactPayload('abc123'),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-sync-audit-linux-x64',
    path.join('docs', 'release', 'release-sync-audit-latest.json'),
    createReleaseSyncAuditArtifactPayload('def456'),
  );

  writeArtifact(
    artifactRoot,
    'release-governance-window-snapshot-web',
    path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    createReleaseWindowSnapshotArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-export-web',
    path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    createTelemetryExportPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-snapshot-web',
    path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    createTelemetrySnapshotPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-slo-evidence-web',
    path.join('docs', 'release', 'slo-governance-latest.json'),
    createSloGovernancePayload(),
  );

  assert.throws(
    () => module.restoreReleaseGovernanceLatestArtifacts({
      artifactDir: artifactRoot,
      repoRoot: targetRoot,
    }),
    /conflicting duplicate governance artifact/i,
  );
});

test('restore release governance latest materializer enables blocked-host governance replay when real latest artifacts are restored', async () => {
  await withCleanedGovernedReleaseArtifacts(async () => {
    const restoreModule = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
      ).href,
    );
    const governanceModule = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
      ).href,
    );

    const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));

    writeArtifact(
      artifactRoot,
      'release-governance-window-snapshot-web',
      path.join('docs', 'release', 'release-window-snapshot-latest.json'),
      createReleaseWindowSnapshotArtifactPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-sync-audit-web',
      path.join('docs', 'release', 'release-sync-audit-latest.json'),
      createReleaseSyncAuditArtifactPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-telemetry-export-web',
      path.join('docs', 'release', 'release-telemetry-export-latest.json'),
      createTelemetryExportPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-telemetry-snapshot-web',
      path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
      createTelemetrySnapshotPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-slo-evidence-web',
      path.join('docs', 'release', 'slo-governance-latest.json'),
      createSloGovernancePayload(),
    );

    restoreModule.restoreReleaseGovernanceLatestArtifacts({
      artifactDir: artifactRoot,
      repoRoot,
    });

    const summary = await governanceModule.runReleaseGovernanceChecks({
      spawnSyncImpl() {
        return {
          status: 1,
          stdout: '',
          stderr: '',
          error: new Error('spawnSync node EPERM'),
        };
      },
      fallbackSpawnSyncImpl() {
        throw new Error('live git replay should not run after latest governance artifacts are restored');
      },
    });

    assert.equal(summary.ok, true);
    assert.equal(summary.blocked, false);
    assert.deepEqual(summary.blockedIds, []);
  });
});
