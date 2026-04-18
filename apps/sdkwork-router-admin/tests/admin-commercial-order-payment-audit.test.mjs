import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadOrderPaymentAuditModule() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-commercial',
      'src',
      'orderPaymentAudit.ts',
    ),
  );
}

test('admin commercial order payment audit keeps refunded order-state fallback and recent event ordering', () => {
  const {
    buildCommercialOrderPaymentAuditRows,
    buildCommercialRefundAuditRows,
  } = loadOrderPaymentAuditModule();

  const orders = [
    {
      order_id: 'order-1',
      project_id: 'project-a',
      user_id: 'user-a',
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      target_name: 'Boost 100k',
      list_price_cents: 4000,
      payable_price_cents: 3200,
      list_price_label: '$40.00',
      payable_price_label: '$32.00',
      granted_units: 100000,
      bonus_units: 0,
      applied_coupon_code: 'SPRING20',
      coupon_reservation_id: null,
      coupon_redemption_id: null,
      marketing_campaign_id: null,
      subsidy_amount_minor: 800,
      status: 'refunded',
      source: 'live',
      created_at_ms: 100,
      updated_at_ms: 300,
    },
    {
      order_id: 'order-2',
      project_id: 'project-b',
      user_id: 'user-b',
      target_kind: 'subscription_plan',
      target_id: 'growth',
      target_name: 'Growth',
      list_price_cents: 7900,
      payable_price_cents: 7900,
      list_price_label: '$79.00',
      payable_price_label: '$79.00',
      granted_units: 100000,
      bonus_units: 0,
      applied_coupon_code: null,
      coupon_reservation_id: null,
      coupon_redemption_id: null,
      marketing_campaign_id: null,
      subsidy_amount_minor: 0,
      status: 'pending_payment',
      source: 'live',
      created_at_ms: 190,
      updated_at_ms: 200,
    },
  ];

  const paymentEvents = [
    {
      payment_event_id: 'payevt-1-settled',
      order_id: 'order-1',
      project_id: 'project-a',
      user_id: 'user-a',
      provider: 'stripe',
      provider_event_id: 'evt_stripe_1',
      dedupe_key: 'stripe:evt_stripe_1',
      event_type: 'settled',
      payload_json: '{"event_type":"settled"}',
      processing_status: 'processed',
      processing_message: null,
      received_at_ms: 150,
      processed_at_ms: 160,
      order_status_after: 'fulfilled',
    },
  ];

  const rows = buildCommercialOrderPaymentAuditRows(orders, paymentEvents);
  const refundRows = buildCommercialRefundAuditRows(rows);

  assert.equal(rows.length, 3);
  assert.equal(rows[0].row_kind, 'refunded_order_state');
  assert.equal(rows[0].order_id, 'order-1');
  assert.equal(rows[0].event_type, 'refunded');
  assert.equal(rows[0].provider, 'stripe');
  assert.equal(rows[1].row_kind, 'order_state');
  assert.equal(rows[1].order_id, 'order-2');
  assert.equal(rows[1].event_type, null);
  assert.equal(rows[2].row_kind, 'payment_event');
  assert.equal(rows[2].payment_event_id, 'payevt-1-settled');
  assert.equal(rows[2].provider_event_id, 'evt_stripe_1');

  assert.equal(refundRows.length, 1);
  assert.equal(refundRows[0].row_kind, 'refunded_order_state');
  assert.equal(refundRows[0].order_id, 'order-1');
});
