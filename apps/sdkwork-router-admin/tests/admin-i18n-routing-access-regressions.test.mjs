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

test('routing and access policy-group copy is overridden by a dedicated zh-CN routing translation slice', () => {
  const routesPageSource = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx');
  const routingSnapshotsSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingSnapshotsDialog.tsx',
  );
  const groupsDialogSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyGroupsDialog.tsx',
  );
  const routingProfilesSource = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');

  assert.match(
    routesPageSource,
    /t\('API key groups currently bound to reusable routing profiles\.'\)/,
  );
  assert.match(
    routingSnapshotsSource,
    /t\('API key groups currently bound to a reusable routing profile\.'\)/,
  );
  assert.match(
    groupsDialogSource,
    /t\('Define reusable policy groups for workspace-scoped key issuance, routing posture, and accounting defaults\.'\)/,
  );
  assert.match(
    groupsDialogSource,
    /t\('Define the defaults that each bound API key should inherit from this group policy\.'\)/,
  );
  assert.match(
    routingProfilesSource,
    /t\('Capture reusable routing posture so API key groups and workspace policy can bind to a named profile instead of repeating provider order, latency, and health rules\.'\)/,
  );
  assert.match(
    routingProfilesSource,
    /t\('Create route providers first, then return to build a reusable routing profile\.'\)/,
  );

  assert.match(i18n, /const ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(
    i18n,
    translationEntry(
      'API key groups currently bound to reusable routing profiles.',
      '\\u5f53\\u524d\\u5df2\\u7ed1\\u5b9a\\u5230\\u53ef\\u590d\\u7528\\u8def\\u7531\\u914d\\u7f6e\\u7684 API \\u5bc6\\u94a5\\u7ec4\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'API key groups currently bound to a reusable routing profile.',
      '\\u5f53\\u524d\\u5df2\\u7ed1\\u5b9a\\u5230\\u53ef\\u590d\\u7528\\u8def\\u7531\\u914d\\u7f6e\\u7684 API \\u5bc6\\u94a5\\u7ec4\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Define reusable policy groups for workspace-scoped key issuance, routing posture, and accounting defaults.',
      '\\u5b9a\\u4e49\\u53ef\\u590d\\u7528\\u7684\\u7b56\\u7565\\u7ec4\\uff0c\\u7528\\u4e8e\\u7ba1\\u7406\\u5de5\\u4f5c\\u533a\\u8303\\u56f4\\u5185\\u7684\\u5bc6\\u94a5\\u7b7e\\u53d1\\u3001\\u8def\\u7531\\u6001\\u52bf\\u548c\\u8bb0\\u8d26\\u9ed8\\u8ba4\\u503c\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Define the defaults that each bound API key should inherit from this group policy.',
      '\\u5b9a\\u4e49\\u6bcf\\u4e2a\\u7ed1\\u5b9a API \\u5bc6\\u94a5\\u5e94\\u4ece\\u8be5\\u7b56\\u7565\\u7ec4\\u7ee7\\u627f\\u7684\\u9ed8\\u8ba4\\u8bbe\\u7f6e\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Capture reusable routing posture so API key groups and workspace policy can bind to a named profile instead of repeating provider order, latency, and health rules.',
      '\\u6c89\\u6dc0\\u53ef\\u590d\\u7528\\u7684\\u8def\\u7531\\u7b56\\u7565\\u6001\\u52bf\\uff0c\\u8ba9 API \\u5bc6\\u94a5\\u7ec4\\u548c\\u5de5\\u4f5c\\u533a\\u7b56\\u7565\\u53ef\\u4ee5\\u7ed1\\u5b9a\\u547d\\u540d\\u914d\\u7f6e\\uff0c\\u65e0\\u9700\\u91cd\\u590d\\u7ef4\\u62a4\\u4f9b\\u5e94\\u5546\\u987a\\u5e8f\\u3001\\u65f6\\u5ef6\\u548c\\u5065\\u5eb7\\u89c4\\u5219\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Create route providers first, then return to build a reusable routing profile.',
      '\\u8bf7\\u5148\\u521b\\u5efa\\u8def\\u7531\\u4f9b\\u5e94\\u5546\\uff0c\\u518d\\u56de\\u6765\\u6784\\u5efa\\u53ef\\u590d\\u7528\\u7684\\u8def\\u7531\\u914d\\u7f6e\\u3002',
    ),
  );
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS,/);
});
