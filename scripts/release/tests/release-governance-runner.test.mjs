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
const governedReleaseArtifactPaths = [
  releaseTelemetryExportPath,
  releaseTelemetrySnapshotPath,
  releaseWindowSnapshotPath,
  releaseSyncAuditPath,
  sloGovernanceEvidencePath,
];

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

async function withCleanedGovernedReleaseArtifacts(callback) {
  const originals = governedReleaseArtifactPaths.map((filePath) => ({
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

function getPlanById(plans, id) {
  const plan = plans.find((entry) => entry.id === id);
  assert.ok(plan, `expected release governance plan ${id}`);
  return plan;
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
  assert.equal(typeof module.parseArgs, 'function');
  assert.deepEqual(module.parseArgs(['--profile', 'preflight', '--format', 'json']), {
    format: 'json',
    profile: 'preflight',
  });

  const plans = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  assert.deepEqual(
    plans.map((plan) => plan.id),
    [
      'release-sync-audit-test',
      'release-workflow-test',
      'release-governance-workflow-test',
      'release-attestation-verify-test',
      'release-observability-test',
      'release-slo-governance-contracts-test',
      'release-slo-governance-test',
      'release-slo-governance',
      'release-runtime-tooling-test',
      'release-unix-installed-runtime-smoke-test',
      'release-windows-installed-runtime-smoke-test',
      'release-materialize-external-deps-test',
      'release-window-snapshot-test',
      'release-window-snapshot-materializer-test',
      'release-sync-audit-materializer-test',
      'release-governance-bundle-test',
      'restore-release-governance-latest-test',
      'release-telemetry-export-test',
      'release-telemetry-snapshot-test',
      'release-slo-evidence-materializer-test',
      'release-window-snapshot',
      'release-sync-audit',
    ],
  );

  const preflightPlans = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
    profile: 'preflight',
  });

  assert.deepEqual(
    preflightPlans.map((plan) => plan.id),
    [
      'release-sync-audit-test',
      'release-workflow-test',
      'release-governance-workflow-test',
      'release-attestation-verify-test',
      'release-observability-test',
      'release-slo-governance-contracts-test',
      'release-slo-governance-test',
      'release-runtime-tooling-test',
      'release-unix-installed-runtime-smoke-test',
      'release-windows-installed-runtime-smoke-test',
      'release-materialize-external-deps-test',
      'release-window-snapshot-test',
      'release-window-snapshot-materializer-test',
      'release-sync-audit-materializer-test',
      'release-governance-bundle-test',
      'restore-release-governance-latest-test',
      'release-telemetry-export-test',
      'release-telemetry-snapshot-test',
      'release-slo-evidence-materializer-test',
    ],
  );

  const planArgsById = new Map(plans.map((plan) => [plan.id, plan.args]));

  assert.deepEqual(
    planArgsById.get('release-sync-audit-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-sync-audit.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-workflow-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-workflow.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-governance-workflow-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release-governance-workflow.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-attestation-verify-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-attestation-verify.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-observability-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-observability-contracts.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-slo-governance-contracts-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-slo-governance-contracts.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-slo-governance-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-slo-governance.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-slo-governance'),
    [
      'scripts/release/slo-governance.mjs',
      '--format',
      'json',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-runtime-tooling-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'bin/tests/router-runtime-tooling.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-unix-installed-runtime-smoke-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-windows-installed-runtime-smoke-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-materialize-external-deps-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-external-deps.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-window-snapshot-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/release-window-snapshot.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-window-snapshot-materializer-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-release-window-snapshot.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-sync-audit-materializer-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-release-sync-audit.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-governance-bundle-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-release-governance-bundle.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('restore-release-governance-latest-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/restore-release-governance-latest.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-telemetry-export-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-release-telemetry-export.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-telemetry-snapshot-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-slo-evidence-materializer-test'),
    [
      '--test',
      '--experimental-test-isolation=none',
      'scripts/release/tests/materialize-slo-governance-evidence.test.mjs',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-window-snapshot'),
    [
      'scripts/release/compute-release-window-snapshot.mjs',
      '--format',
      'json',
      '--live',
    ],
  );
  assert.deepEqual(
    planArgsById.get('release-sync-audit'),
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
      'release-governance-workflow-test',
      {
        status: 0,
        stdout: 'ok release governance workflow test',
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
      'release-slo-governance-contracts-test',
      {
        status: 0,
        stdout: 'ok slo governance contracts test',
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
      'release-unix-installed-runtime-smoke-test',
      {
        status: 0,
        stdout: 'ok unix installed runtime smoke test',
        stderr: '',
      },
    ],
    [
      'release-windows-installed-runtime-smoke-test',
      {
        status: 0,
        stdout: 'ok windows installed runtime smoke test',
        stderr: '',
      },
    ],
    [
      'release-materialize-external-deps-test',
      {
        status: 0,
        stdout: 'ok materialize external deps test',
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
      'release-window-snapshot-materializer-test',
      {
        status: 0,
        stdout: 'ok release window snapshot materializer test',
        stderr: '',
      },
    ],
    [
      'release-sync-audit-materializer-test',
      {
        status: 0,
        stdout: 'ok release sync audit materializer test',
        stderr: '',
      },
    ],
    [
      'release-governance-bundle-test',
      {
        status: 0,
        stdout: 'ok release governance bundle test',
        stderr: '',
      },
    ],
    [
      'restore-release-governance-latest-test',
      {
        status: 0,
        stdout: 'ok restore release governance latest test',
        stderr: '',
      },
    ],
    [
      'release-telemetry-export-test',
      {
        status: 0,
        stdout: 'ok release telemetry export test',
        stderr: '',
      },
    ],
    [
      'release-telemetry-snapshot-test',
      {
        status: 0,
        stdout: 'ok release telemetry snapshot test',
        stderr: '',
      },
    ],
    [
      'release-slo-evidence-materializer-test',
      {
        status: 0,
        stdout: 'ok release slo evidence materializer test',
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
    'release-governance-workflow-test',
    'release-attestation-verify-test',
    'release-observability-test',
    'release-slo-governance-contracts-test',
    'release-slo-governance-test',
    'release-slo-governance',
    'release-runtime-tooling-test',
    'release-unix-installed-runtime-smoke-test',
    'release-windows-installed-runtime-smoke-test',
    'release-materialize-external-deps-test',
    'release-window-snapshot-test',
    'release-window-snapshot-materializer-test',
    'release-sync-audit-materializer-test',
    'release-governance-bundle-test',
    'restore-release-governance-latest-test',
    'release-telemetry-export-test',
    'release-telemetry-snapshot-test',
    'release-slo-evidence-materializer-test',
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
      ['release-governance-workflow-test', true, 0],
      ['release-attestation-verify-test', true, 0],
      ['release-observability-test', true, 0],
      ['release-slo-governance-contracts-test', true, 0],
      ['release-slo-governance-test', true, 0],
      ['release-slo-governance', true, 0],
      ['release-runtime-tooling-test', true, 0],
      ['release-unix-installed-runtime-smoke-test', true, 0],
      ['release-windows-installed-runtime-smoke-test', true, 0],
      ['release-materialize-external-deps-test', true, 0],
      ['release-window-snapshot-test', true, 0],
      ['release-window-snapshot-materializer-test', true, 0],
      ['release-sync-audit-materializer-test', true, 0],
      ['release-governance-bundle-test', true, 0],
      ['restore-release-governance-latest-test', true, 0],
      ['release-telemetry-export-test', true, 0],
      ['release-telemetry-snapshot-test', true, 0],
      ['release-slo-evidence-materializer-test', true, 0],
      ['release-window-snapshot', true, 0],
      ['release-sync-audit', false, 1],
    ],
  );
  assert.equal(
    summary.results.find((result) => result.id === 'release-sync-audit')?.stdout.trim(),
    '{"releasable":false}',
  );
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
      'release-governance-workflow-test',
      {
        status: 0,
        stdout: 'ok release governance workflow test',
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
      'release-slo-governance-contracts-test',
      {
        status: 0,
        stdout: 'ok slo governance contracts test',
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
      'release-unix-installed-runtime-smoke-test',
      {
        status: 0,
        stdout: 'ok unix installed runtime smoke test',
        stderr: '',
      },
    ],
    [
      'release-windows-installed-runtime-smoke-test',
      {
        status: 0,
        stdout: 'ok windows installed runtime smoke test',
        stderr: '',
      },
    ],
    [
      'release-materialize-external-deps-test',
      {
        status: 0,
        stdout: 'ok materialize external deps test',
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
      'release-window-snapshot-materializer-test',
      {
        status: 0,
        stdout: 'ok release window snapshot materializer test',
        stderr: '',
      },
    ],
    [
      'release-sync-audit-materializer-test',
      {
        status: 0,
        stdout: 'ok release sync audit materializer test',
        stderr: '',
      },
    ],
    [
      'release-governance-bundle-test',
      {
        status: 0,
        stdout: 'ok release governance bundle test',
        stderr: '',
      },
    ],
    [
      'restore-release-governance-latest-test',
      {
        status: 0,
        stdout: 'ok restore release governance latest test',
        stderr: '',
      },
    ],
    [
      'release-telemetry-export-test',
      {
        status: 0,
        stdout: 'ok release telemetry export test',
        stderr: '',
      },
    ],
    [
      'release-telemetry-snapshot-test',
      {
        status: 0,
        stdout: 'ok release telemetry snapshot test',
        stderr: '',
      },
    ],
    [
      'release-slo-evidence-materializer-test',
      {
        status: 0,
        stdout: 'ok release slo evidence materializer test',
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
    'release-governance-workflow-test',
    'release-attestation-verify-test',
    'release-observability-test',
    'release-slo-governance-contracts-test',
    'release-slo-governance-test',
    'release-runtime-tooling-test',
    'release-unix-installed-runtime-smoke-test',
    'release-windows-installed-runtime-smoke-test',
    'release-materialize-external-deps-test',
    'release-window-snapshot-test',
    'release-window-snapshot-materializer-test',
    'release-sync-audit-materializer-test',
    'release-governance-bundle-test',
    'restore-release-governance-latest-test',
    'release-telemetry-export-test',
    'release-telemetry-snapshot-test',
    'release-slo-evidence-materializer-test',
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
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-sync-audit-test'),
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

test('release governance runner also falls back for release governance workflow checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-governance-workflow-test'),
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

test('release governance runner also falls back for attestation verification checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-attestation-verify-test'),
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
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-observability-test'),
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

test('release governance runner also falls back for slo governance contract checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-slo-governance-contracts-test'),
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
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-runtime-tooling-test'),
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

test('release governance runner also falls back for unix installed runtime smoke checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-unix-installed-runtime-smoke-test'),
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

test('release governance runner also falls back for windows installed runtime smoke checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-windows-installed-runtime-smoke-test'),
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

test('release governance runner also falls back for external dependency materialization checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-materialize-external-deps-test'),
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

test('release governance runner also falls back for release window snapshot materializer checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-window-snapshot-materializer-test'),
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      throw new Error('git spawn should not run for release window snapshot materializer fallback');
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner also falls back for release sync audit materializer checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-sync-audit-materializer-test'),
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync node EPERM'),
      };
    },
    fallbackSpawnSyncImpl() {
      throw new Error('git spawn should not run for release sync audit materializer fallback');
    },
  });

  assert.equal(result.mode, 'fallback');
  assert.equal(result.ok, true);
  assert.equal(result.status, 0);
});

test('release governance runner also falls back for governance bundle checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-governance-bundle-test'),
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

test('release governance runner also falls back for governance restore checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'restore-release-governance-latest-test'),
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

test('release governance runner also falls back for telemetry export checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-telemetry-export-test'),
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

test('release governance runner also falls back for telemetry snapshot checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-telemetry-snapshot-test'),
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

test('release governance runner also falls back for slo evidence materializer checks when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-slo-evidence-materializer-test'),
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
  await withCleanedGovernedReleaseArtifacts(async () => {
    const module = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
      ).href,
    );

    const plan = module.listReleaseGovernanceCheckPlans({
      nodeExecutable: 'node',
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-slo-governance'),
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
  });
});

test('release governance runner reports telemetry-input-missing when live slo inputs were never materialized', async () => {
  await withCleanedGovernedReleaseArtifacts(async () => {
    const module = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
      ).href,
    );

    const plan = module.listReleaseGovernanceCheckPlans({
      nodeExecutable: 'node',
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-slo-governance'),
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
});

test('release governance runner materializes live slo evidence from a release telemetry export when node child execution is blocked', async () => {
  await withCleanedGovernedReleaseArtifacts(async () => {
    const module = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
      ).href,
    );

    const plan = module.listReleaseGovernanceCheckPlans({
      nodeExecutable: 'node',
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-slo-governance'),
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
  });
});

test('release governance runner materializes live slo evidence from the default release telemetry export artifact when node child execution is blocked', async () => {
  await withCleanedGovernedReleaseArtifacts(async () => {
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
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-slo-governance'),
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
  });
});

test('release governance runner also falls back for release window snapshot checks when node child execution is blocked', async () => {
  await withCleanedGovernedReleaseArtifacts(async () => {
    const module = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
      ).href,
    );

    const plan = module.listReleaseGovernanceCheckPlans({
      nodeExecutable: 'node',
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-window-snapshot'),
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
});

test('release governance runner consumes governed release window snapshot input when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-window-snapshot'),
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
  await withCleanedGovernedReleaseArtifacts(async () => {
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
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-window-snapshot'),
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
  });
});

test('release governance runner consumes governed release sync audit input when node child execution is blocked', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    ).href,
  );

  const plan = module.listReleaseGovernanceCheckPlans({
    nodeExecutable: 'node',
  });

  const result = await module.runReleaseGovernanceCheckPlan({
    plan: getPlanById(plan, 'release-sync-audit'),
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
  await withCleanedGovernedReleaseArtifacts(async () => {
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
    });

    const result = await module.runReleaseGovernanceCheckPlan({
      plan: getPlanById(plan, 'release-sync-audit'),
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
  });
});
