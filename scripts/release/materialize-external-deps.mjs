#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const EXTERNAL_RELEASE_DEPENDENCY_SPECS = Object.freeze([
  Object.freeze({
    id: 'sdkwork-core',
    repository: 'Sdkwork-Cloud/sdkwork-core',
    envRefKey: 'SDKWORK_CORE_GIT_REF',
    defaultRef: 'main',
    targetDir: path.resolve(rootDir, '..', 'sdkwork-core'),
    requiredPaths: Object.freeze(['package.json']),
  }),
  Object.freeze({
    id: 'sdkwork-ui',
    repository: 'Sdkwork-Cloud/sdkwork-ui',
    envRefKey: 'SDKWORK_UI_GIT_REF',
    defaultRef: 'main',
    targetDir: path.resolve(rootDir, '..', 'sdkwork-ui'),
    requiredPaths: Object.freeze(['sdkwork-ui-pc-react/package.json']),
  }),
  Object.freeze({
    id: 'sdkwork-appbase',
    repository: 'Sdkwork-Cloud/sdkwork-appbase',
    envRefKey: 'SDKWORK_APPBASE_GIT_REF',
    defaultRef: 'main',
    targetDir: path.resolve(rootDir, '..', 'sdkwork-appbase'),
    requiredPaths: Object.freeze(['package.json']),
  }),
  Object.freeze({
    id: 'sdkwork-craw-chat-sdk',
    repository: 'Sdkwork-Cloud/craw-chat',
    envRefKey: 'SDKWORK_CRAW_CHAT_SDK_GIT_REF',
    defaultRef: 'main',
    targetDir: path.resolve(
      rootDir,
      '..',
      'craw-chat',
      'sdks',
      'sdkwork-craw-chat-sdk',
      'sdkwork-craw-chat-sdk-typescript',
    ),
    expectedGitRoot: path.resolve(rootDir, '..', 'craw-chat'),
    cloneTargetDir: path.resolve(rootDir, '..', 'craw-chat'),
    requiredPaths: Object.freeze(['package.json']),
  }),
]);

const RELEASE_EXTERNAL_DEPENDENCY_SCAN_ROOTS = Object.freeze([
  path.resolve(rootDir, 'apps', 'sdkwork-router-admin'),
  path.resolve(rootDir, 'apps', 'sdkwork-router-portal'),
]);

const PACKAGE_JSON_DEPENDENCY_FIELDS = Object.freeze([
  'dependencies',
  'devDependencies',
  'optionalDependencies',
  'peerDependencies',
]);

export function listExternalReleaseDependencySpecs() {
  return EXTERNAL_RELEASE_DEPENDENCY_SPECS.map((spec) => ({
    ...spec,
    requiredPaths: [...spec.requiredPaths],
  }));
}

export function listExternalReleaseDependencyScanRoots() {
  return [...RELEASE_EXTERNAL_DEPENDENCY_SCAN_ROOTS];
}

function normalizePathForCompare(value) {
  return path.resolve(String(value ?? '')).replaceAll('\\', '/').toLowerCase();
}

function isPathInside(parentPath, childPath) {
  const normalizedParent = normalizePathForCompare(parentPath);
  const normalizedChild = normalizePathForCompare(childPath);
  return normalizedChild === normalizedParent
    || normalizedChild.startsWith(`${normalizedParent}/`);
}

function toPosixRelativePath(basePath, targetPath) {
  return path.relative(basePath, targetPath).replaceAll('\\', '/');
}

function collectFilesByName(rootPath, fileName) {
  if (!existsSync(rootPath)) {
    return [];
  }

  const results = [];
  const queue = [rootPath];

  while (queue.length > 0) {
    const currentPath = queue.shift();
    for (const entry of readdirSync(currentPath, { withFileTypes: true })) {
      if (entry.name === 'node_modules' || entry.name === '.turbo' || entry.name === 'dist') {
        continue;
      }

      const entryPath = path.join(currentPath, entry.name);
      if (entry.isDirectory()) {
        queue.push(entryPath);
        continue;
      }

      if (entry.isFile() && entry.name === fileName) {
        results.push(entryPath);
      }
    }
  }

  return results.sort((left, right) => left.localeCompare(right));
}

function resolveExternalDependencySpecForPath({
  resolvedPath,
  specs = listExternalReleaseDependencySpecs(),
} = {}) {
  return specs.find((spec) => isPathInside(spec.targetDir, resolvedPath)) ?? null;
}

function createExternalDependencyReference({
  sourceFile,
  kind,
  field,
  name,
  rawValue,
  resolvedPath,
  specs,
} = {}) {
  const spec = resolveExternalDependencySpecForPath({
    resolvedPath,
    specs,
  });

  return {
    sourceFile: toPosixRelativePath(rootDir, sourceFile),
    kind,
    field,
    name,
    rawValue,
    resolvedPath,
    dependencyId: spec?.id ?? null,
    covered: spec !== null,
  };
}

function collectPackageJsonExternalDependencyReferences({
  filePath,
  specs,
} = {}) {
  const packageJson = JSON.parse(readFileSync(filePath, 'utf8'));
  const packageDir = path.dirname(filePath);
  const references = [];

  for (const field of PACKAGE_JSON_DEPENDENCY_FIELDS) {
    const deps = packageJson[field];
    if (!deps || typeof deps !== 'object') {
      continue;
    }

    for (const [name, rawValue] of Object.entries(deps)) {
      if (typeof rawValue !== 'string' || !rawValue.startsWith('file:')) {
        continue;
      }

      const resolvedPath = path.resolve(
        packageDir,
        rawValue.slice('file:'.length).replaceAll('\\', '/'),
      );
      if (isPathInside(rootDir, resolvedPath)) {
        continue;
      }

      references.push(createExternalDependencyReference({
        sourceFile: filePath,
        kind: 'package-json',
        field,
        name,
        rawValue,
        resolvedPath,
        specs,
      }));
    }
  }

  return references;
}

function collectWorkspaceExternalDependencyReferences({
  filePath,
  specs,
} = {}) {
  const text = readFileSync(filePath, 'utf8');
  const workspaceDir = path.dirname(filePath);
  const references = [];

  for (const rawLine of text.split(/\r?\n/u)) {
    const match = rawLine.match(/^\s*-\s*["']?(.+?)["']?\s*$/u);
    if (!match) {
      continue;
    }

    const rawValue = match[1].trim();
    const resolvedPath = path.resolve(workspaceDir, rawValue.replaceAll('\\', '/'));
    if (isPathInside(rootDir, resolvedPath)) {
      continue;
    }

    references.push(createExternalDependencyReference({
      sourceFile: filePath,
      kind: 'pnpm-workspace',
      field: 'packages',
      name: rawValue,
      rawValue,
      resolvedPath,
      specs,
    }));
  }

  return references;
}

function collectTsconfigExternalDependencyReferences({
  filePath,
  specs,
} = {}) {
  const tsconfig = JSON.parse(readFileSync(filePath, 'utf8'));
  const tsconfigDir = path.dirname(filePath);
  const references = [];
  const pathsConfig = tsconfig?.compilerOptions?.paths;

  if (!pathsConfig || typeof pathsConfig !== 'object') {
    return references;
  }

  for (const [name, entries] of Object.entries(pathsConfig)) {
    if (!Array.isArray(entries)) {
      continue;
    }

    for (const rawValue of entries) {
      if (typeof rawValue !== 'string') {
        continue;
      }

      const normalizedValue = rawValue.replaceAll('\\', '/');
      if (normalizedValue.startsWith('node_modules/')) {
        continue;
      }

      const resolvedPath = path.resolve(tsconfigDir, normalizedValue);
      if (isPathInside(rootDir, resolvedPath)) {
        continue;
      }

      references.push(createExternalDependencyReference({
        sourceFile: filePath,
        kind: 'tsconfig-path',
        field: 'compilerOptions.paths',
        name,
        rawValue,
        resolvedPath,
        specs,
      }));
    }
  }

  return references;
}

export function auditExternalReleaseDependencyCoverage({
  specs = listExternalReleaseDependencySpecs(),
  scanRoots = listExternalReleaseDependencyScanRoots(),
} = {}) {
  const references = [];

  for (const scanRoot of scanRoots) {
    for (const filePath of collectFilesByName(scanRoot, 'package.json')) {
      references.push(...collectPackageJsonExternalDependencyReferences({
        filePath,
        specs,
      }));
    }

    for (const filePath of collectFilesByName(scanRoot, 'pnpm-workspace.yaml')) {
      references.push(...collectWorkspaceExternalDependencyReferences({
        filePath,
        specs,
      }));
    }

    for (const filePath of collectFilesByName(scanRoot, 'tsconfig.json')) {
      references.push(...collectTsconfigExternalDependencyReferences({
        filePath,
        specs,
      }));
    }
  }

  const externalDependencyIds = [...new Set(
    references
      .map((reference) => reference.dependencyId)
      .filter((dependencyId) => typeof dependencyId === 'string' && dependencyId.length > 0),
  )].sort((left, right) => left.localeCompare(right));

  const uncoveredReferences = references.filter((reference) => reference.covered !== true);

  return {
    covered: uncoveredReferences.length === 0,
    externalDependencyIds,
    references,
    uncoveredReferences,
  };
}

export function resolveExternalReleaseDependencyRef({
  spec,
  env = process.env,
} = {}) {
  const configuredRef = String(env?.[spec?.envRefKey] ?? '').trim();
  return configuredRef.length > 0 ? configuredRef : String(spec?.defaultRef ?? 'main');
}

function resolveExternalReleaseCloneTargetDir(spec = {}) {
  const explicitCloneTargetDir = String(spec.cloneTargetDir ?? '').trim();
  if (explicitCloneTargetDir.length > 0) {
    return path.resolve(explicitCloneTargetDir);
  }

  return path.resolve(String(spec.targetDir ?? ''));
}

function resolveExternalReleaseExpectedGitRoot(spec = {}) {
  const explicitExpectedGitRoot = String(spec.expectedGitRoot ?? '').trim();
  if (explicitExpectedGitRoot.length > 0) {
    return path.resolve(explicitExpectedGitRoot);
  }

  return resolveExternalReleaseCloneTargetDir(spec);
}

function createExpectedExternalReleaseRemoteUrl(spec = {}) {
  const explicitRemoteUrl = String(spec.expectedRemoteUrl ?? '').trim();
  if (explicitRemoteUrl.length > 0) {
    return explicitRemoteUrl;
  }

  const repository = String(spec.repository ?? '').trim();
  return repository.length > 0 ? `https://github.com/${repository}.git` : '';
}

function normalizeGitHubRemoteUrlForCompare(value = '') {
  const normalized = String(value ?? '').trim();
  if (normalized.length === 0) {
    return '';
  }

  const httpsMatch = normalized.match(/^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/iu);
  if (httpsMatch) {
    return `github.com/${httpsMatch[1].toLowerCase()}/${httpsMatch[2].toLowerCase()}`;
  }

  const sshMatch = normalized.match(/^git@github\.com:([^/]+)\/([^/]+?)(?:\.git)?$/iu);
  if (sshMatch) {
    return `github.com/${sshMatch[1].toLowerCase()}/${sshMatch[2].toLowerCase()}`;
  }

  const sshUrlMatch = normalized.match(/^ssh:\/\/git@github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/iu);
  if (sshUrlMatch) {
    return `github.com/${sshUrlMatch[1].toLowerCase()}/${sshUrlMatch[2].toLowerCase()}`;
  }

  return normalized;
}

function resolveGitRunner({
  platform = process.platform,
} = {}) {
  if (platform === 'win32') {
    return {
      command: 'git.exe',
      shell: false,
    };
  }

  return {
    command: 'git',
    shell: false,
  };
}

function runGitCommand({
  cwd,
  args,
  spawnSyncImpl = spawnSync,
} = {}) {
  const runner = resolveGitRunner();
  const result = spawnSyncImpl(runner.command, args, {
    cwd,
    encoding: 'utf8',
    stdio: 'pipe',
    shell: runner.shell,
  });

  return {
    ok: !result.error && (result.status ?? 1) === 0,
    status: result.status ?? 1,
    stdout: String(result.stdout ?? ''),
    stderr: result.error ? String(result.error.message ?? '') : String(result.stderr ?? ''),
    errorMessage: result.error ? String(result.error.message ?? '') : '',
  };
}

export function auditExistingExternalReleaseDependency({
  spec,
  spawnSyncImpl = spawnSync,
} = {}) {
  const topLevelResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['rev-parse', '--show-toplevel'],
    spawnSyncImpl,
  });
  const remoteUrlResult = runGitCommand({
    cwd: spec.targetDir,
    args: ['remote', 'get-url', 'origin'],
    spawnSyncImpl,
  });

  const topLevel = topLevelResult.ok ? topLevelResult.stdout.trim() : '';
  const remoteUrl = remoteUrlResult.ok ? remoteUrlResult.stdout.trim() : '';
  const expectedRemoteUrl = createExpectedExternalReleaseRemoteUrl(spec);
  const expectedGitRoot = resolveExternalReleaseExpectedGitRoot(spec);
  const reasons = [];

  if (!topLevel || normalizePathForCompare(topLevel) !== normalizePathForCompare(expectedGitRoot)) {
    reasons.push('not-standalone-root');
  }

  if (!remoteUrl) {
    reasons.push('remote-url-unverifiable');
  } else if (
    expectedRemoteUrl
    && normalizeGitHubRemoteUrlForCompare(remoteUrl) !== normalizeGitHubRemoteUrlForCompare(expectedRemoteUrl)
  ) {
    reasons.push('remote-url-mismatch');
  }

  return {
    id: spec.id,
    targetDir: spec.targetDir,
    expectedGitRoot,
    expectedRemoteUrl,
    topLevel,
    remoteUrl,
    reasons,
    ready: reasons.length === 0,
  };
}

function assertGovernedExternalReleaseDependencyCheckout({
  spec,
  spawnSyncImpl = spawnSync,
} = {}) {
  const audit = auditExistingExternalReleaseDependency({
    spec,
    spawnSyncImpl,
  });

  if (audit.ready) {
    return audit;
  }

  const detailLines = [
    `External release dependency target is not a governed standalone checkout: ${spec.targetDir}`,
    `Reasons: ${audit.reasons.join(', ')}`,
    `Expected remote: ${audit.expectedRemoteUrl || 'unconfigured'}`,
    `Observed top-level: ${audit.topLevel || 'unavailable'}`,
    `Observed remote: ${audit.remoteUrl || 'unavailable'}`,
  ];
  throw new Error(detailLines.join('\n'));
}

export function buildExternalReleaseClonePlan({
  spec,
  env = process.env,
} = {}) {
  if (!spec?.repository || !spec?.targetDir) {
    throw new Error('buildExternalReleaseClonePlan requires a repository spec with repository and targetDir.');
  }

  const ref = resolveExternalReleaseDependencyRef({ spec, env });
  const cloneTargetDir = resolveExternalReleaseCloneTargetDir(spec);
  return {
    command: 'git',
    args: [
      'clone',
      '--depth',
      '1',
      '--branch',
      ref,
      `https://github.com/${spec.repository}.git`,
      cloneTargetDir,
    ],
    ref,
  };
}

function collectMissingRequiredPaths({
  spec,
  exists = existsSync,
} = {}) {
  return spec.requiredPaths.filter(
    (relativePath) => !exists(path.join(spec.targetDir, relativePath)),
  );
}

function runGitClonePlan({
  plan,
  spawnSyncImpl = spawnSync,
} = {}) {
  const result = spawnSyncImpl(plan.command, plan.args, {
    cwd: rootDir,
    encoding: 'utf8',
    stdio: 'pipe',
    shell: false,
  });

  if (result.error) {
    throw new Error(
      `Failed to clone external release dependency: ${result.error.message}`,
    );
  }

  if ((result.status ?? 0) !== 0) {
    const stdout = String(result.stdout ?? '').trim();
    const stderr = String(result.stderr ?? '').trim();
    const details = [stdout, stderr].filter(Boolean).join('\n');
    throw new Error(
      `git clone exited with code ${result.status ?? 'unknown'}${details ? `\n${details}` : ''}`,
    );
  }
}

export function materializeExternalReleaseDependency({
  spec,
  env = process.env,
  exists = existsSync,
  mkdir = mkdirSync,
  spawnSyncImpl = spawnSync,
} = {}) {
  const missingBefore = collectMissingRequiredPaths({ spec, exists });
  const ref = resolveExternalReleaseDependencyRef({ spec, env });
  const cloneTargetDir = resolveExternalReleaseCloneTargetDir(spec);

  if (missingBefore.length === 0) {
    assertGovernedExternalReleaseDependencyCheckout({
      spec,
      spawnSyncImpl,
    });

    return {
      id: spec.id,
      repository: spec.repository,
      ref,
      status: 'ready',
      skipped: true,
    };
  }

  if (exists(cloneTargetDir)) {
    throw new Error(
      `External release dependency target already exists but is incomplete: ${cloneTargetDir}\nRequired package path: ${spec.targetDir}\nMissing: ${missingBefore.join(', ')}`,
    );
  }

  mkdir(path.dirname(cloneTargetDir), { recursive: true });

  const plan = buildExternalReleaseClonePlan({ spec, env });
  runGitClonePlan({
    plan,
    spawnSyncImpl,
  });

  const missingAfter = collectMissingRequiredPaths({ spec, exists });
  if (missingAfter.length > 0) {
    throw new Error(
      `External release dependency is still incomplete after clone: ${spec.targetDir}\nMissing: ${missingAfter.join(', ')}`,
    );
  }

  assertGovernedExternalReleaseDependencyCheckout({
    spec,
    spawnSyncImpl,
  });

  return {
    id: spec.id,
    repository: spec.repository,
    ref,
    status: 'cloned',
    skipped: false,
  };
}

export function materializeExternalReleaseDependencies({
  specs = listExternalReleaseDependencySpecs(),
  env = process.env,
  exists = existsSync,
  mkdir = mkdirSync,
  spawnSyncImpl = spawnSync,
} = {}) {
  return specs.map((spec) =>
    materializeExternalReleaseDependency({
      spec,
      env,
      exists,
      mkdir,
      spawnSyncImpl,
    }),
  );
}

function runCli() {
  const coverage = auditExternalReleaseDependencyCoverage();
  if (!coverage.covered) {
    const details = coverage.uncoveredReferences.map((reference) =>
      `${reference.sourceFile} ${reference.field}:${reference.name} -> ${reference.rawValue}`,
    );
    throw new Error(
      `External release dependency coverage is incomplete.\n${details.join('\n')}`,
    );
  }

  const results = materializeExternalReleaseDependencies();

  for (const result of results) {
    const action = result.skipped ? 'reused' : 'cloned';
    console.error(
      `[materialize-external-deps] ${action} ${result.id} from ${result.repository}@${result.ref}`,
    );
  }
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    runCli();
  } catch (error) {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  }
}
