import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('resolveInitialLocale synchronizes the global admin locale before provider effects run', () => {
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.match(i18n, /function resolveInitialLocale\(\): AdminLocale \{[\s\S]*activeAdminLocale = locale;[\s\S]*return locale;/);
});
