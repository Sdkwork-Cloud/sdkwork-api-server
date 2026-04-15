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

test('commercial account posture copy is overridden by a dedicated zh-CN account translation slice', () => {
  const source = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');

  assert.match(source, /t\('Commercial accounts'\)/);
  assert.match(
    source,
    /t\('Canonical payable accounts currently discoverable by the commercial control plane\.'\)/,
  );
  assert.match(source, /t\('Available balance'\)/);
  assert.match(
    source,
    /t\('Spendable credit still available across the commercial account inventory\.'\)/,
  );
  assert.match(source, /t\('Active accounts'\)/);
  assert.match(
    source,
    /t\('Accounts currently able to receive holds and settlement capture\.'\)/,
  );
  assert.match(source, /t\('Suspended accounts'\)/);
  assert.match(
    source,
    /t\('Accounts blocked from new commercial admission until operator review\.'\)/,
  );
  assert.match(source, /t\('Held balance'\)/);
  assert.match(
    source,
    /t\('Credit currently reserved by request admission and pending settlement flows\.'\)/,
  );
  assert.match(
    source,
    /t\('Commercial accounts, settlement explorer, and pricing governance now live as a first-class admin module\.'\)/,
  );
  assert.match(
    source,
    /t\('Account posture keeps status, held balance, and admission readiness visible in one surface\.'\)/,
  );

  assert.match(i18n, /const ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18n, translationEntry('Commercial accounts', '\\u5546\\u4e1a\\u8d26\\u6237'));
  assert.match(
    i18n,
    translationEntry(
      'Canonical payable accounts currently discoverable by the commercial control plane.',
      '\\u5f53\\u524d\\u5546\\u4e1a\\u63a7\\u5236\\u9762\\u53ef\\u8bc6\\u522b\\u7684\\u89c4\\u8303\\u5316\\u5e94\\u4ed8\\u8d26\\u6237\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Available balance', '\\u53ef\\u7528\\u4f59\\u989d'));
  assert.match(
    i18n,
    translationEntry(
      'Spendable credit still available across the commercial account inventory.',
      '\\u5f53\\u524d\\u5546\\u4e1a\\u8d26\\u6237\\u6e05\\u5355\\u4e2d\\u4ecd\\u53ef\\u4f7f\\u7528\\u7684\\u4fe1\\u7528\\u989d\\u5ea6\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Active accounts', '\\u6d3b\\u8dc3\\u8d26\\u6237'));
  assert.match(
    i18n,
    translationEntry(
      'Accounts currently able to receive holds and settlement capture.',
      '\\u5f53\\u524d\\u53ef\\u63a5\\u6536\\u51bb\\u7ed3\\u548c\\u7ed3\\u7b97\\u6355\\u83b7\\u7684\\u8d26\\u6237\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Suspended accounts', '\\u5df2\\u6682\\u505c\\u8d26\\u6237'));
  assert.match(
    i18n,
    translationEntry(
      'Accounts blocked from new commercial admission until operator review.',
      '\\u5728\\u8fd0\\u8425\\u590d\\u6838\\u524d\\u7981\\u6b62\\u65b0\\u7684\\u5546\\u4e1a\\u51c6\\u5165\\u7684\\u8d26\\u6237\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Held balance', '\\u51bb\\u7ed3\\u4f59\\u989d'));
  assert.match(
    i18n,
    translationEntry(
      'Credit currently reserved by request admission and pending settlement flows.',
      '\\u5f53\\u524d\\u88ab\\u8bf7\\u6c42\\u51c6\\u5165\\u548c\\u5f85\\u7ed3\\u7b97\\u6d41\\u7a0b\\u9884\\u7559\\u7684\\u4fe1\\u7528\\u989d\\u5ea6\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Commercial accounts, settlement explorer, and pricing governance now live as a first-class admin module.',
      '\\u5546\\u4e1a\\u8d26\\u6237\\u3001\\u7ed3\\u7b97\\u5206\\u6790\\u548c\\u5b9a\\u4ef7\\u6cbb\\u7406\\u73b0\\u5df2\\u6210\\u4e3a\\u4e00\\u7ea7\\u7ba1\\u7406\\u6a21\\u5757\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Account posture keeps status, held balance, and admission readiness visible in one surface.',
      '\\u8d26\\u6237\\u6001\\u52bf\\u4f1a\\u5728\\u540c\\u4e00\\u89c6\\u56fe\\u4e2d\\u5c55\\u793a\\u72b6\\u6001\\u3001\\u51bb\\u7ed3\\u4f59\\u989d\\u548c\\u51c6\\u5165\\u5c31\\u7eea\\u5ea6\\u3002',
    ),
  );
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS,/);
});
