import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export async function assertReleaseSyncAuditContracts({
  repoRoot,
} = {}) {
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
  assert.equal(typeof module.resolveReleaseSyncAuditInput, 'function');
  assert.equal(typeof module.validateReleaseSyncAuditSummary, 'function');
  assert.equal(typeof module.validateReleaseSyncAuditArtifact, 'function');
  assert.equal(typeof module.auditReleaseSyncRepositories, 'function');

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

  const governedInput = module.resolveReleaseSyncAuditInput({
    env: {
      SDKWORK_RELEASE_SYNC_AUDIT_JSON: JSON.stringify({
        generatedAt: '2026-04-08T13:00:00Z',
        source: {
          kind: 'release-sync-audit-contract-fixture',
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
      }),
    },
  });
  assert.equal(governedInput.source, 'json');
  assert.deepEqual(
    module.auditReleaseSyncRepositories({
      specs: [],
      env: {
        SDKWORK_RELEASE_SYNC_AUDIT_JSON: JSON.stringify({
          generatedAt: '2026-04-08T13:00:00Z',
          source: {
            kind: 'release-sync-audit-contract-fixture',
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
        }),
      },
    }),
    governedInput.summary,
  );
}
