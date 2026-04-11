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

test('coupon governance shell and detail copy is localized through the dedicated zh-CN marketing slice', () => {
  const couponsSource = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const detailSource = read('packages/sdkwork-router-admin-coupons/src/page/CouponsDetailPanel.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const marketingTranslations = extractMap(i18n, 'ADMIN_ZH_MARKETING_TRANSLATIONS');

  const couponOverviewKeys = [
    'Template governance',
    '{count} active templates',
    'Campaign budgets',
    '{count} active campaigns',
    'Code vault',
    '{count} total codes',
    'Redemption ledger',
    '{count} tracked redemptions',
    'Rollback trail',
    '{count} recorded rollbacks',
  ];
  const couponGovernanceKeys = [
    'Activate budget',
    'Activate campaign',
    'Activate template',
    'Archive template',
    'Budget status',
    'Campaign status',
    'Close budget',
    'Code locked by lifecycle',
    'Code status',
    'Disable code',
    'Enable code',
    'Governance controls',
    'No budget linked',
    'No campaign linked',
    'No code linked',
    'No template linked',
    'Pause campaign',
    'Template status',
    'Template, campaign, budget, and code status controls let operators stop risk exposure or restore offers without editing the whole record.',
    'missing',
  ];

  for (const key of couponOverviewKeys) {
    assert.match(
      couponsSource,
      buildTranslationUsagePattern(key),
      `expected coupons overview to render ${key} through t(...)`,
    );
    assert.ok(
      marketingTranslations.has(key),
      `expected marketing translation key ${key}`,
    );
    assert.notEqual(
      marketingTranslations.get(key),
      key,
      `expected marketing translation ${key} to be localized instead of English`,
    );
  }

  for (const key of couponGovernanceKeys) {
    assert.match(
      detailSource,
      buildTranslationUsagePattern(key),
      `expected coupon governance detail to render ${key} through t(...)`,
    );
    assert.ok(
      marketingTranslations.has(key),
      `expected marketing translation key ${key}`,
    );
    assert.notEqual(
      marketingTranslations.get(key),
      key,
      `expected marketing translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_MARKETING_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18n, /\.\.\.ADMIN_ZH_MARKETING_TRANSLATIONS,/);
});
