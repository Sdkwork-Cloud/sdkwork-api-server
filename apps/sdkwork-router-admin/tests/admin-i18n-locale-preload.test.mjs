import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin i18n runtime lazy-loads locale catalogs instead of wiring the zh-CN table synchronously into startup state', () => {
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.match(i18n, /export async function preloadAdminLocale\(/);
  assert.match(i18n, /await import\('\.\/i18nTranslations'\)/);
  assert.doesNotMatch(i18n, /from '\.\/i18nTranslations'/);
  assert.match(i18n, /runtimeTranslationsByLocale/);
  assert.match(i18n, /return runtimeTranslationsByLocale\[locale\]\[text\] \?\? text;/);
});

test('admin shell bootstrap preloads the preferred locale before the React tree mounts', () => {
  const bootstrap = read(
    'packages/sdkwork-router-admin-shell/src/application/bootstrap/bootstrapShellRuntime.ts',
  );

  assert.match(bootstrap, /preloadPreferredAdminLocale/);
  assert.match(bootstrap, /await preloadPreferredAdminLocale\(\)/);
});
