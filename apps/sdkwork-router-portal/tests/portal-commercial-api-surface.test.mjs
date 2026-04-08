import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadPortalApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-portal-api',
      'src',
      'index.ts',
    ),
  );
}

function loadBillingRepository() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-billing',
      'src',
      'repository',
      'index.ts',
    ),
  );
}

function loadBillingServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-billing',
      'src',
      'services',
      'index.ts',
    ),
  );
}

function installPortalApiTestEnvironment(responseMap = {}) {
  const requests = [];
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;
  const previousWindow = globalThis.window;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.router.portal.session-token' ? 'portal-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.window = {
    location: {
      origin: 'http://127.0.0.1:3001',
      port: '3001',
    },
  };
  globalThis.fetch = async (input, init) => {
    const url = String(input);
    const rawBody = typeof init?.body === 'string' ? init.body : null;
    requests.push({
      url,
      method: init?.method ?? 'GET',
      authorization: init?.headers?.authorization ?? init?.headers?.Authorization ?? null,
      body: rawBody ? JSON.parse(rawBody) : null,
    });

    const payload = responseMap[url] ?? {};
    return {
      ok: true,
      status: 200,
      async json() {
        return payload;
      },
    };
  };

  return {
    requests,
    restore() {
      globalThis.fetch = previousFetch;
      globalThis.localStorage = previousLocalStorage;
      globalThis.window = previousWindow;
    },
  };
}

test('portal commercial api client exposes canonical commercial billing methods', async () => {
  const portalApi = loadPortalApi();
  const env = installPortalApiTestEnvironment();

  try {
    await portalApi.getPortalCommercialAccount();
    await portalApi.getPortalCommercialAccountHistory();
    await portalApi.getPortalCommercialAccountBalance();
    await portalApi.listPortalCommercialBenefitLots();
    await portalApi.listPortalCommercialHolds();
    await portalApi.listPortalCommercialRequestSettlements();
    await portalApi.listPortalCommercialPricingPlans();
    await portalApi.listPortalCommercialPricingRates();
    await portalApi.getPortalCommerceOrderCenter();

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/billing/account',
        '/api/portal/billing/account-history',
        '/api/portal/billing/account/balance',
        '/api/portal/billing/account/benefit-lots',
        '/api/portal/billing/account/holds',
        '/api/portal/billing/account/request-settlements',
        '/api/portal/billing/pricing-plans',
        '/api/portal/billing/pricing-rates',
        '/api/portal/commerce/order-center',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(9).fill('Bearer portal-session-token'),
    );
  } finally {
    env.restore();
  }
});

test('portal commercial api client exposes formal commerce order detail and payment detail methods', async () => {
  const portalApi = loadPortalApi();
  const env = installPortalApiTestEnvironment({
    '/api/portal/commerce/orders/order-1': {
      order_id: 'order-1',
    },
    '/api/portal/commerce/orders/order-1/payment-methods': [],
    '/api/portal/commerce/payment-attempts/payatt-1': {
      payment_attempt_id: 'payatt-1',
    },
  });

  try {
    await portalApi.getPortalCommerceOrder('order-1');
    await portalApi.listPortalCommercePaymentMethods('order-1');
    await portalApi.getPortalCommercePaymentAttempt('payatt-1');

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/commerce/orders/order-1',
        '/api/portal/commerce/orders/order-1/payment-methods',
        '/api/portal/commerce/payment-attempts/payatt-1',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(3).fill('Bearer portal-session-token'),
    );
  } finally {
    env.restore();
  }
});

test('portal commercial api client exposes formal payment-attempt mutation methods', async () => {
  const portalApi = loadPortalApi();
  const env = installPortalApiTestEnvironment({
    '/api/portal/commerce/orders/order-1/payment-attempts': [
      {
        payment_attempt_id: 'payatt-1',
      },
    ],
  });

  try {
    await portalApi.listPortalCommercePaymentAttempts('order-1');
    await portalApi.createPortalCommercePaymentAttempt('order-1', {
      payment_method_id: 'pm-stripe',
    });

    assert.deepEqual(
      env.requests.map((request) => ({
        url: request.url,
        method: request.method,
        body: request.body,
      })),
      [
        {
          url: '/api/portal/commerce/orders/order-1/payment-attempts',
          method: 'GET',
          body: null,
        },
        {
          url: '/api/portal/commerce/orders/order-1/payment-attempts',
          method: 'POST',
          body: {
            payment_method_id: 'pm-stripe',
          },
        },
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(2).fill('Bearer portal-session-token'),
    );
  } finally {
    env.restore();
  }
});

test('portal billing repository composes formal commerce detail reads with compatibility aggregates', async () => {
  const billingRepository = loadBillingRepository();
  const env = installPortalApiTestEnvironment({
    '/api/portal/billing/summary': {
      project_id: 'project-demo',
      entry_count: 0,
      used_units: 0,
      booked_amount: 0,
      remaining_units: 0,
      exhausted: false,
    },
    '/api/portal/usage/records': [],
    '/api/portal/billing/events/summary': {
      total_events: 0,
      project_count: 0,
      group_count: 0,
      capability_count: 0,
      total_request_count: 0,
      total_units: 0,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_tokens: 0,
      total_image_count: 0,
      total_audio_seconds: 0,
      total_video_seconds: 0,
      total_music_seconds: 0,
      total_upstream_cost: 0,
      total_customer_charge: 0,
      projects: [],
      groups: [],
      capabilities: [],
      accounting_modes: [],
    },
    '/api/portal/billing/events': [],
    '/api/portal/commerce/catalog': {
      plans: [],
      packs: [],
      recharge_options: [],
      custom_recharge_policy: null,
      coupons: [],
    },
    '/api/portal/commerce/order-center': {
      project_id: 'project-demo',
      payment_simulation_enabled: true,
      membership: {
        membership_id: 'membership-pro',
        project_id: 'project-demo',
        user_id: 'user-demo',
        plan_id: 'pro',
        plan_name: 'Pro',
        price_cents: 9900,
        price_label: '$99.00',
        cadence: 'monthly',
        included_units: 100000,
        status: 'active',
        source: 'workspace_seed',
        activated_at_ms: 1,
        updated_at_ms: 2,
      },
      reconciliation: {
        account_id: 7001,
        last_reconciled_order_id: 'order-0',
        last_reconciled_order_updated_at_ms: 1,
        last_reconciled_order_created_at_ms: 1,
        last_reconciled_at_ms: 1,
        backlog_order_count: 1,
        checkpoint_lag_ms: 1,
        healthy: false,
      },
      orders: [
        {
          order: {
            order_id: 'order-1',
            project_id: 'project-demo',
            user_id: 'user-demo',
            target_kind: 'recharge_pack',
            target_id: 'pack-100k',
            target_name: 'Legacy 100k Pack',
            list_price_cents: 1000,
            payable_price_cents: 1000,
            list_price_label: '$10.00',
            payable_price_label: '$10.00',
            granted_units: 100000,
            bonus_units: 0,
            payment_method_id: 'pm-card',
            latest_payment_attempt_id: 'payatt-1',
            status: 'pending_payment',
            source: 'workspace_seed',
            created_at_ms: 1,
            updated_at_ms: 5,
          },
          payment_events: [
            {
              payment_event_id: 'payevt-order-1-settled',
              order_id: 'order-1',
              project_id: 'project-demo',
              user_id: 'user-demo',
              provider: 'stripe',
              provider_event_id: 'evt_stripe_1',
              dedupe_key: 'stripe_evt_1',
              event_type: 'settled',
              payload_json: '{"amount_cents":1000}',
              processing_status: 'processed',
              processing_message: null,
              received_at_ms: 3,
              processed_at_ms: 4,
              order_status_after: 'fulfilled',
            },
          ],
          latest_payment_event: {
            payment_event_id: 'payevt-order-1-settled',
            order_id: 'order-1',
            project_id: 'project-demo',
            user_id: 'user-demo',
            provider: 'stripe',
            provider_event_id: 'evt_stripe_1',
            dedupe_key: 'stripe_evt_1',
            event_type: 'settled',
            payload_json: '{"amount_cents":1000}',
            processing_status: 'processed',
            processing_message: null,
            received_at_ms: 3,
            processed_at_ms: 4,
            order_status_after: 'fulfilled',
          },
          checkout_session: {
            order_id: 'order-1',
            order_status: 'refunded',
            session_status: 'refunded',
            provider: 'manual_lab',
            mode: 'closed',
            reference: 'PAY-order-legacy',
            payable_price_label: '$10.00',
            guidance: 'refunded',
            methods: [],
          },
        },
      ],
    },
    '/api/portal/commerce/orders/order-1': {
      order_id: 'order-1',
      project_id: 'project-demo',
      user_id: 'user-demo',
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      target_name: '100k Pack canonical',
      list_price_cents: 1000,
      payable_price_cents: 1000,
      list_price_label: '$10.00',
      payable_price_label: '$10.00',
      granted_units: 100000,
      bonus_units: 0,
      payment_method_id: 'pm-card',
      latest_payment_attempt_id: 'payatt-1',
      status: 'refunded',
      settlement_status: 'settled',
      source: 'workspace_seed',
      refundable_amount_minor: 0,
      refunded_amount_minor: 1000,
      created_at_ms: 1,
      updated_at_ms: 6,
    },
    '/api/portal/commerce/orders/order-1/payment-methods': [
      {
        payment_method_id: 'pm-card',
        display_name: 'Primary card',
        description: 'Visa ending 4242',
        provider: 'stripe',
        channel: 'card',
        mode: 'hosted',
        enabled: true,
        sort_order: 1,
        capability_codes: ['checkout'],
        supported_currency_codes: ['USD'],
        supported_country_codes: ['US'],
        supported_order_kinds: ['recharge_pack'],
        callback_strategy: 'webhook',
        webhook_path: '/webhooks/stripe',
        webhook_tolerance_seconds: 300,
        replay_window_seconds: 900,
        max_retry_count: 3,
        config_json: '{}',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    '/api/portal/commerce/payment-attempts/payatt-1': {
      payment_attempt_id: 'payatt-1',
      order_id: 'order-1',
      project_id: 'project-demo',
      user_id: 'user-demo',
      payment_method_id: 'pm-card',
      provider: 'stripe',
      channel: 'card',
      status: 'succeeded',
      idempotency_key: 'idem-order-1-1',
      attempt_sequence: 1,
      amount_minor: 1000,
      currency_code: 'USD',
      captured_amount_minor: 1000,
      refunded_amount_minor: 1000,
      provider_payment_intent_id: 'pi_1',
      provider_checkout_session_id: 'cs_test_1',
      provider_reference: 'pi_1',
      checkout_url: 'https://checkout.stripe.test/session/cs_test_1',
      qr_code_payload: null,
      request_payload_json: '{}',
      response_payload_json: '{}',
      error_code: null,
      error_message: null,
      initiated_at_ms: 2,
      expires_at_ms: null,
      completed_at_ms: 4,
      updated_at_ms: 4,
    },
    '/api/portal/billing/account-history': {
      account: {
        account_id: 7001,
        tenant_id: 1001,
        organization_id: 2002,
        user_id: 9001,
        account_type: 'primary',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        allow_overdraft: false,
        overdraft_limit: 0,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      balance: {
        account_id: 7001,
        available_balance: 150,
        held_balance: 10,
        consumed_balance: 40,
        grant_balance: 240,
        active_lot_count: 1,
        lots: [],
      },
      benefit_lots: [{ lot_id: 8001 }],
      holds: [{ hold_id: 8101 }],
      request_settlements: [{ request_settlement_id: 8301 }],
      ledger: [],
    },
    '/api/portal/billing/pricing-plans': [{ pricing_plan_id: 9101 }],
    '/api/portal/billing/pricing-rates': [{ pricing_rate_id: 9201 }],
  });

  try {
    const result = await billingRepository.loadBillingPageData();

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/billing/summary',
        '/api/portal/usage/records',
        '/api/portal/billing/events/summary',
        '/api/portal/billing/events',
        '/api/portal/commerce/catalog',
        '/api/portal/commerce/order-center',
        '/api/portal/billing/account-history',
        '/api/portal/billing/pricing-plans',
        '/api/portal/billing/pricing-rates',
        '/api/portal/commerce/orders/order-1',
        '/api/portal/commerce/orders/order-1/payment-methods',
        '/api/portal/commerce/payment-attempts/payatt-1',
      ],
    );
    assert.equal(result.orders.length, 1);
    assert.equal(result.orders[0].order_id, 'order-1');
    assert.equal(result.orders[0].target_name, '100k Pack canonical');
    assert.equal(result.orders[0].status, 'refunded');
    assert.equal(result.membership?.membership_id, 'membership-pro');
    assert.equal(result.payment_history.length, 2);
    assert.equal(result.payment_history[0].row_kind, 'refunded_order_state');
    assert.equal(result.payment_history[0].order_id, 'order-1');
    assert.equal(result.payment_history[0].target_name, '100k Pack canonical');
    assert.equal(result.payment_history[0].event_type, 'refunded');
    assert.equal(result.payment_history[0].provider, 'stripe');
    assert.equal(result.payment_history[0].checkout_reference, 'pi_1');
    assert.equal(result.payment_history[0].payment_method_name, 'Primary card');
    assert.equal(result.payment_history[1].row_kind, 'payment_event');
    assert.equal(result.payment_history[1].payment_event_id, 'payevt-order-1-settled');
    assert.equal(result.payment_history[1].provider, 'stripe');
    assert.equal(result.payment_history[1].provider_event_id, 'evt_stripe_1');
    assert.equal(result.payment_history[1].payment_method_name, 'Primary card');
    assert.equal(result.refund_history.length, 1);
    assert.equal(result.refund_history[0].row_kind, 'refunded_order_state');
    assert.equal(result.refund_history[0].event_type, 'refunded');
    assert.equal(result.refund_history[0].checkout_reference, 'pi_1');
    assert.equal(result.commercial_reconciliation?.account_id, 7001);
    assert.equal(result.commercial_reconciliation?.backlog_order_count, 1);
    assert.equal(result.commercial_reconciliation?.healthy, false);
    assert.equal(result.commercial_account.account.account_id, 7001);
    assert.equal(result.commercial_balance.account_id, 7001);
    assert.equal(result.commercial_benefit_lots.length, 1);
    assert.equal(result.commercial_holds.length, 1);
    assert.equal(result.commercial_request_settlements.length, 1);
    assert.equal(result.commercial_pricing_plans.length, 1);
    assert.equal(result.commercial_pricing_rates.length, 1);
  } finally {
    env.restore();
  }
});

test('portal billing repository exposes formal payment-attempt launch for provider checkout methods', async () => {
  const billingRepository = loadBillingRepository();
  const env = installPortalApiTestEnvironment({
    '/api/portal/commerce/orders/order-2/payment-attempts': {
      payment_attempt_id: 'payatt-launch-2',
      order_id: 'order-2',
      project_id: 'project-demo',
      user_id: 'user-demo',
      payment_method_id: 'pm-stripe',
      provider: 'stripe',
      channel: 'hosted_checkout',
      status: 'requires_action',
      idempotency_key: 'idem-order-2-launch',
      attempt_sequence: 2,
      amount_minor: 9900,
      currency_code: 'USD',
      captured_amount_minor: 0,
      refunded_amount_minor: 0,
      provider_payment_intent_id: 'pi_order_2_launch',
      provider_checkout_session_id: 'cs_order_2_launch',
      provider_reference: 'ref_order_2_launch',
      checkout_url: 'https://checkout.stripe.test/session/cs_order_2_launch',
      qr_code_payload: null,
      request_payload_json: '{}',
      response_payload_json: '{}',
      error_code: null,
      error_message: null,
      initiated_at_ms: 20,
      expires_at_ms: null,
      completed_at_ms: null,
      updated_at_ms: 21,
    },
  });

  try {
    const paymentAttempt = await billingRepository.createBillingPaymentAttempt('order-2', {
      payment_method_id: 'pm-stripe',
      success_url: 'http://127.0.0.1:3001/portal/billing?checkout=success',
      cancel_url: 'http://127.0.0.1:3001/portal/billing?checkout=cancel',
    });

    assert.equal(paymentAttempt.payment_attempt_id, 'payatt-launch-2');
    assert.equal(paymentAttempt.checkout_url, 'https://checkout.stripe.test/session/cs_order_2_launch');
    assert.deepEqual(
      env.requests.map((request) => ({
        url: request.url,
        method: request.method,
        body: request.body,
      })),
      [
        {
          url: '/api/portal/commerce/orders/order-2/payment-attempts',
          method: 'POST',
          body: {
            payment_method_id: 'pm-stripe',
            success_url: 'http://127.0.0.1:3001/portal/billing?checkout=success',
            cancel_url: 'http://127.0.0.1:3001/portal/billing?checkout=cancel',
          },
        },
      ],
    );
  } finally {
    env.restore();
  }
});

test('portal billing surface wires payment simulation posture from the aggregate commerce contract', () => {
  const portalTypes = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-types', 'src', 'index.ts'),
    'utf8',
  );
  const billingRepositorySource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'repository', 'index.ts'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(portalTypes, /payment_simulation_enabled: boolean;/);
  assert.match(
    billingRepositorySource,
    /payment_simulation_enabled:\s*order_center\.payment_simulation_enabled/,
  );
  assert.match(billingPageSource, /const \[paymentSimulationEnabled, setPaymentSimulationEnabled\]/);
  assert.match(billingPageSource, /setPaymentSimulationEnabled\(data\.payment_simulation_enabled\)/);
  assert.match(
    billingPageSource,
    /checkoutMethods\.filter\(\(method\) => method\.supports_webhook && paymentSimulationEnabled\)/,
  );
  assert.match(
    billingPageSource,
    /paymentSimulationEnabled \? \(/,
  );
});

test('portal billing page surfaces formal provider checkout launch from payment-attempt flows', () => {
  const portalTypes = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-types', 'src', 'index.ts'),
    'utf8',
  );
  const portalCommonsSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-commons', 'src', 'index.tsx'),
    'utf8',
  );
  const portalApiSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-portal-api', 'src', 'index.ts'),
    'utf8',
  );
  const billingRepositorySource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'repository', 'index.ts'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(portalTypes, /export interface PortalCommercePaymentAttemptCreateRequest {/);
  assert.match(portalApiSource, /export function createPortalCommercePaymentAttempt\(/);
  assert.match(portalApiSource, /export function listPortalCommercePaymentAttempts\(/);
  assert.match(billingRepositorySource, /createBillingPaymentAttempt/);
  assert.match(billingRepositorySource, /listPortalCommercePaymentAttempts/);
  assert.match(billingPageSource, /createBillingPaymentAttempt/);
  assert.match(billingPageSource, /buildBillingCheckoutLaunchDecision/);
  assert.match(billingPageSource, /Start checkout/);
  assert.match(billingPageSource, /Resume checkout/);
  assert.match(billingPageSource, /Opening checkout\.\.\./);
  assert.match(billingPageSource, /Retry with new attempt/);
  assert.match(
    billingPageSource,
    /\{targetName\} now uses the \{provider\} checkout launch path\./,
  );
  assert.match(
    billingPageSource,
    /\{targetName\} created a \{provider\} checkout attempt, but no checkout link was returned\./,
  );
  assert.match(
    portalCommonsSource,
    /'Checkout workbench keeps checkout access, selected reference, and payable price aligned under one payment method\.'/,
  );
  assert.match(
    portalCommonsSource,
    /'No checkout guidance is available for this order yet\.'/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /\{targetName\} now uses the formal \{provider\} checkout launch path\./,
  );
  assert.doesNotMatch(
    billingPageSource,
    /\{targetName\} created a formal \{provider\} checkout attempt, but no checkout URL was returned\./,
  );
  assert.doesNotMatch(
    portalCommonsSource,
    /'Formal checkout keeps checkout access, selected reference, and payable price aligned under one payment method\.'/,
  );
  assert.doesNotMatch(
    portalCommonsSource,
    /'No formal guidance is available for this order yet\.'/,
  );
  assert.doesNotMatch(billingPageSource, /Launch provider checkout/);
  assert.doesNotMatch(billingPageSource, /Resume provider checkout/);
});

test('portal billing services classify provider checkout resume and retry decisions from canonical attempt posture', () => {
  const { buildBillingCheckoutLaunchDecision } = loadBillingServices();

  const checkoutMethod = {
    id: 'pm-stripe',
    label: 'Primary card',
    detail: 'Visa ending 4242',
    action: 'provider_handoff',
    availability: 'available',
    provider: 'stripe',
    channel: 'hosted_checkout',
    session_kind: 'hosted_checkout',
    session_reference: 'pi_order_3_b',
    qr_code_payload: null,
    webhook_verification: 'webhook',
    supports_refund: true,
    supports_partial_refund: false,
    recommended: true,
    supports_webhook: true,
  };

  const reusableDecision = buildBillingCheckoutLaunchDecision({
    checkout_method: checkoutMethod,
    payment_attempts: [
      {
        payment_attempt_id: 'payatt-3-b',
        order_id: 'order-3',
        project_id: 'project-demo',
        user_id: 'user-demo',
        payment_method_id: 'pm-stripe',
        provider: 'stripe',
        channel: 'hosted_checkout',
        status: 'requires_action',
        idempotency_key: 'idem-order-3-b',
        attempt_sequence: 2,
        amount_minor: 9900,
        currency_code: 'USD',
        captured_amount_minor: 0,
        refunded_amount_minor: 0,
        provider_payment_intent_id: 'pi_order_3_b',
        provider_checkout_session_id: 'cs_order_3_b',
        provider_reference: 'pi_order_3_b',
        checkout_url: 'https://checkout.stripe.test/session/cs_order_3_b',
        qr_code_payload: null,
        request_payload_json: '{}',
        response_payload_json: '{}',
        error_code: null,
        error_message: null,
        initiated_at_ms: 40,
        expires_at_ms: null,
        completed_at_ms: null,
        updated_at_ms: 41,
      },
    ],
  });
  assert.equal(reusableDecision.kind, 'resume_existing_attempt');
  assert.equal(reusableDecision.matched_attempt_count, 1);
  assert.equal(reusableDecision.latest_attempt?.payment_attempt_id, 'payatt-3-b');

  const retryDecision = buildBillingCheckoutLaunchDecision({
    checkout_method: checkoutMethod,
    payment_attempts: [
      {
        payment_attempt_id: 'payatt-3-c',
        order_id: 'order-3',
        project_id: 'project-demo',
        user_id: 'user-demo',
        payment_method_id: 'pm-stripe',
        provider: 'stripe',
        channel: 'hosted_checkout',
        status: 'failed',
        idempotency_key: 'idem-order-3-c',
        attempt_sequence: 3,
        amount_minor: 9900,
        currency_code: 'USD',
        captured_amount_minor: 0,
        refunded_amount_minor: 0,
        provider_payment_intent_id: 'pi_order_3_c',
        provider_checkout_session_id: 'cs_order_3_c',
        provider_reference: 'pi_order_3_c',
        checkout_url: null,
        qr_code_payload: null,
        request_payload_json: '{}',
        response_payload_json: '{}',
        error_code: 'card_declined',
        error_message: 'Card declined',
        initiated_at_ms: 50,
        expires_at_ms: null,
        completed_at_ms: 51,
        updated_at_ms: 52,
      },
    ],
  });
  assert.equal(retryDecision.kind, 'create_retry_attempt');
  assert.equal(retryDecision.matched_attempt_count, 1);
  assert.equal(retryDecision.latest_attempt?.payment_attempt_id, 'payatt-3-c');

  const firstAttemptDecision = buildBillingCheckoutLaunchDecision({
    checkout_method: checkoutMethod,
    payment_attempts: [],
  });
  assert.equal(firstAttemptDecision.kind, 'create_first_attempt');
  assert.equal(firstAttemptDecision.matched_attempt_count, 0);
  assert.equal(firstAttemptDecision.latest_attempt, null);
});

test('portal billing services build a formal-first checkout presentation from canonical payment posture', () => {
  const { buildBillingCheckoutPresentation } = loadBillingServices();

  const presentation = buildBillingCheckoutPresentation({
    order: {
      order_id: 'order-4',
      project_id: 'project-demo',
      user_id: 'user-demo',
      target_kind: 'subscription_plan',
      target_id: 'plan-pro',
      target_name: 'Pro monthly',
      list_price_cents: 9900,
      payable_price_cents: 9900,
      list_price_label: '$99.00',
      payable_price_label: '$99.00',
      granted_units: 100000,
      bonus_units: 0,
      payment_method_id: 'pm-stripe',
      latest_payment_attempt_id: 'payatt-4-b',
      status: 'pending_payment',
      source: 'workspace_seed',
      created_at_ms: 1,
      updated_at_ms: 2,
    },
    checkout_session: {
      order_id: 'order-4',
      order_status: 'pending_payment',
      session_status: 'open',
      provider: 'manual_lab',
      mode: 'operator_settlement',
      reference: 'PAY-order-4-legacy',
      payable_price_label: '$99.00',
      guidance: 'Use the compatibility settlement rail.',
      payment_simulation_enabled: true,
      methods: [
        {
          id: 'manual_settlement',
          label: 'Manual settlement',
          detail: 'Use the portal settle action in local or lab mode.',
          action: 'settle_order',
          availability: 'available',
          provider: 'manual_lab',
          channel: 'operator_settlement',
          session_kind: 'operator_action',
          session_reference: 'MANUAL-ORDER-4',
          qr_code_payload: null,
          webhook_verification: 'manual',
          supports_refund: true,
          supports_partial_refund: false,
          recommended: false,
          supports_webhook: false,
        },
      ],
    },
    checkout_methods: [
      {
        id: 'manual_settlement',
        label: 'Manual settlement',
        detail: 'Use the portal settle action in local or lab mode.',
        action: 'settle_order',
        availability: 'available',
        provider: 'manual_lab',
        channel: 'operator_settlement',
        session_kind: 'operator_action',
        session_reference: 'MANUAL-ORDER-4',
        qr_code_payload: null,
        webhook_verification: 'manual',
        supports_refund: true,
        supports_partial_refund: false,
        recommended: false,
        supports_webhook: false,
      },
      {
        id: 'pm-stripe',
        label: 'Primary card',
        detail: 'Visa ending 4242',
        action: 'provider_handoff',
        availability: 'available',
        provider: 'stripe',
        channel: 'hosted_checkout',
        session_kind: 'hosted_checkout',
        session_reference: 'pi_order_4_b',
        qr_code_payload: null,
        webhook_verification: 'webhook',
        supports_refund: true,
        supports_partial_refund: false,
        recommended: true,
        supports_webhook: true,
      },
    ],
    payment_attempts: [
      {
        payment_attempt_id: 'payatt-4-b',
        order_id: 'order-4',
        project_id: 'project-demo',
        user_id: 'user-demo',
        payment_method_id: 'pm-stripe',
        provider: 'stripe',
        channel: 'hosted_checkout',
        status: 'requires_action',
        idempotency_key: 'idem-order-4-b',
        attempt_sequence: 2,
        amount_minor: 9900,
        currency_code: 'USD',
        captured_amount_minor: 0,
        refunded_amount_minor: 0,
        provider_payment_intent_id: 'pi_order_4_b',
        provider_checkout_session_id: 'cs_order_4_b',
        provider_reference: 'pi_order_4_b',
        checkout_url: 'https://checkout.stripe.test/session/cs_order_4_b',
        qr_code_payload: null,
        request_payload_json: '{}',
        response_payload_json: '{}',
        error_code: null,
        error_message: null,
        initiated_at_ms: 10,
        expires_at_ms: null,
        completed_at_ms: null,
        updated_at_ms: 11,
      },
    ],
    payment_methods: [
      {
        payment_method_id: 'pm-stripe',
        display_name: 'Primary card',
        description: 'Visa ending 4242',
        provider: 'stripe',
        channel: 'card',
        mode: 'hosted',
        enabled: true,
        sort_order: 1,
        capability_codes: ['checkout'],
        supported_currency_codes: ['USD'],
        supported_country_codes: ['US'],
        supported_order_kinds: ['subscription_plan'],
        callback_strategy: 'webhook',
        webhook_path: '/webhooks/stripe',
        webhook_tolerance_seconds: 300,
        replay_window_seconds: 900,
        max_retry_count: 3,
        config_json: '{}',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    latest_payment_attempt: {
      payment_attempt_id: 'payatt-4-b',
      order_id: 'order-4',
      project_id: 'project-demo',
      user_id: 'user-demo',
      payment_method_id: 'pm-stripe',
      provider: 'stripe',
      channel: 'hosted_checkout',
      status: 'requires_action',
      idempotency_key: 'idem-order-4-b',
      attempt_sequence: 2,
      amount_minor: 9900,
      currency_code: 'USD',
      captured_amount_minor: 0,
      refunded_amount_minor: 0,
      provider_payment_intent_id: 'pi_order_4_b',
      provider_checkout_session_id: 'cs_order_4_b',
      provider_reference: 'pi_order_4_b',
      checkout_url: 'https://checkout.stripe.test/session/cs_order_4_b',
      qr_code_payload: null,
      request_payload_json: '{}',
      response_payload_json: '{}',
      error_code: null,
      error_message: null,
      initiated_at_ms: 10,
      expires_at_ms: null,
      completed_at_ms: null,
      updated_at_ms: 11,
    },
    selected_payment_method: {
      payment_method_id: 'pm-stripe',
      display_name: 'Primary card',
      description: 'Visa ending 4242',
      provider: 'stripe',
      channel: 'card',
      mode: 'hosted',
      enabled: true,
      sort_order: 1,
      capability_codes: ['checkout'],
      supported_currency_codes: ['USD'],
      supported_country_codes: ['US'],
      supported_order_kinds: ['subscription_plan'],
      callback_strategy: 'webhook',
      webhook_path: '/webhooks/stripe',
      webhook_tolerance_seconds: 300,
      replay_window_seconds: 900,
      max_retry_count: 3,
      config_json: '{}',
      created_at_ms: 1,
      updated_at_ms: 2,
    },
  });

  assert.equal(presentation.reference, 'pi_order_4_b');
  assert.equal(presentation.payable_price_label, '$99.00');
  assert.equal(presentation.payment_method_name, 'Primary card');
  assert.equal(presentation.provider, 'stripe');
  assert.equal(presentation.channel, 'hosted_checkout');
  assert.equal(presentation.status_source, 'payment_attempt');
  assert.equal(presentation.status, 'requires_action');
  assert.equal(presentation.guidance_source, 'launch_decision');
  assert.equal(presentation.launch_decision_kind, 'resume_existing_attempt');
  assert.equal(presentation.launch_method?.id, 'pm-stripe');
});

test('portal billing checkout detail normalizes canonical checkout methods ahead of compatibility checkout-session methods', () => {
  const billingRepositorySource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'repository', 'index.ts'),
    'utf8',
  );
  const billingServicesSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'services', 'index.ts'),
    'utf8',
  );
  const billingTypesSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'types', 'index.ts'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(billingServicesSource, /export function buildBillingCheckoutMethods\(/);
  assert.match(billingServicesSource, /export function buildBillingCheckoutPresentation\(/);
  assert.match(billingRepositorySource, /buildBillingCheckoutMethods/);
  assert.match(billingTypesSource, /checkout_methods: PortalCommerceCheckoutSessionMethod\[];/);
  assert.match(billingPageSource, /buildBillingCheckoutPresentation/);
  assert.match(billingPageSource, /checkoutPresentation\?\.reference/);
  assert.match(billingPageSource, /checkoutPresentation\?\.provider/);
  assert.match(billingPageSource, /checkoutPresentationGuidanceText/);
  assert.match(
    billingPageSource,
    /const checkoutMethods = checkoutDetail\?\.checkout_methods \?\? checkoutSession\?\.methods \?\? \[];/,
  );
  assert.match(
    billingPageSource,
    /const visibleCheckoutMethods = checkoutMethods\.filter\(\(method\) => paymentSimulationEnabled \|\| method\.action !== 'settle_order'\);/,
  );
  assert.match(
    billingPageSource,
    /const activeCheckoutOrder = checkoutDetail\?\.order \?\? null;/,
  );
  assert.match(
    billingPageSource,
    /visibleCheckoutMethods\.map\(\(method\) =>/,
  );
  assert.match(
    billingPageSource,
    /method\.action === 'settle_order' && paymentSimulationEnabled && activeCheckoutOrder/,
  );
  assert.match(
    billingPageSource,
    /handleQueueAction\(activeCheckoutOrder, 'settle'\)/,
  );
  assert.match(
    billingPageSource,
    /method\.action === 'cancel_order' && activeCheckoutOrder/,
  );
  assert.match(
    billingPageSource,
    /handleQueueAction\(activeCheckoutOrder, 'cancel'\)/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /checkoutSession\?\.reference \?\? t\('Awaiting pending order'\)/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /checkoutSession\.methods\.map\(\(method\) => \(/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /checkoutMethods\.map\(\(method\) =>/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /handleQueueAction\(row, 'settle'\)/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /handleQueueAction\(row, 'cancel'\)/,
  );
});

test('portal billing repository composes formal checkout detail reads for pending orders', async () => {
  const billingRepository = loadBillingRepository();
  const env = installPortalApiTestEnvironment({
    '/api/portal/commerce/orders/order-2': {
      order_id: 'order-2',
      project_id: 'project-demo',
      user_id: 'user-demo',
      target_kind: 'subscription_plan',
      target_id: 'plan-pro',
      target_name: 'Pro monthly',
      list_price_cents: 9900,
      payable_price_cents: 9900,
      list_price_label: '$99.00',
      payable_price_label: '$99.00',
      granted_units: 100000,
      bonus_units: 0,
      payment_method_id: 'pm-stripe',
      latest_payment_attempt_id: 'payatt-2',
      status: 'pending_payment',
      source: 'workspace_seed',
      created_at_ms: 10,
      updated_at_ms: 11,
    },
    '/api/portal/commerce/orders/order-2/payment-methods': [
      {
        payment_method_id: 'pm-stripe',
        display_name: 'Primary card',
        description: 'Visa ending 4242',
        provider: 'stripe',
        channel: 'card',
        mode: 'hosted',
        enabled: true,
        sort_order: 1,
        capability_codes: ['checkout'],
        supported_currency_codes: ['USD'],
        supported_country_codes: ['US'],
        supported_order_kinds: ['subscription_plan'],
        callback_strategy: 'webhook',
        webhook_path: '/webhooks/stripe',
        webhook_tolerance_seconds: 300,
        replay_window_seconds: 900,
        max_retry_count: 3,
        config_json: '{}',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    '/api/portal/commerce/orders/order-2/payment-attempts': [
      {
        payment_attempt_id: 'payatt-2',
        order_id: 'order-2',
        project_id: 'project-demo',
        user_id: 'user-demo',
        payment_method_id: 'pm-stripe',
        provider: 'stripe',
        channel: 'card',
        status: 'pending',
        idempotency_key: 'idem-order-2-1',
        attempt_sequence: 1,
        amount_minor: 9900,
        currency_code: 'USD',
        captured_amount_minor: 0,
        refunded_amount_minor: 0,
        provider_payment_intent_id: 'pi_order_2',
        provider_checkout_session_id: 'cs_order_2',
        provider_reference: 'pi_order_2',
        checkout_url: 'https://checkout.stripe.test/session/cs_order_2',
        qr_code_payload: null,
        request_payload_json: '{}',
        response_payload_json: '{}',
        error_code: null,
        error_message: null,
        initiated_at_ms: 10,
        expires_at_ms: null,
        completed_at_ms: null,
        updated_at_ms: 11,
      },
    ],
    '/api/portal/commerce/orders/order-2/checkout-session': {
      order_id: 'order-2',
      order_status: 'pending_payment',
      session_status: 'open',
      provider: 'manual_lab',
      mode: 'operator_settlement',
      reference: 'PAY-order-2-legacy',
      payable_price_label: '$99.00',
      guidance: 'Use the compatibility settlement rail.',
      payment_simulation_enabled: true,
      methods: [
        {
          id: 'manual_settlement',
          label: 'Manual settlement',
          detail: 'Use the portal settle action in local or lab mode.',
          action: 'settle_order',
          availability: 'available',
          provider: 'manual_lab',
          channel: 'operator_settlement',
          session_kind: 'operator_action',
          session_reference: 'MANUAL-ORDER-2',
          qr_code_payload: null,
          webhook_verification: 'manual',
          supports_refund: true,
          supports_partial_refund: false,
          recommended: false,
          supports_webhook: false,
        },
        {
          id: 'cancel_order',
          label: 'Cancel checkout',
          detail: 'Close the pending order without applying quota.',
          action: 'cancel_order',
          availability: 'available',
          provider: 'manual_lab',
          channel: 'operator_settlement',
          session_kind: 'operator_action',
          session_reference: 'CANCEL-ORDER-2',
          qr_code_payload: null,
          webhook_verification: 'manual',
          supports_refund: false,
          supports_partial_refund: false,
          recommended: false,
          supports_webhook: false,
        },
      ],
    },
  });

  try {
    const result = await billingRepository.getBillingCheckoutDetail('order-2');

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/commerce/orders/order-2',
        '/api/portal/commerce/orders/order-2/payment-methods',
        '/api/portal/commerce/orders/order-2/payment-attempts',
        '/api/portal/commerce/orders/order-2/checkout-session',
      ],
    );
    assert.equal(result.order.order_id, 'order-2');
    assert.equal(result.payment_attempts.length, 1);
    assert.equal(result.selected_payment_method?.display_name, 'Primary card');
    assert.equal(result.latest_payment_attempt?.provider_reference, 'pi_order_2');
    assert.equal(result.checkout_session.reference, 'PAY-order-2-legacy');
    assert.equal(result.checkout_methods.length, 3);
    assert.deepEqual(
      result.checkout_methods.map((method) => method.id),
      ['manual_settlement', 'cancel_order', 'pm-stripe'],
    );
    assert.equal(result.checkout_methods[2].action, 'provider_handoff');
    assert.equal(result.checkout_methods[2].provider, 'stripe');
    assert.equal(result.checkout_methods[2].channel, 'hosted_checkout');
    assert.equal(result.checkout_methods[2].supports_webhook, true);
    assert.equal(result.checkout_methods[2].session_reference, 'pi_order_2');
    assert.equal(result.checkout_methods[2].webhook_verification, 'webhook');
    assert.equal(result.checkout_methods[2].recommended, true);
  } finally {
    env.restore();
  }
});

test('portal billing checkout detail composes formal payment-attempt history for the selected order', async () => {
  const billingRepository = loadBillingRepository();
  const env = installPortalApiTestEnvironment({
    '/api/portal/commerce/orders/order-3': {
      order_id: 'order-3',
      project_id: 'project-demo',
      user_id: 'user-demo',
      target_kind: 'subscription_plan',
      target_id: 'plan-pro',
      target_name: 'Pro monthly',
      list_price_cents: 9900,
      payable_price_cents: 9900,
      list_price_label: '$99.00',
      payable_price_label: '$99.00',
      granted_units: 100000,
      bonus_units: 0,
      payment_method_id: 'pm-stripe',
      latest_payment_attempt_id: 'payatt-3-b',
      status: 'pending_payment',
      source: 'workspace_seed',
      created_at_ms: 30,
      updated_at_ms: 31,
    },
    '/api/portal/commerce/orders/order-3/payment-methods': [
      {
        payment_method_id: 'pm-stripe',
        display_name: 'Primary card',
        description: 'Visa ending 4242',
        provider: 'stripe',
        channel: 'card',
        mode: 'hosted',
        enabled: true,
        sort_order: 1,
        capability_codes: ['checkout'],
        supported_currency_codes: ['USD'],
        supported_country_codes: ['US'],
        supported_order_kinds: ['subscription_plan'],
        callback_strategy: 'webhook',
        webhook_path: '/webhooks/stripe',
        webhook_tolerance_seconds: 300,
        replay_window_seconds: 900,
        max_retry_count: 3,
        config_json: '{}',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    '/api/portal/commerce/orders/order-3/payment-attempts': [
      {
        payment_attempt_id: 'payatt-3-a',
        order_id: 'order-3',
        project_id: 'project-demo',
        user_id: 'user-demo',
        payment_method_id: 'pm-stripe',
        provider: 'stripe',
        channel: 'hosted_checkout',
        status: 'failed',
        idempotency_key: 'idem-order-3-a',
        attempt_sequence: 1,
        amount_minor: 9900,
        currency_code: 'USD',
        captured_amount_minor: 0,
        refunded_amount_minor: 0,
        provider_payment_intent_id: 'pi_order_3_a',
        provider_checkout_session_id: 'cs_order_3_a',
        provider_reference: 'pi_order_3_a',
        checkout_url: null,
        qr_code_payload: null,
        request_payload_json: '{}',
        response_payload_json: '{}',
        error_code: 'card_declined',
        error_message: 'Card declined',
        initiated_at_ms: 30,
        expires_at_ms: null,
        completed_at_ms: 31,
        updated_at_ms: 32,
      },
      {
        payment_attempt_id: 'payatt-3-b',
        order_id: 'order-3',
        project_id: 'project-demo',
        user_id: 'user-demo',
        payment_method_id: 'pm-stripe',
        provider: 'stripe',
        channel: 'hosted_checkout',
        status: 'requires_action',
        idempotency_key: 'idem-order-3-b',
        attempt_sequence: 2,
        amount_minor: 9900,
        currency_code: 'USD',
        captured_amount_minor: 0,
        refunded_amount_minor: 0,
        provider_payment_intent_id: 'pi_order_3_b',
        provider_checkout_session_id: 'cs_order_3_b',
        provider_reference: 'pi_order_3_b',
        checkout_url: 'https://checkout.stripe.test/session/cs_order_3_b',
        qr_code_payload: null,
        request_payload_json: '{}',
        response_payload_json: '{}',
        error_code: null,
        error_message: null,
        initiated_at_ms: 40,
        expires_at_ms: null,
        completed_at_ms: null,
        updated_at_ms: 41,
      },
    ],
    '/api/portal/commerce/orders/order-3/checkout-session': {
      order_id: 'order-3',
      order_status: 'pending_payment',
      session_status: 'open',
      provider: 'manual_lab',
      mode: 'operator_settlement',
      reference: 'PAY-order-3-legacy',
      payable_price_label: '$99.00',
      guidance: 'Use the compatibility settlement rail.',
      payment_simulation_enabled: true,
      methods: [
        {
          id: 'manual_settlement',
          label: 'Manual settlement',
          detail: 'Use the portal settle action in local or lab mode.',
          action: 'settle_order',
          availability: 'available',
          provider: 'manual_lab',
          channel: 'operator_settlement',
          session_kind: 'operator_action',
          session_reference: 'MANUAL-ORDER-3',
          qr_code_payload: null,
          webhook_verification: 'manual',
          supports_refund: true,
          supports_partial_refund: false,
          recommended: false,
          supports_webhook: false,
        },
      ],
    },
  });

  try {
    const result = await billingRepository.getBillingCheckoutDetail('order-3');

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/commerce/orders/order-3',
        '/api/portal/commerce/orders/order-3/payment-methods',
        '/api/portal/commerce/orders/order-3/payment-attempts',
        '/api/portal/commerce/orders/order-3/checkout-session',
      ],
    );
    assert.equal(result.payment_attempts.length, 2);
    assert.deepEqual(
      result.payment_attempts.map((attempt) => attempt.payment_attempt_id),
      ['payatt-3-b', 'payatt-3-a'],
    );
    assert.equal(result.latest_payment_attempt?.payment_attempt_id, 'payatt-3-b');
    assert.equal(result.latest_payment_attempt?.checkout_url, 'https://checkout.stripe.test/session/cs_order_3_b');
  } finally {
    env.restore();
  }
});

test('portal billing page surfaces payment-attempt history inside the checkout workbench', () => {
  const billingTypesSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'types', 'index.ts'),
    'utf8',
  );
  const billingRepositorySource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'repository', 'index.ts'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(billingTypesSource, /payment_attempts: CommercePaymentAttemptRecord\[];/);
  assert.match(billingRepositorySource, /listPortalCommercePaymentAttempts/);
  assert.match(billingPageSource, /checkoutDetail\?\.payment_attempts/);
  assert.match(billingPageSource, /Checkout attempts/);
  assert.match(
    billingPageSource,
    /Checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench\./,
  );
  assert.match(billingPageSource, /Latest attempt/);
  assert.doesNotMatch(billingPageSource, /Payment attempts/);
  assert.doesNotMatch(
    billingPageSource,
    /Formal order-scoped checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench\./,
  );
  assert.match(billingPageSource, /Refund history keeps completed refund outcomes, payment method evidence, and the resulting order status visible without reopening each order\./);
  assert.doesNotMatch(billingPageSource, /closed-loop refund outcomes/);
  assert.match(billingPageSource, /Payment history keeps checkout outcomes, payment method evidence, and refund status visible in one billing timeline\./);
  assert.doesNotMatch(billingPageSource, /refund closure/);
  assert.match(billingPageSource, /Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path\./);
  assert.doesNotMatch(billingPageSource, /operators can distinguish degraded routing from normal preference selection/);
});

test('portal billing page productizes commercial-account summary copy instead of exposing canonical state jargon', () => {
  const portalCommonsSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-commons', 'src', 'index.tsx'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(
    billingPageSource,
    /Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture\./,
  );
  assert.match(
    portalCommonsSource,
    /'Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture\.'/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /Commercial account exposes canonical balance, hold, and account identity state beside the workspace billing posture\./,
  );
  assert.doesNotMatch(
    portalCommonsSource,
    /'Commercial account exposes canonical balance, hold, and account identity state beside the workspace billing posture\.'/,
  );
});

test('portal billing page productizes failed-payment lane description instead of isolate jargon', () => {
  const portalCommonsSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-commons', 'src', 'index.tsx'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(
    billingPageSource,
    /Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up\./,
  );
  assert.match(
    portalCommonsSource,
    /'Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up\.'/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /Failed payment isolates checkout attempts that need coupon updates, a different payment method, or a fresh checkout attempt\./,
  );
  assert.doesNotMatch(
    portalCommonsSource,
    /'Failed payment isolates checkout attempts that need coupon updates, a different payment method, or a fresh checkout attempt\.'/,
  );
});

test('portal billing page productizes payment-reference anchor wording instead of anchor jargon', () => {
  const portalCommonsSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-commons', 'src', 'index.tsx'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(
    billingPageSource,
    /\{reference\} is the current \{provider\} \/ \{channel\} payment reference for this order\./,
  );
  assert.match(
    portalCommonsSource,
    /'\{reference\} is the current \{provider\} \/ \{channel\} payment reference for this order\.'/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /\{reference\} anchors the current \{provider\} \/ \{channel\} payment method for this order\./,
  );
  assert.doesNotMatch(
    portalCommonsSource,
    /'\{reference\} anchors the current \{provider\} \/ \{channel\} payment method for this order\.'/,
  );
});

test('portal billing page productizes billing-view status wording instead of posture-and-lifecycle jargon', () => {
  const portalCommonsSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-commons', 'src', 'index.tsx'),
    'utf8',
  );
  const billingPageSource = readFileSync(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-billing', 'src', 'pages', 'index.tsx'),
    'utf8',
  );

  assert.match(
    billingPageSource,
    /Billing view keeps live quota, checkout progress, and payment history in one place\./,
  );
  assert.match(
    portalCommonsSource,
    /'Billing view keeps live quota, checkout progress, and payment history in one place\.'/,
  );
  assert.doesNotMatch(
    billingPageSource,
    /Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline\./,
  );
  assert.doesNotMatch(
    portalCommonsSource,
    /'Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline\.'/,
  );
});
