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

test('commercial workspace summary and order-audit shell copy are overridden by a dedicated zh-CN surface slice', () => {
  const commercialSource = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const commercialSurfaceTranslations = extractMap(i18n, 'ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS');

  const expectedKeys = [
    'Commercial control plane',
    'Operators can audit commercial accounts, request settlement posture, and pricing governance without leaving a dedicated module.',
    'Captured, released, and refunded request settlements ready for operator investigation.',
    'Recent commerce orders stay linked to provider callbacks and operator-visible payment evidence.',
    'Live metric-rate rows currently shaping canonical commercial charging.',
    'Active pricing plans',
    'Commercial pricing plans that are active and currently effective in the control plane.',
    'Priced metrics',
    'Distinct metric codes already governed by canonical pricing rates.',
    'Primary plan',
    'No active plan',
    'The first active pricing plan remains the quickest operator reference point.',
    'Charge unit',
    'Primary metered unit keeps settlement granularity explicit for operator review.',
    'Billing method',
    'Settlement method shows whether the primary rate charges per unit, flat, or step-based.',
    'Display unit makes the commercial rate readable for token and multimodal pricing review.',
    'No order payment evidence yet',
    'The right rail keeps the most recent commercial settlement evidence in view for rapid operator triage.',
    'Order audit detail',
    'Loading selected order',
    'Loading order audit evidence',
    'Order audit detail unavailable',
    'Commercial order, checkout, and coupon evidence stay bundled here so operators can reconstruct fulfillment and refund posture without switching modules.',
    'Order audit detail keeps payment callbacks and coupon lifecycle evidence scoped to the selected order so reconciliation triage stays deterministic.',
  ];

  for (const key of expectedKeys) {
    assert.match(
      commercialSource,
      buildTranslationUsagePattern(key),
      `expected commercial module to render ${key} through t(...)`,
    );
    assert.ok(
      commercialSurfaceTranslations.has(key),
      `expected dedicated commercial surface translation key ${key}`,
    );
    assert.notEqual(
      commercialSurfaceTranslations.get(key),
      key,
      `expected commercial surface translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS,/);
});
