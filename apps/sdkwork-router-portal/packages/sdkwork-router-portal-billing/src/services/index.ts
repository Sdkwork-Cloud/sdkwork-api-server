import { formatUnits } from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';
import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventGroupSummary,
  BillingEventRecord,
  BillingEventSummary,
  CommercePaymentAttemptRecord,
  PaymentMethodRecord,
  PortalCommerceCheckoutSession,
  PortalCommerceCheckoutSessionMethod,
  PortalCommerceOrder,
  PortalCommercePaymentEventRecord,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import type {
  BillingCheckoutDetail,
  BillingRecommendation,
} from '../types';
import type {
  BillingEventAnalyticsViewModel,
  BillingPaymentHistoryRow,
} from '../types';

export type BillingEventCsvDocument = {
  headers: string[];
  rows: Array<Array<string | number>>;
};

export type BillingCheckoutLaunchDecisionKind =
  | 'resume_existing_attempt'
  | 'create_retry_attempt'
  | 'create_first_attempt';

export interface BillingCheckoutLaunchDecision {
  kind: BillingCheckoutLaunchDecisionKind;
  latest_attempt: CommercePaymentAttemptRecord | null;
  matched_attempt_count: number;
}

export interface BillingCheckoutPresentation {
  reference: string | null;
  payable_price_label: string;
  payment_method_name: string | null;
  provider: PortalCommerceCheckoutSessionMethod['provider'] | null;
  channel: PortalCommerceCheckoutSessionMethod['channel'] | null;
  status_source: 'payment_attempt' | 'checkout_session' | 'none';
  status: string | null;
  guidance_source: 'payment_attempt_error' | 'launch_decision' | 'checkout_session' | 'none';
  guidance: string | null;
  launch_decision_kind: BillingCheckoutLaunchDecisionKind | null;
  launch_method: PortalCommerceCheckoutSessionMethod | null;
}

export interface BillingPaymentHistorySource {
  order: PortalCommerceOrder;
  payment_events: PortalCommercePaymentEventRecord[];
  latest_payment_event?: PortalCommercePaymentEventRecord | null;
  compatibility_checkout_session?: PortalCommerceCheckoutSession | null;
  latest_payment_attempt?: CommercePaymentAttemptRecord | null;
  selected_payment_method?: PaymentMethodRecord | null;
}

function sortBillingCheckoutLaunchAttempts(
  paymentAttempts: CommercePaymentAttemptRecord[],
): CommercePaymentAttemptRecord[] {
  return paymentAttempts
    .slice()
    .sort((left, right) => (
      right.attempt_sequence - left.attempt_sequence
      || right.updated_at_ms - left.updated_at_ms
      || right.initiated_at_ms - left.initiated_at_ms
    ));
}

function hasBillingCapability(method: PaymentMethodRecord, capabilityCode: string): boolean {
  return method.capability_codes.some((capability) => (
    capability.trim().toLowerCase() === capabilityCode.trim().toLowerCase()
  ));
}

function resolveBillingCheckoutMethodProvider(
  provider: string,
  fallback?: PortalCommerceCheckoutSessionMethod['provider'],
): PortalCommerceCheckoutSessionMethod['provider'] {
  switch (provider.trim().toLowerCase()) {
    case 'manual_lab':
      return 'manual_lab';
    case 'stripe':
      return 'stripe';
    case 'alipay':
      return 'alipay';
    case 'wechat_pay':
      return 'wechat_pay';
    case 'no_payment_required':
      return 'no_payment_required';
    default:
      return fallback ?? 'manual_lab';
  }
}

function resolveBillingCheckoutMethodChannel(
  channel: string,
  fallback?: PortalCommerceCheckoutSessionMethod['channel'],
): PortalCommerceCheckoutSessionMethod['channel'] {
  switch (channel.trim().toLowerCase()) {
    case 'operator_settlement':
      return 'operator_settlement';
    case 'scan_qr':
      return 'scan_qr';
    case 'hosted_checkout':
      return 'hosted_checkout';
    default:
      return fallback ?? 'hosted_checkout';
  }
}

function resolveBillingCheckoutMethodSessionKind(
  channel: PortalCommerceCheckoutSessionMethod['channel'],
  fallback?: PortalCommerceCheckoutSessionMethod['session_kind'],
): PortalCommerceCheckoutSessionMethod['session_kind'] {
  switch (channel) {
    case 'operator_settlement':
      return 'operator_action';
    case 'scan_qr':
      return 'qr_code';
    case 'hosted_checkout':
      return 'hosted_checkout';
    default:
      return fallback ?? 'hosted_checkout';
  }
}

function buildBillingCheckoutMethodKey(
  method: Pick<PortalCommerceCheckoutSessionMethod, 'provider' | 'channel' | 'action'>,
): string {
  return `${method.provider}::${method.channel}::${method.action}`.toLowerCase();
}

function resolveBillingAttemptReference(
  latestPaymentAttempt: CommercePaymentAttemptRecord | null,
): string | null {
  const providerReference = latestPaymentAttempt?.provider_reference?.trim();
  if (providerReference) {
    return providerReference;
  }

  const checkoutSessionId = latestPaymentAttempt?.provider_checkout_session_id?.trim();
  if (checkoutSessionId) {
    return checkoutSessionId;
  }

  return null;
}

function isReusableBillingCheckoutAttempt(
  paymentAttempt: CommercePaymentAttemptRecord | null,
  nowMs: number,
): boolean {
  const checkoutUrl = paymentAttempt?.checkout_url?.trim();
  if (!checkoutUrl) {
    return false;
  }

  if (paymentAttempt?.expires_at_ms != null && paymentAttempt.expires_at_ms <= nowMs) {
    return false;
  }

  const normalizedStatus = paymentAttempt?.status?.trim().toLowerCase() ?? '';
  switch (normalizedStatus) {
    case 'failed':
    case 'canceled':
    case 'cancelled':
    case 'expired':
    case 'succeeded':
    case 'refunded':
    case 'partially_refunded':
    case 'partially-refunded':
      return false;
    default:
      return true;
  }
}

export function buildBillingCheckoutLaunchDecision(input: {
  checkout_method: Pick<PortalCommerceCheckoutSessionMethod, 'id' | 'action'>;
  payment_attempts: CommercePaymentAttemptRecord[];
  now_ms?: number;
}): BillingCheckoutLaunchDecision {
  const { checkout_method, payment_attempts, now_ms = Date.now() } = input;
  if (checkout_method.action !== 'provider_handoff') {
    return {
      kind: 'create_first_attempt',
      latest_attempt: null,
      matched_attempt_count: 0,
    };
  }

  const matchingAttempts = sortBillingCheckoutLaunchAttempts(
    payment_attempts.filter((paymentAttempt) => (
      paymentAttempt.payment_method_id === checkout_method.id
    )),
  );
  const latestAttempt = matchingAttempts[0] ?? null;

  if (isReusableBillingCheckoutAttempt(latestAttempt, now_ms)) {
    return {
      kind: 'resume_existing_attempt',
      latest_attempt: latestAttempt,
      matched_attempt_count: matchingAttempts.length,
    };
  }

  if (latestAttempt) {
    return {
      kind: 'create_retry_attempt',
      latest_attempt: latestAttempt,
      matched_attempt_count: matchingAttempts.length,
    };
  }

  return {
    kind: 'create_first_attempt',
    latest_attempt: null,
    matched_attempt_count: 0,
  };
}

function trimBillingText(value?: string | null): string | null {
  const normalized = value?.trim();
  return normalized ? normalized : null;
}

function findBillingCheckoutMethodById(
  methods: PortalCommerceCheckoutSessionMethod[],
  id: string | null,
): PortalCommerceCheckoutSessionMethod | null {
  if (!id) {
    return null;
  }

  return methods.find((method) => method.id === id) ?? null;
}

function selectPrimaryBillingCheckoutMethod(
  checkoutDetail: BillingCheckoutDetail,
): PortalCommerceCheckoutSessionMethod | null {
  const providerHandoffMethods = checkoutDetail.checkout_methods.filter((method) => (
    method.action === 'provider_handoff'
  ));
  const preferredMethods = providerHandoffMethods.length
    ? providerHandoffMethods
    : checkoutDetail.checkout_methods;
  const preferredIds = [
    checkoutDetail.selected_payment_method?.payment_method_id ?? null,
    checkoutDetail.latest_payment_attempt?.payment_method_id ?? null,
    checkoutDetail.order.payment_method_id ?? null,
  ];

  for (const preferredId of preferredIds) {
    const match = findBillingCheckoutMethodById(preferredMethods, preferredId);
    if (match) {
      return match;
    }
  }

  return preferredMethods.find((method) => method.recommended)
    ?? preferredMethods[0]
    ?? checkoutDetail.checkout_methods.find((method) => method.recommended)
    ?? checkoutDetail.checkout_methods[0]
    ?? null;
}

export function buildBillingCheckoutPresentation(
  checkoutDetail: BillingCheckoutDetail,
): BillingCheckoutPresentation {
  const primaryMethod = selectPrimaryBillingCheckoutMethod(checkoutDetail);
  const latestAttempt = checkoutDetail.latest_payment_attempt;
  const compatibilityCheckoutSession = checkoutDetail.checkout_session;
  const reference = resolveBillingAttemptReference(latestAttempt)
    ?? trimBillingText(primaryMethod?.session_reference)
    ?? trimBillingText(compatibilityCheckoutSession.reference);
  const payablePriceLabel = trimBillingText(checkoutDetail.order.payable_price_label)
    ?? trimBillingText(compatibilityCheckoutSession.payable_price_label)
    ?? '';
  const paymentMethodName = trimBillingText(checkoutDetail.selected_payment_method?.display_name)
    ?? trimBillingText(primaryMethod?.label);
  const latestAttemptStatus = trimBillingText(latestAttempt?.status);
  const compatibilityStatus = trimBillingText(compatibilityCheckoutSession.session_status);
  const latestAttemptError = trimBillingText(latestAttempt?.error_message);

  if (latestAttemptError) {
    return {
      reference,
      payable_price_label: payablePriceLabel,
      payment_method_name: paymentMethodName,
      provider: primaryMethod?.provider ?? compatibilityCheckoutSession.provider ?? null,
      channel: primaryMethod?.channel ?? null,
      status_source: latestAttemptStatus ? 'payment_attempt' : compatibilityStatus ? 'checkout_session' : 'none',
      status: latestAttemptStatus ?? compatibilityStatus,
      guidance_source: 'payment_attempt_error',
      guidance: latestAttemptError,
      launch_decision_kind: null,
      launch_method: null,
    };
  }

  if (primaryMethod?.action === 'provider_handoff') {
    const launchDecision = buildBillingCheckoutLaunchDecision({
      checkout_method: primaryMethod,
      payment_attempts: checkoutDetail.payment_attempts,
    });

    return {
      reference,
      payable_price_label: payablePriceLabel,
      payment_method_name: paymentMethodName,
      provider: primaryMethod.provider,
      channel: primaryMethod.channel,
      status_source: latestAttemptStatus ? 'payment_attempt' : compatibilityStatus ? 'checkout_session' : 'none',
      status: latestAttemptStatus ?? compatibilityStatus,
      guidance_source: 'launch_decision',
      guidance: null,
      launch_decision_kind: launchDecision.kind,
      launch_method: primaryMethod,
    };
  }

  const compatibilityGuidance = trimBillingText(compatibilityCheckoutSession.guidance);

  return {
    reference,
    payable_price_label: payablePriceLabel,
    payment_method_name: paymentMethodName,
    provider: primaryMethod?.provider ?? compatibilityCheckoutSession.provider ?? null,
    channel: primaryMethod?.channel ?? null,
    status_source: latestAttemptStatus ? 'payment_attempt' : compatibilityStatus ? 'checkout_session' : 'none',
    status: latestAttemptStatus ?? compatibilityStatus,
    guidance_source: compatibilityGuidance ? 'checkout_session' : 'none',
    guidance: compatibilityGuidance,
    launch_decision_kind: null,
    launch_method: null,
  };
}

function findMatchingCompatibilityCheckoutMethod(
  checkoutSession: PortalCommerceCheckoutSession,
  paymentMethod: PaymentMethodRecord,
): PortalCommerceCheckoutSessionMethod | null {
  const directMatch = checkoutSession.methods.find((method) => (
    method.id === paymentMethod.payment_method_id
  ));
  if (directMatch) {
    return directMatch;
  }

  const provider = resolveBillingCheckoutMethodProvider(paymentMethod.provider);
  const channel = resolveBillingCheckoutMethodChannel(paymentMethod.channel);

  return checkoutSession.methods.find((method) => (
    method.provider === provider && method.channel === channel
  )) ?? null;
}

function buildFormalBillingCheckoutMethod(input: {
  order: PortalCommerceOrder;
  payment_method: PaymentMethodRecord;
  latest_payment_attempt: CommercePaymentAttemptRecord | null;
  selected_payment_method_id: string | null;
  compatibility_checkout_method: PortalCommerceCheckoutSessionMethod | null;
}): PortalCommerceCheckoutSessionMethod {
  const {
    order,
    payment_method,
    latest_payment_attempt,
    selected_payment_method_id,
    compatibility_checkout_method,
  } = input;
  const provider = resolveBillingCheckoutMethodProvider(
    payment_method.provider,
    compatibility_checkout_method?.provider,
  );
  const channel = resolveBillingCheckoutMethodChannel(
    payment_method.channel,
    compatibility_checkout_method?.channel,
  );
  const action: PortalCommerceCheckoutSessionMethod['action'] =
    channel === 'operator_settlement'
      ? 'settle_order'
      : 'provider_handoff';
  const attemptReference = latest_payment_attempt?.payment_method_id === payment_method.payment_method_id
    ? resolveBillingAttemptReference(latest_payment_attempt)
    : null;
  const sessionReference = attemptReference
    ?? compatibility_checkout_method?.session_reference
    ?? payment_method.payment_method_id;
  const callbackStrategy = payment_method.callback_strategy.trim();
  const supportsWebhook = callbackStrategy.toLowerCase().includes('webhook');

  return {
    id: payment_method.payment_method_id,
    label: payment_method.display_name,
    detail: payment_method.description || compatibility_checkout_method?.detail || '',
    action,
    availability: payment_method.enabled ? 'available' : 'closed',
    provider,
    channel,
    session_kind: resolveBillingCheckoutMethodSessionKind(
      channel,
      compatibility_checkout_method?.session_kind,
    ),
    session_reference: sessionReference,
    qr_code_payload: latest_payment_attempt?.payment_method_id === payment_method.payment_method_id
      ? (latest_payment_attempt.qr_code_payload ?? compatibility_checkout_method?.qr_code_payload ?? null)
      : (compatibility_checkout_method?.qr_code_payload ?? null),
    webhook_verification: callbackStrategy || compatibility_checkout_method?.webhook_verification || 'manual',
    supports_refund: hasBillingCapability(payment_method, 'refund')
      || Boolean(compatibility_checkout_method?.supports_refund),
    supports_partial_refund: hasBillingCapability(payment_method, 'partial_refund')
      || Boolean(compatibility_checkout_method?.supports_partial_refund),
    recommended: payment_method.payment_method_id === selected_payment_method_id
      || hasBillingCapability(payment_method, 'recommended')
      || Boolean(compatibility_checkout_method?.recommended),
    supports_webhook: supportsWebhook || Boolean(compatibility_checkout_method?.supports_webhook),
  };
}

export function buildBillingCheckoutMethods(input: {
  order: PortalCommerceOrder;
  checkout_session: PortalCommerceCheckoutSession;
  payment_methods: PaymentMethodRecord[];
  latest_payment_attempt: CommercePaymentAttemptRecord | null;
  selected_payment_method: PaymentMethodRecord | null;
}): PortalCommerceCheckoutSessionMethod[] {
  const {
    order,
    checkout_session,
    payment_methods,
    latest_payment_attempt,
    selected_payment_method,
  } = input;
  if (order.status !== 'pending_payment' || checkout_session.session_status !== 'open') {
    return checkout_session.methods;
  }

  const selectedPaymentMethodId = selected_payment_method?.payment_method_id
    ?? order.payment_method_id
    ?? latest_payment_attempt?.payment_method_id
    ?? null;
  const formalMethods = payment_methods
    .map((payment_method) => {
      const compatibilityCheckoutMethod = findMatchingCompatibilityCheckoutMethod(
        checkout_session,
        payment_method,
      );
      const formalMethod = buildFormalBillingCheckoutMethod({
        order,
        payment_method,
        latest_payment_attempt,
        selected_payment_method_id: selectedPaymentMethodId,
        compatibility_checkout_method: compatibilityCheckoutMethod,
      });

      if (
        formalMethod.action === 'settle_order'
        && compatibilityCheckoutMethod
        && compatibilityCheckoutMethod.action === 'settle_order'
      ) {
        return null;
      }

      return formalMethod;
    })
    .filter((method): method is PortalCommerceCheckoutSessionMethod => method !== null);
  const formalMethodKeys = new Set(formalMethods.map((method) => buildBillingCheckoutMethodKey(method)));
  const compatibilityFallbackMethods = checkout_session.methods.filter((method) => (
    !formalMethodKeys.has(buildBillingCheckoutMethodKey(method))
  ));

  return [...compatibilityFallbackMethods, ...formalMethods];
}

function compareBillingPaymentHistoryRows(
  left: BillingPaymentHistoryRow,
  right: BillingPaymentHistoryRow,
): number {
  const leftKindRank = left.row_kind === 'refunded_order_state' ? 0 : 1;
  const rightKindRank = right.row_kind === 'refunded_order_state' ? 0 : 1;

  return right.received_at_ms - left.received_at_ms
    || leftKindRank - rightKindRank
    || left.order_id.localeCompare(right.order_id)
    || left.id.localeCompare(right.id);
}

function resolveBillingHistoryProvider(
  source: BillingPaymentHistorySource,
  paymentEventProvider?: string | null,
): string {
  const directProvider = paymentEventProvider?.trim();
  if (directProvider) {
    return directProvider;
  }

  const paymentAttemptProvider = source.latest_payment_attempt?.provider?.trim();
  if (paymentAttemptProvider) {
    return paymentAttemptProvider;
  }

  const paymentMethodProvider = source.selected_payment_method?.provider?.trim();
  if (paymentMethodProvider) {
    return paymentMethodProvider;
  }

  return source.compatibility_checkout_session?.provider ?? '';
}

function resolveBillingCheckoutReference(
  source: BillingPaymentHistorySource,
): string | null {
  return source.latest_payment_attempt?.provider_reference
    ?? source.latest_payment_attempt?.provider_checkout_session_id
    ?? source.compatibility_checkout_session?.reference
    ?? null;
}

function buildPaymentEventHistoryRow(
  source: BillingPaymentHistorySource,
  event: BillingPaymentHistorySource['payment_events'][number],
): BillingPaymentHistoryRow {
  return {
    row_kind: 'payment_event',
    id: event.payment_event_id,
    order_id: source.order.order_id,
    target_name: source.order.target_name,
    target_kind: source.order.target_kind,
    payable_price_label: source.order.payable_price_label,
    order_status: source.order.status,
    order_status_after: event.order_status_after ?? null,
    provider: resolveBillingHistoryProvider(source, event.provider),
    event_type: event.event_type,
    payment_event_id: event.payment_event_id,
    provider_event_id: event.provider_event_id ?? null,
    payment_method_name: source.selected_payment_method?.display_name ?? null,
    processing_status: event.processing_status,
    processing_message: event.processing_message ?? null,
    checkout_reference: resolveBillingCheckoutReference(source),
    checkout_session_status: source.compatibility_checkout_session?.session_status ?? null,
    guidance: source.compatibility_checkout_session?.guidance ?? null,
    received_at_ms: event.received_at_ms,
    processed_at_ms: event.processed_at_ms ?? null,
  };
}

function hasRefundPaymentEvent(source: BillingPaymentHistorySource): boolean {
  return source.payment_events.some((event) => event.event_type === 'refunded');
}

function buildRefundedOrderStateRow(
  source: BillingPaymentHistorySource,
): BillingPaymentHistoryRow {
  const observedAtMs = Math.max(
    source.order.updated_at_ms ?? 0,
    source.latest_payment_event?.processed_at_ms ?? 0,
    source.latest_payment_event?.received_at_ms ?? 0,
  );

  return {
    row_kind: 'refunded_order_state',
    id: `refund-state:${source.order.order_id}`,
    order_id: source.order.order_id,
    target_name: source.order.target_name,
    target_kind: source.order.target_kind,
    payable_price_label: source.order.payable_price_label,
    order_status: source.order.status,
    order_status_after: 'refunded',
    provider: resolveBillingHistoryProvider(source),
    event_type: 'refunded',
    payment_event_id: null,
    provider_event_id: null,
    payment_method_name: source.selected_payment_method?.display_name ?? null,
    processing_status: null,
    processing_message: null,
    checkout_reference: resolveBillingCheckoutReference(source),
    checkout_session_status: source.compatibility_checkout_session?.session_status ?? null,
    guidance: source.compatibility_checkout_session?.guidance ?? null,
    received_at_ms: observedAtMs,
    processed_at_ms: null,
  };
}

export function buildBillingPaymentHistory(
  sources: BillingPaymentHistorySource[],
): BillingPaymentHistoryRow[] {
  const rows: BillingPaymentHistoryRow[] = [];

  for (const source of sources) {
    for (const event of source.payment_events) {
      rows.push(buildPaymentEventHistoryRow(source, event));
    }

    if (source.order.status === 'refunded' && !hasRefundPaymentEvent(source)) {
      rows.push(buildRefundedOrderStateRow(source));
    }
  }

  return rows.sort(compareBillingPaymentHistoryRows);
}

export function buildBillingRefundHistory(
  sources: BillingPaymentHistorySource[],
): BillingPaymentHistoryRow[] {
  return buildBillingPaymentHistory(sources).filter((row) => row.event_type === 'refunded');
}

function buildDailyUsageSeries(usageRecords: UsageRecord[]): number[] {
  const daily = new Map<string, number>();

  for (const record of usageRecords) {
    if (!record.created_at_ms) {
      continue;
    }

    const key = new Date(record.created_at_ms).toISOString().slice(0, 10);
    daily.set(key, (daily.get(key) ?? 0) + record.units);
  }

  return [...daily.entries()]
    .sort((left, right) => left[0].localeCompare(right[0]))
    .map(([, units]) => units);
}

function exponentialMovingAverage(values: number[], alpha = 0.45): number | null {
  if (!values.length) {
    return null;
  }

  let smoothed = values[0];
  for (let index = 1; index < values.length; index += 1) {
    smoothed = alpha * values[index] + (1 - alpha) * smoothed;
  }

  return smoothed;
}

function estimateDailyUnits(
  summary: ProjectBillingSummary,
  usageRecords: UsageRecord[],
): number | null {
  const smoothedDailyUnits = exponentialMovingAverage(buildDailyUsageSeries(usageRecords));
  if (smoothedDailyUnits && Number.isFinite(smoothedDailyUnits)) {
    return Math.max(1, Math.round(smoothedDailyUnits));
  }

  if (summary.used_units <= 0) {
    return null;
  }

  return Math.max(1, Math.ceil(summary.used_units / 30));
}

function buildRunway(
  summary: ProjectBillingSummary,
  usageRecords: UsageRecord[],
): BillingRecommendation['runway'] {
  const daily_units = estimateDailyUnits(summary, usageRecords);

  if (summary.exhausted) {
    return {
      label: translatePortalText('0 days'),
      detail: translatePortalText(
        'Visible quota is already exhausted, so the workspace needs an immediate recharge or plan change before additional traffic is expected.',
      ),
      projected_days: 0,
      daily_units,
    };
  }

  if (summary.remaining_units === null || summary.remaining_units === undefined) {
    return {
      label: translatePortalText('Unlimited'),
      detail: translatePortalText(
        'The current billing summary exposes no visible quota ceiling, so the portal treats runway as unlimited for this workspace.',
      ),
      projected_days: null,
      daily_units,
    };
  }

  if (!daily_units) {
    return {
      label: translatePortalText('Needs first traffic signal'),
      detail: translatePortalText(
        'There is not enough recorded usage yet to project a meaningful burn pace. Send live traffic, then revisit billing decisions.',
      ),
      projected_days: null,
      daily_units: null,
    };
  }

  const projected_days = Math.floor(summary.remaining_units / daily_units);
  const label = projected_days < 1
    ? translatePortalText('< 1 day')
    : translatePortalText('{days} days', { days: projected_days });

  return {
    label,
    detail: translatePortalText(
      'Estimated from an exponentially smoothed burn pace of {units} token units per day.',
      { units: formatUnits(daily_units) },
    ),
    projected_days,
    daily_units,
  };
}

function buildRecommendedBundle(
  summary: ProjectBillingSummary,
  plan: SubscriptionPlan | null,
  pack: RechargePack | null,
): BillingRecommendation['bundle'] {
  if (!plan && !pack) {
    return {
      title: translatePortalText('Billing catalog unavailable'),
      detail: translatePortalText(
        'The portal could not build a plan-plus-pack recommendation from the current seed catalog.',
      ),
    };
  }

  if (summary.exhausted) {
    return {
      title: translatePortalText('{plan} + {pack}', {
        plan: plan?.name ?? translatePortalText('Subscription'),
        pack: pack?.label ?? translatePortalText('Recharge pack'),
      }),
      detail: translatePortalText(
        'The workspace needs both immediate runway recovery and a steadier monthly posture, so the portal recommends a plan and a recharge together.',
      ),
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000) {
    return {
      title: translatePortalText('{plan} with {pack} as buffer', {
        plan: plan?.name ?? translatePortalText('Subscription'),
        pack: pack?.label ?? translatePortalText('Recharge pack'),
      }),
      detail: translatePortalText(
        'Current quota is still active, but remaining headroom is tight enough that a plan-plus-buffer path is the lowest-friction next move.',
      ),
    };
  }

  return {
    title: translatePortalText('{plan} as the next growth step', {
      plan: plan?.name ?? translatePortalText('Subscription'),
    }),
    detail: translatePortalText(
      'The workspace is stable today, so the recommended bundle focuses on the cleanest subscription path while keeping the top-up pack available only if demand spikes.',
    ),
  };
}

export function recommendBillingChange(
  summary: ProjectBillingSummary,
  plans: SubscriptionPlan[],
  packs: RechargePack[],
  usageRecords: UsageRecord[] = [],
): BillingRecommendation {
  const runway = buildRunway(summary, usageRecords);
  const projectedMonthlyUnits = runway.daily_units
    ? runway.daily_units * 30
    : Math.max(summary.used_units, 1);
  const recommendedPlan = plans.length
    ? (plans.find((plan) => plan.included_units >= projectedMonthlyUnits) ?? plans[plans.length - 1])
    : null;
  const recommendedPack = packs.length
    ? (packs.find((pack) => pack.points >= Math.max(10_000, Math.round(projectedMonthlyUnits / 4))) ??
      packs[packs.length - 1])
    : null;
  const bundle = buildRecommendedBundle(summary, recommendedPlan, recommendedPack);

  if (summary.exhausted && recommendedPlan && recommendedPack) {
    return {
      title: translatePortalText('Quota is exhausted'),
      detail: translatePortalText(
        'Move to {plan} or add {pack} to restore headroom immediately.',
        {
          plan: recommendedPlan.name,
          pack: recommendedPack.label,
        },
      ),
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000 && recommendedPlan && recommendedPack) {
    return {
      title: translatePortalText('Headroom is getting tight'),
      detail: translatePortalText(
        'Add {pack} for near-term coverage, or move to {plan} for a steadier monthly posture.',
        {
          pack: recommendedPack.label,
          plan: recommendedPlan.name,
        },
      ),
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  return {
    title: recommendedPlan
      ? translatePortalText('Current workspace is stable')
      : translatePortalText('Billing catalog unavailable'),
    detail: recommendedPlan
      ? translatePortalText(
        'Based on a projected monthly demand of {units} units, {plan} is the cleanest next subscription step when traffic grows.',
        {
          units: formatUnits(projectedMonthlyUnits),
          plan: recommendedPlan.name,
        },
      )
      : translatePortalText('The portal could not load a live commerce catalog for this workspace.'),
    plan: recommendedPlan,
    pack: recommendedPack,
    runway,
    bundle,
  };
}

export function isRecommendedPlan(
  plan: SubscriptionPlan,
  recommendation: BillingRecommendation,
): boolean {
  return recommendation.plan?.id === plan.id;
}

export function isRecommendedPack(
  pack: RechargePack,
  recommendation: BillingRecommendation,
): boolean {
  return recommendation.pack?.id === pack.id;
}

function sortCapabilityMix(
  items: BillingEventCapabilitySummary[],
): BillingEventCapabilitySummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || left.capability.localeCompare(right.capability),
    );
}

function sortGroupChargeback(
  items: BillingEventGroupSummary[],
): BillingEventGroupSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || (left.api_key_group_id ?? '').localeCompare(right.api_key_group_id ?? ''),
    );
}

function sortAccountingModeMix(
  items: BillingEventAccountingModeSummary[],
): BillingEventAccountingModeSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || left.accounting_mode.localeCompare(right.accounting_mode),
    );
}

function sortRecentEvents(events: BillingEventRecord[]): BillingEventRecord[] {
  return [...events].sort((left, right) =>
    right.created_at_ms - left.created_at_ms
    || right.customer_charge - left.customer_charge
    || right.units - left.units
    || left.event_id.localeCompare(right.event_id),
  );
}

export function buildBillingEventAnalytics(
  summary: BillingEventSummary,
  events: BillingEventRecord[],
  limits: {
    capabilities?: number;
    groups?: number;
    accounting_modes?: number;
    recent_events?: number;
  } = {},
): BillingEventAnalyticsViewModel {
  const capabilityLimit = limits.capabilities ?? 6;
  const groupLimit = limits.groups ?? 6;
  const accountingModeLimit = limits.accounting_modes ?? 3;
  const recentEventLimit = limits.recent_events ?? 6;

  return {
    totals: {
      total_events: summary.total_events,
      total_request_count: summary.total_request_count,
      total_tokens: summary.total_tokens,
      total_image_count: summary.total_image_count,
      total_audio_seconds: summary.total_audio_seconds,
      total_video_seconds: summary.total_video_seconds,
      total_music_seconds: summary.total_music_seconds,
      total_upstream_cost: summary.total_upstream_cost,
      total_customer_charge: summary.total_customer_charge,
    },
    top_capabilities: sortCapabilityMix(summary.capabilities).slice(0, capabilityLimit),
    group_chargeback: sortGroupChargeback(summary.groups).slice(0, groupLimit),
    accounting_mode_mix: sortAccountingModeMix(summary.accounting_modes).slice(
      0,
      accountingModeLimit,
    ),
    recent_events: sortRecentEvents(events).slice(0, recentEventLimit),
    routing_evidence: {
      events_with_profile: events.filter((event) => event.applied_routing_profile_id).length,
      events_with_compiled_snapshot: events.filter(
        (event) => event.compiled_routing_snapshot_id,
      ).length,
      events_with_fallback_reason: events.filter((event) => event.fallback_reason).length,
    },
  };
}

export function buildBillingEventCsvDocument(
  events: BillingEventRecord[],
): BillingEventCsvDocument {
  return {
    headers: [
      'event_id',
      'tenant_id',
      'project_id',
      'api_key_group_id',
      'capability',
      'route_key',
      'usage_model',
      'provider_id',
      'accounting_mode',
      'operation_kind',
      'modality',
      'api_key_hash',
      'channel_id',
      'reference_id',
      'latency_ms',
      'units',
      'request_count',
      'input_tokens',
      'output_tokens',
      'total_tokens',
      'cache_read_tokens',
      'cache_write_tokens',
      'image_count',
      'audio_seconds',
      'video_seconds',
      'music_seconds',
      'upstream_cost',
      'customer_charge',
      'applied_routing_profile_id',
      'compiled_routing_snapshot_id',
      'fallback_reason',
      'created_at',
    ],
    rows: events.map((event) => [
      event.event_id,
      event.tenant_id,
      event.project_id,
      event.api_key_group_id ?? '',
      event.capability,
      event.route_key,
      event.usage_model,
      event.provider_id,
      event.accounting_mode,
      event.operation_kind,
      event.modality,
      event.api_key_hash ?? '',
      event.channel_id ?? '',
      event.reference_id ?? '',
      event.latency_ms ?? '',
      event.units,
      event.request_count,
      event.input_tokens,
      event.output_tokens,
      event.total_tokens,
      event.cache_read_tokens,
      event.cache_write_tokens,
      event.image_count,
      event.audio_seconds,
      event.video_seconds,
      event.music_seconds,
      event.upstream_cost.toFixed(4),
      event.customer_charge.toFixed(4),
      event.applied_routing_profile_id ?? '',
      event.compiled_routing_snapshot_id ?? '',
      event.fallback_reason ?? '',
      new Date(event.created_at_ms).toISOString(),
    ]),
  };
}
