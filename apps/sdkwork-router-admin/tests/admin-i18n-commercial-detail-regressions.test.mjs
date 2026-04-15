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

test('commercial ledger and audit detail labels are overridden by a dedicated zh-CN detail slice', () => {
  const commercialSource = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const commercialDetailTranslations = extractMap(
    i18n,
    'ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS',
  );

  const expectedKeys = [
    'Pricing governance',
    'Account',
    'Account #{id}',
    'Request',
    'Request #{id}',
    'Hold #{id}',
    'Order',
    'Order #{id}',
    'Investigation',
    'View order audit',
    'Entry',
    'Credits',
    'Settlement',
    'Retail charge',
    'Refund credits',
    'Provider cost',
    'Event',
    'Processing',
    'Refund state',
    'Target kind',
    'List price',
    'Payable price',
    'Granted units',
    'Bonus units',
    'Order status after',
    'Payment event id',
    'Dedupe key',
    'No linked request',
    'No linked hold',
    'No provider event id',
    'No derived order status',
    'No payment evidence',
    'Pending evidence',
    'Unlinked',
    'Loading',
    'n/a',
    'Retail charge: {amount}',
    'Provider cost: {amount}',
    'Captured credits: {count}',
    'Commercial holds that still need capture, release, expiry, or operator intervention.',
    'Settlements already converted into captured commercial liability evidence.',
    'Rejected or failed provider callbacks stay visible before they drift into silent payment reconciliation gaps.',
  ];

  for (const key of expectedKeys) {
    assert.match(
      commercialSource,
      buildTranslationUsagePattern(key),
      `expected commercial module to render ${key} through t(...)`,
    );
    assert.ok(
      commercialDetailTranslations.has(key),
      `expected dedicated commercial detail translation key ${key}`,
    );
    assert.notEqual(
      commercialDetailTranslations.get(key),
      key,
      `expected commercial detail translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS,/);
});
