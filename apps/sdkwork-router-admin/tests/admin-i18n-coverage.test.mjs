import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const packagesRoot = path.join(appRoot, 'packages');
const commonsPath = path.join(packagesRoot, 'sdkwork-router-admin-commons', 'src', 'index.tsx');

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

    if (/\.(ts|tsx)$/.test(entry.name)) {
      output.push(fullPath);
    }
  }

  return output;
}

test('admin shared dictionary covers static user-facing i18n keys across product packages', () => {
  const commons = readFileSync(commonsPath, 'utf8');
  const files = walkFiles(packagesRoot);
  const patterns = [/\bt\('([^']+)'/g, /translateAdminText\('([^']+)'/g];
  const missing = new Map();

  for (const file of files) {
    const source = readFileSync(file, 'utf8');
    for (const pattern of patterns) {
      let match;
      while ((match = pattern.exec(source))) {
        const key = match[1];
        if (!commons.includes(`'${key}'`) && !commons.includes(`${key}: '`)) {
          if (!missing.has(key)) {
            missing.set(key, []);
          }
          missing.get(key).push(path.relative(appRoot, file));
        }
      }
    }
  }

  assert.deepEqual(
    [...missing.entries()].map(([key, refs]) => ({ key, refs })),
    [],
  );
});
