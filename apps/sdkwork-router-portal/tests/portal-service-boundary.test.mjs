import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const packagesRoot = path.join(appRoot, 'packages');

function walkFiles(dir, output = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'dist' || entry.name === 'tests') {
        continue;
      }
      walkFiles(fullPath, output);
      continue;
    }

    if (/src[\\/]+services[\\/].+\.(ts|tsx)$/.test(fullPath)) {
      output.push(fullPath);
    }
  }

  return output;
}

test('portal service-layer modules avoid the TSX-heavy commons root export', () => {
  const files = walkFiles(packagesRoot);
  const offenders = [];

  for (const file of files) {
    const source = readFileSync(file, 'utf8');
    if (source.includes("from 'sdkwork-router-portal-commons'")) {
      offenders.push(path.relative(appRoot, file));
    }
  }

  assert.deepEqual(offenders, []);
});
