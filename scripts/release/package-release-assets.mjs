#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  normalizeDesktopArch,
  resolveDesktopReleaseTarget,
} from './desktop-targets.mjs';
import { materializeReleaseCatalog } from './materialize-release-catalog.mjs';
import { resolveManagedWindowsTauriTargetDir } from '../run-tauri-cli.mjs';
import { resolveWorkspaceTargetDir } from '../workspace-target-dir.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DESKTOP_APP_DIRS = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
};

const DESKTOP_APP_TARGET_DIR_NAMES = {
  admin: 'sdkwork-router-admin-tauri',
  portal: 'sdkwork-router-portal-tauri',
};

const NATIVE_RELEASE_DESKTOP_APP_IDS = ['portal'];

const DESKTOP_RELEASE_ARTIFACT_RULES = {
  windows: {
    artifactKind: 'nsis',
    expectedBundleDirectory: 'nsis',
    expectedFileSuffix: '.exe',
  },
  linux: {
    artifactKind: 'deb',
    expectedBundleDirectory: 'deb',
    expectedFileSuffix: '.deb',
  },
  macos: {
    artifactKind: 'dmg',
    expectedBundleDirectory: 'dmg',
    expectedFileSuffix: '.dmg',
  },
};

const SERVICE_BINARY_NAMES = [
  'admin-api-service',
  'gateway-service',
  'portal-api-service',
  'router-web-service',
  'router-product-service',
];

const productServerSiteAssetRoots = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin', 'dist'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal', 'dist'),
};

const productServerBootstrapDataRoots = {
  data: path.join(rootDir, 'data'),
};

const productServerDeploymentAssetRoots = {
  deploy: path.join(rootDir, 'deploy'),
};

export function normalizePlatformId(platform = process.platform) {
  if (platform === 'win32' || platform === 'windows') {
    return 'windows';
  }
  if (platform === 'darwin' || platform === 'macos') {
    return 'macos';
  }
  if (platform === 'linux') {
    return 'linux';
  }

  throw new Error(`Unsupported release platform: ${platform}`);
}

export function shouldIncludeDesktopBundleFile(platformId, relativePath) {
  const normalizedPlatform = normalizePlatformId(platformId);
  const artifactRule = DESKTOP_RELEASE_ARTIFACT_RULES[normalizedPlatform];
  const normalizedPath = relativePath.replaceAll('\\', '/');
  const [topLevelDirectory] = normalizedPath.split('/');
  if (!artifactRule || topLevelDirectory !== artifactRule.expectedBundleDirectory) {
    return false;
  }

  const lowerCasePath = normalizedPath.toLowerCase();
  return lowerCasePath.endsWith(artifactRule.expectedFileSuffix);
}

export function resolveNativeBuildRoot({ appId, targetTriple = '' } = {}) {
  const appDir = DESKTOP_APP_DIRS[appId];
  if (!appDir) {
    throw new Error(`Unsupported desktop application id: ${appId}`);
  }

  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const targetSegments = normalizedTargetTriple.length > 0
    ? [normalizedTargetTriple]
    : [];

  return path.join(
    appDir,
    'src-tauri',
    'target',
    ...targetSegments,
    'release',
    'bundle',
  );
}

export function resolveNativeBuildRootCandidates({
  appId,
  targetTriple = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  const roots = [];
  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const appDir = DESKTOP_APP_DIRS[appId];
  const workspaceTargetDirName = DESKTOP_APP_TARGET_DIR_NAMES[appId];
  if (!appDir || !workspaceTargetDirName) {
    throw new Error(`Unsupported desktop application id: ${appId}`);
  }

  const appTargetRoot = path.join(appDir, 'target');
  if (normalizedTargetTriple.length > 0) {
    roots.push(path.join(appTargetRoot, normalizedTargetTriple, 'release', 'bundle'));
  }
  roots.push(path.join(appTargetRoot, 'release', 'bundle'));

  const repositoryTargetRoot = resolveWorkspaceTargetDir({
    workspaceRoot: rootDir,
    env,
    platform,
  });
  if (normalizedTargetTriple.length > 0) {
    roots.push(path.join(repositoryTargetRoot, normalizedTargetTriple, 'release', 'bundle'));
  }
  roots.push(path.join(repositoryTargetRoot, 'release', 'bundle'));

  if (normalizePlatformId(platform) === 'windows') {
    const managedWindowsTauriTargetDir = resolveManagedWindowsTauriTargetDir({
      cwd: appDir,
      env,
      platform: 'win32',
    });
    if (managedWindowsTauriTargetDir) {
      if (normalizedTargetTriple.length > 0) {
        roots.push(path.join(managedWindowsTauriTargetDir, normalizedTargetTriple, 'release', 'bundle'));
      }
      roots.push(path.join(managedWindowsTauriTargetDir, 'release', 'bundle'));
    }
  }

  const workspaceTargetRoot = path.join(rootDir, 'target', workspaceTargetDirName);
  if (normalizedTargetTriple.length > 0) {
    roots.push(path.join(workspaceTargetRoot, normalizedTargetTriple, 'release', 'bundle'));
  }
  roots.push(path.join(workspaceTargetRoot, 'release', 'bundle'));

  roots.push(resolveNativeBuildRoot({
    appId,
    targetTriple,
  }));

  if (normalizedTargetTriple.length > 0) {
    roots.push(resolveNativeBuildRoot({ appId }));
  }

  return [...new Set(roots)];
}

export function listNativeServiceBinaryNames() {
  return [...SERVICE_BINARY_NAMES];
}

export function listNativeDesktopAppIds() {
  return [...NATIVE_RELEASE_DESKTOP_APP_IDS];
}

export function listNativeProductServerBootstrapDataRoots() {
  return { ...productServerBootstrapDataRoots };
}

export function listNativeProductServerDeploymentAssetRoots() {
  return { ...productServerDeploymentAssetRoots };
}

export function buildNativeProductServerArchiveBaseName({ platformId, archId } = {}) {
  return `sdkwork-api-router-product-server-${platformId}-${archId}`;
}

export function createNativeProductServerReleaseAssetSpec({ platformId, archId } = {}) {
  const baseName = buildNativeProductServerArchiveBaseName({
    platformId,
    archId,
  });
  const fileName = `${baseName}.tar.gz`;
  return {
    productId: 'sdkwork-api-router-product-server',
    fileName,
    checksumFileName: `${fileName}.sha256.txt`,
    manifestFileName: `${baseName}.manifest.json`,
  };
}

export function buildNativePortalDesktopArtifactBaseName({ platformId, archId } = {}) {
  return `sdkwork-router-portal-desktop-${platformId}-${archId}`;
}

export function createNativePortalDesktopReleaseAssetSpec({ platformId, archId } = {}) {
  const normalizedPlatformId = normalizePlatformId(platformId);
  const normalizedArchId = normalizeDesktopArch(archId);
  const artifactRule = DESKTOP_RELEASE_ARTIFACT_RULES[normalizedPlatformId];
  if (!artifactRule) {
    throw new Error(`Unsupported desktop release platform: ${platformId}`);
  }

  const baseName = buildNativePortalDesktopArtifactBaseName({
    platformId: normalizedPlatformId,
    archId: normalizedArchId,
  });
  const fileName = `${baseName}${artifactRule.expectedFileSuffix}`;

  return {
    appId: 'portal',
    artifactKind: artifactRule.artifactKind,
    fileName,
    checksumFileName: `${fileName}.sha256.txt`,
    manifestFileName: `${baseName}.manifest.json`,
    expectedBundleDirectory: artifactRule.expectedBundleDirectory,
    expectedFileSuffix: artifactRule.expectedFileSuffix,
  };
}

export function resolveAvailableNativeBuildRoot({
  appId,
  targetTriple = '',
  buildRoots,
  exists = existsSync,
  listFiles = listFilesRecursively,
} = {}) {
  const candidates = Array.isArray(buildRoots) && buildRoots.length > 0
    ? buildRoots
    : resolveNativeBuildRootCandidates({
        appId,
        targetTriple,
      });

  let firstExistingRoot = '';
  for (const candidate of candidates) {
    if (!exists(candidate)) {
      continue;
    }

    if (firstExistingRoot.length === 0) {
      firstExistingRoot = candidate;
    }

    if (listFiles(candidate).length > 0) {
      return candidate;
    }
  }

  return firstExistingRoot;
}

function normalizeNodePlatform(platform = process.platform) {
  if (platform === 'windows') {
    return 'win32';
  }
  if (platform === 'macos') {
    return 'darwin';
  }

  return platform;
}

function buildWorkspaceServiceReleaseRoot({
  targetTriple = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const targetSegments = normalizedTargetTriple.length > 0
    ? [normalizedTargetTriple]
    : [];

  return path.join(
    resolveWorkspaceTargetDir({
      workspaceRoot: rootDir,
      env,
      platform: normalizeNodePlatform(platform),
    }),
    ...targetSegments,
    'release',
  );
}

export function resolveServiceReleaseRootCandidates({
  targetTriple = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  const candidates = [];
  const normalizedTargetTriple = String(targetTriple ?? '').trim();

  candidates.push(
    buildWorkspaceServiceReleaseRoot({
      targetTriple,
      env,
      platform,
    }),
  );
  if (normalizedTargetTriple.length > 0) {
    candidates.push(
      buildWorkspaceServiceReleaseRoot({
        env,
        platform,
      }),
    );
  }

  const repositoryTargetRoot = path.join(rootDir, 'target');
  if (normalizedTargetTriple.length > 0) {
    candidates.push(path.join(repositoryTargetRoot, normalizedTargetTriple, 'release'));
  }
  candidates.push(path.join(repositoryTargetRoot, 'release'));

  return [...new Set(candidates)];
}

export function resolveAvailableServiceReleaseRoot({
  targetTriple = '',
  env = process.env,
  platform = process.platform,
  serviceBinaryNames = SERVICE_BINARY_NAMES,
  serviceReleaseRoots,
  exists = existsSync,
} = {}) {
  const platformId = normalizePlatformId(platform);
  const candidates = Array.isArray(serviceReleaseRoots) && serviceReleaseRoots.length > 0
    ? [...new Set(serviceReleaseRoots)]
    : resolveServiceReleaseRootCandidates({
        targetTriple,
        env,
        platform,
      });

  let firstExistingRoot = '';
  for (const candidate of candidates) {
    if (!exists(candidate)) {
      continue;
    }

    if (firstExistingRoot.length === 0) {
      firstExistingRoot = candidate;
    }

    const hasAllServiceBinaries = serviceBinaryNames.every((binaryName) =>
      exists(path.join(candidate, withExecutable(binaryName, platformId))));
    if (hasAllServiceBinaries) {
      return candidate;
    }
  }

  return firstExistingRoot || candidates[0] || buildWorkspaceServiceReleaseRoot({
    targetTriple,
    env,
    platform,
  });
}

function resolveServiceReleaseRoot(options = {}) {
  return resolveAvailableServiceReleaseRoot(options);
}

function parseArgs(argv) {
  const [mode, ...rest] = argv;
  const options = {
    mode,
    platform: process.platform,
    arch: process.arch,
    target: '',
    outputDir: path.join(rootDir, 'artifacts', 'release'),
  };

  for (let index = 0; index < rest.length; index += 1) {
    const token = rest[index];
    const next = rest[index + 1];

    if (token === '--platform') {
      options.platform = next;
      index += 1;
      continue;
    }

    if (token === '--arch') {
      options.arch = next;
      index += 1;
      continue;
    }

    if (token === '--target') {
      options.target = next;
      index += 1;
      continue;
    }

    if (token === '--output-dir') {
      options.outputDir = path.resolve(next);
      index += 1;
      continue;
    }
  }

  return options;
}

function listFilesRecursively(sourceDir, relativePrefix = '') {
  const entries = readdirSync(sourceDir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const relativePath = path.join(relativePrefix, entry.name);
    const absolutePath = path.join(sourceDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursively(absolutePath, relativePath));
      continue;
    }

    if (entry.isFile()) {
      files.push({
        absolutePath,
        relativePath,
      });
    }
  }

  return files;
}

function ensureDirectory(directoryPath) {
  mkdirSync(directoryPath, { recursive: true });
}

function createManagedStagingRoot(stagingParent, prefix) {
  ensureDirectory(stagingParent);
  return mkdtempSync(path.join(stagingParent, prefix));
}

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function escapeGitHubActionsCommandValue(value, { property = false } = {}) {
  let escaped = String(value ?? '');
  escaped = escaped.replaceAll('%', '%25');
  escaped = escaped.replaceAll('\r', '%0D');
  escaped = escaped.replaceAll('\n', '%0A');
  if (property) {
    escaped = escaped.replaceAll(':', '%3A');
    escaped = escaped.replaceAll(',', '%2C');
  }

  return escaped;
}

export function buildGitHubActionsErrorAnnotation({
  title = 'package-release-assets',
  error,
} = {}) {
  const message = truncateText(
    error instanceof Error ? error.message : String(error),
    8000,
  );
  const escapedTitle = escapeGitHubActionsCommandValue(title, { property: true });
  const escapedMessage = escapeGitHubActionsCommandValue(message);
  return `::error title=${escapedTitle}::${escapedMessage}`;
}

function describeDirectoryState(sourceDir, maxEntries = 12) {
  if (!existsSync(sourceDir)) {
    return `${sourceDir} [missing]`;
  }

  const files = listFilesRecursively(sourceDir);
  if (files.length === 0) {
    return `${sourceDir} [exists, empty]`;
  }

  const sample = files
    .slice(0, maxEntries)
    .map((file) => file.relativePath.replaceAll('\\', '/'))
    .join(', ');
  const remainingCount = files.length - Math.min(files.length, maxEntries);
  const remainingSuffix = remainingCount > 0 ? ` (+${remainingCount} more)` : '';
  return `${sourceDir} [${files.length} files: ${sample}${remainingSuffix}]`;
}

function writeSha256File(filePath) {
  const checksum = createHash('sha256').update(readFileSync(filePath)).digest('hex');
  writeFileSync(
    `${filePath}.sha256.txt`,
    `${checksum}  ${path.basename(filePath)}\n`,
    'utf8',
  );
}

function writeJsonFile(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function withExecutable(binaryName, platformId) {
  return platformId === 'windows' ? `${binaryName}.exe` : binaryName;
}

function createPortalDesktopEmbeddedRuntimeManifest({ platformId } = {}) {
  return {
    routerBinary: `router-product/bin/${withExecutable('router-product-service', platformId)}`,
    adminSiteDir: 'router-product/sites/admin/dist',
    portalSiteDir: 'router-product/sites/portal/dist',
    bootstrapDataDir: 'router-product/data',
    releaseManifestFile: 'router-product/release-manifest.json',
    readmeFile: 'router-product/README.txt',
  };
}

function copyServiceBinaries({
  platformId,
  targetTriple,
  targetDir,
  writeChecksums = false,
  resolveServiceRoot = resolveServiceReleaseRoot,
  serviceReleaseRoots,
  serviceBinaryNames = SERVICE_BINARY_NAMES,
}) {
  const serviceReleaseRoot = resolveServiceRoot({
    targetTriple,
    platform: platformId,
    serviceReleaseRoots,
    serviceBinaryNames,
  });
  ensureDirectory(targetDir);

  for (const binaryName of serviceBinaryNames) {
    const fileName = withExecutable(binaryName, platformId);
    const sourcePath = path.join(serviceReleaseRoot, fileName);
    if (!existsSync(sourcePath)) {
      throw new Error(
        `Missing release service binary: ${sourcePath}\nservice release root: ${describeDirectoryState(serviceReleaseRoot)}`,
      );
    }

    const targetPath = path.join(targetDir, fileName);
    cpSync(sourcePath, targetPath);
    if (writeChecksums) {
      writeSha256File(targetPath);
    }
  }
}

export function packageDesktopBundles({
  platformId,
  archId,
  targetTriple,
  outputDir,
  resolveBuildRoots = resolveNativeBuildRootCandidates,
  resolveBuildRoot = resolveAvailableNativeBuildRoot,
} = {}) {
  const packagedAssets = [];

  for (const appId of NATIVE_RELEASE_DESKTOP_APP_IDS) {
    const releaseAssetSpec = createNativePortalDesktopReleaseAssetSpec({
      platformId,
      archId,
    });
    const buildRoots = resolveBuildRoots({ appId, targetTriple });
    const buildRoot = resolveBuildRoot({
      appId,
      targetTriple,
      buildRoots,
    });
    if (!buildRoot) {
      throw new Error(
        `Missing desktop bundle output directory for ${appId}. candidates: ${buildRoots.map((root) => describeDirectoryState(root)).join(' | ')}`,
      );
    }

    const allBundleFiles = listFilesRecursively(buildRoot);
    const bundleFiles = allBundleFiles
      .filter((file) => shouldIncludeDesktopBundleFile(platformId, file.relativePath));

    if (bundleFiles.length === 0) {
      throw new Error(
        [
          `Missing official ${platformId} desktop installer for ${appId}.`,
          `Expected ${releaseAssetSpec.expectedBundleDirectory}/*${releaseAssetSpec.expectedFileSuffix} under ${buildRoot}`,
          `bundle root: ${describeDirectoryState(buildRoot)}`,
        ].join('\n'),
      );
    }
    if (bundleFiles.length !== 1) {
      throw new Error(
        [
          `Expected exactly one official ${platformId} desktop installer for ${appId}, found ${bundleFiles.length}.`,
          `Matched files: ${bundleFiles.map((file) => file.relativePath.replaceAll('\\', '/')).join(', ')}`,
          `Expected ${releaseAssetSpec.expectedBundleDirectory}/*${releaseAssetSpec.expectedFileSuffix} under ${buildRoot}`,
        ].join('\n'),
      );
    }

    const appOutputDir = path.join(outputDir, 'native', platformId, archId, 'desktop', appId);
    rmSync(appOutputDir, { recursive: true, force: true });
    ensureDirectory(appOutputDir);

    const [bundleFile] = bundleFiles;
    const installerTargetPath = path.join(appOutputDir, releaseAssetSpec.fileName);
    cpSync(bundleFile.absolutePath, installerTargetPath);
    writeSha256File(installerTargetPath);
    writeJsonFile(
      path.join(appOutputDir, releaseAssetSpec.manifestFileName),
      {
        type: 'portal-desktop-installer',
        productId: 'sdkwork-router-portal-desktop',
        appId,
        platform: platformId,
        arch: archId,
        target: targetTriple,
        artifactKind: releaseAssetSpec.artifactKind,
        installerFile: releaseAssetSpec.fileName,
        checksumFile: releaseAssetSpec.checksumFileName,
        sourceBundlePath: bundleFile.relativePath.replaceAll('\\', '/'),
        embeddedRuntime: createPortalDesktopEmbeddedRuntimeManifest({ platformId }),
      },
    );

    packagedAssets.push({
      appId,
      platformId,
      archId,
      targetTriple,
      fileName: releaseAssetSpec.fileName,
      checksumFileName: releaseAssetSpec.checksumFileName,
      manifestFileName: releaseAssetSpec.manifestFileName,
      outputDir: appOutputDir,
    });
  }

  return packagedAssets;
}

function writeProductServerBundleReadme({ archiveRoot, platformId, archId, targetTriple }) {
  writeFileSync(
    path.join(archiveRoot, 'README.txt'),
    [
      'SDKWork API Router Product Server Bundle',
      '',
      `platform: ${platformId}`,
      `arch: ${archId}`,
      `target: ${targetTriple}`,
      '',
      'Contents:',
      '- bin/: standalone services plus router-product-service',
      '- sites/admin/dist/: admin web assets',
      '- sites/portal/dist/: portal web assets',
      '- data/: bootstrap data packs for first-start initialization',
      '- deploy/: docker, compose, and helm deployment assets',
      '',
      'Example startup:',
      platformId === 'windows'
        ? '  set SDKWORK_BOOTSTRAP_DATA_DIR=data && set SDKWORK_ADMIN_SITE_DIR=sites\\admin\\dist && set SDKWORK_PORTAL_SITE_DIR=sites\\portal\\dist && bin\\router-product-service.exe'
        : '  SDKWORK_BOOTSTRAP_DATA_DIR=data SDKWORK_ADMIN_SITE_DIR=sites/admin/dist SDKWORK_PORTAL_SITE_DIR=sites/portal/dist ./bin/router-product-service',
      '',
      'Container image builds reuse the Linux product-server bundle with:',
      '  docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:<tag> .',
      '',
      'Override SDKWORK_CONFIG_DIR, SDKWORK_CONFIG_FILE, SDKWORK_DATABASE_URL, and role/upstream flags as needed.',
      '',
    ].join('\n'),
    'utf8',
  );
}

export function packageProductServerBundle({
  platformId,
  archId,
  targetTriple,
  outputDir,
  resolveServiceRoot = resolveServiceReleaseRoot,
  resolveServiceRootCandidates,
  serviceBinaryNames = SERVICE_BINARY_NAMES,
  siteAssetRoots = productServerSiteAssetRoots,
  bootstrapDataRoots = productServerBootstrapDataRoots,
  deploymentAssetRoots = productServerDeploymentAssetRoots,
  runTar = runTarCommand,
} = {}) {
  for (const [label, sourceDir] of Object.entries(siteAssetRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(
        `Missing product server site assets for ${label}: ${sourceDir}\nsite asset root: ${describeDirectoryState(sourceDir)}`,
      );
    }
  }
  for (const [label, sourceDir] of Object.entries(bootstrapDataRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(
        `Missing product server bootstrap data for ${label}: ${sourceDir}\nbootstrap data root: ${describeDirectoryState(sourceDir)}`,
      );
    }
  }
  for (const [label, sourceDir] of Object.entries(deploymentAssetRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(
        `Missing product server deployment assets for ${label}: ${sourceDir}\ndeployment asset root: ${describeDirectoryState(sourceDir)}`,
      );
    }
  }

  const archiveBaseName = buildNativeProductServerArchiveBaseName({
    platformId,
    archId,
  });
  const releaseAssetSpec = createNativeProductServerReleaseAssetSpec({
    platformId,
    archId,
  });
  const bundleOutputDir = path.join(outputDir, 'native', platformId, archId, 'bundles');
  const serviceReleaseRoots = typeof resolveServiceRootCandidates === 'function'
    ? resolveServiceRootCandidates({
        targetTriple,
        platform: platformId,
        serviceBinaryNames,
      })
    : undefined;
  ensureDirectory(bundleOutputDir);

  const stagingRoot = createManagedStagingRoot(bundleOutputDir, '.sdkwork-api-router-native-server-');
  const archiveRoot = path.join(stagingRoot, archiveBaseName);

  try {
    copyServiceBinaries({
      platformId,
      targetTriple,
      targetDir: path.join(archiveRoot, 'bin'),
      resolveServiceRoot,
      serviceReleaseRoots,
      serviceBinaryNames,
    });

    for (const [label, sourceDir] of Object.entries(siteAssetRoots)) {
      const targetDir = path.join(archiveRoot, 'sites', label, 'dist');
      ensureDirectory(path.dirname(targetDir));
      cpSync(sourceDir, targetDir, { recursive: true });
    }

    for (const [label, sourceDir] of Object.entries(bootstrapDataRoots)) {
      const targetDir = path.join(archiveRoot, label);
      ensureDirectory(path.dirname(targetDir));
      cpSync(sourceDir, targetDir, { recursive: true });
    }

    for (const [label, sourceDir] of Object.entries(deploymentAssetRoots)) {
      const targetDir = path.join(archiveRoot, label);
      ensureDirectory(path.dirname(targetDir));
      cpSync(sourceDir, targetDir, { recursive: true });
    }

    writeProductServerBundleReadme({
      archiveRoot,
      platformId,
      archId,
      targetTriple,
    });

    writeFileSync(
      path.join(archiveRoot, 'release-manifest.json'),
      JSON.stringify(
        {
          type: 'product-server-bundle',
          productId: releaseAssetSpec.productId,
          platform: platformId,
          arch: archId,
          target: targetTriple,
          services: [...serviceBinaryNames],
          sites: Object.keys(siteAssetRoots),
          bootstrapDataRoots: Object.keys(bootstrapDataRoots),
          deploymentAssetRoots: Object.keys(deploymentAssetRoots),
        },
        null,
        2,
      ),
      'utf8',
    );

    const archivePath = path.join(bundleOutputDir, releaseAssetSpec.fileName);
    rmSync(archivePath, { force: true });
    rmSync(path.join(bundleOutputDir, releaseAssetSpec.checksumFileName), { force: true });
    rmSync(path.join(bundleOutputDir, releaseAssetSpec.manifestFileName), { force: true });
    runTar(archivePath, stagingRoot, archiveBaseName);
    writeSha256File(archivePath);
    writeJsonFile(
      path.join(bundleOutputDir, releaseAssetSpec.manifestFileName),
      {
        type: 'product-server-archive',
        productId: releaseAssetSpec.productId,
        platform: platformId,
        arch: archId,
        target: targetTriple,
        archiveFile: releaseAssetSpec.fileName,
        checksumFile: releaseAssetSpec.checksumFileName,
        embeddedManifestFile: 'release-manifest.json',
        services: [...serviceBinaryNames],
        sites: Object.keys(siteAssetRoots),
        bootstrapDataRoots: Object.keys(bootstrapDataRoots),
        deploymentAssetRoots: Object.keys(deploymentAssetRoots),
      },
    );
    return {
      productId: releaseAssetSpec.productId,
      platformId,
      archId,
      targetTriple,
      fileName: releaseAssetSpec.fileName,
      checksumFileName: releaseAssetSpec.checksumFileName,
      manifestFileName: releaseAssetSpec.manifestFileName,
      outputDir: bundleOutputDir,
    };
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
}

export function packageNativeAssets({
  platform,
  arch,
  target,
  outputDir,
  releaseTag = '',
  generatedAt = '',
  packageDesktopBundlesImpl = packageDesktopBundles,
  packageProductServerBundleImpl = packageProductServerBundle,
  materializeReleaseCatalogImpl = materializeReleaseCatalog,
} = {}) {
  const targetSpec = resolveDesktopReleaseTarget({
    targetTriple: target,
    platform,
    arch,
  });
  const platformId = normalizePlatformId(targetSpec.platform);
  const archId = normalizeDesktopArch(targetSpec.arch);
  const nativeOutputDir = path.join(outputDir, 'native', platformId, archId);
  rmSync(nativeOutputDir, { recursive: true, force: true });
  ensureDirectory(nativeOutputDir);

  const desktopAssets = packageDesktopBundlesImpl({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
  const productServerBundle = packageProductServerBundleImpl({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
  const releaseCatalog = materializeReleaseCatalogImpl({
    assetsRoot: outputDir,
    releaseTag,
    generatedAt,
    outputPath: path.join(outputDir, 'release-catalog.json'),
  });

  return {
    target: targetSpec,
    desktopAssets,
    productServerBundle,
    releaseCatalog,
  };
}

export function detectTarFlavor({
  platform = process.platform,
  spawn = spawnSync,
} = {}) {
  if (platform !== 'win32') {
    return 'default';
  }

  const result = spawn('tar', ['--version'], {
    cwd: rootDir,
    shell: false,
    encoding: 'utf8',
  });
  if (result.error || result.status !== 0) {
    return 'unknown';
  }

  const versionOutput = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.toLowerCase();
  if (versionOutput.includes('gnu tar')) {
    return 'gnu';
  }
  if (versionOutput.includes('bsdtar') || versionOutput.includes('libarchive')) {
    return 'bsd';
  }

  return 'unknown';
}

export function createTarCommandPlan({
  archivePath,
  workingDirectory,
  entryName,
  platform = process.platform,
  tarFlavor = platform === 'win32' ? 'unknown' : 'default',
} = {}) {
  const args = [];
  if (platform === 'win32' && tarFlavor === 'gnu') {
    args.push('--force-local');
  }
  args.push('-czf', archivePath, '-C', workingDirectory, entryName);

  return {
    command: 'tar',
    args,
    shell: platform === 'win32',
  };
}

function runTarCommand(archivePath, workingDirectory, entryName) {
  const tarFlavor = detectTarFlavor();
  const plan = createTarCommandPlan({
    archivePath,
    workingDirectory,
    entryName,
    tarFlavor,
  });
  const result = spawnSync(plan.command, plan.args, {
    cwd: rootDir,
    shell: plan.shell,
    encoding: 'utf8',
  });

  if (result.error) {
    throw new Error(`tar failed while packaging ${archivePath}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    const stdout = truncateText(result.stdout, 2000);
    const stderr = truncateText(result.stderr, 2000);
    const output = [stdout && `stdout: ${stdout}`, stderr && `stderr: ${stderr}`]
      .filter(Boolean)
      .join('\n');
    throw new Error(
      `tar failed while packaging ${archivePath} with exit code ${result.status ?? 'unknown'}${output ? `\n${output}` : ''}`,
    );
  }
}

function printUsage() {
  console.error(
    [
      'Usage:',
      '  node scripts/release/package-release-assets.mjs native --platform <windows|linux|macos> --arch <x64|arm64> --target <triple> --output-dir <dir>',
    ].join('\n'),
  );
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!options.mode) {
    printUsage();
    process.exit(1);
  }

  ensureDirectory(options.outputDir);

  if (options.mode === 'native') {
    packageNativeAssets(options);
    return;
  }

  console.error(`Unsupported packaging mode: ${options.mode}`);
  printUsage();
  process.exit(1);
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    main();
  } catch (error) {
    if (process.env.GITHUB_ACTIONS === 'true') {
      console.error(buildGitHubActionsErrorAnnotation({ error }));
    }
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  }
}
