import assert from 'node:assert/strict';
import {
  existsSync,
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
const defaultReleaseWindowSnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-window-snapshot-latest.json',
);

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
      workingTreeEntryCount: 627,
      hasReleaseBaseline: true,
    },
  };
}

test('release window snapshot materializer writes the standard governed artifact from direct snapshot input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveReleaseWindowSnapshotProducerInput, 'function');
  assert.equal(typeof module.materializeReleaseWindowSnapshot, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-window-snapshot-'));
  const outputPath = path.join(fixtureRoot, 'release-window-snapshot-latest.json');

  const result = module.materializeReleaseWindowSnapshot({
    snapshotJson: JSON.stringify(createReleaseWindowSnapshotArtifactPayload()),
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  assert.equal(existsSync(outputPath), true);

  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.version, 1);
  assert.equal(written.generatedAt, '2026-04-08T12:00:00Z');
  assert.equal(written.source.kind, 'release-window-snapshot-fixture');
  assert.equal(written.snapshot.latestReleaseTag, 'release-2026-03-28-8');
  assert.equal(written.snapshot.commitsSinceLatestRelease, 16);
});

test('release window snapshot materializer can derive the latest artifact from live git facts', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-window-snapshot-'));
  const outputPath = path.join(fixtureRoot, 'release-window-snapshot-latest.json');
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
      'status\u0000--short',
      {
        status: 0,
        stdout: ' M docs/release/CHANGELOG.md\n?? docs/review/new-note.md\n',
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
  ]);

  const result = module.materializeReleaseWindowSnapshot({
    generatedAt: '2026-04-08T12:00:00Z',
    sourceKind: 'release-window-live-git',
    sourceProvenance: 'synthetic-test',
    outputPath,
    spawnSyncImpl(_command, args) {
      const response = responses.get(args.join('\u0000'));
      if (!response) {
        throw new Error(`unexpected git command: ${args.join(' ')}`);
      }
      return response;
    },
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.generatedAt, '2026-04-08T12:00:00Z');
  assert.equal(written.source.kind, 'release-window-live-git');
  assert.equal(written.source.provenance, 'synthetic-test');
  assert.equal(written.snapshot.workingTreeEntryCount, 2);
});

test('release window snapshot materializer prefers live git over the default latest artifact when no explicit input is supplied', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    ).href,
  );

  const original = {
    hadFile: existsSync(defaultReleaseWindowSnapshotPath),
    content: existsSync(defaultReleaseWindowSnapshotPath)
      ? readFileSync(defaultReleaseWindowSnapshotPath, 'utf8')
      : null,
  };
  const staleArtifact = createReleaseWindowSnapshotArtifactPayload();
  staleArtifact.snapshot.latestReleaseTag = 'release-stale';
  staleArtifact.snapshot.commitsSinceLatestRelease = 999;
  staleArtifact.snapshot.workingTreeEntryCount = 999;

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-window-live-'));
  const outputPath = path.join(fixtureRoot, 'release-window-snapshot-latest.json');

  writeFileSync(
    defaultReleaseWindowSnapshotPath,
    `${JSON.stringify(staleArtifact, null, 2)}\n`,
    'utf8',
  );

  try {
    const result = module.materializeReleaseWindowSnapshot({
      generatedAt: '2026-04-15T10:00:00Z',
      sourceKind: 'release-window-live-git',
      outputPath,
      spawnSyncImpl(_command, args) {
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
            'status\u0000--short',
            {
              status: 0,
              stdout: ' M docs/release/CHANGELOG.md\n?? docs/review/new-note.md\n',
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
        ]);
        const response = responses.get(args.join('\u0000'));
        if (!response) {
          throw new Error(`unexpected git command: ${args.join(' ')}`);
        }
        return response;
      },
    });

    assert.equal(result.source, 'live-git');
    const written = JSON.parse(readFileSync(outputPath, 'utf8'));
    assert.equal(written.snapshot.latestReleaseTag, 'release-2026-03-28-8');
    assert.equal(written.snapshot.commitsSinceLatestRelease, 16);
    assert.equal(written.snapshot.workingTreeEntryCount, 2);
  } finally {
    if (original.hadFile) {
      writeFileSync(defaultReleaseWindowSnapshotPath, original.content, 'utf8');
    } else if (existsSync(defaultReleaseWindowSnapshotPath)) {
      rmSync(defaultReleaseWindowSnapshotPath, { force: true });
    }
  }
});

test('release window snapshot materializer rejects blocked live git execution when no governed input is supplied', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    ).href,
  );

  assert.throws(
    () => module.materializeReleaseWindowSnapshot({
      env: {},
      outputPath: path.join(os.tmpdir(), 'unused-release-window-snapshot.json'),
      spawnSyncImpl() {
        return {
          status: 1,
          stdout: '',
          stderr: '',
          error: new Error('spawnSync git EPERM'),
        };
      },
    }),
    /command-exec-blocked/i,
  );
});
