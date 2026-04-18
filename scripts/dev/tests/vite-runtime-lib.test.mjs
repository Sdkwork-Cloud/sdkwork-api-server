import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import {
  findReadableModuleResolution,
  probeReadableFile,
  resolveReadablePackageEntry,
  resolveReadablePackageRoot,
} from '../vite-runtime-lib.mjs';

const adminRoot = path.resolve('/workspace/apps/admin');
const portalRoot = path.resolve('/workspace/apps/portal');

test('resolveReadablePackageEntry prefers the current app package path when it is readable', () => {
  const expectedEntry = path.join(adminRoot, 'node_modules', 'vite', 'bin', 'vite.js');
  const entry = resolveReadablePackageEntry({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    packageName: 'vite',
    relativeEntry: ['bin', 'vite.js'],
    fileExists(filePath) {
      return filePath === expectedEntry;
    },
    isReadable(filePath) {
      return filePath === expectedEntry;
    },
  });

  assert.equal(entry, expectedEntry);
});

test('resolveReadablePackageEntry falls back to a donor app when the current app copy is unreadable', () => {
  const currentEntry = path.join(
    adminRoot,
    'node_modules',
    '@vitejs',
    'plugin-react',
    'dist',
    'index.js',
  );
  const donorEntry = path.join(
    portalRoot,
    'node_modules',
    '@vitejs',
    'plugin-react',
    'dist',
    'index.js',
  );
  const entry = resolveReadablePackageEntry({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    packageName: '@vitejs/plugin-react',
    relativeEntry: ['dist', 'index.js'],
    fileExists(filePath) {
      return [currentEntry, donorEntry].includes(filePath);
    },
    isReadable(filePath) {
      return filePath === donorEntry;
    },
  });

  assert.equal(entry, donorEntry);
});

test('resolveReadablePackageEntry can recover a readable TypeScript compiler runtime from a donor app', () => {
  const currentEntry = path.join(
    adminRoot,
    'node_modules',
    'typescript',
    'lib',
    'tsc.js',
  );
  const donorEntry = path.join(
    portalRoot,
    'node_modules',
    'typescript',
    'lib',
    'tsc.js',
  );
  const entry = resolveReadablePackageEntry({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    packageName: 'typescript',
    relativeEntry: ['lib', 'tsc.js'],
    fileExists(filePath) {
      return [currentEntry, donorEntry].includes(filePath);
    },
    isReadable(filePath) {
      return filePath === donorEntry;
    },
  });

  assert.equal(entry, donorEntry);
});

test('resolveReadablePackageEntry throws when no readable package entry exists', () => {
  assert.throws(() => resolveReadablePackageEntry({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    packageName: '@tailwindcss/vite',
    relativeEntry: ['dist', 'index.mjs'],
    fileExists() {
      return false;
    },
    isReadable() {
      return false;
    },
  }), /unable to resolve a readable @tailwindcss\/vite entry/);
});

test('resolveReadablePackageRoot falls back to a donor app when the current app package root is unreadable', () => {
  const currentPackageJson = path.join(
    adminRoot,
    'node_modules',
    'react-router-dom',
    'package.json',
  );
  const donorPackageJson = path.join(
    portalRoot,
    'node_modules',
    'react-router-dom',
    'package.json',
  );

  const packageRoot = resolveReadablePackageRoot({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    packageName: 'react-router-dom',
    fileExists(filePath) {
      return [currentPackageJson, donorPackageJson].includes(filePath);
    },
    isReadable(filePath) {
      return filePath === donorPackageJson;
    },
  });

  assert.equal(packageRoot, path.dirname(donorPackageJson));
});

test('resolveReadablePackageRoot can recover a transitive pnpm package from the current app install', () => {
  const pnpmRoot = path.join(adminRoot, 'node_modules', '.pnpm');
  const transitivePackageJson = path.join(
    pnpmRoot,
    'react-router@7.13.1_react@19.2.4',
    'node_modules',
    'react-router',
    'package.json',
  );

  const packageRoot = resolveReadablePackageRoot({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    packageName: 'react-router',
    fileExists(filePath) {
      return filePath === transitivePackageJson || filePath === pnpmRoot;
    },
    isReadable(filePath) {
      return filePath === transitivePackageJson;
    },
    readDir(directoryPath) {
      if (directoryPath !== pnpmRoot) {
        throw new Error(`unexpected directory scan: ${directoryPath}`);
      }

      return [{
        isDirectory() {
          return true;
        },
        name: 'react-router@7.13.1_react@19.2.4',
      }];
    },
  });

  assert.equal(
    packageRoot,
    path.dirname(transitivePackageJson),
  );
});

test('findReadableModuleResolution falls back to a donor app when the current app resolution is unreadable', () => {
  const currentResolvedPath = path.join(adminRoot, 'node_modules', 'lucide-react', 'dist', 'index.js');
  const donorResolvedPath = path.join(portalRoot, 'node_modules', 'lucide-react', 'dist', 'index.js');

  const resolution = findReadableModuleResolution({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    specifier: 'lucide-react',
    resolveFromRoot(root, specifier) {
      if (root === adminRoot && specifier === 'lucide-react') {
        return currentResolvedPath;
      }

      if (root === portalRoot && specifier === 'lucide-react') {
        return donorResolvedPath;
      }

      throw new Error(`unexpected resolution request for ${root}: ${specifier}`);
    },
    isReadable(filePath) {
      return filePath === donorResolvedPath;
    },
  });

  assert.deepEqual(resolution, {
    candidateRoot: portalRoot,
    resolvedPath: donorResolvedPath,
  });
});

test('findReadableModuleResolution falls back to a donor app when the current app cannot resolve the specifier', () => {
  const donorResolvedPath = path.join(portalRoot, 'node_modules', 'zustand', 'index.js');

  const resolution = findReadableModuleResolution({
    appRoot: adminRoot,
    donorRoots: [portalRoot],
    specifier: 'zustand',
    resolveFromRoot(root, specifier) {
      if (root === adminRoot && specifier === 'zustand') {
        const error = new Error('cannot find module');
        error.code = 'MODULE_NOT_FOUND';
        throw error;
      }

      if (root === portalRoot && specifier === 'zustand') {
        return donorResolvedPath;
      }

      throw new Error(`unexpected resolution request for ${root}: ${specifier}`);
    },
    isReadable(filePath) {
      return filePath === donorResolvedPath;
    },
  });

  assert.deepEqual(resolution, {
    candidateRoot: portalRoot,
    resolvedPath: donorResolvedPath,
  });
});

test('probeReadableFile returns false when open fails even if the path exists', () => {
  assert.equal(probeReadableFile('/workspace/apps/admin/node_modules/vite/bin/vite.js', {
    fileExists() {
      return true;
    },
    openFile() {
      const error = new Error('operation not permitted');
      error.code = 'EPERM';
      throw error;
    },
    closeFile() {
      throw new Error('close should not be called when open fails');
    },
  }), false);
});

test('probeReadableFile returns true when open succeeds', () => {
  let closedHandle = null;

  assert.equal(probeReadableFile('/workspace/apps/portal/node_modules/vite/bin/vite.js', {
    fileExists() {
      return true;
    },
    openFile() {
      return 42;
    },
    closeFile(handle) {
      closedHandle = handle;
    },
  }), true);

  assert.equal(closedHandle, 42);
});
