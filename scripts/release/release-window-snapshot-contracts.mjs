import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export async function assertReleaseWindowSnapshotContracts({
  repoRoot,
} = {}) {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  assert.equal(typeof module.findLatestReleaseTag, 'function');
  assert.equal(typeof module.countWorkingTreeEntries, 'function');
  assert.equal(typeof module.collectReleaseWindowSnapshot, 'function');
  assert.equal(typeof module.resolveGitRunner, 'function');
  assert.equal(typeof module.resolveReleaseWindowSnapshotInput, 'function');
  assert.equal(typeof module.validateReleaseWindowSnapshotArtifact, 'function');

  assert.equal(
    module.findLatestReleaseTag('\nrelease-2026-03-28-8\nrelease-2026-03-28-7\n'),
    'release-2026-03-28-8',
  );
  assert.equal(module.findLatestReleaseTag('\n\n'), '');
  assert.equal(
    module.countWorkingTreeEntries(' M docs/release/CHANGELOG.md\n?? docs/release/new-note.md\n'),
    2,
  );
  assert.equal(module.countWorkingTreeEntries('\n'), 0);

  const windowsGitRunner = module.resolveGitRunner({
    platform: 'win32',
  });
  assert.equal(windowsGitRunner.command, 'git.exe');
  assert.equal(windowsGitRunner.shell, false);

  const linuxGitRunner = module.resolveGitRunner({
    platform: 'linux',
  });
  assert.equal(linuxGitRunner.command, 'git');
  assert.equal(linuxGitRunner.shell, false);

  const governedSnapshotResult = module.collectReleaseWindowSnapshotResult({
    env: {
      SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON: JSON.stringify({
        generatedAt: '2026-04-08T12:00:00Z',
        source: {
          kind: 'release-window-snapshot-contract-fixture',
        },
        snapshot: {
          latestReleaseTag: 'release-2026-03-28-8',
          commitsSinceLatestRelease: 16,
          workingTreeEntryCount: 2,
          hasReleaseBaseline: true,
        },
      }),
    },
    spawnSyncImpl() {
      throw new Error('git spawn should not run when governed snapshot input is provided');
    },
  });
  assert.equal(governedSnapshotResult.ok, true);
  assert.equal(governedSnapshotResult.blocked, false);
  assert.deepEqual(governedSnapshotResult.snapshot, {
    latestReleaseTag: 'release-2026-03-28-8',
    commitsSinceLatestRelease: 16,
    workingTreeEntryCount: 2,
    hasReleaseBaseline: true,
  });

  const baselineResponses = new Map([
    [
      'tag\u0000--list\u0000release-*\u0000--sort=-creatordate',
      {
        status: 0,
        stdout: 'release-2026-03-28-8\nrelease-2026-03-28-7\n',
        stderr: '',
      },
    ],
    [
      'rev-list\u0000--count\u0000release-2026-03-28-8..HEAD',
      {
        status: 0,
        stdout: '16\n',
        stderr: '',
      },
    ],
    [
      'status\u0000--short',
      {
        status: 0,
        stdout: ' M docs/release/CHANGELOG.md\n?? docs/release/new-note.md\n',
        stderr: '',
      },
    ],
  ]);

  const baselineSnapshot = module.collectReleaseWindowSnapshot({
    spawnSyncImpl(command, args) {
      const key = args.join('\u0000');
      const response = baselineResponses.get(key);
      if (!response) {
        throw new Error(`unexpected command: ${command}\u0000${key}`);
      }

      return response;
    },
  });

  assert.equal(baselineSnapshot.latestReleaseTag, 'release-2026-03-28-8');
  assert.equal(baselineSnapshot.commitsSinceLatestRelease, 16);
  assert.equal(baselineSnapshot.workingTreeEntryCount, 2);
  assert.equal(baselineSnapshot.hasReleaseBaseline, true);

  const missingBaselineResponses = new Map([
    [
      'tag\u0000--list\u0000release-*\u0000--sort=-creatordate',
      {
        status: 0,
        stdout: '',
        stderr: '',
      },
    ],
    [
      'status\u0000--short',
      {
        status: 0,
        stdout: ' M docs/release/CHANGELOG.md\n',
        stderr: '',
      },
    ],
  ]);

  const missingBaselineSnapshot = module.collectReleaseWindowSnapshot({
    spawnSyncImpl(command, args) {
      const key = args.join('\u0000');
      const response = missingBaselineResponses.get(key);
      if (!response) {
        throw new Error(`unexpected command: ${command}\u0000${key}`);
      }

      return response;
    },
  });

  assert.equal(missingBaselineSnapshot.latestReleaseTag, '');
  assert.equal(missingBaselineSnapshot.commitsSinceLatestRelease, null);
  assert.equal(missingBaselineSnapshot.workingTreeEntryCount, 1);
  assert.equal(missingBaselineSnapshot.hasReleaseBaseline, false);
}
