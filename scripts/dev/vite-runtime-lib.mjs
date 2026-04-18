import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..', '..');

function normalizeRelativeEntry(relativeEntry) {
  if (Array.isArray(relativeEntry)) {
    return relativeEntry;
  }

  return [relativeEntry];
}

function defaultFileExists(filePath) {
  return fs.existsSync(filePath);
}

function defaultReadDir(directoryPath, options) {
  return fs.readdirSync(directoryPath, options);
}

function listWorkspaceAppRoots(appsRoot) {
  if (!defaultFileExists(appsRoot)) {
    return [];
  }

  let entries;
  try {
    entries = fs.readdirSync(appsRoot, { withFileTypes: true });
  } catch {
    return [];
  }

  return entries
    .filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
    .map((entry) => path.join(appsRoot, entry.name))
    .filter((candidateRoot) => (
      defaultFileExists(path.join(candidateRoot, 'package.json'))
      && defaultFileExists(path.join(candidateRoot, 'node_modules'))
    ));
}

function defaultOpenFile(filePath) {
  return fs.openSync(filePath, 'r');
}

function defaultCloseFile(fileDescriptor) {
  fs.closeSync(fileDescriptor);
}

function defaultResolveFromRoot(root, specifier) {
  return createRequire(path.join(root, 'package.json')).resolve(specifier);
}

export function probeReadableFile(
  filePath,
  {
    fileExists = defaultFileExists,
    openFile = defaultOpenFile,
    closeFile = defaultCloseFile,
  } = {},
) {
  if (!fileExists(filePath)) {
    return false;
  }

  try {
    const fileDescriptor = openFile(filePath);
    closeFile(fileDescriptor);
    return true;
  } catch {
    return false;
  }
}

function defaultIsReadable(filePath) {
  return probeReadableFile(filePath);
}

export function resolveWorkspaceDonorRoots(appRoot) {
  const normalizedAppRoot = path.resolve(appRoot);
  const knownWorkspaceApps = [
    ...listWorkspaceAppRoots(path.join(repoRoot, 'apps')),
    ...listWorkspaceAppRoots(path.resolve(repoRoot, '..')),
  ];

  return knownWorkspaceApps
    .map((candidateRoot) => path.resolve(candidateRoot))
    .filter((candidateRoot) => candidateRoot !== normalizedAppRoot);
}

export function resolveReadablePackageEntry({
  appRoot,
  donorRoots = [],
  packageName,
  relativeEntry,
  fileExists = defaultFileExists,
  readDir = defaultReadDir,
  isReadable = defaultIsReadable,
}) {
  const entrySegments = normalizeRelativeEntry(relativeEntry);
  const candidateRoots = [appRoot, ...donorRoots]
    .map((candidateRoot) => path.resolve(candidateRoot))
    .filter((candidateRoot, index, roots) => roots.indexOf(candidateRoot) === index);

  for (const candidateRoot of candidateRoots) {
    const directEntry = path.join(
      candidateRoot,
      'node_modules',
      packageName,
      ...entrySegments,
    );

    const candidateEntries = [directEntry];
    const pnpmRoot = path.join(candidateRoot, 'node_modules', '.pnpm');
    if (fileExists(pnpmRoot)) {
      const pnpmDirectoryPrefix = `${packageName.replace('/', '+')}@`;

      let pnpmEntries = [];
      try {
        pnpmEntries = readDir(pnpmRoot, { withFileTypes: true });
      } catch {
        pnpmEntries = [];
      }

      candidateEntries.push(...pnpmEntries
        .filter((entry) => entry.isDirectory() && entry.name.startsWith(pnpmDirectoryPrefix))
        .sort((left, right) => right.name.localeCompare(left.name))
        .map((entry) => path.join(
          pnpmRoot,
          entry.name,
          'node_modules',
          packageName,
          ...entrySegments,
        )));
    }

    for (const candidateEntry of candidateEntries) {
      if (fileExists(candidateEntry) && isReadable(candidateEntry)) {
        return candidateEntry;
      }
    }
  }

  throw new Error(
    `unable to resolve a readable ${packageName} entry (${entrySegments.join('/')}) from ${candidateRoots.join(', ')}`,
  );
}

export function resolveReadablePackageImportUrl(options) {
  return pathToFileURL(resolveReadablePackageEntry(options)).href;
}

export function findReadableModuleResolution({
  appRoot,
  donorRoots = [],
  specifier,
  resolveFromRoot = defaultResolveFromRoot,
  isReadable = defaultIsReadable,
}) {
  const candidateRoots = [appRoot, ...donorRoots]
    .map((candidateRoot) => path.resolve(candidateRoot))
    .filter((candidateRoot, index, roots) => roots.indexOf(candidateRoot) === index);

  for (const candidateRoot of candidateRoots) {
    let resolvedPath;
    try {
      resolvedPath = resolveFromRoot(candidateRoot, specifier);
    } catch {
      continue;
    }

    if (isReadable(resolvedPath)) {
      return {
        candidateRoot,
        resolvedPath,
      };
    }
  }

  throw new Error(
    `unable to resolve readable module specifier "${specifier}" from ${candidateRoots.join(', ')}`,
  );
}

export function resolveReadableModuleSpecifier(options) {
  return findReadableModuleResolution(options).resolvedPath;
}

export function resolveReadablePackageRoot({
  relativeEntry,
  ...options
}) {
  return path.dirname(resolveReadablePackageEntry({
    ...options,
    relativeEntry: 'package.json',
  }));
}

export async function importReadablePackageDefault(options) {
  const moduleUrl = resolveReadablePackageImportUrl(options);
  const loadedModule = await import(moduleUrl);
  return loadedModule.default ?? loadedModule;
}

export async function applyWindowsVitePreload({
  platform = process.platform,
} = {}) {
  if (platform !== 'win32') {
    return;
  }

  await import(pathToFileURL(path.join(scriptDir, 'vite-windows-realpath-preload.mjs')).href);
}
