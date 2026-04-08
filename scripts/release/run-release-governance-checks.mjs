#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { materializeReleaseTelemetrySnapshot } from './materialize-release-telemetry-snapshot.mjs';
import { materializeSloGovernanceEvidence } from './materialize-slo-governance-evidence.mjs';
import { collectReleaseWindowSnapshotResult } from './compute-release-window-snapshot.mjs';
import { assertObservabilityContracts } from './observability-contracts.mjs';
import { assertReleaseAttestationVerificationContracts } from './release-attestation-verification-contracts.mjs';
import { assertReleaseSyncAuditContracts } from './release-sync-audit-contracts.mjs';
import { assertReleaseWindowSnapshotContracts } from './release-window-snapshot-contracts.mjs';
import { assertReleaseWorkflowContracts } from './release-workflow-contracts.mjs';
import { assertRuntimeToolingContracts } from './runtime-tooling-contracts.mjs';
import { collectSloGovernanceResult } from './slo-governance.mjs';
import { assertSloGovernanceContracts } from './slo-governance-contracts.mjs';
import { auditReleaseSyncRepositories } from './verify-release-sync.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');
const defaultReleaseTelemetrySnapshotPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const defaultReleaseWindowSnapshotPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-window-snapshot-latest.json',
);
const defaultReleaseSyncAuditPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-sync-audit-latest.json',
);

function materializeLiveSloGovernanceEvidence({
  env = process.env,
} = {}) {
  let telemetrySnapshotPath = existsSync(defaultReleaseTelemetrySnapshotPath)
    ? defaultReleaseTelemetrySnapshotPath
    : '';

  if (!telemetrySnapshotPath) {
    try {
      telemetrySnapshotPath = materializeReleaseTelemetrySnapshot({
        env,
      }).outputPath;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      if (/missing release telemetry input/i.test(errorMessage)) {
        return {
          ok: false,
          blocked: true,
          reason: 'telemetry-input-missing',
          errorMessage,
          summary: null,
        };
      }

      return {
        ok: false,
        blocked: false,
        reason: 'telemetry-materialization-failed',
        errorMessage,
        summary: null,
      };
    }
  }

  try {
    materializeSloGovernanceEvidence({
      telemetrySnapshotPath,
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      blocked: false,
      reason: 'slo-evidence-materialization-failed',
      errorMessage,
      summary: null,
    };
  }

  return collectSloGovernanceResult();
}

export function resolveNodeRunner({
  platform = process.platform,
  nodeExecutable = process.execPath,
} = {}) {
  return {
    command: nodeExecutable,
    shell: false,
  };
}

export function listReleaseGovernanceCheckPlans({
  nodeExecutable = process.execPath,
} = {}) {
  return [
    {
      id: 'release-sync-audit-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'scripts/release/tests/release-sync-audit.test.mjs',
      ],
    },
    {
      id: 'release-workflow-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'scripts/release/tests/release-workflow.test.mjs',
      ],
    },
    {
      id: 'release-attestation-verify-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'scripts/release/tests/release-attestation-verify.test.mjs',
      ],
    },
    {
      id: 'release-observability-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'scripts/release/tests/release-observability-contracts.test.mjs',
      ],
    },
    {
      id: 'release-slo-governance-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'scripts/release/tests/release-slo-governance.test.mjs',
      ],
    },
    {
      id: 'release-slo-governance',
      command: nodeExecutable,
      args: [
        'scripts/release/slo-governance.mjs',
        '--format',
        'json',
      ],
    },
    {
      id: 'release-runtime-tooling-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'bin/tests/router-runtime-tooling.test.mjs',
      ],
    },
    {
      id: 'release-window-snapshot-test',
      command: nodeExecutable,
      args: [
        '--test',
        '--experimental-test-isolation=none',
        'scripts/release/tests/release-window-snapshot.test.mjs',
      ],
    },
    {
      id: 'release-window-snapshot',
      command: nodeExecutable,
      args: [
        'scripts/release/compute-release-window-snapshot.mjs',
        '--format',
        'json',
      ],
    },
    {
      id: 'release-sync-audit',
      command: nodeExecutable,
      args: [
        'scripts/release/verify-release-sync.mjs',
        '--format',
        'json',
      ],
    },
  ];
}

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function parseJsonObject(value) {
  const text = String(value ?? '').trim();
  if (!text.startsWith('{')) {
    return null;
  }

  try {
    const payload = JSON.parse(text);
    return payload && typeof payload === 'object' ? payload : null;
  } catch {
    return null;
  }
}

function hasCommandExecBlockedReason(reasons = []) {
  return Array.isArray(reasons)
    && reasons.some((reason) => String(reason ?? '').trim() === 'command-exec-blocked');
}

function isReleaseGovernanceResultBlocked(result) {
  const errorText = [
    String(result?.errorMessage ?? ''),
    String(result?.stderr ?? ''),
    String(result?.stdout ?? ''),
  ].join('\n');
  if (/(eperm|eacces)/i.test(errorText) || /command-exec-blocked/i.test(errorText)) {
    return true;
  }

  const payload = parseJsonObject(result?.stdout);
  if (!payload) {
    return false;
  }

  if (payload.blocked === true || String(payload.reason ?? '').trim() === 'command-exec-blocked') {
    return true;
  }

  if (Array.isArray(payload.reports)) {
    return payload.reports.some((report) => hasCommandExecBlockedReason(report?.reasons));
  }

  return false;
}

function summarizeReleaseGovernanceResults(results = []) {
  const passingIds = [];
  const blockedIds = [];
  const failingIds = [];

  for (const result of results) {
    if (result?.ok === true) {
      passingIds.push(result.id);
      continue;
    }

    if (isReleaseGovernanceResultBlocked(result)) {
      blockedIds.push(result.id);
      continue;
    }

    failingIds.push(result.id);
  }

  return {
    blocked: blockedIds.length > 0,
    passingIds,
    blockedIds,
    failingIds,
  };
}

async function runFallbackReleaseGovernanceCheck({
  plan,
  env = process.env,
  fallbackSpawnSyncImpl = spawnSync,
} = {}) {
  if (plan.id === 'release-sync-audit-test') {
    await assertReleaseSyncAuditContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-workflow-test') {
    await assertReleaseWorkflowContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-observability-test') {
    await assertObservabilityContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-attestation-verify-test') {
    await assertReleaseAttestationVerificationContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-runtime-tooling-test') {
    await assertRuntimeToolingContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-slo-governance-test') {
    await assertSloGovernanceContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-slo-governance') {
    const currentResult = collectSloGovernanceResult();
    const result = currentResult.reason === 'evidence-missing'
      ? materializeLiveSloGovernanceEvidence({ env })
      : currentResult;
    const payload = result.summary
      ? {
        ok: result.summary.ok,
        blocked: result.summary.blocked,
        reason: result.summary.reason,
        summary: result.summary,
      }
      : result;
    return {
      id: plan.id,
      ok: result.ok,
      status: result.ok ? 0 : 1,
      stdout: `${JSON.stringify(payload, null, 2)}\n`,
      stderr: '',
      errorMessage: result.ok ? '' : String(result.errorMessage ?? ''),
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-window-snapshot-test') {
    await assertReleaseWindowSnapshotContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-window-snapshot') {
    const hasGovernedWindowInput = [
      env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH,
      env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON,
    ].some((value) => String(value ?? '').trim().length > 0);

    const replayResult = hasGovernedWindowInput
      ? collectReleaseWindowSnapshotResult({
        env,
        spawnSyncImpl: fallbackSpawnSyncImpl,
      })
      : existsSync(defaultReleaseWindowSnapshotPath)
        ? collectReleaseWindowSnapshotResult({
        snapshotPath: defaultReleaseWindowSnapshotPath,
        env: {},
      })
        : collectReleaseWindowSnapshotResult({
          env,
          spawnSyncImpl: fallbackSpawnSyncImpl,
        });
    return {
      id: plan.id,
      ok: replayResult.ok,
      status: replayResult.ok ? 0 : 1,
      stdout: `${JSON.stringify(replayResult, null, 2)}\n`,
      stderr: '',
      errorMessage: replayResult.ok ? '' : String(replayResult.errorMessage ?? ''),
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-sync-audit') {
    const hasGovernedSyncInput = [
      env.SDKWORK_RELEASE_SYNC_AUDIT_PATH,
      env.SDKWORK_RELEASE_SYNC_AUDIT_JSON,
    ].some((value) => String(value ?? '').trim().length > 0);

    const replaySummary = hasGovernedSyncInput
      ? auditReleaseSyncRepositories({
        env,
        spawnSyncImpl: fallbackSpawnSyncImpl,
      })
      : existsSync(defaultReleaseSyncAuditPath)
        ? auditReleaseSyncRepositories({
        auditPath: defaultReleaseSyncAuditPath,
        env: {},
      })
        : auditReleaseSyncRepositories({
          env,
          spawnSyncImpl: fallbackSpawnSyncImpl,
        });
    return {
      id: plan.id,
      ok: replaySummary.releasable,
      status: replaySummary.releasable ? 0 : 1,
      stdout: `${JSON.stringify(replaySummary, null, 2)}\n`,
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  return null;
}

export async function runReleaseGovernanceCheckPlan({
  plan,
  env = process.env,
  spawnSyncImpl = spawnSync,
  fallbackSpawnSyncImpl = spawnSync,
} = {}) {
  const runner = resolveNodeRunner({
    nodeExecutable: plan.command,
  });
  const result = spawnSyncImpl(runner.command, plan.args, {
    cwd: rootDir,
    encoding: 'utf8',
    env,
    shell: runner.shell,
    stdio: 'pipe',
  });

  const stdout = String(result.stdout ?? '');
  const stderr = String(result.stderr ?? '');
  const errorMessage = result.error ? String(result.error.message ?? '') : '';
  const ok = !result.error && (result.status ?? 1) === 0;

  if (/(eperm|eacces)/i.test(errorMessage)) {
    const fallbackResult = await runFallbackReleaseGovernanceCheck({
      plan,
      env,
      fallbackSpawnSyncImpl,
    });
    if (fallbackResult) {
      return {
        ...fallbackResult,
        command: runner.command,
        args: [...plan.args],
        shell: runner.shell,
      };
    }
  }

  return {
    id: plan.id,
    command: runner.command,
    args: [...plan.args],
    shell: runner.shell,
    ok,
    status: result.status ?? (ok ? 0 : 1),
    stdout,
    stderr,
    errorMessage,
    mode: 'spawn',
  };
}

export async function runReleaseGovernanceChecks({
  plans = listReleaseGovernanceCheckPlans(),
  env = process.env,
  spawnSyncImpl = spawnSync,
  fallbackSpawnSyncImpl = spawnSync,
} = {}) {
  const results = [];
  for (const plan of plans) {
    results.push(await runReleaseGovernanceCheckPlan({
      plan,
      env,
      spawnSyncImpl,
      fallbackSpawnSyncImpl,
    }));
  }

  const summary = summarizeReleaseGovernanceResults(results);

  return {
    ok: results.every((result) => result.ok === true),
    blocked: summary.blocked,
    passingIds: summary.passingIds,
    blockedIds: summary.blockedIds,
    failingIds: summary.failingIds,
    results,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  let format = 'text';

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--format') {
      format = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!['text', 'json'].includes(format)) {
    throw new Error(`unsupported format: ${format}`);
  }

  return {
    format,
  };
}

function printTextReport(summary) {
  for (const result of summary.results) {
    const state = result.ok
      ? 'PASS'
      : (summary.blockedIds ?? []).includes(result.id) ? 'BLOCK' : 'FAIL';
    const commandLine = [result.command, ...result.args].join(' ');
    console.error(`[release-governance] ${state} ${result.id}: ${commandLine}`);
    if (!result.ok) {
      if (result.errorMessage) {
        console.error(`  error: ${truncateText(result.errorMessage, 1000)}`);
      }
      if (result.stderr.trim()) {
        console.error(`  stderr: ${truncateText(result.stderr, 1000)}`);
      }
      if (result.stdout.trim()) {
        console.error(`  stdout: ${truncateText(result.stdout, 1000)}`);
      }
    }
  }
}

function main() {
  const { format } = parseArgs();
  return runReleaseGovernanceChecks().then((summary) => {
    if (format === 'json') {
      console.log(JSON.stringify(summary, null, 2));
    } else {
      printTextReport(summary);
    }

    if (!summary.ok) {
      process.exit(1);
    }
  });
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
