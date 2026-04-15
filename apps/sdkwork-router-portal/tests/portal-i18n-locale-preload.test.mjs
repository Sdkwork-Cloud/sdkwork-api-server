import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal i18n runtime lazy-loads zh-CN catalogs instead of wiring them synchronously into startup state', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const core = read('packages/sdkwork-router-portal-commons/src/i18n-core.ts');

  assert.match(commons, /export async function preloadPortalLocale\(/);
  assert.match(commons, /export async function preloadPreferredPortalLocale\(/);
  assert.match(core, /await import\('\.\/portalMessages\.zh-CN'\)/);
  assert.doesNotMatch(commons, /from '\.\/portalMessages\.zh-CN'/);
  assert.doesNotMatch(core, /from '\.\/portalMessages\.zh-CN'/);
  assert.match(core, /runtimePortalMessagesByLocale/);
});

test('portal entry preloads the preferred locale before the React tree mounts', () => {
  const main = read('src/main.tsx');

  assert.match(main, /preloadPreferredPortalLocale/);
  assert.match(main, /await preloadPreferredPortalLocale\(\)/);
});

test('portal initial locale sync updates global helper locales before provider effects run', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(
    commons,
    /function resolveInitialLocale\(\): PortalLocale \{[\s\S]*activePortalLocale = locale;[\s\S]*setActivePortalCoreLocale\(locale\);[\s\S]*setActivePortalFormatLocale\(locale\);[\s\S]*return locale;/,
  );
});
