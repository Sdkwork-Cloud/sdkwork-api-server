import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadAccountServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-account',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('portal account services aggregate total, today, 7-day, and monthly finance windows', () => {
  const { buildPortalAccountViewModel } = loadAccountServices();
  const now = new Date('2026-04-03T10:00:00.000Z').getTime();

  const viewModel = buildPortalAccountViewModel({
    summary: {
      project_id: 'project-demo',
      entry_count: 4,
      used_units: 9000,
      booked_amount: 240.5,
      quota_policy_id: 'quota-enterprise',
      quota_limit_units: 12000,
      remaining_units: 3000,
      exhausted: false,
    },
    membership: {
      membership_id: 'member-1',
      project_id: 'project-demo',
      user_id: 'user-1',
      plan_id: 'growth',
      plan_name: 'Growth',
      price_cents: 19900,
      price_label: '$199 / month',
      cadence: 'monthly',
      included_units: 12000,
      status: 'active',
      source: 'workspace_seed',
      activated_at_ms: now - 20 * 24 * 60 * 60 * 1000,
      updated_at_ms: now,
    },
    usageSummary: {
      total_requests: 120,
      project_count: 1,
      model_count: 3,
      provider_count: 2,
      projects: [{ project_id: 'project-demo', request_count: 120 }],
      providers: [
        { provider: 'provider-openai-official', request_count: 90, project_count: 1 },
        { provider: 'provider-anthropic', request_count: 30, project_count: 1 },
      ],
      models: [
        { model: 'gpt-4.1', request_count: 72, provider_count: 1 },
        { model: 'gpt-4.1-mini', request_count: 28, provider_count: 1 },
        { model: 'claude-3-7-sonnet', request_count: 20, provider_count: 1 },
      ],
    },
    usageRecords: [
      {
        project_id: 'project-demo',
        model: 'gpt-4.1',
        provider: 'provider-openai-official',
        units: 300,
        amount: 12,
        api_key_hash: 'key-live',
        channel_id: 'openai',
        input_tokens: 200,
        output_tokens: 100,
        total_tokens: 300,
        latency_ms: 400,
        reference_amount: 14,
        created_at_ms: now - 2 * 60 * 60 * 1000,
      },
      {
        project_id: 'project-demo',
        model: 'gpt-4.1-mini',
        provider: 'provider-openai-official',
        units: 250,
        amount: 8,
        api_key_hash: 'key-live',
        channel_id: 'openai',
        input_tokens: 160,
        output_tokens: 90,
        total_tokens: 250,
        latency_ms: 380,
        reference_amount: 9,
        created_at_ms: now - 2 * 24 * 60 * 60 * 1000,
      },
      {
        project_id: 'project-demo',
        model: 'claude-3-7-sonnet',
        provider: 'provider-anthropic',
        units: 500,
        amount: 20,
        api_key_hash: 'key-server',
        channel_id: 'anthropic',
        input_tokens: 300,
        output_tokens: 200,
        total_tokens: 500,
        latency_ms: 650,
        reference_amount: 22,
        created_at_ms: now - 8 * 24 * 60 * 60 * 1000,
      },
      {
        project_id: 'project-demo',
        model: 'gpt-4.1',
        provider: 'provider-openai-official',
        units: 200,
        amount: 6,
        api_key_hash: 'key-live',
        channel_id: 'openai',
        input_tokens: 110,
        output_tokens: 90,
        total_tokens: 200,
        latency_ms: 350,
        reference_amount: 7,
        created_at_ms: new Date('2026-04-01T08:00:00.000Z').getTime(),
      },
      {
        project_id: 'project-demo',
        model: 'gpt-4.1-mini',
        provider: 'provider-openai-official',
        units: 150,
        amount: 5,
        api_key_hash: 'key-live',
        channel_id: 'openai',
        input_tokens: 80,
        output_tokens: 70,
        total_tokens: 150,
        latency_ms: 330,
        reference_amount: 6,
        created_at_ms: new Date('2026-03-20T08:00:00.000Z').getTime(),
      },
    ],
    ledger: [
      { project_id: 'project-demo', units: 8000, amount: 180 },
      { project_id: 'project-linked-a', units: 650, amount: 35 },
      { project_id: 'project-linked-b', units: 250, amount: 15.5 },
      { project_id: 'sandbox-ops', units: 100, amount: 10 },
    ],
    searchQuery: 'project',
    historyView: 'all',
    page: 1,
    pageSize: 2,
    now,
  });

  const revenueView = buildPortalAccountViewModel({
    summary: {
      project_id: 'project-demo',
      entry_count: 4,
      used_units: 9000,
      booked_amount: 240.5,
      quota_policy_id: 'quota-enterprise',
      quota_limit_units: 12000,
      remaining_units: 3000,
      exhausted: false,
    },
    membership: null,
    usageSummary: {
      total_requests: 120,
      project_count: 1,
      model_count: 3,
      provider_count: 2,
      projects: [{ project_id: 'project-demo', request_count: 120 }],
      providers: [],
      models: [],
    },
    usageRecords: [
      {
        project_id: 'project-demo',
        model: 'gpt-4.1',
        provider: 'provider-openai-official',
        units: 300,
        amount: 12,
        api_key_hash: 'key-live',
        channel_id: 'openai',
        input_tokens: 200,
        output_tokens: 100,
        total_tokens: 300,
        latency_ms: 400,
        reference_amount: 14,
        created_at_ms: now - 2 * 60 * 60 * 1000,
      },
    ],
    ledger: [
      { project_id: 'project-demo', units: 8000, amount: 180 },
      { project_id: 'project-linked-a', units: 650, amount: 35 },
      { project_id: 'project-linked-b', units: 250, amount: 15.5 },
      { project_id: 'sandbox-ops', units: 100, amount: 10 },
    ],
    searchQuery: 'project',
    historyView: 'revenue',
    page: 1,
    pageSize: 2,
    now,
  });

  assert.equal(viewModel.balance.remaining_units, 3000);
  assert.equal(viewModel.balance.quota_limit_units, 12000);
  assert.equal(viewModel.balance.used_units, 9000);
  assert.equal(viewModel.balance.utilization_ratio, 0.75);
  assert.equal(viewModel.totals.revenue, 240.5);
  assert.equal(viewModel.totals.request_count, 120);
  assert.equal(viewModel.totals.used_units, 9000);
  assert.equal(viewModel.totals.average_booked_spend, 240.5 / 120);

  assert.equal(viewModel.today.revenue, 12);
  assert.equal(viewModel.today.request_count, 1);
  assert.equal(viewModel.today.used_units, 300);
  assert.equal(viewModel.today.average_booked_spend, 12);

  assert.equal(viewModel.trailing_7d.revenue, 26);
  assert.equal(viewModel.trailing_7d.request_count, 3);
  assert.equal(viewModel.trailing_7d.used_units, 750);
  assert.equal(viewModel.trailing_7d.average_booked_spend, 26 / 3);

  assert.equal(viewModel.current_month.revenue, 26);
  assert.equal(viewModel.current_month.request_count, 3);
  assert.equal(viewModel.current_month.used_units, 750);
  assert.equal(viewModel.current_month.average_booked_spend, 26 / 3);

  assert.deepEqual(viewModel.history_counts, {
    all: 8,
    expense: 5,
    revenue: 3,
  });
  assert.equal(viewModel.pagination.total_items, 8);
  assert.equal(viewModel.pagination.total_pages, 4);
  assert.equal(viewModel.visible_history.length, 2);
  assert.equal(viewModel.visible_history[0].kind, 'expense');
  assert.equal(viewModel.visible_history[0].source, 'usage');
  assert.equal(viewModel.visible_history[0].project_id, 'project-demo');
  assert.equal(viewModel.visible_history[1].kind, 'expense');
  assert.equal(viewModel.visible_history[1].source, 'usage');

  assert.equal(revenueView.pagination.total_items, 3);
  assert.equal(revenueView.pagination.total_pages, 2);
  assert.equal(revenueView.visible_history.length, 2);
  assert.equal(revenueView.visible_history[0].kind, 'revenue');
  assert.equal(revenueView.visible_history[0].source, 'ledger');
  assert.equal(revenueView.visible_history[0].project_id, 'project-demo');
  assert.equal(revenueView.visible_history[0].scope, 'current');
  assert.equal(revenueView.visible_history[1].project_id, 'project-linked-a');
  assert.equal(revenueView.visible_history[1].scope, 'linked');
  assert.equal(revenueView.visible_history[0].share_of_booked_amount, 180 / 240.5);
});

test('portal account services recover commerce order lineage from commercial ledger history', () => {
  const { buildPortalAccountViewModel } = loadAccountServices();
  const now = new Date('2026-04-03T10:00:00.000Z').getTime();

  const viewModel = buildPortalAccountViewModel({
    summary: {
      project_id: 'project-demo',
      entry_count: 1,
      used_units: 0,
      booked_amount: 180,
      quota_limit_units: 8000,
      remaining_units: 8000,
      exhausted: false,
    },
    membership: null,
    usageSummary: {
      total_requests: 0,
      project_count: 1,
      model_count: 0,
      provider_count: 0,
      projects: [{ project_id: 'project-demo', request_count: 0 }],
      providers: [],
      models: [],
    },
    usageRecords: [],
    ledger: [
      { project_id: 'project-demo', units: 8000, amount: 180 },
      { project_id: 'project-linked-a', units: 650, amount: 35 },
    ],
    billingEventSummary: {
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
    benefitLots: [
      {
        lot_id: 8001,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        benefit_type: 'cash_credit',
        source_type: 'order',
        source_id: 12345,
        scope_json: '{"order_id":"order-recharge-1","project_id":"project-demo","target_kind":"recharge_pack"}',
        original_quantity: 8000,
        remaining_quantity: 8000,
        held_quantity: 0,
        priority: 10,
        acquired_unit_cost: 0.0225,
        issued_at_ms: now - 60 * 1000,
        expires_at_ms: null,
        status: 'active',
        created_at_ms: now - 60 * 1000,
        updated_at_ms: now - 60 * 1000,
      },
    ],
    accountLedgerHistory: [
      {
        entry: {
          ledger_entry_id: 8401,
          tenant_id: 1001,
          organization_id: 2002,
          account_id: 7001,
          user_id: 9001,
          request_id: null,
          hold_id: null,
          entry_type: 'grant_issue',
          benefit_type: 'cash_credit',
          quantity: 8000,
          amount: 180,
          created_at_ms: now - 60 * 1000,
        },
        allocations: [
          {
            ledger_allocation_id: 8501,
            tenant_id: 1001,
            organization_id: 2002,
            ledger_entry_id: 8401,
            lot_id: 8001,
            quantity_delta: 8000,
            created_at_ms: now - 60 * 1000,
          },
        ],
      },
    ],
    searchQuery: 'order-recharge-1',
    historyView: 'revenue',
    page: 1,
    pageSize: 4,
    now,
  });

  assert.equal(viewModel.pagination.total_items, 1);
  assert.equal(viewModel.visible_history.length, 1);
  assert.equal(viewModel.visible_history[0].order_id, 'order-recharge-1');
  assert.equal(viewModel.visible_history[0].ledger_entry_type, 'grant_issue');
  assert.equal(viewModel.visible_history[0].scope, 'current');
});
