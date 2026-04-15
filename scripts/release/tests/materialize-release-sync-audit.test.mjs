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
const defaultReleaseSyncAuditPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-sync-audit-latest.json',
);

function createReleaseSyncAuditArtifactPayload(targetDir = repoRoot) {
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
          targetDir,
          expectedGitRoot: targetDir,
          topLevel: targetDir,
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

test('release sync audit materializer writes the standard governed artifact from direct audit input', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveReleaseSyncAuditProducerInput, 'function');
  assert.equal(typeof module.materializeReleaseSyncAudit, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-sync-audit-'));
  const outputPath = path.join(fixtureRoot, 'release-sync-audit-latest.json');

  const result = module.materializeReleaseSyncAudit({
    auditJson: JSON.stringify(createReleaseSyncAuditArtifactPayload()),
    outputPath,
  });

  assert.equal(result.outputPath, outputPath);
  assert.equal(existsSync(outputPath), true);

  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.version, 1);
  assert.equal(written.generatedAt, '2026-04-08T13:00:00Z');
  assert.equal(written.source.kind, 'release-sync-audit-fixture');
  assert.equal(written.summary.releasable, true);
  assert.equal(written.summary.reports[0].id, 'sdkwork-api-router');
});

test('release sync audit materializer can derive the latest artifact from live multi-repository git facts', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-sync-audit-'));
  const outputPath = path.join(fixtureRoot, 'release-sync-audit-latest.json');
  const targetDir = fixtureRoot;

  const result = module.materializeReleaseSyncAudit({
    generatedAt: '2026-04-08T13:00:00Z',
    sourceKind: 'release-sync-audit-live-git',
    sourceProvenance: 'synthetic-test',
    outputPath,
    specs: [
      {
        id: 'sdkwork-api-router',
        targetDir,
        expectedGitRoot: targetDir,
        expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
        envRefKey: 'SDKWORK_API_ROUTER_GIT_REF',
        defaultRef: 'main',
      },
    ],
    spawnSyncImpl(_command, args) {
      const key = args.join('\u0000');
      const responses = new Map([
        [
          'rev-parse\u0000--show-toplevel',
          {
            status: 0,
            stdout: `${targetDir}\n`,
            stderr: '',
          },
        ],
        [
          'status\u0000--short\u0000--branch',
          {
            status: 0,
            stdout: '## main...origin/main\n',
            stderr: '',
          },
        ],
        [
          'remote\u0000get-url\u0000origin',
          {
            status: 0,
            stdout: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git\n',
            stderr: '',
          },
        ],
        [
          'rev-parse\u0000HEAD',
          {
            status: 0,
            stdout: 'abc123\n',
            stderr: '',
          },
        ],
        [
          'ls-remote\u0000origin\u0000main',
          {
            status: 0,
            stdout: 'abc123\trefs/heads/main\n',
            stderr: '',
          },
        ],
      ]);
      const response = responses.get(key);
      if (!response) {
        throw new Error(`unexpected git command: ${args.join(' ')}`);
      }
      return response;
    },
  });

  assert.equal(result.outputPath, outputPath);
  const written = JSON.parse(readFileSync(outputPath, 'utf8'));
  assert.equal(written.generatedAt, '2026-04-08T13:00:00Z');
  assert.equal(written.source.kind, 'release-sync-audit-live-git');
  assert.equal(written.source.provenance, 'synthetic-test');
  assert.equal(written.summary.releasable, true);
  assert.equal(written.summary.reports[0].remoteHead, 'abc123');
});

test('release sync audit materializer prefers live git over the default latest artifact when no explicit input is supplied', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    ).href,
  );

  const original = {
    hadFile: existsSync(defaultReleaseSyncAuditPath),
    content: existsSync(defaultReleaseSyncAuditPath)
      ? readFileSync(defaultReleaseSyncAuditPath, 'utf8')
      : null,
  };
  const staleArtifact = createReleaseSyncAuditArtifactPayload(repoRoot);
  staleArtifact.summary.reports[0].remoteHead = 'stale-head';
  staleArtifact.summary.reports[0].localHead = 'stale-head';

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-sync-audit-live-'));
  const outputPath = path.join(fixtureRoot, 'release-sync-audit-latest.json');

  writeFileSync(
    defaultReleaseSyncAuditPath,
    `${JSON.stringify(staleArtifact, null, 2)}\n`,
    'utf8',
  );

  try {
    const result = module.materializeReleaseSyncAudit({
      generatedAt: '2026-04-15T10:00:00Z',
      sourceKind: 'release-sync-audit-live-git',
      outputPath,
      specs: [
        {
          id: 'sdkwork-api-router',
          targetDir: repoRoot,
          expectedGitRoot: repoRoot,
          expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
          envRefKey: 'SDKWORK_API_ROUTER_GIT_REF',
          defaultRef: 'main',
        },
      ],
      spawnSyncImpl(_command, args) {
        const key = args.join('\u0000');
        const responses = new Map([
          [
            'rev-parse\u0000--show-toplevel',
            {
              status: 0,
              stdout: `${repoRoot}\n`,
              stderr: '',
            },
          ],
          [
            'status\u0000--short\u0000--branch',
            {
              status: 0,
              stdout: '## main...origin/main\n',
              stderr: '',
            },
          ],
          [
            'remote\u0000get-url\u0000origin',
            {
              status: 0,
              stdout: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git\n',
              stderr: '',
            },
          ],
          [
            'rev-parse\u0000HEAD',
            {
              status: 0,
              stdout: 'live-head\n',
              stderr: '',
            },
          ],
          [
            'ls-remote\u0000origin\u0000main',
            {
              status: 0,
              stdout: 'live-head\trefs/heads/main\n',
              stderr: '',
            },
          ],
        ]);
        const response = responses.get(key);
        if (!response) {
          throw new Error(`unexpected git command: ${args.join(' ')}`);
        }
        return response;
      },
    });

    assert.equal(result.source, 'live-git');
    const written = JSON.parse(readFileSync(outputPath, 'utf8'));
    assert.equal(written.summary.reports[0].remoteHead, 'live-head');
    assert.equal(written.summary.reports[0].localHead, 'live-head');
  } finally {
    if (original.hadFile) {
      writeFileSync(defaultReleaseSyncAuditPath, original.content, 'utf8');
    } else if (existsSync(defaultReleaseSyncAuditPath)) {
      rmSync(defaultReleaseSyncAuditPath, { force: true });
    }
  }
});

test('release sync audit materializer rejects blocked live git execution when no governed input is supplied', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    ).href,
  );

  assert.throws(
    () => module.materializeReleaseSyncAudit({
      env: {},
      outputPath: path.join(os.tmpdir(), 'unused-release-sync-audit.json'),
      specs: [
        {
          id: 'sdkwork-api-router',
          targetDir: repoRoot,
          expectedGitRoot: repoRoot,
          expectedRemoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
          envRefKey: 'SDKWORK_API_ROUTER_GIT_REF',
          defaultRef: 'main',
        },
      ],
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
