import assert from 'node:assert/strict';
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
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

function withTemporaryFile(filePath, content, callback) {
  const hadOriginalFile = existsSync(filePath);
  const originalContent = hadOriginalFile ? readFileSync(filePath, 'utf8') : null;

  writeFileSync(filePath, content, 'utf8');

  try {
    return callback();
  } finally {
    if (hadOriginalFile) {
      writeFileSync(filePath, originalContent, 'utf8');
    } else {
      rmSync(filePath, { force: true });
    }
  }
}

function createReleaseWindowSnapshotArtifact() {
  return {
    generatedAt: '2026-04-08T11:00:00Z',
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

test('release window snapshot helpers expose release baseline parsing and working-tree counting', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveGitRunner, 'function');
  assert.equal(typeof module.resolveReleaseWindowSnapshotInput, 'function');
  assert.equal(typeof module.validateReleaseWindowSnapshotArtifact, 'function');
  assert.equal(typeof module.findLatestReleaseTag, 'function');
  assert.equal(typeof module.countWorkingTreeEntries, 'function');
  assert.equal(typeof module.collectReleaseWindowSnapshot, 'function');
  assert.equal(typeof module.collectReleaseWindowSnapshotResult, 'function');
  assert.equal(typeof module.isGitCommandExecutionBlockedError, 'function');

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
  assert.equal(
    windowsGitRunner.shell,
    false,
    'Windows release-window Git commands must not route through cmd.exe',
  );

  const linuxGitRunner = module.resolveGitRunner({
    platform: 'linux',
  });
  assert.equal(linuxGitRunner.command, 'git');
  assert.equal(linuxGitRunner.shell, false);
});

test('release window snapshot consumes governed JSON input without spawning git', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  const artifact = createReleaseWindowSnapshotArtifact();
  const result = module.collectReleaseWindowSnapshotResult({
    env: {
      SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON: JSON.stringify(artifact),
    },
    spawnSyncImpl() {
      throw new Error('git spawn should not run when governed snapshot input is provided');
    },
  });

  assert.equal(result.ok, true);
  assert.equal(result.blocked, false);
  assert.equal(result.reason, '');
  assert.deepEqual(result.snapshot, artifact.snapshot);
});

test('release window snapshot prefers the default latest artifact before live git collection', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  const artifact = createReleaseWindowSnapshotArtifact();

  withTemporaryFile(
    releaseWindowSnapshotPath,
    `${JSON.stringify(artifact, null, 2)}\n`,
    () => {
      let gitSpawned = false;
      const result = module.collectReleaseWindowSnapshotResult({
        env: {},
        spawnSyncImpl() {
          gitSpawned = true;
          return {
            status: 1,
            stdout: '',
            stderr: '',
            error: new Error('spawnSync git EPERM'),
          };
        },
      });

      assert.equal(gitSpawned, false);
      assert.equal(result.ok, true);
      assert.equal(result.blocked, false);
      assert.equal(result.reason, '');
      assert.deepEqual(result.snapshot, artifact.snapshot);
    },
  );
});

test('release window snapshot collects the latest release tag, commit delta, and working-tree size', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  const responses = new Map([
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

  const snapshot = module.collectReleaseWindowSnapshot({
    spawnSyncImpl(command, args) {
      const key = args.join('\u0000');
      const response = responses.get(key);
      if (!response) {
        throw new Error(`unexpected command: ${command}\u0000${key}`);
      }

      return response;
    },
  });

  assert.equal(snapshot.latestReleaseTag, 'release-2026-03-28-8');
  assert.equal(snapshot.commitsSinceLatestRelease, 16);
  assert.equal(snapshot.workingTreeEntryCount, 2);
  assert.equal(snapshot.hasReleaseBaseline, true);
});

test('release window snapshot tolerates missing release tags and still reports the working-tree size', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  const responses = new Map([
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

  const snapshot = module.collectReleaseWindowSnapshot({
    spawnSyncImpl(command, args) {
      const key = args.join('\u0000');
      const response = responses.get(key);
      if (!response) {
        throw new Error(`unexpected command: ${command}\u0000${key}`);
      }

      return response;
    },
  });

  assert.equal(snapshot.latestReleaseTag, '');
  assert.equal(snapshot.commitsSinceLatestRelease, null);
  assert.equal(snapshot.workingTreeEntryCount, 1);
  assert.equal(snapshot.hasReleaseBaseline, false);
});

test('release window snapshot reports command-exec-blocked when git child execution is denied', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'compute-release-window-snapshot.mjs'),
    ).href,
  );

  const result = module.collectReleaseWindowSnapshotResult({
    spawnSyncImpl() {
      return {
        status: 1,
        stdout: '',
        stderr: '',
        error: new Error('spawnSync git EPERM'),
      };
    },
  });

  assert.equal(module.isGitCommandExecutionBlockedError(new Error('spawnSync git EPERM')), true);
  assert.equal(module.isGitCommandExecutionBlockedError(new Error('spawnSync git EACCES')), true);
  assert.equal(result.ok, false);
  assert.equal(result.blocked, true);
  assert.equal(result.reason, 'command-exec-blocked');
  assert.equal(result.snapshot, null);
  assert.match(String(result.errorMessage ?? ''), /EPERM/i);
});
