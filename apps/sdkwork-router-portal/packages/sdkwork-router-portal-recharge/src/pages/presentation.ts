import type { PortalCommerceOrder } from 'sdkwork-router-portal-types';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

type HandoffMode = 'billing_handoff' | 'create_order';

export interface PortalRechargePrimaryActionState {
  mode: HandoffMode;
  disabled: boolean;
  label: string;
}

export interface PortalRechargeMobileActionState {
  mode: HandoffMode;
  amountLabel: string;
  eyebrow: string;
  supportingText: string;
  buttonLabel: string;
  disabled: boolean;
}

export interface PortalRechargePendingPaymentSpotlightLike {
  latestOrder: Pick<PortalCommerceOrder, 'order_id'>;
}

function resolvePortalRechargeActionDisabled(input: {
  postOrderHandoffActive: boolean;
  quoteLoading: boolean;
  createLoading: boolean;
  hasSelection: boolean;
}) {
  if (input.postOrderHandoffActive) {
    return false;
  }

  return input.quoteLoading || input.createLoading || !input.hasSelection;
}

export function resolvePortalRechargePostOrderHandoffActive(input: {
  lastCreatedOrderId: string | null;
  pendingPaymentSpotlight: PortalRechargePendingPaymentSpotlightLike | null;
}) {
  const { lastCreatedOrderId, pendingPaymentSpotlight } = input;
  return Boolean(
    lastCreatedOrderId
      && pendingPaymentSpotlight
      && pendingPaymentSpotlight.latestOrder.order_id === lastCreatedOrderId,
  );
}

export function buildPortalRechargePrimaryActionState(input: {
  postOrderHandoffActive: boolean;
  quoteLoading: boolean;
  createLoading: boolean;
  hasSelection: boolean;
  t: TranslateFn;
}): PortalRechargePrimaryActionState {
  const { postOrderHandoffActive, quoteLoading, createLoading, hasSelection, t } = input;

  if (postOrderHandoffActive) {
    return {
      mode: 'billing_handoff',
      disabled: false,
      label: t('Continue in billing'),
    };
  }

  return {
    mode: 'create_order',
    disabled: resolvePortalRechargeActionDisabled({
      postOrderHandoffActive,
      quoteLoading,
      createLoading,
      hasSelection,
    }),
    label: createLoading ? t('Creating...') : t('Create recharge order'),
  };
}

export function buildPortalRechargeMobileActionState(input: {
  postOrderHandoffActive: boolean;
  selectedAmountLabel: string;
  grantedUnitsLabel: string;
  quoteLoading: boolean;
  createLoading: boolean;
  hasSelection: boolean;
  t: TranslateFn;
}): PortalRechargeMobileActionState {
  const {
    postOrderHandoffActive,
    selectedAmountLabel,
    grantedUnitsLabel,
    quoteLoading,
    createLoading,
    hasSelection,
    t,
  } = input;

  if (postOrderHandoffActive) {
    return {
      mode: 'billing_handoff',
      amountLabel: selectedAmountLabel,
      eyebrow: t('Order ready for payment'),
      supportingText: t('Continue in billing'),
      buttonLabel: t('Continue in billing'),
      disabled: false,
    };
  }

  return {
    mode: 'create_order',
    amountLabel: selectedAmountLabel,
    eyebrow: t('Create order in billing'),
    supportingText: t('Granted units: {units}', { units: grantedUnitsLabel }),
    buttonLabel: createLoading ? t('Creating...') : t('Create order in billing'),
    disabled: resolvePortalRechargeActionDisabled({
      postOrderHandoffActive,
      quoteLoading,
      createLoading,
      hasSelection,
    }),
  };
}
