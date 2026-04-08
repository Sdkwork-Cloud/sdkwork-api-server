#!/usr/bin/env node

import process from 'node:process';
import { pathToFileURL } from 'node:url';

import {
  resolveReadablePackageEntry,
  resolveWorkspaceDonorRoots,
} from './vite-runtime-lib.mjs';

export function resolveReadableTypeScriptCliPath({
  appRoot,
  donorRoots = resolveWorkspaceDonorRoots(appRoot),
} = {}) {
  if (!appRoot) {
    throw new Error('appRoot is required');
  }

  return resolveReadablePackageEntry({
    appRoot,
    donorRoots,
    packageName: 'typescript',
    relativeEntry: ['lib', 'tsc.js'],
  });
}

const appRoot = process.cwd();
const tscCliPath = resolveReadableTypeScriptCliPath({ appRoot });

process.argv = [
  process.argv[0],
  tscCliPath,
  ...process.argv.slice(2),
];

await import(pathToFileURL(tscCliPath).href);
