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

test('apirouter access detail and routing evidence copy are overridden by a dedicated zh-CN detail slice', () => {
  const sources = [
    read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailDrawer.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailPanel.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesDetailPanel.tsx'),
  ];
  const joinedSources = sources.join('\n');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const apirouterDetailTranslations = extractMap(
    i18n,
    'ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS',
  );

  const expectedKeys = [
    'Suspended',
    'Manage groups',
    'Commercial governance',
    'Credit holds, settlement capture, and liability posture stay visible while governing API access.',
    'Operators can confirm that API key issuance is mapped onto live commercial account inventory.',
    'Group policy',
    'API key lifecycle, route posture, and bootstrap workflows stay attached to the selected registry row.',
    'Group defaults and inherited posture bound to this key.',
    'API key group',
    'No group assigned',
    'No routing profile',
    'No group',
    'Direct key policy',
    'Compiled snapshots currently loaded from the routing evidence layer.',
    'Snapshots carrying an applied routing profile id.',
    'Review how routing profiles compile into route-key and capability evidence before changing provider posture.',
    '{snapshots} snapshots',
    'No compiled routing evidence is available yet.',
    'Routing impact',
    'Inspect how the selected provider participates in compiled snapshots, reusable routing profiles, and default-route posture.',
    'Top affected routing profiles',
    '{count} snapshots',
    '{count} groups',
    'No compiled routing evidence currently references this provider through a reusable routing profile.',
    'Routing evidence is empty',
    'Recent compiled snapshots',
    'Fallback path',
  ];

  for (const key of expectedKeys) {
    assert.match(
      joinedSources,
      buildTranslationUsagePattern(key),
      `expected apirouter detail surface to render ${key} through t(...)`,
    );
    assert.ok(
      apirouterDetailTranslations.has(key),
      `expected dedicated apirouter detail translation key ${key}`,
    );
    assert.notEqual(
      apirouterDetailTranslations.get(key),
      key,
      `expected apirouter detail translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS,/);
});
