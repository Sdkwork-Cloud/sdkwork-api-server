import assert from 'node:assert/strict';
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const releaseTelemetrySnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const releaseTelemetryExportPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
);
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

function createReleaseWindowSnapshotArtifactPayload() {
  return {
    generatedAt: '2026-04-08T12:00:00Z',
    source: {
      kind: 'release-window-snapshot-fixture',
      provenance: 'synthetic-test',
    },
    snapshot: {
      latestReleaseTag: 'release-2026-03-28-8',
      commitsSinceLatestRelease: 16,
      workingTreeEntryCount: 631,
      hasReleaseBaseline: true,
    },
  };
}

function createReleaseSyncAuditArtifactPayload() {
  return {
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
  };
}

function cleanupGovernedReleaseArtifacts() {
  if (existsSync(releaseTelemetryExportPath)) {
    rmSync(releaseTelemetryExportPath, { force: true });
  }
  if (existsSync(releaseTelemetrySnapshotPath)) {
    rmSync(releaseTelemetrySnapshotPath, { force: true });
  }
  if (existsSync(releaseWindowSnapshotPath)) {
    rmSync(releaseWindowSnapshotPath, { force: true });
  }
  if (existsSync(releaseSyncAuditPath)) {
    rmSync(releaseSyncAuditPath, { force: true });
  }
  if (existsSync(sloGovernanceEvidencePath)) {
    rmSync(sloGovernanceEvidencePath, { force: true });
  }
}

test('release governance runner exposes the expected fixed verification sequence', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  assert.equal(typeof module.listReleaseGovernanceCheckPlans, 'function');
  assert.equal(typeof module.resolveNodeRunner, 'function');
  assert.equal(typeof module.runReleaseGovernanceCheckPlan, 'function');
  assert.equal(typeof module.runReleaseGovernanceChecks, 'function');

  const plans = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  assert.deepEqual(
    plans.map((plan) => plan.id),
    [
      'release-sync-audit-test',
      'release-workflow-test',
      'release-attestation-verify-test',
      'release-observability-test',
      'release-slo-governance-test',
      'release-slo-governance',
      'release-runtime-tooling-test',
      'release-window-snapshot-test',
      'release-window-snapshot',
      'release-sync-audit',
    ],
  );

  assert.deepEqual(
    plans[0].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-sync-audit.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[1].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-workflow.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[2].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-attestation-verify.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[3].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-observability-contracts.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[4].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-slo-governance.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[5].args,
    [
      'scripts/release/slo-governance.mjs',
      '--format',
      'json',
    ],
  );
  assert.deepEqual(
    plans[6].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'bin/tests/router-runtime-tooling.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[7].args,
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-window-snapshot.test.mjs',
    ],
  );
  assert.deepEqual(
    plans[8].args,
    [
      'scripts/release/compute-release-window-snapshot.mjs',
      '--format',
      'json',
    ],
  );
  assert.deepEqual(
    plans[9].args,
    [
      'scripts/release/verify-release-sync.mjs',
      '--format',
      'json',
    ],
  );

  const windowsRunner = module.resolveNodeRunner({
    platform: 'win32',
    nodeExecutable: 'node.exe',
  });
  assert.equal(windowsRunner.command, 'node.exe');
  assert.equal(windowsRunner.shell, false);

  const linuxRunner = module.resolveNodeRunner({
    platform: 'linux',
    nodeExecutable: 'node',
  });
  assert.equal(linuxRunner.command, 'node');
  assert.equal(linuxRunner.shell, false);
});

test('release governance runner aggregates passing tests and blocking live release audits', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const responses = new Map([
    [
      'release-sync-audit-test',
      {
        status: 0,
        stdout: 'ok sync test',
        stderr: '',
      },
    ],
    [
      'release-workflow-test',
      {
        status: 0,
        stdout: 'ok workflow test',
        stderr: '',
      },
    ],
    [
      'release-attestation-verify-test',
      {
        status: 0,
        stdout: 'ok attestation verify test',
        stderr: '',
      },
    ],
    [
      'release-observability-test',
      {
        status: 0,
        stdout: 'ok observability test',
        stderr: '',
      },
    ],
    [
      'release-slo-governance-test',
      {
        status: 0,
        stdout: 'ok slo governance test',
        stderr: '',
      },
    ],
    [
      'release-slo-governance',
      {
        status: 0,
        stdout: '{"ok":true,"blocked":false,"reason":"","summary":{"ok":true}}',
        stderr: '',
      },
    ],
    [
      'release-runtime-tooling-test',
      {
        status: 0,
        stdout: 'ok runtime tooling test',
        stderr: '',
      },
    ],
    [
      'release-window-snapshot-test',
      {
        status: 0,
        stdout: 'ok release window snapshot test',
        stderr: '',
      },
    ],
    [
      'release-window-snapshot',
      {
        status: 0,
        stdout: '{"ok":true,"blocked":false,"snapshot":{"latestReleaseTag":"release-2026-03-28-8","commitsSinceLatestRelease":16,"workingTreeEntryCount":2,"hasReleaseBaseline":true}}',
        stderr: '',
      },
    ],
    [
      'release-sync-audit',
      {
        status: 1,
        stdout: '{"releasable":false}',
        stderr: '',
      },
    ],
  ]);

  const plans = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });
  const planByArgs = new Map(
    plans.map((plan) => [[plan.command, ...plan.args].join('\u0000'), plan.id]),
  );

  const summary = await module.runReleaseGovernanceChecks({
    plans,
    spawnSyncImpl(command, args) {
      const planId = planByArgs.get([command, ...args].join('\u0000'));
      const response = responses.get(planId);
      return {
        status: response.status,
        stdout: response.stdout,
        stderr: response.stderr,
      };
    },
  });

  assert.equal(summary.ok, false);
  assert.equal(summary.blocked, false);
  assert.deepEqual(summary.passingIds, [
    'release-sync-audit-test',
    'release-workflow-test',
    'release-attestation-verify-test',
    'release-observability-test',
    'release-slo-governance-test',
    'release-slo-governance',
    'release-runtime-tooling-test',
    'release-window-snapshot-test',
    'release-window-snapshot',
  ]);
  assert.deepEqual(summary.blockedIds, []);
  assert.deepEqual(summary.failingIds, [
    'release-sync-audit',
  ]);
  assert.deepEqual(
    summary.results.map((result) => [result.id, result.ok, result.status]),
    [
      ['release-sync-audit-test', true, 0],
      ['release-workflow-test', true, 0],
      ['release-attestation-verify-test', true, 0],
      ['release-observability-test', true, 0],
      ['release-slo-governance-test', true, 0],
      ['release-slo-governance', true, 0],
      ['release-runtime-tooling-test', true, 0],
      ['release-window-snapshot-test', true, 0],
      ['release-window-snapshot', true, 0],
      ['release-sync-audit', false, 1],
    ],
  );
  assert.equal(summary.results[9].stdout.trim(), '{"releasable":false}');
});

test('release governance runner distinguishes blocked lanes from real failing lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const responses = new Map([
    [
      'release-sync-audit-test',
      {
        status: 0,
        stdout: 'ok sync test',
        stderr: '',
      },
    ],
    [
      'release-workflow-test',
      {
        status: 0,
        stdout: 'ok workflow test',
        stderr: '',
      },
    ],
    [
      'release-attestation-verify-test',
      {
        status: 0,
        stdout: 'ok attestation verify test',
        stderr: '',
      },
    ],
    [
      'release-observability-test',
      {
        status: 0,
        stdout: 'ok observability test',
        stderr: '',
      },
    ],
    [
      'release-slo-governance-test',
      {
        status: 0,
        stdout: 'ok slo governance test',
        stderr: '',
      },
    ],
    [
      'release-slo-governance',
      {
        status: 1,
        stdout: '{"ok":false,"blocked":true,"reason":"evidence-missing","summary":null}',
        stderr: '',
      },
    ],
    [
      'release-runtime-tooling-test',
      {
        status: 0,
        stdout: 'ok runtime tooling test',
        stderr: '',
      },
    ],
    [
      'release-window-snapshot-test',
      {
        status: 0,
        stdout: 'ok release window snapshot test',
        stderr: '',
      },
    ],
    [
      'release-window-snapshot',
      {
        status: 1,
        stdout: '{"ok":false,"blocked":true,"reason":"command-exec-blocked","snapshot":null}',
        stderr: '',
      },
    ],
    [
      'release-sync-audit',
      {
        status: 1,
        stdout: '{"releasable":false,"reports":[{"id":"sdkwork-api-router","reasons":["remote-url-mismatch"],"releasable":false}]}',
        stderr: '',
      },
    ],
  ]);

  const plans = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });
  const planByArgs = new Map(
    plans.map((plan) => [[plan.command, ...plan.args].join('\u0000'), plan.id]),
  );

  const summary = await module.runReleaseGovernanceChecks({
    plans,
    spawnSyncImpl(command, args) {
      const planId = planByArgs.get([command, ...args].join('\u0000'));
      const response = responses.get(planId);
      return {
        status: response.status,
        stdout: response.stdout,
        stderr: response.stderr,
      };
    },
  });

  assert.equal(summary.ok, false);
  assert.equal(summary.blocked, true);
  assert.deepEqual(summary.passingIds, [
    'release-sync-audit-test',
    'release-workflow-test',
    'release-attestation-verify-test',
    'release-observability-test',
    'release-slo-governance-test',
    'release-runtime-tooling-test',
    'release-window-snapshot-test',
  ]);
  assert.deepEqual(summary.blockedIds, [
    'release-slo-governance',
    'release-window-snapshot',
  ]);
  assert.deepEqual(summary.failingIds, [
    'release-sync-audit',
  ]);
});

test('release governance runner falls back to in-process checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[0];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EACCES'),
      };
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner also falls back for attestation verification checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[2];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner also falls back for observability checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[3];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner also falls back for runtime tooling checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[6];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner also falls back for slo governance checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[4];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner reports telemetry-input-missing when live slo inputs were never materialized', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[5];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {},
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.id, 'release-slo-governance');
  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, false);
  assert.equal(result.status, 1);
  assert.match(result.stdout, /telemetry-input-missing/);
});

test('release governance runner materializes live slo evidence from a release telemetry export when node child execution is blocked', async () => {
  cleanupGovernedReleaseArtifacts();

  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[5];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {
      SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: JSON.stringify(createTelemetryExportPayload()),
    },
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.id, 'release-slo-governance');
  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
  assert.equal(existsSync(releaseTelemetrySnapshotPath), true);
  assert.equal(existsSync(sloGovernanceEvidencePath), true);
  assert.equal(
    JSON.parse(readFileSync(sloGovernanceEvidencePath, 'utf8')).baselineId,
    'release-slo-governance-baseline-2026-04-08',
  );
  assert.match(result.stdout, /"ok": true/);

  cleanupGovernedReleaseArtifacts();
});

test('release governance runner materializes live slo evidence from the default release telemetry export artifact when node child execution is blocked', async () => {
  cleanupGovernedReleaseArtifacts();
  writeFileSync(
    releaseTelemetryExportPath,
    `${JSON.stringify(createTelemetryExportPayload(), null, 2)}\n`,
    'utf8',
  );

  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[5];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {},
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
  });

  assert.equal(result.id, 'release-slo-governance');
  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
  assert.equal(existsSync(releaseTelemetrySnapshotPath), true);
  assert.equal(existsSync(sloGovernanceEvidencePath), true);
  assert.match(result.stdout, /"ok": true/);

  cleanupGovernedReleaseArtifacts();
});

test('release governance runner also falls back for release window snapshot checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[8];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync git EPERM'),
      };
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, false);
  assert.equal(result.status, 1);
  assert.match(result.stdout, /command-exec-blocked/);
});

test('release governance runner consumes governed release window snapshot input when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[8];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {
      SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON: JSON.stringify(
        createReleaseWindowSnapshotArtifactPayload(),
      ),
    },
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      throw new Error('git spawn should not run when governed snapshot input is provided');
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
  assert.match(result.stdout, /release-2026-03-28-8/);
});

test('release governance runner replays the default release window snapshot artifact when node child execution is blocked', async () => {
  cleanupGovernedReleaseArtifacts();
  writeFileSync(
    releaseWindowSnapshotPath,
    `${JSON.stringify(createReleaseWindowSnapshotArtifactPayload(), null, 2)}\n`,
    'utf8',
  );

  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[8];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {},
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      throw new Error('git spawn should not run when a default latest artifact is available');
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
  assert.match(result.stdout, /release-2026-03-28-8/);

  cleanupGovernedReleaseArtifacts();
});

test('release governance runner consumes governed release sync audit input when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[9];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {
      SDKWORK_RELEASE_SYNC_AUDIT_JSON: JSON.stringify(
        createReleaseSyncAuditArtifactPayload(),
      ),
    },
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      throw new Error('git spawn should not run when governed sync audit input is provided');
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
  assert.match(result.stdout, /"releasable": true/);
});

test('release governance runner replays the default release sync audit artifact when node child execution is blocked', async () => {
  cleanupGovernedReleaseArtifacts();
  writeFileSync(
    releaseSyncAuditPath,
    `${JSON.stringify(createReleaseSyncAuditArtifactPayload(), null, 2)}\n`,
    'utf8',
  );

  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  })[9];

  const result = await module.runReleaseGovernanceCheckPlan({
    plan,
    env: {},
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      throw new Error('git spawn should not run when a default latest artifact is available');
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
  assert.match(result.stdout, /"releasable": true/);

  cleanupGovernedReleaseArtifacts();
});
