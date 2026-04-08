#!/usr/bin/env node

import process from 'node:process';
import { pathToFileURL } from 'node:url';

import {
  applyWindowsVitePreload,
  resolveReadablePackageEntry,
  resolveWorkspaceDonorRoots,
} from './vite-runtime-lib.mjs';

const appRoot = process.cwd();
const donorRoots = resolveWorkspaceDonorRoots(appRoot);
const viteCliPath = resolveReadablePackageEntry({
  appRoot,
  donorRoots,
  packageName: 'vite',
  relativeEntry: ['bin', 'vite.js'],
});

await applyWindowsVitePreload();

process.argv = [
  process.argv[0],
  viteCliPath,
  ...process.argv.slice(2),
];

await import(pathToFileURL(viteCliPath).href);
