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
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  normalizeDesktopArch,
  resolveDesktopReleaseTarget,
} from './desktop-targets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DESKTOP_APP_DIRS = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
  console: path.join(rootDir, 'console'),
};

const SERVICE_BINARY_NAMES = [
  'admin-api-service',
  'gateway-service',
  'portal-api-service',
  'router-web-service',
  'router-product-service',
];

const desktopBundleRules = {
  windows: {
    directories: new Set(['msi', 'nsis']),
    suffixes: ['.msi', '.exe'],
  },
  linux: {
    directories: new Set(['appimage', 'deb', 'rpm']),
    suffixes: ['.appimage', '.deb', '.rpm'],
  },
  macos: {
    directories: new Set(['dmg', 'macos']),
    suffixes: ['.dmg', '.app.tar.gz', '.app.zip', '.zip'],
  },
};

const webAssetRoots = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin', 'dist'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal', 'dist'),
  console: path.join(rootDir, 'console', 'dist'),
  docs: path.join(rootDir, 'docs', '.vitepress', 'dist'),
};

const productServerSiteAssetRoots = {
  admin: path.join(rootDir, 'apps', 'sdkwork-router-admin', 'dist'),
  portal: path.join(rootDir, 'apps', 'sdkwork-router-portal', 'dist'),
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
  const normalizedPath = relativePath.replaceAll('\\', '/');
  const [topLevelDirectory] = normalizedPath.split('/');
  const rule = desktopBundleRules[normalizedPlatform];
  if (!rule.directories.has(topLevelDirectory)) {
    return false;
  }

  const lowerCasePath = normalizedPath.toLowerCase();
  return rule.suffixes.some((suffix) => lowerCasePath.endsWith(suffix));
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

export function listNativeServiceBinaryNames() {
  return [...SERVICE_BINARY_NAMES];
}

export function buildNativeProductServerArchiveBaseName({ platformId, archId } = {}) {
  return `sdkwork-api-router-product-server-${platformId}-${archId}`;
}

function resolveServiceReleaseRoot({ targetTriple = '' } = {}) {
  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const targetSegments = normalizedTargetTriple.length > 0
    ? [normalizedTargetTriple]
    : [];

  return path.join(rootDir, 'target', ...targetSegments, 'release');
}

function buildWebArchiveBaseName(releaseTag) {
  if (typeof releaseTag !== 'string' || releaseTag.trim().length === 0) {
    throw new Error('releaseTag is required to package web release assets.');
  }

  return `sdkwork-api-router-web-assets-${releaseTag.trim()}`;
}

function parseArgs(argv) {
  const [mode, ...rest] = argv;
  const options = {
    mode,
    platform: process.platform,
    arch: process.arch,
    target: '',
    outputDir: path.join(rootDir, 'artifacts', 'release'),
    releaseTag: '',
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

    if (token === '--release-tag') {
      options.releaseTag = next;
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

function writeSha256File(filePath) {
  const checksum = createHash('sha256').update(readFileSync(filePath)).digest('hex');
  writeFileSync(
    `${filePath}.sha256.txt`,
    `${checksum}  ${path.basename(filePath)}\n`,
    'utf8',
  );
}

function withExecutable(binaryName, platformId) {
  return platformId === 'windows' ? `${binaryName}.exe` : binaryName;
}

function copyServiceBinaries({ platformId, targetTriple, targetDir, writeChecksums = false }) {
  const serviceReleaseRoot = resolveServiceReleaseRoot({ targetTriple });
  ensureDirectory(targetDir);

  for (const binaryName of SERVICE_BINARY_NAMES) {
    const fileName = withExecutable(binaryName, platformId);
    const sourcePath = path.join(serviceReleaseRoot, fileName);
    if (!existsSync(sourcePath)) {
      throw new Error(`Missing release service binary: ${sourcePath}`);
    }

    const targetPath = path.join(targetDir, fileName);
    cpSync(sourcePath, targetPath);
    if (writeChecksums) {
      writeSha256File(targetPath);
    }
  }
}

function packageServiceBinaries({ platformId, archId, targetTriple, outputDir }) {
  const serviceOutputDir = path.join(outputDir, 'native', platformId, archId, 'services');
  copyServiceBinaries({
    platformId,
    targetTriple,
    targetDir: serviceOutputDir,
    writeChecksums: true,
  });
}

function packageDesktopBundles({ platformId, archId, targetTriple, outputDir }) {
  for (const appId of Object.keys(DESKTOP_APP_DIRS)) {
    const buildRoot = resolveNativeBuildRoot({
      appId,
      targetTriple,
    });
    if (!existsSync(buildRoot)) {
      throw new Error(`Missing desktop bundle output directory: ${buildRoot}`);
    }

    const bundleFiles = listFilesRecursively(buildRoot)
      .filter((file) => shouldIncludeDesktopBundleFile(platformId, file.relativePath));

    if (bundleFiles.length === 0) {
      throw new Error(`No ${platformId} desktop release assets matched under ${buildRoot}`);
    }

    const appOutputDir = path.join(outputDir, 'native', platformId, archId, 'desktop', appId);
    ensureDirectory(appOutputDir);

    for (const bundleFile of bundleFiles) {
      const targetPath = path.join(appOutputDir, bundleFile.relativePath);
      ensureDirectory(path.dirname(targetPath));
      cpSync(bundleFile.absolutePath, targetPath);
      writeSha256File(targetPath);
    }
  }
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
      '',
      'Example startup:',
      platformId === 'windows'
        ? '  set SDKWORK_ADMIN_SITE_DIR=sites\\admin\\dist && set SDKWORK_PORTAL_SITE_DIR=sites\\portal\\dist && bin\\router-product-service.exe'
        : '  SDKWORK_ADMIN_SITE_DIR=sites/admin/dist SDKWORK_PORTAL_SITE_DIR=sites/portal/dist ./bin/router-product-service',
      '',
      'Override SDKWORK_CONFIG_DIR, SDKWORK_CONFIG_FILE, SDKWORK_DATABASE_URL, and role/upstream flags as needed.',
      '',
    ].join('\n'),
    'utf8',
  );
}

function packageProductServerBundle({ platformId, archId, targetTriple, outputDir }) {
  for (const [label, sourceDir] of Object.entries(productServerSiteAssetRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(`Missing product server site assets for ${label}: ${sourceDir}`);
    }
  }

  const archiveBaseName = buildNativeProductServerArchiveBaseName({
    platformId,
    archId,
  });
  const bundleOutputDir = path.join(outputDir, 'native', platformId, archId, 'bundles');
  ensureDirectory(bundleOutputDir);

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-api-router-native-server-'));
  const archiveRoot = path.join(stagingRoot, archiveBaseName);

  try {
    copyServiceBinaries({
      platformId,
      targetTriple,
      targetDir: path.join(archiveRoot, 'bin'),
    });

    for (const [label, sourceDir] of Object.entries(productServerSiteAssetRoots)) {
      const targetDir = path.join(archiveRoot, 'sites', label, 'dist');
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
          platform: platformId,
          arch: archId,
          target: targetTriple,
          services: listNativeServiceBinaryNames(),
          sites: Object.keys(productServerSiteAssetRoots),
        },
        null,
        2,
      ),
      'utf8',
    );

    const archivePath = path.join(bundleOutputDir, `${archiveBaseName}.tar.gz`);
    rmSync(archivePath, { force: true });
    rmSync(`${archivePath}.sha256.txt`, { force: true });
    runTarCommand(archivePath, stagingRoot, archiveBaseName);
    writeSha256File(archivePath);
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
}

function packageNativeAssets({ platform, arch, target, outputDir }) {
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

  packageServiceBinaries({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
  packageDesktopBundles({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
  packageProductServerBundle({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
}

function runTarCommand(archivePath, workingDirectory, entryName) {
  const result = spawnSync('tar', ['-czf', archivePath, '-C', workingDirectory, entryName], {
    cwd: rootDir,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  if (result.error) {
    throw new Error(`tar failed while packaging ${archivePath}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`tar failed while packaging ${archivePath} with exit code ${result.status ?? 'unknown'}`);
  }
}

function packageWebAssets({ releaseTag, outputDir }) {
  const archiveBaseName = buildWebArchiveBaseName(releaseTag);
  ensureDirectory(outputDir);

  for (const [label, sourceDir] of Object.entries(webAssetRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(`Missing web release asset root for ${label}: ${sourceDir}`);
    }
  }

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-api-router-release-web-'));
  const archiveRoot = path.join(stagingRoot, archiveBaseName);

  try {
    for (const [label, sourceDir] of Object.entries(webAssetRoots)) {
      ensureDirectory(path.join(archiveRoot, label));
      cpSync(sourceDir, path.join(archiveRoot, label, 'dist'), { recursive: true });
    }

    const archivePath = path.join(outputDir, `${archiveBaseName}.tar.gz`);
    rmSync(archivePath, { force: true });
    rmSync(`${archivePath}.sha256.txt`, { force: true });
    runTarCommand(archivePath, stagingRoot, archiveBaseName);
    writeSha256File(archivePath);
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
}

function printUsage() {
  console.error(
    [
      'Usage:',
      '  node scripts/release/package-release-assets.mjs native --platform <windows|linux|macos> --arch <x64|arm64> --target <triple> --output-dir <dir>',
      '  node scripts/release/package-release-assets.mjs web --release-tag <tag> --output-dir <dir>',
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

  if (options.mode === 'web') {
    packageWebAssets(options);
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
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
