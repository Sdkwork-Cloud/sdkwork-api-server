import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createFixtureRoot() {
  return mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-attestation-'));
}

function writeFixtureFile(root, relativePath, contents = '{}\n') {
  const filePath = path.join(root, relativePath);
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, contents, 'utf8');
  return filePath;
}

function toPortableRelativePath(root, targetPath) {
  return path.relative(root, targetPath).replaceAll('\\', '/');
}

test('release attestation verifier exposes governed subjects and gh command plans', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-attestations.mjs'),
    ).href,
  );

  assert.equal(typeof module.listReleaseAttestationSubjectSpecs, 'function');
  assert.equal(typeof module.resolveReleaseAttestationRepositorySlug, 'function');
  assert.equal(typeof module.createReleaseAttestationVerificationPlan, 'function');
  assert.equal(typeof module.verifyReleaseAttestations, 'function');

  const fixtureRoot = createFixtureRoot();
  try {
    writeFixtureFile(fixtureRoot, 'docs/release/release-window-snapshot-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-sync-audit-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-telemetry-export-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-telemetry-snapshot-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/slo-governance-latest.json');
    writeFixtureFile(
      fixtureRoot,
      'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
    );
    writeFixtureFile(
      fixtureRoot,
      'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
    );
    writeFixtureFile(fixtureRoot, 'artifacts/release/admin/router-admin.zip', 'binary');
    writeFixtureFile(fixtureRoot, 'artifacts/release/portal/router-portal.zip', 'binary');

    const plan = module.createReleaseAttestationVerificationPlan({
      repoRoot: fixtureRoot,
      repoSlug: 'Sdkwork-Cloud/sdkwork-api-router',
    });

    assert.equal(plan.repoSlug, 'Sdkwork-Cloud/sdkwork-api-router');
    assert.deepEqual(
      plan.subjects.map((subject) => toPortableRelativePath(fixtureRoot, subject.subjectPath)),
      [
        'docs/release/release-window-snapshot-latest.json',
        'docs/release/release-sync-audit-latest.json',
        'docs/release/release-telemetry-export-latest.json',
        'docs/release/release-telemetry-snapshot-latest.json',
        'docs/release/slo-governance-latest.json',
        'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
        'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
        'artifacts/release/admin/router-admin.zip',
        'artifacts/release/portal/router-portal.zip',
      ],
    );
    assert.deepEqual(
      plan.commands[0].args,
      [
        'attestation',
        'verify',
        plan.subjects[0].subjectPath,
        '--repo',
        'Sdkwork-Cloud/sdkwork-api-router',
      ],
    );
    assert.deepEqual(
      module.listReleaseAttestationSubjectSpecs().map((spec) => spec.id),
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
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('release attestation verifier reports blocked when required subject paths are missing', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-attestations.mjs'),
    ).href,
  );

  const fixtureRoot = createFixtureRoot();
  try {
    const result = await module.verifyReleaseAttestations({
      repoRoot: fixtureRoot,
      repoSlug: 'Sdkwork-Cloud/sdkwork-api-router',
      spawnSyncImpl() {
        throw new Error('spawn should not be called when no subjects are available');
      },
    });

    assert.equal(result.ok, false);
    assert.equal(result.blocked, true);
    assert.equal(result.reason, 'subject-path-missing');
    assert.deepEqual(
      result.reports.map((report) => report.id),
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
    assert.ok(result.reports.every((report) => report.blocked === true));
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('release attestation verifier reports blocked when gh execution is unavailable', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-attestations.mjs'),
    ).href,
  );

  const fixtureRoot = createFixtureRoot();
  try {
    writeFixtureFile(fixtureRoot, 'docs/release/release-window-snapshot-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-sync-audit-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-telemetry-export-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-telemetry-snapshot-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/slo-governance-latest.json');
    writeFixtureFile(
      fixtureRoot,
      'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
    );
    writeFixtureFile(
      fixtureRoot,
      'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
    );
    writeFixtureFile(fixtureRoot, 'artifacts/release/admin/router-admin.zip', 'binary');

    const result = await module.verifyReleaseAttestations({
      repoRoot: fixtureRoot,
      repoSlug: 'Sdkwork-Cloud/sdkwork-api-router',
      spawnSyncImpl() {
        return {
          status: 1,
          stdout: '',
          stderr: '',
          error: new Error('spawnSync gh EACCES'),
        };
      },
    });

    assert.equal(result.ok, false);
    assert.equal(result.blocked, true);
    assert.equal(result.reason, 'command-exec-blocked');
    assert.ok(result.reports.every((report) => report.blocked === true));
    assert.ok(result.reports.every((report) => report.reason === 'command-exec-blocked'));
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('release attestation verifier reports real verification failures separately from blocked states', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-attestations.mjs'),
    ).href,
  );

  const fixtureRoot = createFixtureRoot();
  try {
    writeFixtureFile(fixtureRoot, 'docs/release/release-window-snapshot-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-sync-audit-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-telemetry-export-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/release-telemetry-snapshot-latest.json');
    writeFixtureFile(fixtureRoot, 'docs/release/slo-governance-latest.json');
    writeFixtureFile(
      fixtureRoot,
      'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
    );
    writeFixtureFile(
      fixtureRoot,
      'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
    );
    writeFixtureFile(fixtureRoot, 'artifacts/release/admin/router-admin.zip', 'binary');

    let callCount = 0;
    const result = await module.verifyReleaseAttestations({
      repoRoot: fixtureRoot,
      repoSlug: 'Sdkwork-Cloud/sdkwork-api-router',
      spawnSyncImpl() {
        callCount += 1;
        if (callCount === 2) {
          return {
            status: 1,
            stdout: '',
            stderr: 'no matching attestation',
          };
        }

        return {
          status: 0,
          stdout: 'verified',
          stderr: '',
        };
      },
    });

    assert.equal(result.ok, false);
    assert.equal(result.blocked, false);
    assert.equal(result.reason, 'attestation-verify-failed');
    assert.equal(result.failedCount, 1);
    assert.equal(result.verifiedCount, 7);
    assert.match(
      result.reports.find((report) => report.ok === false)?.stderr ?? '',
      /no matching attestation/,
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
