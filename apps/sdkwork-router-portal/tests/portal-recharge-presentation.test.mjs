import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadRechargePresentation() {
  const load = jiti(import.meta.url, {
    moduleCache: false,
  });

  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-recharge',
      'src',
      'pages',
      'presentation.ts',
    ),
  );
}

function createTranslator() {
  return (text, values) => {
    if (!values) {
      return text;
    }

    return Object.entries(values).reduce(
      (current, [key, value]) => current.replace(`{${key}}`, String(value)),
      text,
    );
  };
}

test('recharge presentation helpers switch the primary action into billing handoff mode only for the latest created pending order', () => {
  const {
    buildPortalRechargePrimaryActionState,
    resolvePortalRechargePostOrderHandoffActive,
  } = loadRechargePresentation();
  const t = createTranslator();
  const spotlight = {
    latestOrder: {
      order_id: 'order-2',
    },
  };

  assert.equal(
    resolvePortalRechargePostOrderHandoffActive({
      lastCreatedOrderId: 'order-2',
      pendingPaymentSpotlight: spotlight,
    }),
    true,
  );
  assert.equal(
    resolvePortalRechargePostOrderHandoffActive({
      lastCreatedOrderId: 'order-1',
      pendingPaymentSpotlight: spotlight,
    }),
    false,
  );
  assert.equal(
    resolvePortalRechargePostOrderHandoffActive({
      lastCreatedOrderId: null,
      pendingPaymentSpotlight: spotlight,
    }),
    false,
  );

  assert.deepEqual(
    buildPortalRechargePrimaryActionState({
      postOrderHandoffActive: true,
      quoteLoading: true,
      createLoading: true,
      hasSelection: true,
      t,
    }),
    {
      mode: 'billing_handoff',
      disabled: false,
      label: 'Continue in billing',
    },
  );
  assert.deepEqual(
    buildPortalRechargePrimaryActionState({
      postOrderHandoffActive: false,
      quoteLoading: false,
      createLoading: true,
      hasSelection: true,
      t,
    }),
    {
      mode: 'create_order',
      disabled: true,
      label: 'Creating...',
    },
  );
  assert.deepEqual(
    buildPortalRechargePrimaryActionState({
      postOrderHandoffActive: false,
      quoteLoading: false,
      createLoading: false,
      hasSelection: true,
      t,
    }),
    {
      mode: 'create_order',
      disabled: false,
      label: 'Create recharge order',
    },
  );
});

test('recharge presentation helpers keep the mobile CTA copy aligned with desktop handoff mode', () => {
  const { buildPortalRechargeMobileActionState } = loadRechargePresentation();
  const t = createTranslator();

  assert.deepEqual(
    buildPortalRechargeMobileActionState({
      postOrderHandoffActive: true,
      selectedAmountLabel: '$90.00',
      grantedUnitsLabel: '1,080,000',
      quoteLoading: true,
      createLoading: true,
      hasSelection: true,
      t,
    }),
    {
      mode: 'billing_handoff',
      amountLabel: '$90.00',
      eyebrow: 'Order ready for payment',
      supportingText: 'Continue in billing',
      buttonLabel: 'Continue in billing',
      disabled: false,
    },
  );
  assert.deepEqual(
    buildPortalRechargeMobileActionState({
      postOrderHandoffActive: false,
      selectedAmountLabel: '$30.00',
      grantedUnitsLabel: '330,000',
      quoteLoading: false,
      createLoading: false,
      hasSelection: true,
      t,
    }),
    {
      mode: 'create_order',
      amountLabel: '$30.00',
      eyebrow: 'Create order in billing',
      supportingText: 'Granted units: 330,000',
      buttonLabel: 'Create order in billing',
      disabled: false,
    },
  );
  assert.deepEqual(
    buildPortalRechargeMobileActionState({
      postOrderHandoffActive: false,
      selectedAmountLabel: '$30.00',
      grantedUnitsLabel: '330,000',
      quoteLoading: false,
      createLoading: true,
      hasSelection: true,
      t,
    }),
    {
      mode: 'create_order',
      amountLabel: '$30.00',
      eyebrow: 'Create order in billing',
      supportingText: 'Granted units: 330,000',
      buttonLabel: 'Creating...',
      disabled: true,
    },
  );
});

test('recharge presentation helpers derive a three-stage flow tracker for selection, order creation, and settlement', () => {
  const { buildPortalRechargeFlowTrackerState } = loadRechargePresentation();
  const t = createTranslator();

  assert.deepEqual(
    buildPortalRechargeFlowTrackerState({
      hasSelection: false,
      hasQuote: false,
      postOrderHandoffActive: false,
      pendingPaymentCount: 0,
      t,
    }),
    {
      title: 'Funding flow',
      steps: [
        {
          id: 'choose_amount',
          label: 'Choose amount',
          detail: 'Pick a package or custom amount to unlock the live quote.',
          status: 'current',
        },
        {
          id: 'create_order',
          label: 'Create order',
          detail: 'Order creation unlocks once an amount is ready.',
          status: 'pending',
        },
        {
          id: 'complete_payment',
          label: 'Complete payment in billing',
          detail: 'Settlement starts after the recharge order is created.',
          status: 'pending',
        },
      ],
    },
  );

  assert.deepEqual(
    buildPortalRechargeFlowTrackerState({
      hasSelection: true,
      hasQuote: true,
      postOrderHandoffActive: false,
      pendingPaymentCount: 0,
      t,
    }),
    {
      title: 'Funding flow',
      steps: [
        {
          id: 'choose_amount',
          label: 'Choose amount',
          detail: 'Live quote ready for the selected recharge path.',
          status: 'complete',
        },
        {
          id: 'create_order',
          label: 'Create order',
          detail: 'Create the recharge order to hand settlement into billing.',
          status: 'current',
        },
        {
          id: 'complete_payment',
          label: 'Complete payment in billing',
          detail: 'Settlement starts after the recharge order is created.',
          status: 'pending',
        },
      ],
    },
  );

  assert.deepEqual(
    buildPortalRechargeFlowTrackerState({
      hasSelection: true,
      hasQuote: true,
      postOrderHandoffActive: true,
      pendingPaymentCount: 1,
      t,
    }),
    {
      title: 'Funding flow',
      steps: [
        {
          id: 'choose_amount',
          label: 'Choose amount',
          detail: 'Live quote ready for the selected recharge path.',
          status: 'complete',
        },
        {
          id: 'create_order',
          label: 'Create order',
          detail: 'Recharge order created and recorded.',
          status: 'complete',
        },
        {
          id: 'complete_payment',
          label: 'Complete payment in billing',
          detail: 'Continue in billing to finish payment capture.',
          status: 'current',
        },
      ],
    },
  );

  assert.deepEqual(
    buildPortalRechargeFlowTrackerState({
      hasSelection: true,
      hasQuote: true,
      postOrderHandoffActive: false,
      pendingPaymentCount: 2,
      t,
    }),
    {
      title: 'Funding flow',
      steps: [
        {
          id: 'choose_amount',
          label: 'Choose amount',
          detail: 'Live quote ready for the selected recharge path.',
          status: 'complete',
        },
        {
          id: 'create_order',
          label: 'Create order',
          detail: 'Create the recharge order to hand settlement into billing.',
          status: 'current',
        },
        {
          id: 'complete_payment',
          label: 'Complete payment in billing',
          detail: 'Pending settlement queue already needs billing follow-up.',
          status: 'attention',
        },
      ],
    },
  );
});
