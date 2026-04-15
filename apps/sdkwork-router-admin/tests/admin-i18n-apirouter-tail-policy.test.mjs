import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function extractMap(source, name) {
  const start = source.indexOf(`const ${name}: Record<string, string> = {`);
  assert.notEqual(start, -1, `missing map ${name}`);

  const open = source.indexOf('{', start);
  const close = source.indexOf('\n};', open);
  assert.notEqual(close, -1, `missing closing brace for ${name}`);

  const body = source.slice(open + 1, close);
  return new Map(
    [...body.matchAll(/\n\s*"([^"]+)":\s*(?:"([^"]*)"|\n\s*"([^"]*)"),/g)].map((match) => [
      match[1],
      match[2] ?? match[3] ?? '',
    ]),
  );
}

function buildTranslationUsagePattern(key) {
  const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`t\\(\\s*'${escapedKey}'\\s*(?:,|\\))`, 's');
}

test('apirouter rate-limit labels localize ui copy while protocol literals remain raw examples', () => {
  const gatewayRateLimitsSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRateLimitsPage.tsx',
  );
  const gatewayRoutesSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx',
  );
  const apiKeyCreateDialogSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyCreateDialog.tsx',
  );
  const apiKeyUsageDialogSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyUsageDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const apirouterSurfaceTranslations = extractMap(i18n, 'ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS');

  const localizedUiKeys = [
    'Window',
    'Policies',
    'Live windows',
    'Manage routing profiles',
    'Snapshot evidence',
  ];

  for (const key of localizedUiKeys) {
    assert.match(
      `${gatewayRateLimitsSource}\n${gatewayRoutesSource}`,
      buildTranslationUsagePattern(key),
      `expected apirouter ui to render ${key} through t(...)`,
    );
    assert.ok(
      apirouterSurfaceTranslations.has(key),
      `expected apirouter surface translation key ${key}`,
    );
    assert.notEqual(
      apirouterSurfaceTranslations.get(key),
      key,
      `expected apirouter surface translation ${key} to be localized instead of English`,
    );
  }

  assert.match(
    apiKeyCreateDialogSource,
    buildTranslationUsagePattern('sk-router-live-demo'),
    'expected sample key placeholder to flow through t(...)',
  );
  assert.match(
    i18n,
    /'sk-router-live-demo': 'sk-router-live-demo'/,
    'expected sample key placeholder to remain a raw example literal',
  );

  assert.match(
    apiKeyUsageDialogSource,
    buildTranslationUsagePattern('Authorization: Bearer {token}'),
    'expected authorization header example to flow through t(...)',
  );
  assert.match(
    i18n,
    /'Authorization: Bearer \{token\}': 'Authorization: Bearer \{token\}'/,
    'expected authorization header example to remain a protocol literal',
  );

  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS,/);
});
