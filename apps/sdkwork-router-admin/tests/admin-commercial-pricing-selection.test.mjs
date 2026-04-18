import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadCommercialPricing() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-core',
      'src',
      'commercialPricing.ts',
    ),
  );
}

test('admin commercial pricing selection prefers currently effective active plans over future active versions', () => {
  const { selectPrimaryCommercialPricingPlan } = loadCommercialPricing();
  const now = 1_717_171_730_000;

  const primaryPlan = selectPrimaryCommercialPricingPlan(
    [
      {
        pricing_plan_id: 9102,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 2,
        display_name: 'Workspace Retail Future',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: now + 86_400_000,
        effective_to_ms: null,
        created_at_ms: now - 1000,
        updated_at_ms: now + 1000,
      },
      {
        pricing_plan_id: 9101,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 1,
        display_name: 'Workspace Retail Current',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: now - 86_400_000,
        effective_to_ms: now + 86_400_000,
        created_at_ms: now - 2000,
        updated_at_ms: now,
      },
    ],
    now,
  );

  assert.equal(primaryPlan?.pricing_plan_id, 9101);
  assert.equal(primaryPlan?.display_name, 'Workspace Retail Current');
});

test('admin commercial pricing selection keeps unsafe pricing ids sortable without number coercion', () => {
  const {
    selectPrimaryCommercialPricingPlan,
    selectPrimaryCommercialPricingRate,
  } = loadCommercialPricing();
  const now = 1_717_171_730_000;

  const pricingPlans = [
    {
      pricing_plan_id: '9007199254740992',
      tenant_id: 1001,
      organization_id: 2002,
      plan_code: 'workspace-retail',
      plan_version: 1,
      display_name: 'Workspace Retail Older Id',
      currency_code: 'USD',
      credit_unit_code: 'credit',
      status: 'active',
      effective_from_ms: now - 86_400_000,
      effective_to_ms: now + 86_400_000,
      created_at_ms: now - 2000,
      updated_at_ms: now,
    },
    {
      pricing_plan_id: '9007199254740993',
      tenant_id: 1001,
      organization_id: 2002,
      plan_code: 'workspace-retail',
      plan_version: 1,
      display_name: 'Workspace Retail Higher Id',
      currency_code: 'USD',
      credit_unit_code: 'credit',
      status: 'active',
      effective_from_ms: now - 86_400_000,
      effective_to_ms: now + 86_400_000,
      created_at_ms: now - 2000,
      updated_at_ms: now,
    },
  ];
  const pricingRates = [
    {
      pricing_rate_id: '9007199254740992',
      tenant_id: 1001,
      organization_id: 2002,
      pricing_plan_id: '9007199254740993',
      metric_code: 'token.input',
      capability_code: 'responses',
      model_code: 'gpt-5.4',
      provider_code: 'provider-openai',
      charge_unit: 'input_token',
      pricing_method: 'per_unit',
      quantity_step: 1000000,
      unit_price: 0.2,
      display_price_unit: 'USD / 1M input tokens',
      minimum_billable_quantity: 0,
      minimum_charge: 0,
      rounding_increment: 1,
      rounding_mode: 'ceil',
      included_quantity: 0,
      priority: 100,
      notes: null,
      status: 'active',
      created_at_ms: now - 1000,
      updated_at_ms: now,
    },
    {
      pricing_rate_id: '9007199254740993',
      tenant_id: 1001,
      organization_id: 2002,
      pricing_plan_id: '9007199254740993',
      metric_code: 'token.output',
      capability_code: 'responses',
      model_code: 'gpt-5.4',
      provider_code: 'provider-openai',
      charge_unit: 'output_token',
      pricing_method: 'per_unit',
      quantity_step: 1000000,
      unit_price: 0.3,
      display_price_unit: 'USD / 1M output tokens',
      minimum_billable_quantity: 0,
      minimum_charge: 0,
      rounding_increment: 1,
      rounding_mode: 'ceil',
      included_quantity: 0,
      priority: 100,
      notes: null,
      status: 'active',
      created_at_ms: now - 1000,
      updated_at_ms: now,
    },
  ];

  const primaryPlan = selectPrimaryCommercialPricingPlan(pricingPlans, now);
  const primaryRate = selectPrimaryCommercialPricingRate(pricingRates, primaryPlan);

  assert.equal(primaryPlan?.pricing_plan_id, '9007199254740993');
  assert.equal(primaryPlan?.display_name, 'Workspace Retail Higher Id');
  assert.equal(primaryRate?.pricing_rate_id, '9007199254740993');
  assert.equal(primaryRate?.metric_code, 'token.output');
});
