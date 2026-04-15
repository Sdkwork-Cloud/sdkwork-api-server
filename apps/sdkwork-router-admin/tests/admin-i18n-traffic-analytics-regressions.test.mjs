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

test('traffic analytics shell copy is localized through the dedicated zh-CN traffic slice', () => {
  const trafficSource = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const trafficTranslations = extractMap(i18n, 'ADMIN_ZH_TRAFFIC_TRANSLATIONS');

  const expectedKeys = [
    'Billing events stay aligned with quota posture and remaining project headroom.',
    'Billing events summarize project chargeback, request volume, and quota posture in one view.',
    'Billing-event analytics stay visible across all traffic lenses.',
    'No accounting-mode mix is visible for this slice.',
    'No billing events match the current filters',
    'No capability mix is visible for this slice.',
    'No fallback used',
    'No group chargeback data is visible for this slice.',
    'Not captured',
    'Platform credit, BYOK, and passthrough mix remain visible.',
    'Provider selection, fallback evidence, compiled snapshots, and SLO posture remain visible for every routing decision.',
    'Top billed capabilities in the active time slice.',
    'Try a broader query to inspect more billing events.',
    'Upstream cost',
  ];

  for (const key of expectedKeys) {
    assert.match(
      trafficSource,
      buildTranslationUsagePattern(key),
      `expected traffic analytics to render ${key} through t(...)`,
    );
    assert.ok(
      trafficTranslations.has(key),
      `expected traffic translation key ${key}`,
    );
    assert.notEqual(
      trafficTranslations.get(key),
      key,
      `expected traffic translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_TRAFFIC_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_TRAFFIC_TRANSLATIONS,/);
});
