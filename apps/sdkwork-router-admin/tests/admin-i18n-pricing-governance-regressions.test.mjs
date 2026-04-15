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

test('pricing governance operator copy is overridden by a dedicated zh-CN pricing translation slice', () => {
  const commercialSource = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const pricingSource = read('packages/sdkwork-router-admin-pricing/src/index.tsx');
  const gatewayAccessSource = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');

  assert.match(
    commercialSource,
    /t\('Pricing governance keeps commercial plan activation and metric-rate coverage visible for operator review\.'\)/,
  );
  assert.match(
    pricingSource,
    /t\('Pricing plans, charge units, and billing methods are maintained here for token, image, audio, video, and music APIs\.'\)/,
  );
  assert.match(
    pricingSource,
    /t\('Operators define versioned commercial plans before adding rate rows\.'\)/,
  );
  assert.match(
    pricingSource,
    /t\('Create a pricing plan before maintaining pricing rates\.'\)/,
  );
  assert.match(
    gatewayAccessSource,
    /t\('Pricing plans and rates define the commercial surface that gateway access policies must honor\.'\)/,
  );

  assert.match(i18n, /const ADMIN_ZH_PRICING_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(
    i18n,
    translationEntry(
      'Pricing governance keeps commercial plan activation and metric-rate coverage visible for operator review.',
      '\\u5b9a\\u4ef7\\u6cbb\\u7406\\u4f1a\\u6301\\u7eed\\u5c55\\u793a\\u5546\\u4e1a\\u5957\\u9910\\u542f\\u7528\\u548c\\u6309\\u6307\\u6807\\u8d39\\u7387\\u8986\\u76d6\\u60c5\\u51b5\\uff0c\\u4fbf\\u4e8e\\u8fd0\\u8425\\u590d\\u6838\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Pricing plans, charge units, and billing methods are maintained here for token, image, audio, video, and music APIs.',
      '\\u6b64\\u5904\\u7ef4\\u62a4 Token\\u3001\\u56fe\\u50cf\\u3001\\u97f3\\u9891\\u3001\\u89c6\\u9891\\u548c\\u97f3\\u4e50 API \\u7684\\u5b9a\\u4ef7\\u8ba1\\u5212\\u3001\\u8ba1\\u8d39\\u5355\\u4f4d\\u4e0e\\u7ed3\\u7b97\\u65b9\\u5f0f\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Operators define versioned commercial plans before adding rate rows.',
      '\\u8fd0\\u8425\\u9700\\u5148\\u5b9a\\u4e49\\u5e26\\u7248\\u672c\\u7684\\u5546\\u4e1a\\u5957\\u9910\\uff0c\\u518d\\u65b0\\u589e\\u8d39\\u7387\\u884c\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Create a pricing plan before maintaining pricing rates.',
      '\\u8bf7\\u5148\\u521b\\u5efa\\u5b9a\\u4ef7\\u8ba1\\u5212\\uff0c\\u518d\\u7ef4\\u62a4\\u5b9a\\u4ef7\\u8d39\\u7387\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Pricing plans and rates define the commercial surface that gateway access policies must honor.',
      '\\u5b9a\\u4ef7\\u8ba1\\u5212\\u548c\\u8d39\\u7387\\u5171\\u540c\\u5b9a\\u4e49\\u4e86\\u7f51\\u5173\\u8bbf\\u95ee\\u7b56\\u7565\\u5fc5\\u987b\\u9075\\u5faa\\u7684\\u5546\\u4e1a\\u9762\\u89c4\\u3002',
    ),
  );
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_PRICING_TRANSLATIONS,/);
});
