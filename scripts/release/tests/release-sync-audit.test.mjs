import assert from 'node:assert/strict';
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const releaseSyncAuditPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-sync-audit-latest.json',
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

test('release sync audit exposes repository specs and blocks non-standalone, dirty, or remote-unverifiable repositories', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-sync.mjs'),
    ).href,
  );

  assert.equal(typeof module.listReleaseSyncRepositorySpecs, 'function');
  assert.equal(typeof module.parseGitStatusBranchSummary, 'function');
  assert.equal(typeof module.evaluateReleaseSyncRepositoryAudit, 'function');
  assert.equal(typeof module.isReleaseSyncAuditPassing, 'function');
  assert.equal(typeof module.resolveGitRunner, 'function');
  assert.equal(typeof module.isGitCommandExecutionBlocked, 'function');
  assert.equal(typeof module.formatReleaseSyncTextReport, 'function');
  assert.equal(typeof module.resolveReleaseSyncRepositoryRef, 'function');
  assert.equal(typeof module.parseRemoteHeadStdout, 'function');

  const specs = module.listReleaseSyncRepositorySpecs();
  assert.deepEqual(
    specs.map((spec) => spec.id),
    [
      'sdkwork-api-router',
      'sdkwork-core',
      'sdkwork-ui',
      'sdkwork-appbase',
      'sdkwork-craw-chat-sdk',
    ],
  );
  assert.equal(specs[0].envRefKey, 'SDKWORK_API_ROUTER_GIT_REF');
  assert.equal(specs[0].defaultRef, 'main');
  assert.equal(specs[4].envRefKey, 'SDKWORK_CRAW_CHAT_SDK_GIT_REF');
  assert.match(
    specs[4].targetDir.replaceAll('\\', '/'),
    /\/craw-chat\/sdks\/sdkwork-craw-chat-sdk\/sdkwork-craw-chat-sdk-typescript\/composed$/,
  );
  assert.match(
    specs[4].expectedGitRoot.replaceAll('\\', '/'),
    /\/craw-chat$/,
  );

  assert.equal(
    module.resolveReleaseSyncRepositoryRef({
      spec: specs[0],
      env: {
        SDKWORK_API_ROUTER_GIT_REF: 'refs/tags/release-2026-03-28-8',
      },
    }),
    'refs/tags/release-2026-03-28-8',
  );
  assert.equal(
    module.resolveReleaseSyncRepositoryRef({
      spec: specs[1],
      env: {},
    }),
    'main',
  );

  const branchSummary = module.parseGitStatusBranchSummary(
    [
      '## main...origin/main [ahead 2, behind 1]',
      ' M src/index.ts',
      '?? tmp.txt',
    ].join('\n'),
  );
  assert.equal(branchSummary.branch, 'main');
  assert.equal(branchSummary.upstream, 'origin/main');
  assert.equal(branchSummary.ahead, 2);
  assert.equal(branchSummary.behind, 1);
  assert.equal(branchSummary.isDirty, true);
  assert.equal(branchSummary.hasTrackingDivergence, true);

  const cleanAudit = module.evaluateReleaseSyncRepositoryAudit({
    spec: specs[0],
    expectedRef: 'main',
    topLevel: specs[0].expectedGitRoot,
    statusText: '## main...origin/main',
    remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
    remoteHeadResult: {
      ok: true,
      stdout: 'abc123\tHEAD',
    },
  });
  assert.equal(cleanAudit.releasable, true);
  assert.deepEqual(cleanAudit.reasons, []);

  const sshRemoteAudit = module.evaluateReleaseSyncRepositoryAudit({
    spec: specs[0],
    expectedRef: 'main',
    topLevel: specs[0].expectedGitRoot,
    statusText: '## main...origin/main',
    remoteUrl: 'git@github.com:Sdkwork-Cloud/sdkwork-api-router.git',
    remoteHeadResult: {
      ok: true,
      stdout: 'abc123\trefs/heads/main',
    },
  });
  assert.equal(sshRemoteAudit.releasable, true);
  assert.deepEqual(sshRemoteAudit.reasons, []);

  const nonStandaloneAudit = module.evaluateReleaseSyncRepositoryAudit({
    spec: specs[1],
    expectedRef: 'main',
    topLevel: path.resolve(specs[1].expectedGitRoot, '..', '..'),
    statusText: '## main...origin/main [ahead 2]',
    remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-core.git',
    remoteHeadResult: {
      ok: false,
      stderr: 'TLS connect error',
    },
  });
  assert.equal(nonStandaloneAudit.releasable, false);
  assert.deepEqual(
    nonStandaloneAudit.reasons,
    ['not-standalone-root', 'branch-not-synced', 'remote-unverifiable'],
  );

  const dirtyAudit = module.evaluateReleaseSyncRepositoryAudit({
    spec: specs[2],
    expectedRef: 'main',
    topLevel: specs[2].expectedGitRoot,
    statusText: ['## main...origin/main', ' M sdkwork-ui-pc-react/src/theme/sdkwork-theme.ts'].join('\n'),
    remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-ui.git',
    remoteHeadResult: {
      ok: true,
      stdout: 'def456\tHEAD',
    },
  });
  assert.equal(dirtyAudit.releasable, false);
  assert.deepEqual(dirtyAudit.reasons, ['dirty-working-tree']);

  assert.equal(
    module.parseRemoteHeadStdout('abc123\trefs/tags/release-1\nfed456\trefs/tags/release-1^{}\n'),
    'fed456',
  );

  const detachedTagAudit = module.evaluateReleaseSyncRepositoryAudit({
    spec: specs[0],
    expectedRef: 'refs/tags/release-2026-03-28-8',
    topLevel: specs[0].expectedGitRoot,
    statusText: '## HEAD (no branch)',
    remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
    localHead: 'fed456',
    remoteHeadResult: {
      ok: true,
      stdout: 'abc123\trefs/tags/release-2026-03-28-8\nfed456\trefs/tags/release-2026-03-28-8^{}\n',
    },
  });
  assert.equal(detachedTagAudit.releasable, true);
  assert.deepEqual(detachedTagAudit.reasons, []);

  assert.equal(
    module.isReleaseSyncAuditPassing([cleanAudit]),
    true,
  );
  assert.equal(
    module.isReleaseSyncAuditPassing([cleanAudit, dirtyAudit]),
    false,
  );

  const reportText = module.formatReleaseSyncTextReport([cleanAudit, dirtyAudit]);
  assert.match(
    reportText,
    /\[verify-release-sync\] PASS sdkwork-api-router branch=main upstream=origin\/main dirty=false reasons=none/,
  );
  assert.match(
    reportText,
    /\[verify-release-sync\] BLOCK sdkwork-ui branch=main upstream=origin\/main dirty=true reasons=dirty-working-tree/,
  );

  const windowsGitRunner = module.resolveGitRunner({
    platform: 'win32',
  });
  assert.equal(windowsGitRunner.command, 'git.exe');
  assert.equal(
    windowsGitRunner.shell,
    false,
    'Windows release-sync Git commands must not route through cmd.exe',
  );

  const linuxGitRunner = module.resolveGitRunner({
    platform: 'linux',
  });
  assert.equal(linuxGitRunner.command, 'git');
  assert.equal(linuxGitRunner.shell, false);

  assert.equal(
    module.isGitCommandExecutionBlocked([
      { ok: false, errorMessage: 'spawnSync git EPERM' },
    ]),
    true,
  );
  assert.equal(
    module.isGitCommandExecutionBlocked([
      { ok: false, errorMessage: 'spawnSync git EACCES' },
    ]),
    true,
  );
  assert.equal(
    module.isGitCommandExecutionBlocked([
      { ok: false, errorMessage: 'TLS connect error' },
    ]),
    false,
  );
});

test('release sync audit consumes governed JSON input without spawning git', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-sync.mjs'),
    ).href,
  );

  const artifact = createReleaseSyncAuditArtifactPayload();

  assert.equal(typeof module.resolveReleaseSyncAuditInput, 'function');

  const summary = module.auditReleaseSyncRepositories({
    specs: [],
    env: {
      SDKWORK_RELEASE_SYNC_AUDIT_JSON: JSON.stringify(artifact),
    },
  });

  assert.deepEqual(summary, artifact.summary);
});

test('release sync audit prefers the default latest artifact before live git collection', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-sync.mjs'),
    ).href,
  );

  const artifact = createReleaseSyncAuditArtifactPayload();

  withTemporaryFile(
    releaseSyncAuditPath,
    `${JSON.stringify(artifact, null, 2)}\n`,
    () => {
      let gitSpawned = false;
      const summary = module.auditReleaseSyncRepositories({
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
      assert.deepEqual(summary, artifact.summary);
    },
  );
});

test('release sync audit can bypass the default latest artifact in explicit live mode', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-sync.mjs'),
    ).href,
  );

  const artifact = createReleaseSyncAuditArtifactPayload();
  const [spec] = module.listReleaseSyncRepositorySpecs();
  let gitSpawnCount = 0;

  withTemporaryFile(
    releaseSyncAuditPath,
    `${JSON.stringify(artifact, null, 2)}\n`,
    () => {
      const summary = module.auditReleaseSyncRepositories({
        specs: [spec],
        env: {},
        preferDefaultArtifact: false,
        spawnSyncImpl(command, args) {
          gitSpawnCount += 1;

          const key = args.join('\u0000');
          if (key === 'rev-parse\u0000--show-toplevel') {
            return {
              status: 0,
              stdout: `${spec.expectedGitRoot}\n`,
              stderr: '',
            };
          }

          if (key === 'status\u0000--short\u0000--branch') {
            return {
              status: 0,
              stdout: '## main...origin/main\n',
              stderr: '',
            };
          }

          if (key === 'remote\u0000get-url\u0000origin') {
            return {
              status: 0,
              stdout: `${spec.expectedRemoteUrl}\n`,
              stderr: '',
            };
          }

          if (key === 'rev-parse\u0000HEAD') {
            return {
              status: 0,
              stdout: 'fed456\n',
              stderr: '',
            };
          }

          if (key === 'ls-remote\u0000origin\u0000main') {
            return {
              status: 0,
              stdout: 'fed456\trefs/heads/main\n',
              stderr: '',
            };
          }

          throw new Error(`unexpected command: ${command} ${args.join(' ')}`);
        },
      });

      assert.ok(gitSpawnCount > 0);
      assert.equal(summary.releasable, true);
      assert.equal(summary.reports[0].id, spec.id);
      assert.equal(summary.reports[0].expectedGitRoot, spec.expectedGitRoot);
      assert.equal(summary.reports[0].localHead, 'fed456');
    },
  );
});

test('release sync audit specs can remap governed external repositories into a dedicated release root without moving the main repository', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'verify-release-sync.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveReleaseSyncRepositorySpecs, 'function');

  const governedRoot = path.join(repoRoot, 'artifacts', 'release-governance', 'external-deps');
  const specs = module.resolveReleaseSyncRepositorySpecs({
    env: {
      SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT: governedRoot,
    },
  });

  const routerSpec = specs.find((spec) => spec.id === 'sdkwork-api-router');
  assert.ok(routerSpec);
  assert.equal(routerSpec.targetDir, repoRoot);
  assert.equal(routerSpec.expectedGitRoot, repoRoot);

  const sdkworkCore = specs.find((spec) => spec.id === 'sdkwork-core');
  assert.ok(sdkworkCore);
  assert.equal(
    sdkworkCore.targetDir,
    path.join(governedRoot, 'sdkwork-core'),
  );
  assert.equal(
    sdkworkCore.expectedGitRoot,
    path.join(governedRoot, 'sdkwork-core'),
  );

  const sdkworkUi = specs.find((spec) => spec.id === 'sdkwork-ui');
  assert.ok(sdkworkUi);
  assert.equal(
    sdkworkUi.targetDir,
    path.join(governedRoot, 'sdkwork-ui'),
  );

  const sdkworkCrawChatSdk = specs.find((spec) => spec.id === 'sdkwork-craw-chat-sdk');
  assert.ok(sdkworkCrawChatSdk);
  assert.equal(
    sdkworkCrawChatSdk.expectedGitRoot,
    path.join(governedRoot, 'craw-chat'),
  );
  assert.equal(
    sdkworkCrawChatSdk.targetDir,
    path.join(
      governedRoot,
      'craw-chat',
      'sdks',
      'sdkwork-craw-chat-sdk',
      'sdkwork-craw-chat-sdk-typescript',
      'composed',
    ),
  );
});
