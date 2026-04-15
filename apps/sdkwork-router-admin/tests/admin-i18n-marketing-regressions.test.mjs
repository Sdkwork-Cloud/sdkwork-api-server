import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function escapeForRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function translationEntry(key, value) {
  return new RegExp(`"${escapeForRegex(key)}":\\s*"${escapeForRegex(value)}"`);
}

test('commercial coupon order-audit copy is overridden by a dedicated zh-CN marketing translation slice', () => {
  const source = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');

  assert.match(source, /t\('Payment, coupon, and campaign evidence is being loaded for the selected order\.'\)/);
  assert.match(source, /t\('No coupon applied'\)/);
  assert.match(source, /t\('Coupon evidence chain'\)/);
  assert.match(
    source,
    /t\('Reservation, redemption, rollback, code, template, and campaign evidence stays attached so discount posture can be audited together with payment callbacks\.'\)/,
  );
  assert.match(source, /t\('Reservation'\)/);
  assert.match(source, /t\('Redemption'\)/);
  assert.match(source, /t\('Rollback count'\)/);
  assert.match(source, /t\('No reservation evidence'\)/);
  assert.match(source, /t\('No redemption evidence'\)/);
  assert.match(source, /t\('No coupon code evidence'\)/);
  assert.match(source, /t\('Coupon template'\)/);
  assert.match(source, /t\('No template evidence'\)/);
  assert.match(source, /t\('Marketing campaign'\)/);
  assert.match(source, /t\('No campaign evidence'\)/);
  assert.match(source, /t\('Coupon rollback timeline'\)/);
  assert.match(
    source,
    /t\('Rollback evidence confirms whether coupon subsidy and inventory were restored during refund handling\.'\)/,
  );
  assert.match(source, /t\('Restored budget'\)/);
  assert.match(source, /t\('Restored inventory'\)/);
  assert.match(source, /t\('No coupon rollback evidence has been recorded for this order\.'\)/);

  assert.match(i18n, /const ADMIN_ZH_MARKETING_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(
    i18n,
    translationEntry(
      'Payment, coupon, and campaign evidence is being loaded for the selected order.',
      '\\u6b63\\u5728\\u4e3a\\u6240\\u9009\\u8ba2\\u5355\\u52a0\\u8f7d\\u652f\\u4ed8\\u3001\\u4f18\\u60e0\\u5238\\u548c\\u6d3b\\u52a8\\u8bc1\\u636e\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry('No coupon applied', '\\u672a\\u4f7f\\u7528\\u4f18\\u60e0\\u5238'),
  );
  assert.match(
    i18n,
    translationEntry('Coupon evidence chain', '\\u4f18\\u60e0\\u5238\\u8bc1\\u636e\\u94fe'),
  );
  assert.match(
    i18n,
    translationEntry(
      'Reservation, redemption, rollback, code, template, and campaign evidence stays attached so discount posture can be audited together with payment callbacks.',
      '\\u9884\\u7559\\u3001\\u5151\\u6362\\u3001\\u56de\\u6eda\\u3001\\u5238\\u7801\\u3001\\u6a21\\u677f\\u548c\\u6d3b\\u52a8\\u8bc1\\u636e\\u4f1a\\u7edf\\u4e00\\u4fdd\\u7559\\uff0c\\u4fbf\\u4e8e\\u7ed3\\u5408\\u652f\\u4ed8\\u56de\\u8c03\\u4e00\\u8d77\\u5ba1\\u8ba1\\u4f18\\u60e0\\u72b6\\u6001\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Reservation', '\\u9884\\u7559'));
  assert.match(i18n, translationEntry('Redemption', '\\u5151\\u6362'));
  assert.match(i18n, translationEntry('Rollback count', '\\u56de\\u6eda\\u6b21\\u6570'));
  assert.match(
    i18n,
    translationEntry('No reservation evidence', '\\u6682\\u65e0\\u9884\\u7559\\u8bc1\\u636e'),
  );
  assert.match(
    i18n,
    translationEntry('No redemption evidence', '\\u6682\\u65e0\\u5151\\u6362\\u8bc1\\u636e'),
  );
  assert.match(
    i18n,
    translationEntry('No coupon code evidence', '\\u6682\\u65e0\\u4f18\\u60e0\\u7801\\u8bc1\\u636e'),
  );
  assert.match(
    i18n,
    translationEntry('Coupon template', '\\u4f18\\u60e0\\u5238\\u6a21\\u677f'),
  );
  assert.match(
    i18n,
    translationEntry('No template evidence', '\\u6682\\u65e0\\u6a21\\u677f\\u8bc1\\u636e'),
  );
  assert.match(
    i18n,
    translationEntry('Marketing campaign', '\\u8425\\u9500\\u6d3b\\u52a8'),
  );
  assert.match(
    i18n,
    translationEntry('No campaign evidence', '\\u6682\\u65e0\\u6d3b\\u52a8\\u8bc1\\u636e'),
  );
  assert.match(
    i18n,
    translationEntry('Coupon rollback timeline', '\\u4f18\\u60e0\\u5238\\u56de\\u6eda\\u65f6\\u95f4\\u7ebf'),
  );
  assert.match(
    i18n,
    translationEntry(
      'Rollback evidence confirms whether coupon subsidy and inventory were restored during refund handling.',
      '\\u56de\\u6eda\\u8bc1\\u636e\\u7528\\u4e8e\\u786e\\u8ba4\\u9000\\u6b3e\\u5904\\u7406\\u4e2d\\u662f\\u5426\\u5df2\\u6062\\u590d\\u4f18\\u60e0\\u8865\\u8d34\\u548c\\u5e93\\u5b58\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Restored budget', '\\u6062\\u590d\\u9884\\u7b97'));
  assert.match(i18n, translationEntry('Restored inventory', '\\u6062\\u590d\\u5e93\\u5b58'));
  assert.match(
    i18n,
    translationEntry(
      'No coupon rollback evidence has been recorded for this order.',
      '\\u8be5\\u8ba2\\u5355\\u5c1a\\u672a\\u8bb0\\u5f55\\u4f18\\u60e0\\u5238\\u56de\\u6eda\\u8bc1\\u636e\\u3002',
    ),
  );
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_MARKETING_TRANSLATIONS,/);
});
