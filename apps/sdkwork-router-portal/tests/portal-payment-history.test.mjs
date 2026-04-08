import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal billing exposes dedicated payment and refund history audit views', () => {
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const billingTypes = read('packages/sdkwork-router-portal-billing/src/types/index.ts');
  const billingServices = read('packages/sdkwork-router-portal-billing/src/services/index.ts');
  const zhMessages = read('packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts');

  assert.match(billingTypes, /export type BillingPaymentHistoryRowKind =/);
  assert.match(billingTypes, /'payment_event'/);
  assert.match(billingTypes, /'refunded_order_state'/);
  assert.match(billingTypes, /export interface BillingPaymentHistoryRow {/);
  assert.match(billingTypes, /payment_history: BillingPaymentHistoryRow\[];/);
  assert.match(billingTypes, /refund_history: BillingPaymentHistoryRow\[];/);
  assert.match(billingTypes, /payment_method_name\?: string \| null;/);
  assert.match(billingTypes, /checkout_reference\?: string \| null;/);
  assert.match(billingTypes, /checkout_session_status\?: PortalCommerceCheckoutSessionStatus \| null;/);

  assert.match(billingServices, /export function buildBillingPaymentHistory\(/);
  assert.match(billingServices, /export function buildBillingRefundHistory\(/);
  assert.match(billingServices, /payment_method_name: source\.selected_payment_method\?\.display_name \?\? null,/);
  assert.match(billingServices, /provider: resolveBillingHistoryProvider\(source\)/);
  assert.match(billingServices, /row_kind: 'refunded_order_state'/);

  assert.match(billingPage, /const \[paymentHistory, setPaymentHistory\] = useState<BillingPaymentHistoryRow\[]>\(\[]\);/);
  assert.match(billingPage, /const \[refundHistory, setRefundHistory\] = useState<BillingPaymentHistoryRow\[]>\(\[]\);/);
  assert.match(billingPage, /setPaymentHistory\(data\.payment_history\);/);
  assert.match(billingPage, /setRefundHistory\(data\.refund_history\);/);
  assert.match(billingPage, /function paymentHistoryRailCell\(/);
  assert.match(billingPage, /function paymentHistoryRowKindLabel\(/);
  assert.match(billingPage, /function paymentEventTypeLabel\(/);
  assert.match(billingPage, /title=\{t\('Payment history'\)\}/);
  assert.match(billingPage, /title=\{t\('Refund history'\)\}/);
  assert.match(billingPage, /Order refund state/);
  assert.match(billingPage, /Payment update reference/);
  assert.match(billingPage, /Processing/);
  assert.doesNotMatch(billingPage, /Provider event/);

  assert.match(zhMessages, /'Payment history':/);
  assert.match(zhMessages, /'Refund history':/);
  assert.match(zhMessages, /'Order refund state':/);
  assert.match(zhMessages, /'Payment update reference':/);
  assert.match(zhMessages, /'Processing':/);
  assert.doesNotMatch(zhMessages, /'Provider event':/);
});
