import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createExternalReleaseSpec({
  id = 'sdkwork-core',
  repository = 'Sdkwork-Cloud/sdkwork-core',
  envRefKey = 'SDKWORK_CORE_GIT_REF',
  defaultRef = 'main',
  targetDir = path.join(repoRoot, '..', 'sdkwork-core'),
  expectedGitRoot = targetDir,
  cloneTargetDir = targetDir,
  requiredPaths = ['package.json'],
} = {}) {
  return {
    id,
    repository,
    envRefKey,
    defaultRef,
    targetDir,
    expectedGitRoot,
    cloneTargetDir,
    requiredPaths,
  };
}

function createExistingPathProbe(spec) {
  const readyPaths = new Set([
    path.resolve(spec.cloneTargetDir ?? spec.targetDir),
    path.resolve(spec.targetDir),
    ...spec.requiredPaths.map((relativePath) => path.resolve(spec.targetDir, relativePath)),
  ]);

  return function exists(candidatePath) {
    return readyPaths.has(path.resolve(candidatePath));
  };
}

function createGitAuditSpawn({
  cwd,
  topLevel,
  remoteUrl,
} = {}) {
  return function spawnSyncImpl(command, args, options = {}) {
    assert.match(String(command), /git(?:\.exe)?$/i);
    assert.equal(options.cwd, cwd);
    assert.equal(options.encoding, 'utf8');
    assert.equal(options.shell, false);

    const key = args.join('\u0000');
    if (key === 'rev-parse\u0000--show-toplevel') {
      return {
        status: 0,
        stdout: `${topLevel}\n`,
        stderr: '',
      };
    }

    if (key === 'remote\u0000get-url\u0000origin') {
      return {
        status: 0,
        stdout: `${remoteUrl}\n`,
        stderr: '',
      };
    }

    throw new Error(`unexpected git command: ${args.join(' ')}`);
  };
}

test('external release dependency materializer reuses a governed standalone checkout', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  const spec = createExternalReleaseSpec();
  const result = module.materializeExternalReleaseDependency({
    spec,
    env: {},
    exists: createExistingPathProbe(spec),
    spawnSyncImpl: createGitAuditSpawn({
      cwd: spec.targetDir,
      topLevel: spec.targetDir,
      remoteUrl: 'git@github.com:Sdkwork-Cloud/sdkwork-core.git',
    }),
  });

  assert.deepEqual(result, {
    id: 'sdkwork-core',
    repository: 'Sdkwork-Cloud/sdkwork-core',
    ref: 'main',
    status: 'ready',
    skipped: true,
  });
});

test('external release dependency materializer rejects an occupied target that is not the governed standalone repository', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  const spec = createExternalReleaseSpec();

  assert.throws(
    () => module.materializeExternalReleaseDependency({
      spec,
      env: {},
      exists: createExistingPathProbe(spec),
      spawnSyncImpl: createGitAuditSpawn({
        cwd: spec.targetDir,
        topLevel: path.resolve(spec.targetDir, '..', '..'),
        remoteUrl: 'git@github.com:Sdkwork-Cloud/spring-ai-plus2.git',
      }),
    }),
    /not-standalone-root[\s\S]*remote-url-mismatch/i,
  );
});

test('external release dependency materializer accepts a governed nested package checkout inside the craw-chat repository', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  const spec = createExternalReleaseSpec({
    id: 'sdkwork-craw-chat-sdk',
    repository: 'Sdkwork-Cloud/craw-chat',
    envRefKey: 'SDKWORK_CRAW_CHAT_SDK_GIT_REF',
    targetDir: path.join(
      repoRoot,
      '..',
      'craw-chat',
      'sdks',
      'sdkwork-craw-chat-sdk',
      'sdkwork-craw-chat-sdk-typescript',
    ),
    expectedGitRoot: path.join(repoRoot, '..', 'craw-chat'),
    cloneTargetDir: path.join(repoRoot, '..', 'craw-chat'),
    requiredPaths: ['package.json'],
  });

  const result = module.materializeExternalReleaseDependency({
    spec,
    env: {},
    exists: createExistingPathProbe(spec),
    spawnSyncImpl: createGitAuditSpawn({
      cwd: spec.targetDir,
      topLevel: spec.expectedGitRoot,
      remoteUrl: 'git@github.com:Sdkwork-Cloud/craw-chat.git',
    }),
  });

  assert.deepEqual(result, {
    id: 'sdkwork-craw-chat-sdk',
    repository: 'Sdkwork-Cloud/craw-chat',
    ref: 'main',
    status: 'ready',
    skipped: true,
  });
});
