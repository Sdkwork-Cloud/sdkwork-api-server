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

test('commercial settlement and refund operator copy is overridden by a dedicated zh-CN billing translation slice', () => {
  const source = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');

  assert.match(source, /t\('Settlement explorer'\)/);
  assert.match(
    source,
    /t\('Settlement explorer highlights open holds, captured requests, and correction posture from canonical settlement records\.'\)/,
  );
  assert.match(source, /t\('Settlement ledger'\)/);
  assert.match(
    source,
    /t\('Settlement ledger keeps capture and refund entries linked to request settlements so operators can audit credits, retail charge, and final correction posture without leaving the commercial module\.'\)/,
  );
  assert.match(
    source,
    /t\('Settlement ledger entries will appear here once commercial account history begins landing for the selected control-plane slice\.'\)/,
  );
  assert.match(source, /t\('No settlement ledger entries yet'\)/);
  assert.match(source, /t\('Refund timeline'\)/);
  assert.match(
    source,
    /t\('Refund timeline isolates correction entries so support and finance can verify credited quantity, linked request, and refund cost posture at a glance\.'\)/,
  );
  assert.match(
    source,
    /t\('Refund activity will appear here once commercial refunds are posted into the account ledger history\.'\)/,
  );
  assert.match(source, /t\('No refunds recorded yet'\)/);
  assert.match(source, /t\('Order payment audit'\)/);
  assert.match(
    source,
    /t\('Order payment audit keeps recent commercial orders linked to payment callbacks, provider evidence, and operator-visible processing posture without loading unbounded order history into the commercial module\.'\)/,
  );
  assert.match(
    source,
    /t\('Recent commerce orders will appear here once checkout, webhook, and settlement evidence starts landing in the commercial audit stream\.'\)/,
  );
  assert.match(source, /t\('Order refund audit'\)/);
  assert.match(
    source,
    /t\('Order refund audit keeps explicit refund callbacks and refunded-order fallback evidence visible so operators can spot missing callback closure before it becomes a reconciliation blind spot\.'\)/,
  );
  assert.match(
    source,
    /t\('Refund audit rows will appear here once commercial orders begin entering explicit refund or refunded-order-state correction flows\.'\)/,
  );
  assert.match(source, /t\('No refund evidence yet'\)/);
  assert.match(source, /t\('Latest settlements'\)/);
  assert.match(source, /t\('No settlement evidence yet'\)/);
  assert.match(
    source,
    /t\('Latest settlements will appear here once request settlement records start landing from the canonical commercial kernel\.'\)/,
  );
  assert.match(source, /t\('Payment evidence timeline'\)/);
  assert.match(
    source,
    /t\('Provider callbacks remain ordered here so operators can verify settlement, rejection, and refund sequencing for the selected order\.'\)/,
  );
  assert.match(source, /t\('No payment evidence has been recorded for this order yet\.'\)/);
  assert.match(source, /t\('Open holds'\)/);
  assert.match(source, /t\('Captured settlements'\)/);
  assert.match(source, /t\('Refunded settlements'\)/);
  assert.match(source, /t\('Rejected callbacks'\)/);
  assert.match(source, /t\('Refund posture keeps correction flows visible inside the settlement explorer\.'\)/);

  assert.match(i18n, /const ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18n, translationEntry('Settlement explorer', '\\u7ed3\\u7b97\\u5206\\u6790'));
  assert.match(
    i18n,
    translationEntry(
      'Settlement explorer highlights open holds, captured requests, and correction posture from canonical settlement records.',
      '\\u7ed3\\u7b97\\u5206\\u6790\\u4f1a\\u57fa\\u4e8e\\u89c4\\u8303\\u5316\\u7ed3\\u7b97\\u8bb0\\u5f55\\u7a81\\u51fa\\u663e\\u793a\\u672a\\u5b8c\\u6210\\u51bb\\u7ed3\\u3001\\u5df2\\u6355\\u83b7\\u8bf7\\u6c42\\u548c\\u7ea0\\u504f\\u72b6\\u6001\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Settlement ledger', '\\u7ed3\\u7b97\\u53f0\\u8d26'));
  assert.match(
    i18n,
    translationEntry(
      'Settlement ledger keeps capture and refund entries linked to request settlements so operators can audit credits, retail charge, and final correction posture without leaving the commercial module.',
      '\\u7ed3\\u7b97\\u53f0\\u8d26\\u4f1a\\u5c06\\u6355\\u83b7\\u548c\\u9000\\u6b3e\\u5206\\u5f55\\u4e0e\\u8bf7\\u6c42\\u7ed3\\u7b97\\u5173\\u8054\\u8d77\\u6765\\uff0c\\u4fbf\\u4e8e\\u8fd0\\u8425\\u5728\\u5546\\u4e1a\\u6a21\\u5757\\u5185\\u5ba1\\u8ba1\\u989d\\u5ea6\\u3001\\u96f6\\u552e\\u6536\\u8d39\\u548c\\u6700\\u7ec8\\u7ea0\\u504f\\u72b6\\u6001\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Settlement ledger entries will appear here once commercial account history begins landing for the selected control-plane slice.',
      '\\u6240\\u9009\\u63a7\\u5236\\u9762\\u8303\\u56f4\\u5f00\\u59cb\\u5199\\u5165\\u5546\\u4e1a\\u8d26\\u6237\\u5386\\u53f2\\u540e\\uff0c\\u7ed3\\u7b97\\u53f0\\u8d26\\u5206\\u5f55\\u4f1a\\u663e\\u793a\\u5728\\u8fd9\\u91cc\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry('No settlement ledger entries yet', '\\u6682\\u65e0\\u7ed3\\u7b97\\u53f0\\u8d26\\u5206\\u5f55'),
  );
  assert.match(i18n, translationEntry('Refund timeline', '\\u9000\\u6b3e\\u65f6\\u95f4\\u7ebf'));
  assert.match(
    i18n,
    translationEntry(
      'Refund timeline isolates correction entries so support and finance can verify credited quantity, linked request, and refund cost posture at a glance.',
      '\\u9000\\u6b3e\\u65f6\\u95f4\\u7ebf\\u4f1a\\u9694\\u79bb\\u5c55\\u793a\\u7ea0\\u504f\\u5206\\u5f55\\uff0c\\u4fbf\\u4e8e\\u652f\\u6301\\u548c\\u8d22\\u52a1\\u5feb\\u901f\\u6838\\u5bf9\\u5165\\u8d26\\u6570\\u91cf\\u3001\\u5173\\u8054\\u8bf7\\u6c42\\u548c\\u9000\\u6b3e\\u6210\\u672c\\u72b6\\u6001\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Refund activity will appear here once commercial refunds are posted into the account ledger history.',
      '\\u5546\\u4e1a\\u9000\\u6b3e\\u8fc7\\u8d26\\u5230\\u8d26\\u6237\\u53f0\\u8d26\\u5386\\u53f2\\u540e\\uff0c\\u9000\\u6b3e\\u6d3b\\u52a8\\u4f1a\\u663e\\u793a\\u5728\\u8fd9\\u91cc\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('No refunds recorded yet', '\\u6682\\u65e0\\u9000\\u6b3e\\u8bb0\\u5f55'));
  assert.match(i18n, translationEntry('Order payment audit', '\\u8ba2\\u5355\\u652f\\u4ed8\\u5ba1\\u8ba1'));
  assert.match(
    i18n,
    translationEntry(
      'Order payment audit keeps recent commercial orders linked to payment callbacks, provider evidence, and operator-visible processing posture without loading unbounded order history into the commercial module.',
      '\\u8ba2\\u5355\\u652f\\u4ed8\\u5ba1\\u8ba1\\u4f1a\\u5c06\\u8fd1\\u671f\\u5546\\u4e1a\\u8ba2\\u5355\\u4e0e\\u652f\\u4ed8\\u56de\\u8c03\\u3001\\u4f9b\\u5e94\\u5546\\u8bc1\\u636e\\u4ee5\\u53ca\\u9762\\u5411\\u8fd0\\u8425\\u7684\\u5904\\u7406\\u72b6\\u6001\\u5173\\u8054\\u5c55\\u793a\\uff0c\\u65e0\\u9700\\u5728\\u5546\\u4e1a\\u6a21\\u5757\\u4e2d\\u52a0\\u8f7d\\u65e0\\u754c\\u8ba2\\u5355\\u5386\\u53f2\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Recent commerce orders will appear here once checkout, webhook, and settlement evidence starts landing in the commercial audit stream.',
      '\\u7ed3\\u8d26\\u3001Webhook \\u548c\\u7ed3\\u7b97\\u8bc1\\u636e\\u5f00\\u59cb\\u5199\\u5165\\u5546\\u4e1a\\u5ba1\\u8ba1\\u6d41\\u540e\\uff0c\\u8fd1\\u671f\\u5546\\u4e1a\\u8ba2\\u5355\\u4f1a\\u663e\\u793a\\u5728\\u8fd9\\u91cc\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Order refund audit', '\\u8ba2\\u5355\\u9000\\u6b3e\\u5ba1\\u8ba1'));
  assert.match(
    i18n,
    translationEntry(
      'Order refund audit keeps explicit refund callbacks and refunded-order fallback evidence visible so operators can spot missing callback closure before it becomes a reconciliation blind spot.',
      '\\u8ba2\\u5355\\u9000\\u6b3e\\u5ba1\\u8ba1\\u4f1a\\u6301\\u7eed\\u663e\\u793a\\u660e\\u786e\\u7684\\u9000\\u6b3e\\u56de\\u8c03\\u548c\\u9000\\u6b3e\\u8ba2\\u5355\\u515c\\u5e95\\u8bc1\\u636e\\uff0c\\u4fbf\\u4e8e\\u8fd0\\u8425\\u5728\\u5b83\\u6f14\\u53d8\\u4e3a\\u5bf9\\u8d26\\u76f2\\u70b9\\u524d\\u53d1\\u73b0\\u7f3a\\u5931\\u7684\\u56de\\u8c03\\u95ed\\u73af\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry(
      'Refund audit rows will appear here once commercial orders begin entering explicit refund or refunded-order-state correction flows.',
      '\\u5546\\u4e1a\\u8ba2\\u5355\\u5f00\\u59cb\\u8fdb\\u5165\\u663e\\u5f0f\\u9000\\u6b3e\\u6216\\u9000\\u6b3e\\u8ba2\\u5355\\u72b6\\u6001\\u7ea0\\u504f\\u6d41\\u7a0b\\u540e\\uff0c\\u9000\\u6b3e\\u5ba1\\u8ba1\\u8bb0\\u5f55\\u4f1a\\u663e\\u793a\\u5728\\u8fd9\\u91cc\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('No refund evidence yet', '\\u6682\\u65e0\\u9000\\u6b3e\\u8bc1\\u636e'));
  assert.match(i18n, translationEntry('Latest settlements', '\\u6700\\u65b0\\u7ed3\\u7b97'));
  assert.match(i18n, translationEntry('No settlement evidence yet', '\\u6682\\u65e0\\u7ed3\\u7b97\\u8bc1\\u636e'));
  assert.match(
    i18n,
    translationEntry(
      'Latest settlements will appear here once request settlement records start landing from the canonical commercial kernel.',
      '\\u89c4\\u8303\\u5316\\u5546\\u4e1a\\u5185\\u6838\\u5f00\\u59cb\\u5199\\u5165\\u8bf7\\u6c42\\u7ed3\\u7b97\\u8bb0\\u5f55\\u540e\\uff0c\\u6700\\u65b0\\u7ed3\\u7b97\\u4f1a\\u663e\\u793a\\u5728\\u8fd9\\u91cc\\u3002',
    ),
  );
  assert.match(i18n, translationEntry('Payment evidence timeline', '\\u652f\\u4ed8\\u8bc1\\u636e\\u65f6\\u95f4\\u7ebf'));
  assert.match(
    i18n,
    translationEntry(
      'Provider callbacks remain ordered here so operators can verify settlement, rejection, and refund sequencing for the selected order.',
      '\\u4f9b\\u5e94\\u5546\\u56de\\u8c03\\u4f1a\\u5728\\u8fd9\\u91cc\\u6309\\u987a\\u5e8f\\u4fdd\\u7559\\uff0c\\u4fbf\\u4e8e\\u8fd0\\u8425\\u6838\\u5bf9\\u6240\\u9009\\u8ba2\\u5355\\u7684\\u7ed3\\u7b97\\u3001\\u62d2\\u7edd\\u548c\\u9000\\u6b3e\\u65f6\\u5e8f\\u3002',
    ),
  );
  assert.match(
    i18n,
    translationEntry('No payment evidence has been recorded for this order yet.', '\\u8be5\\u8ba2\\u5355\\u5c1a\\u672a\\u8bb0\\u5f55\\u652f\\u4ed8\\u8bc1\\u636e\\u3002'),
  );
  assert.match(i18n, translationEntry('Open holds', '\\u672a\\u5b8c\\u6210\\u51bb\\u7ed3'));
  assert.match(i18n, translationEntry('Captured settlements', '\\u5df2\\u6355\\u83b7\\u7ed3\\u7b97'));
  assert.match(i18n, translationEntry('Refunded settlements', '\\u5df2\\u9000\\u6b3e\\u7ed3\\u7b97'));
  assert.match(i18n, translationEntry('Rejected callbacks', '\\u5df2\\u62d2\\u7edd\\u56de\\u8c03'));
  assert.match(
    i18n,
    translationEntry(
      'Refund posture keeps correction flows visible inside the settlement explorer.',
      '\\u9000\\u6b3e\\u72b6\\u6001\\u8ba9\\u7ea0\\u504f\\u6d41\\u7a0b\\u5728\\u7ed3\\u7b97\\u5206\\u6790\\u4e2d\\u4fdd\\u6301\\u53ef\\u89c1\\u3002',
    ),
  );
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS,/);
});
