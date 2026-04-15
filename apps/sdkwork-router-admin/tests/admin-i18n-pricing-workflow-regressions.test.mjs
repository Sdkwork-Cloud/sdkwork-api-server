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

test('pricing plan and rate workflow copy is localized through the dedicated zh-CN pricing slice', () => {
  const pricingSource = read('packages/sdkwork-router-admin-pricing/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const pricingTranslations = extractMap(i18n, 'ADMIN_ZH_PRICING_TRANSLATIONS');

  const expectedKeys = [
    'A dedicated pricing module keeps settlement-facing pricing governance separate from catalog market prices.',
    'Force lifecycle convergence when due planned versions should become active before the next automatic pricing read.',
    'Plan code',
    'Plan version',
    'Credit unit code',
    'Saving...',
    'Create plan',
    'Update plan',
    'Create new plan',
    'Immediate',
    'Schedule plan',
    'Publish plan',
    'Retire plan',
    'Clone plan',
    'Edit plan',
    'Pricing rate composer',
    'Create commercial pricing rows with explicit charge units, billing methods, rounding, and minimums.',
    'Pricing plan',
    'No pricing plan available',
    'Metric code',
    'Capability code',
    'Model code',
    'Provider code',
    'Quantity step',
    'Unit price',
    'Display unit',
    'Minimum billable quantity',
    'Minimum charge',
    'Rounding',
    'Rounding increment',
    'Included quantity',
    'Priority',
    'Create pricing rate',
    'Update rate',
    'Create new rate',
    'Edit rate',
  ];

  for (const key of expectedKeys) {
    assert.match(
      pricingSource,
      buildTranslationUsagePattern(key),
      `expected pricing workflow to render ${key} through t(...)`,
    );
    assert.ok(
      pricingTranslations.has(key),
      `expected pricing translation key ${key}`,
    );
    assert.notEqual(
      pricingTranslations.get(key),
      key,
      `expected pricing translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_PRICING_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_PRICING_TRANSLATIONS,/);
});
