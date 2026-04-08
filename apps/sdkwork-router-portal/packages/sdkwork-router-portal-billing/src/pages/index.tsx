import { useDeferredValue, useEffect, useState } from 'react';
import type { ChangeEvent, FormEvent, ReactNode } from 'react';

import {
  formatCurrency,
  formatDateTime,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  DataTable,
} from 'sdkwork-router-portal-commons/framework/display';
import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import {
  FilterBar,
  FilterBarActions,
  FilterBarSection,
  FilterField,
  SearchInput,
  SettingsField,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import { WorkspacePanel } from 'sdkwork-router-portal-commons/framework/workspace';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  BillingAccountingMode,
  BillingEventCapabilitySummary,
  BillingEventRecord,
  BillingEventSummary,
  CommercePaymentAttemptRecord,
  CommercialAccountBalanceSnapshot,
  CommercialAccountBenefitLotRecord,
  CommercialAccountHoldRecord,
  CommercialAccountSummary,
  CommercialPricingPlanRecord,
  CommercialPricingRateRecord,
  CommercialRequestSettlementRecord,
  PortalCommerceCheckoutSession,
  PortalCommerceCheckoutMethodChannel,
  PortalCommerceCheckoutSessionMethod,
  PortalCommerceCheckoutSessionStatus,
  PortalCommerceMembership,
  PortalCommerceReconciliationSummary,
  PortalCommerceOrder,
  PortalCommerceOrderStatus,
  PortalCommercePaymentEventType,
  PortalCommercePaymentProvider,
  PortalCommerceQuoteKind,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import { BillingRecommendationCard } from '../components';
import {
  cancelBillingOrder,
  createBillingPaymentAttempt,
  createBillingOrder,
  getBillingCheckoutDetail,
  loadBillingPageData,
  previewBillingCheckout,
  sendBillingPaymentEvent,
  settleBillingOrder,
} from '../repository';
import {
  buildBillingEventAnalytics,
  buildBillingCheckoutPresentation,
  buildBillingCheckoutLaunchDecision,
  buildBillingEventCsvDocument,
  isRecommendedPack,
  isRecommendedPlan,
  recommendBillingChange,
} from '../services';
import type {
  BillingCheckoutDetail,
  BillingEventAnalyticsViewModel,
  BillingCheckoutPreview,
  BillingPageData,
  BillingPaymentHistoryRow,
  PortalBillingPageProps,
} from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

const emptyBillingEventSummary: BillingEventSummary = {
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
};

type BillingSelection =
  | {
      kind: 'subscription_plan';
      target: SubscriptionPlan;
    }
  | {
      kind: 'recharge_pack';
      target: RechargePack;
    };

type OrderWorkbenchLane = 'all' | 'pending_payment' | 'failed' | 'timeline';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;
type CsvValue = string | number | boolean | null | undefined;
type BillingCheckoutPresentationView = ReturnType<typeof buildBillingCheckoutPresentation>;

const BillingEventsAuditTable = DataTable;
const PaymentHistoryAuditTable = DataTable;
const RefundHistoryAuditTable = DataTable;

function csvValue(value: CsvValue): string {
  const normalized = value == null ? '' : String(value);
  return `"${normalized.replaceAll('"', '""')}"`;
}

function downloadCsv(
  filename: string,
  headers: string[],
  rows: Array<CsvValue[]>,
): void {
  const contents = [
    headers.map(csvValue).join(','),
    ...rows.map((row) => row.map(csvValue).join(',')),
  ].join('\n');
  const blob = new Blob([contents], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

function titleCaseToken(value: string): string {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length <= 3
        ? segment.toUpperCase()
        : `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`,
    )
    .join(' ');
}

function resolveMembershipStatusLabel(
  status: string | null | undefined,
  t: TranslateFn,
): string {
  const normalized = status?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'active':
      return t('Active');
    case 'inactive':
      return t('Inactive');
    case 'canceled':
    case 'cancelled':
      return t('Canceled');
    case 'past_due':
    case 'past-due':
      return t('Past due');
    case 'grace_period':
    case 'grace-period':
      return t('Grace period');
    case 'paused':
      return t('Paused');
    default:
      return status?.trim() ? titleCaseToken(status) : t('Inactive');
  }
}

function targetKindLabel(
  kind: BillingSelection['kind'] | PortalCommerceQuoteKind,
  t: TranslateFn,
): string {
  switch (kind) {
    case 'subscription_plan':
      return t('Subscription plan');
    case 'recharge_pack':
      return t('Recharge pack');
    default:
      return t('Offer');
  }
}

function orderStatusLabel(status: PortalCommerceOrderStatus, t: TranslateFn): string {
  switch (status) {
    case 'pending_payment':
      return t('Payment pending');
    case 'fulfilled':
      return t('Fulfilled');
    case 'canceled':
      return t('Canceled');
    case 'failed':
      return t('Failed');
    case 'refunded':
      return t('Refunded');
    default:
      return t('Status');
  }
}

function checkoutSessionStatusLabel(
  status: PortalCommerceCheckoutSessionStatus,
  t: TranslateFn,
): string {
  switch (status) {
    case 'open':
      return t('Open');
    case 'settled':
      return t('Settled');
    case 'canceled':
      return t('Canceled');
    case 'failed':
      return t('Failed');
    case 'refunded':
      return t('Refunded');
    case 'not_required':
      return t('Not required');
    case 'closed':
      return t('Closed');
    default:
      return t('Status');
  }
}

function checkoutMethodActionLabel(
  action: PortalCommerceCheckoutSessionMethod['action'],
  t: TranslateFn,
): string {
  switch (action) {
    case 'settle_order':
      return t('Settle order');
    case 'cancel_order':
      return t('Cancel order');
    case 'provider_handoff':
      return t('Checkout access');
    default:
      return t('Actions');
  }
}

function checkoutMethodProviderLabel(
  provider: PortalCommercePaymentProvider | string,
  t: TranslateFn,
): string {
  switch (provider) {
    case 'manual_lab':
      return t('Manual lab');
    case 'stripe':
      return t('Stripe');
    case 'alipay':
      return t('Alipay');
    case 'wechat_pay':
      return t('WeChat Pay');
    case 'no_payment_required':
      return t('No payment required');
    default:
      return t('Payment method');
  }
}

function checkoutMethodChannelLabel(
  channel: PortalCommerceCheckoutMethodChannel,
  t: TranslateFn,
): string {
  switch (channel) {
    case 'operator_settlement':
      return t('Manual settlement');
    case 'hosted_checkout':
      return t('Hosted checkout');
    case 'scan_qr':
      return t('Scan QR');
    default:
      return t('Payment channel');
  }
}

function checkoutMethodSessionKindLabel(
  sessionKind: PortalCommerceCheckoutSessionMethod['session_kind'],
  t: TranslateFn,
): string {
  switch (sessionKind) {
    case 'operator_action':
      return t('Manual step');
    case 'hosted_checkout':
      return t('Hosted checkout flow');
    case 'qr_code':
      return t('QR checkout flow');
    default:
      return t('Checkout flow');
  }
}

function checkoutMethodVerificationLabel(value: string | null | undefined, t: TranslateFn): string {
  const normalized = value?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'manual':
      return t('Manual confirmation');
    case 'webhook':
    case 'webhook_signed':
      return t('Signed callback check');
    case 'stripe_signature':
      return t('Stripe signature check');
    case 'alipay_rsa_sha256':
      return t('Alipay RSA-SHA256 check');
    case 'wechatpay_rsa_sha256':
      return t('WeChat Pay RSA-SHA256 check');
    default:
      return value?.trim() ? titleCaseToken(value) : t('Verification method');
  }
}

function checkoutMethodAvailabilityLabel(
  availability: PortalCommerceCheckoutSessionMethod['availability'],
  t: TranslateFn,
): string {
  switch (availability) {
    case 'available':
      return t('Available');
    case 'planned':
      return t('Planned');
    case 'closed':
      return t('Closed');
    default:
      return t('Status');
  }
}

function checkoutMethodAvailabilityTone(
  availability: PortalCommerceCheckoutSessionMethod['availability'],
): 'success' | 'warning' | 'secondary' {
  switch (availability) {
    case 'available':
      return 'success';
    case 'planned':
      return 'warning';
    case 'closed':
      return 'secondary';
    default:
      return 'secondary';
  }
}

function paymentAttemptStatusLabel(
  status: CommercePaymentAttemptRecord['status'],
  t: TranslateFn,
): string {
  const normalized = status?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'created':
      return t('Created');
    case 'provider_pending':
      return t('Pending');
    case 'requires_action':
      return t('Action required');
    case 'processing':
      return t('Processing');
    case 'succeeded':
      return t('Succeeded');
    case 'failed':
      return t('Failed');
    case 'canceled':
    case 'cancelled':
      return t('Canceled');
    case 'expired':
      return t('Expired');
    case 'partially_refunded':
    case 'partially-refunded':
      return t('Partially refunded');
    case 'refunded':
      return t('Refunded');
    default:
      return status?.trim() ? titleCaseToken(status) : t('Status');
  }
}

function paymentAttemptStatusTone(
  status: CommercePaymentAttemptRecord['status'],
): 'default' | 'secondary' | 'success' | 'warning' {
  const normalized = status?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'requires_action':
    case 'provider_pending':
    case 'processing':
      return 'default';
    case 'succeeded':
      return 'success';
    case 'failed':
    case 'canceled':
    case 'cancelled':
    case 'expired':
      return 'warning';
    case 'refunded':
    case 'partially_refunded':
    case 'partially-refunded':
    case 'created':
    default:
      return 'secondary';
  }
}

function paymentAttemptReference(attempt: CommercePaymentAttemptRecord): string {
  return attempt.provider_reference?.trim()
    || attempt.provider_checkout_session_id?.trim()
    || attempt.provider_payment_intent_id?.trim()
    || attempt.payment_attempt_id;
}

function buildProviderEventReplayId(
  orderId: string,
  eventType: PortalCommercePaymentEventType,
  method: PortalCommerceCheckoutSessionMethod,
): string {
  const normalizedOrderId = orderId.trim().replaceAll(/[^a-zA-Z0-9]+/g, '_');
  return `portal_replay_${method.provider}_${method.id}_${eventType}_${normalizedOrderId}`;
}

function membershipDescription(
  membership: PortalCommerceMembership | null,
  t: TranslateFn,
): string {
  if (membership) {
    return t(
      '{planName} is the active workspace membership and defines the current subscription entitlement baseline.',
      {
        planName: membership.plan_name,
      },
    );
  }

  return t(
    'No active membership is recorded yet. Complete a subscription checkout to activate monthly entitlement posture.',
  );
}

function orderMatchesSearch(order: PortalCommerceOrder, search: string, t: TranslateFn): boolean {
  if (!search) {
    return true;
  }

  const haystack = [
    order.order_id,
    order.target_name,
    order.target_kind,
    targetKindLabel(order.target_kind, t),
    order.status,
    orderStatusLabel(order.status, t),
    order.applied_coupon_code ?? '',
    order.payable_price_label,
  ]
    .join(' ')
    .toLowerCase();

  return haystack.includes(search);
}

function selectionLabel(selection: BillingSelection | null): string | null {
  if (!selection) {
    return null;
  }

  return selection.kind === 'subscription_plan'
    ? selection.target.name
    : selection.target.label;
}

function isPendingPaymentOrder(order: PortalCommerceOrder): boolean {
  return order.status === 'pending_payment';
}

function matchesOrderLane(
  order: PortalCommerceOrder,
  lane: OrderWorkbenchLane,
): boolean {
  switch (lane) {
    case 'pending_payment':
      return isPendingPaymentOrder(order);
    case 'failed':
      return order.status === 'failed';
    case 'timeline':
      return !isPendingPaymentOrder(order);
    default:
      return true;
  }
}

function orderWorkbenchDetail(lane: OrderWorkbenchLane, t: TranslateFn): string {
  switch (lane) {
    case 'pending_payment':
      return t(
        'Pending payment queue keeps unpaid or unfulfilled orders visible until payment completes or the order leaves the queue.',
      );
    case 'failed':
      return t(
        'Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up.',
      );
    case 'timeline':
      return t('Order timeline shows completed or closed outcomes after checkout attempts resolve.');
    default:
      return t(
        'Switch between pending payment queue, failed payment, and order timeline without leaving the main order table.',
      );
  }
}

function checkoutModeLabel(
  session: PortalCommerceCheckoutSession | null,
  t: TranslateFn,
): string {
  switch (session?.mode) {
    case 'operator_settlement':
      return t('Manual settlement');
    case 'instant_fulfillment':
      return t('Instant fulfillment');
    default:
      return t('Closed checkout');
  }
}

function hasProviderHandoff(methods: PortalCommerceCheckoutSessionMethod[]): boolean {
  return methods.some((method) => method.supports_webhook);
}

function preferredProviderCallbackMethod(
  methods: PortalCommerceCheckoutSessionMethod[],
): PortalCommerceCheckoutSessionMethod | null {
  const callbackMethods = methods.filter((method) => method.supports_webhook);
  return callbackMethods.find((method) => method.recommended) ?? callbackMethods[0] ?? null;
}

function orderStatusTone(
  status: PortalCommerceOrder['status'],
): 'secondary' | 'default' | 'success' | 'warning' {
  switch (status) {
    case 'fulfilled':
      return 'success';
    case 'failed':
      return 'warning';
    case 'pending_payment':
      return 'default';
    default:
      return 'secondary';
  }
}

function paymentEventTypeLabel(
  eventType: PortalCommercePaymentEventType,
  t: TranslateFn,
): string {
  switch (eventType) {
    case 'settled':
      return t('Settled');
    case 'failed':
      return t('Failed');
    case 'canceled':
      return t('Canceled');
    case 'refunded':
      return t('Refunded');
    default:
      return t('Status');
  }
}

function paymentHistoryRowKindLabel(
  rowKind: BillingPaymentHistoryRow['row_kind'],
  t: TranslateFn,
): string {
  switch (rowKind) {
    case 'payment_event':
      return t('Payment event');
    case 'refunded_order_state':
      return t('Order refund state');
    default:
      return t('Timeline');
  }
}

function paymentHistoryRailCell(
  row: BillingPaymentHistoryRow,
  t: TranslateFn,
): ReactNode {
  const providerLabel = row.provider
    ? checkoutMethodProviderLabel(row.provider, t)
    : t('Not recorded');

  if (!row.payment_method_name) {
    return providerLabel;
  }

  return (
    <div className="grid gap-1">
      <strong className="text-zinc-950 dark:text-zinc-50">{providerLabel}</strong>
      <span className="text-xs text-zinc-500 dark:text-zinc-400">
        {row.payment_method_name}
      </span>
    </div>
  );
}

function paymentProcessingStatusLabel(
  status: BillingPaymentHistoryRow['processing_status'],
  t: TranslateFn,
): string {
  switch (status) {
    case 'received':
      return t('Received');
    case 'processed':
      return t('Processed');
    case 'ignored':
      return t('Ignored');
    case 'rejected':
      return t('Rejected');
    case 'failed':
      return t('Failed');
    default:
      return t('Not recorded');
  }
}

function paymentProcessingStatusTone(
  status: BillingPaymentHistoryRow['processing_status'],
): 'default' | 'secondary' | 'success' | 'warning' {
  switch (status) {
    case 'processed':
      return 'success';
    case 'failed':
    case 'rejected':
      return 'warning';
    case 'received':
      return 'default';
    default:
      return 'secondary';
  }
}

function accountingModeLabel(mode: BillingAccountingMode, t: TranslateFn): string {
  switch (mode) {
    case 'platform_credit':
      return t('Platform credit');
    case 'byok':
      return t('BYOK');
    case 'passthrough':
      return t('Passthrough');
    default:
      return t('Accounting mode');
  }
}

function accountingModeTone(
  mode: BillingAccountingMode,
): 'default' | 'secondary' | 'success' | 'warning' {
  switch (mode) {
    case 'platform_credit':
      return 'default';
    case 'byok':
      return 'success';
    case 'passthrough':
      return 'warning';
    default:
      return 'secondary';
  }
}

function commercialPricingChargeUnitLabel(
  chargeUnit: CommercialPricingRateRecord['charge_unit'],
  t: TranslateFn,
): string {
  switch (chargeUnit) {
    case 'input_token':
      return t('Input token');
    case 'output_token':
      return t('Output token');
    case 'cache_read_token':
      return t('Cache read token');
    case 'cache_write_token':
      return t('Cache write token');
    case 'request':
      return t('Request');
    case 'image':
      return t('Image');
    case 'audio_second':
      return t('Audio second');
    case 'audio_minute':
      return t('Audio minute');
    case 'video_second':
      return t('Video second');
    case 'video_minute':
      return t('Video minute');
    case 'music_track':
      return t('Music track');
    case 'character':
      return t('Character');
    case 'storage_mb_day':
      return t('Storage MB day');
    case 'tool_call':
      return t('Tool call');
    case 'unit':
    default:
      return t('Unit');
  }
}

function commercialPricingMethodLabel(
  pricingMethod: CommercialPricingRateRecord['pricing_method'],
  t: TranslateFn,
): string {
  switch (pricingMethod) {
    case 'per_unit':
      return t('Per unit');
    case 'flat':
      return t('Flat');
    case 'step':
      return t('Step');
    case 'included_then_per_unit':
      return t('Included then per unit');
    default:
      return t('Billing method');
  }
}

function commercialPricingStatusTone(
  status: string | null | undefined,
): 'success' | 'warning' | 'secondary' {
  switch (status?.trim().toLowerCase()) {
    case 'active':
      return 'success';
    case 'draft':
    case 'planned':
      return 'warning';
    default:
      return 'secondary';
  }
}

function isCommercialPricingPlanEffectiveAt(
  plan: Pick<CommercialPricingPlanRecord, 'effective_from_ms' | 'effective_to_ms'>,
  nowMs: number,
): boolean {
  return plan.effective_from_ms <= nowMs
    && (plan.effective_to_ms == null || plan.effective_to_ms >= nowMs);
}

function selectPrimaryCommercialPricingPlan(
  pricingPlans: CommercialPricingPlanRecord[],
  nowMs: number,
): CommercialPricingPlanRecord | null {
  const comparePlans = (
    left: CommercialPricingPlanRecord,
    right: CommercialPricingPlanRecord,
  ): number => {
    const leftRank = left.status.trim().toLowerCase() === 'active'
      ? (isCommercialPricingPlanEffectiveAt(left, nowMs) ? 0 : 1)
      : 2;
    const rightRank = right.status.trim().toLowerCase() === 'active'
      ? (isCommercialPricingPlanEffectiveAt(right, nowMs) ? 0 : 1)
      : 2;

    return leftRank - rightRank
      || right.plan_version - left.plan_version
      || right.updated_at_ms - left.updated_at_ms
      || right.created_at_ms - left.created_at_ms
      || right.pricing_plan_id - left.pricing_plan_id;
  };

  return [...pricingPlans].sort(comparePlans)[0] ?? null;
}

function compareCommercialPricingRates(
  left: CommercialPricingRateRecord,
  right: CommercialPricingRateRecord,
): number {
  const leftStatusRank = left.status.trim().toLowerCase() === 'active' ? 0 : 1;
  const rightStatusRank = right.status.trim().toLowerCase() === 'active' ? 0 : 1;

  return leftStatusRank - rightStatusRank
    || right.priority - left.priority
    || right.updated_at_ms - left.updated_at_ms
    || right.created_at_ms - left.created_at_ms
    || right.pricing_rate_id - left.pricing_rate_id;
}

function isTokenPricingRate(rate: CommercialPricingRateRecord): boolean {
  switch (rate.charge_unit) {
    case 'input_token':
    case 'output_token':
    case 'cache_read_token':
    case 'cache_write_token':
      return true;
    default:
      return false;
  }
}

function isMediaPricingRate(rate: CommercialPricingRateRecord): boolean {
  switch (rate.charge_unit) {
    case 'image':
    case 'audio_second':
    case 'audio_minute':
    case 'video_second':
    case 'video_minute':
    case 'music_track':
      return true;
    default:
      return false;
  }
}

function commercialPricingDisplayUnit(
  rate: CommercialPricingRateRecord,
  t: TranslateFn,
): string {
  if (rate.display_price_unit.trim()) {
    return rate.display_price_unit;
  }

  switch (rate.charge_unit) {
    case 'input_token':
      return rate.quantity_step === 1_000_000
        ? t('USD / 1M input tokens')
        : t('USD / input token');
    case 'image':
      return t('USD / image');
    case 'music_track':
      return t('USD / music track');
    default:
      return t('{count} x {unit}', {
        count: formatUnits(rate.quantity_step),
        unit: commercialPricingChargeUnitLabel(rate.charge_unit, t).toLowerCase(),
      });
  }
}

function commercialPricingRuleSummary(
  rate: CommercialPricingRateRecord,
  t: TranslateFn,
): string {
  const details: string[] = [];

  if (rate.included_quantity > 0) {
    details.push(
      t('{count} included', {
        count: formatUnits(rate.included_quantity),
      }),
    );
  }

  if (rate.minimum_billable_quantity > 0) {
    details.push(
      t('Minimum quantity {count}', {
        count: formatUnits(rate.minimum_billable_quantity),
      }),
    );
  }

  if (rate.minimum_charge > 0) {
    details.push(
      t('Minimum charge {amount}', {
        amount: formatCurrency(rate.minimum_charge),
      }),
    );
  }

  if (rate.rounding_increment > 1) {
    details.push(
      t('Rounds by {count} ({mode})', {
        count: formatUnits(rate.rounding_increment),
        mode: titleCaseToken(rate.rounding_mode),
      }),
    );
  }

  return details.length
    ? details.join(' / ')
    : t('Settles directly from the configured billing method and charge unit.');
}

function groupChargebackLabel(groupId: string | null | undefined, t: TranslateFn): string {
  return groupId?.trim() ? groupId : t('Unassigned');
}

function billingCapabilitySignalLabel(
  capability: BillingEventCapabilitySummary,
  t: TranslateFn,
): string {
  const signals: string[] = [];

  if (capability.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(capability.total_tokens) }));
  }
  if (capability.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(capability.image_count) }));
  }
  if (capability.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(capability.audio_seconds) }));
  }
  if (capability.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(capability.video_seconds) }));
  }
  if (capability.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(capability.music_seconds) }));
  }

  return signals.length
    ? signals.join(' · ')
    : t('{count} requests', { count: formatUnits(capability.request_count) });
}

function billingEventSignalLabel(event: BillingEventRecord, t: TranslateFn): string {
  const signals: string[] = [];

  if (event.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(event.total_tokens) }));
  }
  if (event.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(event.image_count) }));
  }
  if (event.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(event.audio_seconds) }));
  }
  if (event.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(event.video_seconds) }));
  }
  if (event.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(event.music_seconds) }));
  }

  return signals.length
    ? signals.join(' · ')
    : t('{count} requests', { count: formatUnits(event.request_count) });
}

function capabilitySignalText(
  capability: BillingEventCapabilitySummary,
  t: TranslateFn,
): string {
  const signals: string[] = [];

  if (capability.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(capability.total_tokens) }));
  }
  if (capability.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(capability.image_count) }));
  }
  if (capability.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(capability.audio_seconds) }));
  }
  if (capability.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(capability.video_seconds) }));
  }
  if (capability.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(capability.music_seconds) }));
  }

  return signals.length
    ? signals.join(' / ')
    : t('{count} requests', { count: formatUnits(capability.request_count) });
}

function eventSignalText(event: BillingEventRecord, t: TranslateFn): string {
  const signals: string[] = [];

  if (event.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(event.total_tokens) }));
  }
  if (event.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(event.image_count) }));
  }
  if (event.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(event.audio_seconds) }));
  }
  if (event.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(event.video_seconds) }));
  }
  if (event.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(event.music_seconds) }));
  }

  return signals.length
    ? signals.join(' / ')
    : t('{count} requests', { count: formatUnits(event.request_count) });
}

function downloadBillingEventsCsv(events: BillingEventRecord[]): void {
  const document = buildBillingEventCsvDocument(events);
  downloadCsv('sdkwork-router-billing-events.csv', document.headers, document.rows);
}

function openPortalExternalUrl(href: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.open(href, '_blank', 'noopener,noreferrer');
}

function resolvePortalCheckoutReturnUrl(
  orderId: string,
  result: 'success' | 'cancel',
): string | null {
  if (typeof window === 'undefined') {
    return null;
  }

  const href = window.location?.href?.trim();
  if (!href) {
    return null;
  }

  try {
    const url = new URL(href);
    url.searchParams.set('billing_order_id', orderId);
    url.searchParams.set('checkout_result', result);
    return url.toString();
  } catch {
    return href;
  }
}

function supportsFormalProviderCheckoutLaunch(
  method: PortalCommerceCheckoutSessionMethod,
): boolean {
  return method.action === 'provider_handoff'
    && method.availability === 'available'
    && method.provider === 'stripe'
    && method.channel === 'hosted_checkout';
}

function providerCheckoutLaunchActionLabel(
  kind: 'resume_existing_attempt' | 'create_retry_attempt' | 'create_first_attempt',
  t: TranslateFn,
): string {
  switch (kind) {
    case 'resume_existing_attempt':
      return t('Resume checkout');
    case 'create_retry_attempt':
      return t('Retry with new attempt');
    case 'create_first_attempt':
    default:
      return t('Start checkout');
  }
}

function providerCheckoutLaunchDecisionDetail(
  kind: 'resume_existing_attempt' | 'create_retry_attempt' | 'create_first_attempt',
  providerLabel: string,
  t: TranslateFn,
): string {
  switch (kind) {
    case 'resume_existing_attempt':
      return t(
        'The latest {provider} checkout can still be resumed, so the workbench will reopen the existing checkout.',
        {
          provider: providerLabel,
        },
      );
    case 'create_retry_attempt':
      return t(
        'The latest {provider} attempt no longer has a reusable checkout link, so the workbench will create a fresh checkout attempt.',
        {
          provider: providerLabel,
        },
      );
    case 'create_first_attempt':
    default:
      return t(
        'No {provider} checkout attempt exists yet for this order. Start the first checkout now.',
        {
          provider: providerLabel,
        },
      );
  }
}

function checkoutPresentationStatusText(
  checkoutPresentation: BillingCheckoutPresentationView | null,
  compatibilityStatusLabel: string,
  t: TranslateFn,
): string {
  if (!checkoutPresentation?.status) {
    return compatibilityStatusLabel;
  }

  switch (checkoutPresentation.status_source) {
    case 'payment_attempt':
      return paymentAttemptStatusLabel(checkoutPresentation.status, t);
    case 'checkout_session':
      return compatibilityStatusLabel;
    case 'none':
    default:
      return t('Status');
  }
}

function checkoutPresentationGuidanceText(
  checkoutPresentation: BillingCheckoutPresentationView | null,
  t: TranslateFn,
): string {
  if (!checkoutPresentation) {
    return t('No checkout guidance is available for this order yet.');
  }

  switch (checkoutPresentation.guidance_source) {
    case 'payment_attempt_error':
    case 'checkout_session':
      return checkoutPresentation.guidance ?? t('No checkout guidance is available for this order yet.');
    case 'launch_decision':
      return providerCheckoutLaunchDecisionDetail(
        checkoutPresentation.launch_decision_kind ?? 'create_first_attempt',
        checkoutPresentation.provider
          ? checkoutMethodProviderLabel(checkoutPresentation.provider, t)
          : t('Payment method'),
        t,
      );
    case 'none':
    default:
      if (!checkoutPresentation.reference) {
        return t('No checkout guidance is available for this order yet.');
      }

      if (checkoutPresentation.provider && checkoutPresentation.channel) {
        return t('{reference} is the current {provider} / {channel} payment reference for this order.', {
          reference: checkoutPresentation.reference,
          provider: checkoutMethodProviderLabel(checkoutPresentation.provider, t),
          channel: checkoutMethodChannelLabel(checkoutPresentation.channel, t),
        });
      }

      return t('{reference} is the current checkout reference for this order.', {
        reference: checkoutPresentation.reference,
      });
  }
}

export function PortalBillingPage({ onNavigate }: PortalBillingPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [billingEventSummary, setBillingEventSummary] = useState<BillingEventSummary>(
    emptyBillingEventSummary,
  );
  const [billingEvents, setBillingEvents] = useState<BillingEventRecord[]>([]);
  const [usageRecords, setUsageRecords] = useState<UsageRecord[]>([]);
  const [plans, setPlans] = useState<SubscriptionPlan[]>([]);
  const [packs, setPacks] = useState<RechargePack[]>([]);
  const [orders, setOrders] = useState<PortalCommerceOrder[]>([]);
  const [paymentHistory, setPaymentHistory] = useState<BillingPaymentHistoryRow[]>([]);
  const [refundHistory, setRefundHistory] = useState<BillingPaymentHistoryRow[]>([]);
  const [paymentSimulationEnabled, setPaymentSimulationEnabled] = useState(false);
  const [membership, setMembership] = useState<PortalCommerceMembership | null>(null);
  const [commercialReconciliation, setCommercialReconciliation] =
    useState<PortalCommerceReconciliationSummary | null>(null);
  const [commercialAccount, setCommercialAccount] = useState<CommercialAccountSummary | null>(null);
  const [commercialBalance, setCommercialBalance] = useState<CommercialAccountBalanceSnapshot | null>(null);
  const [commercialBenefitLots, setCommercialBenefitLots] = useState<CommercialAccountBenefitLotRecord[]>([]);
  const [commercialHolds, setCommercialHolds] = useState<CommercialAccountHoldRecord[]>([]);
  const [commercialRequestSettlements, setCommercialRequestSettlements] = useState<CommercialRequestSettlementRecord[]>([]);
  const [commercialPricingPlans, setCommercialPricingPlans] = useState<CommercialPricingPlanRecord[]>([]);
  const [commercialPricingRates, setCommercialPricingRates] = useState<CommercialPricingRateRecord[]>([]);
  const [status, setStatus] = useState(t('Loading billing posture...'));
  const [searchQuery, setSearchQuery] = useState('');
  const [orderLane, setOrderLane] = useState<OrderWorkbenchLane>('all');
  const [checkoutOpen, setCheckoutOpen] = useState(false);
  const [checkoutSelection, setCheckoutSelection] = useState<BillingSelection | null>(null);
  const [couponCode, setCouponCode] = useState('');
  const [checkoutPreview, setCheckoutPreview] = useState<BillingCheckoutPreview | null>(null);
  const [checkoutStatus, setCheckoutStatus] = useState(
    t('Choose a plan or recharge path to price the next checkout.'),
  );
  const [previewLoading, setPreviewLoading] = useState(false);
  const [orderLoading, setOrderLoading] = useState(false);
  const [queueActionOrderId, setQueueActionOrderId] = useState<string | null>(null);
  const [queueActionType, setQueueActionType] = useState<'settle' | 'cancel' | null>(null);
  const [checkoutSession, setCheckoutSession] = useState<PortalCommerceCheckoutSession | null>(null);
  const [checkoutDetail, setCheckoutDetail] = useState<BillingCheckoutDetail | null>(null);
  const [checkoutSessionOrderId, setCheckoutSessionOrderId] = useState<string | null>(null);
  const [providerCheckoutMethodId, setProviderCheckoutMethodId] = useState<string | null>(null);
  const [providerCallbackMethodId, setProviderCallbackMethodId] = useState<string | null>(null);
  const [providerEventOrderId, setProviderEventOrderId] = useState<string | null>(null);
  const [providerEventType, setProviderEventType] = useState<PortalCommercePaymentEventType | null>(
    null,
  );
  const [checkoutSessionStatus, setCheckoutSessionStatus] = useState(
    t('Open the checkout workbench from Pending payment queue to inspect the selected payment method.'),
  );
  const [checkoutSessionLoading, setCheckoutSessionLoading] = useState(false);
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  const recommendation = recommendBillingChange(summary, plans, packs, usageRecords);
  const billingEventAnalytics: BillingEventAnalyticsViewModel = buildBillingEventAnalytics(
    billingEventSummary,
    billingEvents,
  );

  useEffect(() => {
    let cancelled = false;

    void loadBillingPageData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        applyBillingPageData(data);
        setStatus(
          t(
            'Billing view keeps live quota, checkout progress, and payment history in one place.',
          ),
        );
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [t]);

  function applyBillingPageData(data: BillingPageData) {
    setSummary(data.summary);
    setBillingEventSummary(data.billing_event_summary);
    setBillingEvents(data.billing_events);
    setUsageRecords(data.usage_records);
    setPlans(data.plans);
    setPacks(data.packs);
    setOrders(data.orders);
    setPaymentHistory(data.payment_history);
    setRefundHistory(data.refund_history);
    setPaymentSimulationEnabled(data.payment_simulation_enabled);
    setMembership(data.membership);
    setCommercialReconciliation(data.commercial_reconciliation);
    setCommercialAccount(data.commercial_account);
    setCommercialBalance(data.commercial_balance);
    setCommercialBenefitLots(data.commercial_benefit_lots);
    setCommercialHolds(data.commercial_holds);
    setCommercialRequestSettlements(data.commercial_request_settlements);
    setCommercialPricingPlans(data.commercial_pricing_plans);
    setCommercialPricingRates(data.commercial_pricing_rates);
  }

  useEffect(() => {
    if (checkoutSessionOrderId && orders.some((order) => order.order_id === checkoutSessionOrderId)) {
      return;
    }

    const nextPendingOrder = orders.find((order) => isPendingPaymentOrder(order));
    if (!nextPendingOrder) {
      setCheckoutSessionOrderId(null);
      setCheckoutSession(null);
      setCheckoutDetail(null);
      setProviderCheckoutMethodId(null);
      setProviderCallbackMethodId(null);
      setCheckoutSessionStatus(
        t('Open the checkout workbench from Pending payment queue to inspect the selected payment method.'),
      );
      return;
    }

    void loadCheckoutSession(nextPendingOrder.order_id);
  }, [orders, checkoutSessionOrderId]);

  async function refreshBillingPage(nextStatus?: string): Promise<void> {
    const data = await loadBillingPageData();
    applyBillingPageData(data);
    if (nextStatus) {
      setStatus(nextStatus);
    }
  }

  function currentRemainingUnits(): number | null {
    return summary.remaining_units ?? null;
  }

  function handleBillingEventExport(): void {
    if (!billingEvents.length) {
      return;
    }

    downloadBillingEventsCsv(billingEvents);
  }

  async function loadCheckoutSession(orderId: string): Promise<void> {
    setCheckoutSessionOrderId(orderId);
    setCheckoutSessionLoading(true);
    setCheckoutSessionStatus(t('Loading checkout for {orderId}...', { orderId }));

    try {
      const detail = await getBillingCheckoutDetail(orderId);
      const session = detail.checkout_session;
      const checkoutPresentation = buildBillingCheckoutPresentation(detail);
      const callbackMethod = preferredProviderCallbackMethod(detail.checkout_methods);
      setCheckoutDetail(detail);
      setCheckoutSession(session);
      setProviderCheckoutMethodId(null);
      setPaymentSimulationEnabled(session.payment_simulation_enabled);
      setProviderCallbackMethodId(callbackMethod?.id ?? null);
      if (checkoutPresentation.reference) {
        if (checkoutPresentation.provider && checkoutPresentation.channel) {
          setCheckoutSessionStatus(
            t('{reference} is the current {provider} / {channel} payment reference for this order.', {
              reference: checkoutPresentation.reference,
              provider: checkoutMethodProviderLabel(checkoutPresentation.provider, t),
              channel: checkoutMethodChannelLabel(checkoutPresentation.channel, t),
            }),
          );
        } else {
          setCheckoutSessionStatus(
            t('{reference} is the current checkout reference for this order.', {
              reference: checkoutPresentation.reference,
            }),
          );
        }
      } else {
        setCheckoutSessionStatus(
          t('Open the checkout workbench from Pending payment queue to inspect the selected payment method.'),
        );
      }
    } catch (error) {
      setCheckoutDetail(null);
      setCheckoutSession(null);
      setProviderCheckoutMethodId(null);
      setProviderCallbackMethodId(null);
      setCheckoutSessionStatus(portalErrorMessage(error));
    } finally {
      setCheckoutSessionLoading(false);
    }
  }

  async function loadCheckoutPreview(
    selection: BillingSelection,
    nextCouponCode = couponCode,
  ): Promise<void> {
    setPreviewLoading(true);
    setCheckoutStatus(
      t('Loading live checkout pricing for {targetId}...', {
        targetId: selection.target.id,
      }),
    );
    setCheckoutPreview(null);

    try {
      const quote = await previewBillingCheckout({
        target_kind: selection.kind,
        target_id: selection.target.id,
        coupon_code: nextCouponCode.trim() ? nextCouponCode.trim().toUpperCase() : null,
        current_remaining_units: currentRemainingUnits(),
      });
      setCheckoutPreview(quote);
      setCheckoutStatus(
        t(
          '{targetName} is priced by the live commerce quote service and ready to create as a pending payment order.',
          {
            targetName: quote.target_name,
          },
        ),
      );
    } catch (error) {
      setCheckoutStatus(portalErrorMessage(error));
    } finally {
      setPreviewLoading(false);
    }
  }

  function openCheckout(selection: BillingSelection) {
    setCheckoutSelection(selection);
    setCheckoutOpen(true);
    setCouponCode('');
    setCheckoutPreview(null);
    void loadCheckoutPreview(selection, '');
  }

  async function handleCheckoutPreviewRefresh(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!checkoutSelection) {
      return;
    }

    await loadCheckoutPreview(checkoutSelection);
  }

  async function placeOrder() {
    if (!checkoutSelection) {
      return;
    }

    setOrderLoading(true);
    setCheckoutStatus(
      t('Creating a checkout order for {targetId}...', {
        targetId: checkoutSelection.target.id,
      }),
    );

    try {
      const order = await createBillingOrder({
        target_kind: checkoutSelection.kind,
        target_id: checkoutSelection.target.id,
        coupon_code: couponCode.trim() ? couponCode.trim().toUpperCase() : null,
      });
      await refreshBillingPage(
        t(
          '{targetName} was queued in Pending payment queue. Open the checkout workbench to complete payment before quota or membership changes are applied.',
          {
            targetName: order.target_name,
          },
        ),
      );
      await loadCheckoutSession(order.order_id);
      setCheckoutOpen(false);
      setCheckoutPreview(null);
      setCouponCode('');
      setCheckoutSelection(null);
    } catch (error) {
      setCheckoutStatus(portalErrorMessage(error));
    } finally {
      setOrderLoading(false);
    }
  }

  async function handleQueueAction(
    order: PortalCommerceOrder,
    action: 'settle' | 'cancel',
  ): Promise<void> {
    setQueueActionOrderId(order.order_id);
    setQueueActionType(action);
    setStatus(
      action === 'settle'
        ? t('Settling {targetName} into active workspace quota...', {
            targetName: order.target_name,
          })
        : t('Canceling {targetName} before fulfillment is applied...', {
            targetName: order.target_name,
          }),
    );

    try {
      const nextOrder =
        action === 'settle'
          ? await settleBillingOrder(order.order_id)
          : await cancelBillingOrder(order.order_id);
      await refreshBillingPage(
        action === 'settle'
          ? t('{targetName} was settled and moved into Order timeline.', {
              targetName: nextOrder.target_name,
            })
          : t('{targetName} was canceled and left out of quota fulfillment.', {
              targetName: nextOrder.target_name,
            }),
      );
      await loadCheckoutSession(nextOrder.order_id);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setQueueActionOrderId(null);
      setQueueActionType(null);
    }
  }

  async function handleProviderEvent(
    eventType: PortalCommercePaymentEventType,
    method: PortalCommerceCheckoutSessionMethod | null,
  ): Promise<void> {
    if (!checkoutSessionOrderId || !method) {
      return;
    }

    const providerLabel = checkoutMethodProviderLabel(method.provider, t);
    setProviderEventOrderId(checkoutSessionOrderId);
    setProviderEventType(eventType);
    setStatus(
      eventType === 'settled'
        ? t('Applying {provider} settlement outcome for {orderId}...', {
            provider: providerLabel,
            orderId: checkoutSessionOrderId,
          })
        : eventType === 'failed'
          ? t('Applying {provider} failure outcome for {orderId}...', {
              provider: providerLabel,
              orderId: checkoutSessionOrderId,
            })
          : t('Applying {provider} cancellation outcome for {orderId}...', {
              provider: providerLabel,
              orderId: checkoutSessionOrderId,
            }),
    );

    try {
      const nextOrder = await sendBillingPaymentEvent(checkoutSessionOrderId, {
        event_type: eventType,
        provider: method.provider,
        provider_event_id: buildProviderEventReplayId(
          checkoutSessionOrderId,
          eventType,
          method,
        ),
        checkout_method_id: method.id,
      });
      await refreshBillingPage(
        eventType === 'settled'
          ? t('{targetName} was settled after the {provider} payment confirmation.', {
              targetName: nextOrder.target_name,
              provider: providerLabel,
            })
          : eventType === 'failed'
            ? t('{targetName} was marked failed after the {provider} payment confirmation.', {
                targetName: nextOrder.target_name,
                provider: providerLabel,
              })
            : t('{targetName} was canceled after the {provider} payment confirmation.', {
                targetName: nextOrder.target_name,
                provider: providerLabel,
              }),
      );
      await loadCheckoutSession(nextOrder.order_id);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setProviderEventOrderId(null);
      setProviderEventType(null);
    }
  }

  async function handleProviderCheckoutLaunch(
    method: PortalCommerceCheckoutSessionMethod,
  ): Promise<void> {
    if (!checkoutSessionOrderId || !checkoutDetail || !supportsFormalProviderCheckoutLaunch(method)) {
      return;
    }

    const targetName = checkoutDetail.order.target_name;
    const providerLabel = checkoutMethodProviderLabel(method.provider, t);
    const checkoutLaunchDecision = buildBillingCheckoutLaunchDecision({
      checkout_method: method,
      payment_attempts: checkoutDetail.payment_attempts,
    });
    const activeAttempt = checkoutLaunchDecision.kind === 'resume_existing_attempt'
      ? checkoutLaunchDecision.latest_attempt
      : null;

    if (activeAttempt?.checkout_url) {
      setStatus(
        t('Reopening {provider} checkout for {targetName}...', {
          provider: providerLabel,
          targetName,
        }),
      );
      openPortalExternalUrl(activeAttempt.checkout_url);
      return;
    }

    setProviderCheckoutMethodId(method.id);
    setStatus(
      checkoutLaunchDecision.kind === 'create_retry_attempt'
        ? t('Creating a fresh {provider} checkout attempt for {targetName}...', {
            provider: providerLabel,
            targetName,
          })
        : t('Starting {provider} checkout for {targetName}...', {
            provider: providerLabel,
            targetName,
          }),
    );

    try {
      const paymentAttempt = await createBillingPaymentAttempt(checkoutSessionOrderId, {
        payment_method_id: method.id,
        success_url: resolvePortalCheckoutReturnUrl(checkoutSessionOrderId, 'success'),
        cancel_url: resolvePortalCheckoutReturnUrl(checkoutSessionOrderId, 'cancel'),
      });

      await loadCheckoutSession(checkoutSessionOrderId);

      if (paymentAttempt.checkout_url) {
        openPortalExternalUrl(paymentAttempt.checkout_url);
        setStatus(
          t('{targetName} now uses the {provider} checkout launch path.', {
            targetName,
            provider: providerLabel,
          }),
        );
        return;
      }

      setStatus(
        t('{targetName} created a {provider} checkout attempt, but no checkout link was returned.', {
          targetName,
          provider: providerLabel,
        }),
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setProviderCheckoutMethodId(null);
    }
  }

  const remainingUnitsLabel =
    summary.remaining_units === null || summary.remaining_units === undefined
      ? t('Unlimited')
      : formatUnits(summary.remaining_units);
  const searchedOrders = orders.filter((order) => orderMatchesSearch(order, deferredSearch, t));
  const pendingOrders = orders.filter((order) => isPendingPaymentOrder(order));
  const failedOrders = orders.filter((order) => order.status === 'failed');
  const timelineOrders = orders.filter((order) => !isPendingPaymentOrder(order));
  const visibleOrders = searchedOrders.filter((order) => matchesOrderLane(order, orderLane));
  const pendingPaymentCount = orders.filter((order) => isPendingPaymentOrder(order)).length;
  const failedPaymentCount = failedOrders.length;
  const timelineOrderCount = timelineOrders.length;
  const selectedTargetLabel = selectionLabel(checkoutSelection);
  const selectedTargetKindLabel = checkoutSelection
    ? targetKindLabel(checkoutSelection.kind, t)
    : null;
  const checkoutMethods = checkoutDetail?.checkout_methods ?? checkoutSession?.methods ?? [];
  const checkoutPresentation = checkoutDetail
    ? buildBillingCheckoutPresentation(checkoutDetail)
    : null;
  const checkoutPaymentAttempts = checkoutDetail?.payment_attempts ?? [];
  const latestCheckoutPaymentAttemptId =
    checkoutDetail?.latest_payment_attempt?.payment_attempt_id ?? null;
  const activeCheckoutOrder = checkoutDetail?.order ?? null;
  const visibleCheckoutMethods = checkoutMethods.filter((method) => paymentSimulationEnabled || method.action !== 'settle_order');
  const providerCallbackMethods = checkoutMethods.filter((method) => method.supports_webhook && paymentSimulationEnabled);
  const activeProviderCallbackMethod = providerCallbackMethods.find(
    (method) => method.id === providerCallbackMethodId,
  ) ?? preferredProviderCallbackMethod(checkoutMethods);
  const activeProviderLabel = activeProviderCallbackMethod
    ? checkoutMethodProviderLabel(activeProviderCallbackMethod.provider, t)
    : null;
  const activeProviderChannelLabel = activeProviderCallbackMethod
    ? checkoutMethodChannelLabel(activeProviderCallbackMethod.channel, t)
    : null;
  const checkoutReference = checkoutPresentation?.reference
    ?? checkoutSession?.reference
    ?? t('Not recorded');
  const checkoutRailProvider = checkoutPresentation?.provider
    ?? checkoutDetail?.selected_payment_method?.provider
    ?? (paymentSimulationEnabled ? checkoutSession?.provider : null)
    ?? null;
  const checkoutPaymentMethodName = checkoutPresentation?.payment_method_name
    ?? checkoutDetail?.selected_payment_method?.display_name
    ?? null;
  const checkoutCompatibilityStatusLabel = checkoutSession
    ? checkoutSessionStatusLabel(checkoutSession.session_status, t)
    : t('Status');
  const checkoutPresentationProviderLabel = checkoutPresentation?.provider
    ? checkoutMethodProviderLabel(checkoutPresentation.provider, t)
    : null;
  const checkoutPresentationChannelLabel = checkoutPresentation?.channel
    ? checkoutMethodChannelLabel(checkoutPresentation.channel, t)
    : null;
  const checkoutPrimaryRailLabel = checkoutPresentationProviderLabel
    ? checkoutPresentationChannelLabel
      ? t('{provider} / {channel}', {
          provider: checkoutPresentationProviderLabel,
          channel: checkoutPresentationChannelLabel,
        })
      : checkoutPresentationProviderLabel
    : paymentSimulationEnabled
      ? checkoutModeLabel(checkoutSession, t)
      : t('Not recorded');
  const checkoutPresentationStatusLabelText = checkoutPresentationStatusText(
    checkoutPresentation,
    checkoutCompatibilityStatusLabel,
    t,
  );
  const orderWorkbenchCopy = orderWorkbenchDetail(orderLane, t);
  const membershipPanelDescription = membershipDescription(membership, t);
  const orderEmptyTitle =
    orderLane === 'pending_payment'
      ? t('No pending payment orders for this slice')
      : orderLane === 'failed'
        ? t('No failed payment orders for this slice')
        : orderLane === 'timeline'
          ? t('No timeline orders for this slice')
          : t('No orders for this slice');
  const orderEmptyDetail = orders.length
    ? orderLane === 'pending_payment'
      ? t('Adjust the search or switch Order lane to reveal a different pending checkout.')
      : orderLane === 'failed'
        ? t('No failed payment orders match the current search or lane selection.')
        : orderLane === 'timeline'
          ? t('Adjust the search or switch Order lane to reveal a different settled or canceled order.')
          : t('Adjust the search or switch Order lane to reveal a different checkout.')
    : t('Create the first subscription or recharge checkout and it will appear here.');
  const detailCardClassName =
    'rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80';
  const catalogCardClassName =
    'rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50';
  const decisionSupportFacts = [
    {
      detail: t('Live quota still available to the current workspace.'),
      label: t('Remaining units'),
      value: remainingUnitsLabel,
    },
    {
      detail: t('Total token units already consumed.'),
      label: t('Used units'),
      value: formatUnits(summary.used_units),
    },
    {
      detail: t('Live amount visible in the billing summary.'),
      label: t('Booked amount'),
      value: formatCurrency(summary.booked_amount),
    },
    {
      detail: t('Open checkout orders that still need settlement before quota changes apply.'),
      label: t('Pending payment'),
      value: String(pendingPaymentCount),
    },
    {
      detail: t('Checkout attempts that closed on the failure path and need a fresh checkout decision.'),
      label: t('Failed payment'),
      value: String(failedPaymentCount),
    },
  ];
  const membershipFacts = [
    {
      label: t('Plan'),
      value: membership?.plan_name ?? t('No membership'),
    },
    {
      label: t('Cadence'),
      value: membership?.cadence ?? t('n/a'),
    },
    {
      label: t('Included units'),
      value: membership ? formatUnits(membership.included_units) : t('n/a'),
    },
    {
      label: t('Status'),
      value: resolveMembershipStatusLabel(membership?.status, t),
    },
  ];
  const runwayFacts = [
    {
      label: t('Projected coverage'),
      value: recommendation.runway.label,
    },
    {
      label: t('Daily burn'),
      value: recommendation.runway.daily_units
        ? t('{units} / day', { units: formatUnits(recommendation.runway.daily_units) })
        : t('Needs data'),
    },
    {
      label: t('Quota posture'),
      value: summary.exhausted ? t('Exhausted') : t('Active'),
    },
  ];
  const recommendedBundleFacts = [
    {
      label: t('Subscription'),
      value: recommendation.plan?.name ?? t('None'),
    },
    {
      label: t('Recharge buffer'),
      value: recommendation.pack?.label ?? t('Optional'),
    },
    {
      label: t('Bundle posture'),
      value: recommendation.bundle.title,
    },
  ];
  const multimodalFacts = [
    {
      detail: t('Workspace-scoped billing requests recorded by the Billing 2.0 event ledger.'),
      label: t('Requests'),
      value: formatUnits(billingEventAnalytics.totals.total_request_count),
    },
    {
      detail: t('Text tokens charged across chat, responses, and other token-driven routes.'),
      label: t('Tokens'),
      value: formatUnits(billingEventAnalytics.totals.total_tokens),
    },
    {
      detail: t('Generated image count tracked by event-level metering.'),
      label: t('Images'),
      value: formatUnits(billingEventAnalytics.totals.total_image_count),
    },
    {
      detail: t('Audio seconds tracked for speech, transcription, or audio generation routes.'),
      label: t('Audio sec'),
      value: formatUnits(billingEventAnalytics.totals.total_audio_seconds),
    },
    {
      detail: t('Video seconds tracked for multimodal video generation or relay traffic.'),
      label: t('Video sec'),
      value: formatUnits(billingEventAnalytics.totals.total_video_seconds),
    },
    {
      detail: t('Music seconds tracked for music generation workloads.'),
      label: t('Music sec'),
      value: formatUnits(billingEventAnalytics.totals.total_music_seconds),
    },
  ];
  const activeCommercialPlan = selectPrimaryCommercialPricingPlan(
    commercialPricingPlans,
    Date.now(),
  );
  const prioritizedCommercialRates = commercialPricingRates
    .slice()
    .sort(compareCommercialPricingRates);
  const primaryCommercialRate = activeCommercialPlan
    ? prioritizedCommercialRates.find(
      (rate) => rate.pricing_plan_id === activeCommercialPlan.pricing_plan_id,
    ) ?? prioritizedCommercialRates[0] ?? null
    : prioritizedCommercialRates[0] ?? null;
  const tokenPricingRates = prioritizedCommercialRates.filter(isTokenPricingRate).slice(0, 3);
  const mediaPricingRates = prioritizedCommercialRates.filter(isMediaPricingRate).slice(0, 4);
  const commercialAccountFacts = [
    {
      label: t('Account'),
      value: commercialAccount ? String(commercialAccount.account.account_id) : t('n/a'),
    },
    {
      label: t('Status'),
      value: commercialAccount ? titleCaseToken(commercialAccount.account.status) : t('n/a'),
    },
    {
      label: t('Available balance'),
      value: formatUnits(commercialBalance?.available_balance ?? commercialAccount?.available_balance ?? 0),
    },
    {
      label: t('Held balance'),
      value: formatUnits(commercialBalance?.held_balance ?? commercialAccount?.held_balance ?? 0),
    },
    {
      label: t('Consumed balance'),
      value: formatUnits(commercialBalance?.consumed_balance ?? commercialAccount?.consumed_balance ?? 0),
    },
    {
      label: t('Active lots'),
      value: formatUnits(commercialBalance?.active_lot_count ?? commercialAccount?.active_lot_count ?? 0),
    },
  ];
  const commercialReconciliationFacts = [
    {
      label: t('Health'),
      value: commercialReconciliation
        ? (commercialReconciliation.healthy ? t('Healthy') : t('Lagging'))
        : t('n/a'),
    },
    {
      label: t('Backlog orders'),
      value: formatUnits(commercialReconciliation?.backlog_order_count ?? 0),
    },
    {
      label: t('Checkpoint lag'),
      value: commercialReconciliation
        ? (
          commercialReconciliation.checkpoint_lag_ms >= 1000
            ? `${formatUnits(Math.round(commercialReconciliation.checkpoint_lag_ms / 1000))} s`
            : `${commercialReconciliation.checkpoint_lag_ms} ms`
        )
        : t('n/a'),
    },
    {
      label: t('Last reconciled order'),
      value: commercialReconciliation?.last_reconciled_order_id || t('n/a'),
    },
    {
      label: t('Last checkpoint'),
      value: commercialReconciliation?.last_reconciled_at_ms
        ? formatDateTime(commercialReconciliation.last_reconciled_at_ms)
        : t('n/a'),
    },
    {
      label: t('Checkpoint watermark'),
      value: commercialReconciliation?.last_reconciled_order_updated_at_ms
        ? formatDateTime(commercialReconciliation.last_reconciled_order_updated_at_ms)
        : t('n/a'),
    },
  ];
  const commercialSettlementFacts = [
    {
      label: t('Benefit lots'),
      value: formatUnits(commercialBenefitLots.length),
    },
    {
      label: t('Open holds'),
      value: formatUnits(
        commercialHolds.filter((hold) =>
          hold.status === 'held'
          || hold.status === 'captured'
          || hold.status === 'partially_released').length,
      ),
    },
    {
      label: t('Request settlements'),
      value: formatUnits(commercialRequestSettlements.length),
    },
    {
      label: t('Captured credits'),
      value: formatUnits(
        commercialRequestSettlements.reduce(
          (sum, settlement) => sum + settlement.captured_credit_amount,
          0,
        ),
      ),
    },
    {
      label: t('Grant balance'),
      value: formatUnits(commercialBalance?.grant_balance ?? commercialAccount?.grant_balance ?? 0),
    },
    {
      label: t('Latest settlement'),
      value: commercialRequestSettlements.length
        ? formatDateTime(
          commercialRequestSettlements
            .slice()
            .sort((left, right) => right.settled_at_ms - left.settled_at_ms)[0]
            .settled_at_ms,
        )
        : t('n/a'),
    },
  ];
  const pricingPostureFacts = [
    {
      label: t('Pricing plans'),
      value: formatUnits(commercialPricingPlans.length),
    },
    {
      label: t('Pricing rates'),
      value: formatUnits(commercialPricingRates.length),
    },
    {
      label: t('Primary plan'),
      value: activeCommercialPlan?.display_name ?? t('n/a'),
    },
    {
      label: t('Plan code'),
      value: activeCommercialPlan?.plan_code ?? t('n/a'),
    },
    {
      label: t('Primary metric'),
      value: primaryCommercialRate?.metric_code ?? t('n/a'),
    },
    {
      label: t('Effective from'),
      value: activeCommercialPlan
        ? (
          activeCommercialPlan.effective_from_ms > 0
            ? formatDateTime(activeCommercialPlan.effective_from_ms)
            : t('Immediate')
        )
        : t('n/a'),
    },
    {
      label: t('Effective to'),
      value: activeCommercialPlan
        ? (
          activeCommercialPlan.effective_to_ms != null
            ? formatDateTime(activeCommercialPlan.effective_to_ms)
            : t('Open ended')
        )
        : t('n/a'),
    },
    {
      label: t('Charge unit'),
      value: primaryCommercialRate
        ? commercialPricingChargeUnitLabel(primaryCommercialRate.charge_unit, t)
        : t('n/a'),
    },
    {
      label: t('Billing method'),
      value: primaryCommercialRate
        ? commercialPricingMethodLabel(primaryCommercialRate.pricing_method, t)
        : t('n/a'),
    },
    {
      label: t('Price unit'),
      value: primaryCommercialRate
        ? commercialPricingDisplayUnit(primaryCommercialRate, t)
        : t('n/a'),
    },
  ];

  return (
    <>
      <Dialog open={checkoutOpen} onOpenChange={setCheckoutOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Checkout preview')}</DialogTitle>
            <DialogDescription>{checkoutStatus}</DialogDescription>
          </DialogHeader>

          <form className="grid gap-4" onSubmit={(event) => void handleCheckoutPreviewRefresh(event)}>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                  {t('Selected offer')}
                </p>
                <h3 className="mt-3 text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                  {selectedTargetLabel ?? t('Checkout preview')}
                </h3>
                <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                  {checkoutPreview
                    ? t('{kind} / {price}', {
                        kind: targetKindLabel(checkoutPreview.target_kind, t),
                        price: checkoutPreview.payable_price_label,
                      })
                    : t('Preview the live quote before creating a checkout.')}
                </p>
                <div className="mt-4 flex flex-wrap gap-2">
                  {selectedTargetKindLabel ? (
                    <Badge variant="default">{selectedTargetKindLabel}</Badge>
                  ) : null}
                  {checkoutPreview?.applied_coupon ? (
                    <Badge variant="warning">{checkoutPreview.applied_coupon.code}</Badge>
                  ) : null}
                </div>
              </div>

              <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                  {t('Order impact')}
                </p>
                <div className="mt-3 grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Payable price')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview?.payable_price_label ?? t('Pending')}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Granted units')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview ? formatUnits(checkoutPreview.granted_units) : t('Pending')}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Projected remaining')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview?.projected_remaining_units === null
                      || checkoutPreview?.projected_remaining_units === undefined
                        ? remainingUnitsLabel
                        : formatUnits(checkoutPreview.projected_remaining_units)}
                    </strong>
                  </div>
                </div>
              </div>
            </div>

            <SettingsField
              description={t('Optional coupon codes are priced by the live quote service before checkout creation.')}
              label={t('Coupon code')}
              layout="vertical"
            >
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) => {
                  setCouponCode(event.target.value);
                  setCheckoutPreview(null);
                }}
                placeholder={t('SPRING20')}
                value={couponCode}
              />
            </SettingsField>

            <DialogFooter>
              <Button onClick={() => setCheckoutOpen(false)} type="button" variant="ghost">
                {t('Close')}
              </Button>
              <Button onClick={() => onNavigate('credits')} type="button" variant="secondary">
                {t('Open redeem')}
              </Button>
              <Button disabled={previewLoading || orderLoading} type="submit" variant="secondary">
                {previewLoading ? t('Loading preview...') : t('Refresh preview')}
              </Button>
              <Button
                disabled={!checkoutPreview || previewLoading || orderLoading}
                onClick={() => void placeOrder()}
                type="button"
              >
                {orderLoading ? t('Creating checkout...') : t('Create checkout')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4">
        <section
          data-slot="portal-billing-toolbar"
          className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
        >
          <FilterBar>
            <FilterBarSection className="min-w-[15rem] flex-[0_1_20rem]" grow={false}>
              <FilterField
                className="w-full"
                controlClassName="min-w-0"
                label={t('Search order lifecycle')}
              >
                <SearchInput
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                  placeholder={t('Search order lifecycle')}
                />
              </FilterField>
            </FilterBarSection>
            <FilterBarSection className="min-w-[12rem] shrink-0" grow={false}>
              <FilterField className="w-full" label={t('Order lane')}>
                <Select
                  value={orderLane}
                  onValueChange={(value: string) => setOrderLane(value as OrderWorkbenchLane)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('Order lane')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">{t('All orders')}</SelectItem>
                    <SelectItem value="pending_payment">{t('Pending payment queue')}</SelectItem>
                    <SelectItem value="failed">{t('Failed payment')}</SelectItem>
                    <SelectItem value="timeline">{t('Order timeline')}</SelectItem>
                  </SelectContent>
                </Select>
              </FilterField>
            </FilterBarSection>
            <FilterBarActions className="gap-2.5 whitespace-nowrap">
              <Button type="button" onClick={() => onNavigate('credits')}>
                {t('Open redeem')}
              </Button>
              <Button onClick={() => onNavigate('usage')} variant="secondary">
                {t('Open usage')}
              </Button>
              <Button onClick={() => onNavigate('account')} variant="secondary">
                {t('Open account')}
              </Button>
            </FilterBarActions>
          </FilterBar>
        </section>

        <section className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
          <WorkspacePanel description={status} title={t('Decision support')}>
            <div className="grid gap-4">
              <BillingRecommendationCard recommendation={recommendation} />
              <div className="grid gap-3 md:grid-cols-2">
                {decisionSupportFacts.map((item) => (
                  <article className={detailCardClassName} key={item.label}>
                    <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                      {item.label}
                    </p>
                    <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                      {item.value}
                    </strong>
                    <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">{item.detail}</p>
                  </article>
                ))}
              </div>
            </div>
          </WorkspacePanel>

          <div className="grid gap-4">
            <WorkspacePanel
              description={membershipPanelDescription}
              title={t('Active membership')}
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {membershipFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>
            </WorkspacePanel>

            <WorkspacePanel
              description={recommendation.runway.detail}
              title={t('Estimated runway')}
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {runwayFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>
            </WorkspacePanel>

            <WorkspacePanel
              description={recommendation.bundle.detail}
              title={t('Recommended bundle')}
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {recommendedBundleFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>
            </WorkspacePanel>
          </div>
        </section>

        <section className="grid gap-4 xl:grid-cols-4">
          <WorkspacePanel
            description={t(
              'Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture.',
            )}
            title={t('Commercial account')}
          >
            <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              {commercialAccountFacts.map((item) => (
                <div className="flex items-center justify-between gap-3" key={item.label}>
                  <span>{item.label}</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                </div>
              ))}
            </div>
          </WorkspacePanel>

          <WorkspacePanel
            description={t(
              'Commerce reconciliation shows whether account history processing has caught up with the latest order mutations and refund activity.',
            )}
            title={t('Commerce reconciliation')}
          >
            <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              {commercialReconciliationFacts.map((item) => (
                <div className="flex items-center justify-between gap-3" key={item.label}>
                  <span>{item.label}</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                </div>
              ))}
            </div>
          </WorkspacePanel>

          <WorkspacePanel
            description={t(
              'Settlement coverage keeps benefit lots, credit holds, and request capture aligned in one billing snapshot.',
            )}
            title={t('Settlement coverage')}
          >
            <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              {commercialSettlementFacts.map((item) => (
                <div className="flex items-center justify-between gap-3" key={item.label}>
                  <span>{item.label}</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                </div>
              ))}
            </div>
          </WorkspacePanel>

          <WorkspacePanel
            description={t(
              'Pricing posture shows which commercial plans and rates currently define the workspace charging envelope.',
            )}
            title={t('Pricing posture')}
          >
            <div className="grid gap-4">
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {pricingPostureFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>

              {primaryCommercialRate ? (
                <div className="grid gap-3">
                  <article className={detailCardClassName}>
                    <div className="flex flex-wrap items-start justify-between gap-3">
                      <div className="grid gap-2">
                        <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                          {t('Primary pricing rule')}
                        </p>
                        <div className="flex flex-wrap gap-2">
                          <Badge variant="default">
                            {commercialPricingChargeUnitLabel(primaryCommercialRate.charge_unit, t)}
                          </Badge>
                          <Badge variant="secondary">
                            {commercialPricingMethodLabel(primaryCommercialRate.pricing_method, t)}
                          </Badge>
                          <Badge variant={commercialPricingStatusTone(primaryCommercialRate.status)}>
                            {titleCaseToken(primaryCommercialRate.status)}
                          </Badge>
                        </div>
                      </div>
                      <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                        {commercialPricingDisplayUnit(primaryCommercialRate, t)}
                      </strong>
                    </div>
                    <div className="mt-4 grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                      <div className="flex items-center justify-between gap-3">
                        <span>{t('Model')}</span>
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {primaryCommercialRate.model_code ?? t('n/a')}
                        </strong>
                      </div>
                      <div className="flex items-center justify-between gap-3">
                        <span>{t('Provider')}</span>
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {primaryCommercialRate.provider_code ?? t('n/a')}
                        </strong>
                      </div>
                      <div className="flex items-center justify-between gap-3">
                        <span>{t('Billing method')}</span>
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {commercialPricingMethodLabel(primaryCommercialRate.pricing_method, t)}
                        </strong>
                      </div>
                      <div className="flex items-center justify-between gap-3">
                        <span>{t('Price unit')}</span>
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {commercialPricingDisplayUnit(primaryCommercialRate, t)}
                        </strong>
                      </div>
                    </div>
                    <p className="mt-4 text-sm text-zinc-600 dark:text-zinc-300">
                      {commercialPricingRuleSummary(primaryCommercialRate, t)}
                    </p>
                  </article>

                  <div className="grid gap-3 lg:grid-cols-2">
                    <article className={catalogCardClassName}>
                      <div className="grid gap-3">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                            {t('Token pricing')}
                          </strong>
                          <Badge variant="secondary">
                            {t('{count} rules', { count: formatUnits(tokenPricingRates.length) })}
                          </Badge>
                        </div>
                        {tokenPricingRates.length ? (
                          tokenPricingRates.map((rate) => (
                            <div className="grid gap-1" key={rate.pricing_rate_id}>
                              <div className="flex flex-wrap items-center justify-between gap-2">
                                <span className="text-sm text-zinc-950 dark:text-zinc-50">
                                  {commercialPricingChargeUnitLabel(rate.charge_unit, t)}
                                </span>
                                <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                                  {commercialPricingDisplayUnit(rate, t)}
                                </strong>
                              </div>
                              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                                {commercialPricingRuleSummary(rate, t)}
                              </p>
                            </div>
                          ))
                        ) : (
                          <p className="text-sm text-zinc-500 dark:text-zinc-400">
                            {t('No token pricing rules are active yet.')}
                          </p>
                        )}
                      </div>
                    </article>

                    <article className={catalogCardClassName}>
                      <div className="grid gap-3">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                            {t('Media pricing')}
                          </strong>
                          <Badge variant="secondary">
                            {t('{count} rules', { count: formatUnits(mediaPricingRates.length) })}
                          </Badge>
                        </div>
                        {mediaPricingRates.length ? (
                          mediaPricingRates.map((rate) => (
                            <div className="grid gap-1" key={rate.pricing_rate_id}>
                              <div className="flex flex-wrap items-center justify-between gap-2">
                                <span className="text-sm text-zinc-950 dark:text-zinc-50">
                                  {commercialPricingChargeUnitLabel(rate.charge_unit, t)}
                                </span>
                                <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                                  {commercialPricingDisplayUnit(rate, t)}
                                </strong>
                              </div>
                              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                                {commercialPricingRuleSummary(rate, t)}
                              </p>
                            </div>
                          ))
                        ) : (
                          <p className="text-sm text-zinc-500 dark:text-zinc-400">
                            {t(
                              'Standard price units include USD / 1M input tokens, USD / image, and USD / music track once live pricing is configured.',
                            )}
                          </p>
                        )}
                      </div>
                    </article>
                  </div>
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Standard price units include USD / 1M input tokens, USD / image, and USD / music track once live pricing is configured.',
                  )}
                  title={t('No commercial pricing rules yet')}
                />
              )}
            </div>
          </WorkspacePanel>
        </section>

        <section className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant="default">
                  {t('{count} billing events', {
                    count: formatUnits(billingEventAnalytics.totals.total_events),
                  })}
                </Badge>
                <Badge variant="secondary">
                  {t('{amount} customer charge', {
                    amount: formatCurrency(billingEventAnalytics.totals.total_customer_charge),
                  })}
                </Badge>
              </div>
            )}
            description={t(
              'Billing event analytics turns route-level metering into multimodal, group, and accounting evidence for commercial reviews.',
            )}
            title={t('Billing event analytics')}
          >
            {billingEventAnalytics.totals.total_events ? (
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                {multimodalFacts.map((item) => (
                  <article className={detailCardClassName} key={item.label}>
                    <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                      {item.label}
                    </p>
                    <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                      {item.value}
                    </strong>
                    <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">{item.detail}</p>
                  </article>
                ))}
              </div>
            ) : (
              <EmptyState
                description={t(
                  'Billing event analytics appears after the workspace records routed usage, multimodal traffic, or chargeback activity.',
                )}
                title={t('No billing event analytics yet')}
              />
            )}
          </WorkspacePanel>

          <div className="grid gap-4">
            <WorkspacePanel
              description={t(
                'Top capabilities show where customer charge is landing across text, image, audio, video, and music traffic.',
              )}
              title={t('Capability mix')}
            >
              {billingEventAnalytics.top_capabilities.length ? (
                <div className="grid gap-3">
                  {billingEventAnalytics.top_capabilities.map((item) => (
                    <article className={catalogCardClassName} key={item.capability}>
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="grid gap-2">
                          <div className="flex flex-wrap gap-2">
                            <Badge variant="default">{titleCaseToken(item.capability)}</Badge>
                            <Badge variant="secondary">
                              {t('{count} events', { count: formatUnits(item.event_count) })}
                            </Badge>
                          </div>
                          <strong className="text-base text-zinc-950 dark:text-zinc-50">
                            {capabilitySignalText(item, t)}
                          </strong>
                          <p className="text-sm text-zinc-600 dark:text-zinc-300">
                            {t('{count} requests routed through this capability slice.', {
                              count: formatUnits(item.request_count),
                            })}
                          </p>
                        </div>
                        <div className="grid gap-1 text-right text-sm">
                          <span className="text-zinc-500 dark:text-zinc-400">
                            {t('Customer charge')}
                          </span>
                          <strong className="text-zinc-950 dark:text-zinc-50">
                            {formatCurrency(item.total_customer_charge)}
                          </strong>
                          <span className="text-zinc-500 dark:text-zinc-400">
                            {t('Upstream cost')}: {formatCurrency(item.total_upstream_cost)}
                          </span>
                        </div>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Capability charge mix will appear after the workspace records billing events.',
                  )}
                  title={t('No capability mix yet')}
                />
              )}
            </WorkspacePanel>

            <WorkspacePanel
              description={t(
                'API key group chargeback keeps environment and tenant-level billing accountability visible without leaving the billing workspace.',
              )}
              title={t('API key group chargeback')}
            >
              {billingEventAnalytics.group_chargeback.length ? (
                <div className="grid gap-3">
                  {billingEventAnalytics.group_chargeback.map((item) => (
                    <article
                      className="flex items-center justify-between gap-3 border-b border-zinc-200/80 pb-3 last:border-b-0 last:pb-0 dark:border-zinc-800/80"
                      key={item.api_key_group_id ?? 'unassigned'}
                    >
                      <div className="grid gap-2">
                        <div className="flex flex-wrap gap-2">
                          <Badge variant={item.api_key_group_id ? 'default' : 'secondary'}>
                            {groupChargebackLabel(item.api_key_group_id, t)}
                          </Badge>
                          <Badge variant="secondary">
                            {t('{count} requests', { count: formatUnits(item.request_count) })}
                          </Badge>
                        </div>
                        <p className="text-sm text-zinc-600 dark:text-zinc-300">
                          {t('{count} billing events contributed to this group chargeback slice.', {
                            count: formatUnits(item.event_count),
                          })}
                        </p>
                      </div>
                      <div className="grid gap-1 text-right text-sm">
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {formatCurrency(item.total_customer_charge)}
                        </strong>
                        <span className="text-zinc-500 dark:text-zinc-400">
                          {t('Upstream cost')}: {formatCurrency(item.total_upstream_cost)}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Group chargeback will appear once billing events are attributed to API key groups.',
                  )}
                  title={t('No group chargeback yet')}
                />
              )}
            </WorkspacePanel>

            <WorkspacePanel
              description={t(
                'Accounting mode mix separates platform credit, BYOK, and passthrough consumption in one billing review slice.',
              )}
              title={t('Accounting mode mix')}
            >
              {billingEventAnalytics.accounting_mode_mix.length ? (
                <div className="grid gap-3">
                  {billingEventAnalytics.accounting_mode_mix.map((item) => (
                    <article
                      className="flex items-center justify-between gap-3 border-b border-zinc-200/80 pb-3 last:border-b-0 last:pb-0 dark:border-zinc-800/80"
                      key={item.accounting_mode}
                    >
                      <div className="grid gap-2">
                        <div className="flex flex-wrap gap-2">
                          <Badge variant={accountingModeTone(item.accounting_mode)}>
                            {accountingModeLabel(item.accounting_mode, t)}
                          </Badge>
                          <Badge variant="secondary">
                            {t('{count} requests', { count: formatUnits(item.request_count) })}
                          </Badge>
                        </div>
                        <p className="text-sm text-zinc-600 dark:text-zinc-300">
                          {t('{count} billing events used this accounting mode.', {
                            count: formatUnits(item.event_count),
                          })}
                        </p>
                      </div>
                      <div className="grid gap-1 text-right text-sm">
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {formatCurrency(item.total_customer_charge)}
                        </strong>
                        <span className="text-zinc-500 dark:text-zinc-400">
                          {t('Upstream cost')}: {formatCurrency(item.total_upstream_cost)}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Accounting mode evidence appears after billing events record platform credit, BYOK, or passthrough traffic.',
                  )}
                  title={t('No accounting mode mix yet')}
                />
              )}
            </WorkspacePanel>

            <WorkspacePanel
              description={t(
                'Inspect the compiled routing evidence for this workspace after policy, project defaults, and API key group profile overlays are combined.',
              )}
              title={t('Routing evidence')}
            >
              <div className="grid gap-3 md:grid-cols-3">
                <article className={detailCardClassName}>
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Applied routing profile')}
                  </p>
                  <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                    {formatUnits(billingEventAnalytics.routing_evidence.events_with_profile)}
                  </strong>
                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {t('{count} billing events retained an applied profile id.', {
                      count: formatUnits(billingEventAnalytics.routing_evidence.events_with_profile),
                    })}
                  </p>
                </article>

                <article className={detailCardClassName}>
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Compiled snapshot')}
                  </p>
                  <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                    {formatUnits(
                      billingEventAnalytics.routing_evidence.events_with_compiled_snapshot,
                    )}
                  </strong>
                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {t('{count} billing events retained a compiled snapshot id.', {
                      count: formatUnits(
                        billingEventAnalytics.routing_evidence.events_with_compiled_snapshot,
                      ),
                    })}
                  </p>
                </article>

                <article className={detailCardClassName}>
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Fallback reason')}
                  </p>
                  <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                    {formatUnits(
                      billingEventAnalytics.routing_evidence.events_with_fallback_reason,
                    )}
                  </strong>
                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {t(
                      'Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path.',
                    )}
                  </p>
                </article>
              </div>
            </WorkspacePanel>
          </div>
        </section>

        <section>
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant="secondary">
                  {t('{count} capabilities', {
                    count: formatUnits(billingEventSummary.capability_count),
                  })}
                </Badge>
                <Badge variant="secondary">
                  {t('{count} groups', {
                    count: formatUnits(billingEventSummary.group_count),
                  })}
                </Badge>
                <Button
                  disabled={!billingEvents.length}
                  onClick={handleBillingEventExport}
                  type="button"
                  variant="secondary"
                >
                  {t('Export billing events CSV')}
                </Button>
              </div>
            )}
            description={t(
              'Recent billing events keep multimodal chargeback, provider cost, and routing evidence in one finance-ready table.',
            )}
            title={t('Recent billing events')}
          >
            <BillingEventsAuditTable
              columns={[
                {
                  id: 'event',
                  header: t('Event'),
                  cell: (row: BillingEventRecord) => row.event_id,
                },
                {
                  id: 'capability',
                  header: t('Capability'),
                  cell: (row: BillingEventRecord) => titleCaseToken(row.capability),
                },
                {
                  id: 'group',
                  header: t('Group'),
                  cell: (row: BillingEventRecord) => groupChargebackLabel(row.api_key_group_id, t),
                },
                {
                  id: 'signals',
                  header: t('Signals'),
                  cell: (row: BillingEventRecord) => eventSignalText(row, t),
                },
                {
                  id: 'accounting',
                  header: t('Accounting'),
                  cell: (row: BillingEventRecord) => (
                    <Badge variant={accountingModeTone(row.accounting_mode)}>
                      {accountingModeLabel(row.accounting_mode, t)}
                    </Badge>
                  ),
                },
                {
                  id: 'applied_routing_profile_id',
                  header: t('Applied routing profile'),
                  cell: (row: BillingEventRecord) => (
                    <div className="max-w-[12rem] truncate">
                      {row.applied_routing_profile_id ?? t('Not recorded')}
                    </div>
                  ),
                },
                {
                  id: 'compiled_routing_snapshot_id',
                  header: t('Compiled snapshot'),
                  cell: (row: BillingEventRecord) => (
                    <div className="max-w-[12rem] truncate">
                      {row.compiled_routing_snapshot_id ?? t('Not recorded')}
                    </div>
                  ),
                },
                {
                  id: 'fallback_reason',
                  header: t('Fallback reason'),
                  cell: (row: BillingEventRecord) => (
                    <div className="max-w-[14rem] truncate">
                      {row.fallback_reason ?? t('None')}
                    </div>
                  ),
                },
                {
                  id: 'customer_charge',
                  header: t('Customer charge'),
                  cell: (row: BillingEventRecord) => formatCurrency(row.customer_charge),
                },
                {
                  id: 'upstream_cost',
                  header: t('Upstream cost'),
                  cell: (row: BillingEventRecord) => formatCurrency(row.upstream_cost),
                },
                {
                  id: 'time',
                  header: t('Created'),
                  cell: (row: BillingEventRecord) => formatDateTime(row.created_at_ms),
                },
              ]}
              emptyState={(
                <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                  <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {t('No recent billing events yet')}
                  </strong>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {t(
                      'Recent billing events appear once the workspace records billable routed traffic.',
                    )}
                  </p>
                </div>
              )}
              getRowId={(row: BillingEventRecord) => row.event_id}
              rows={billingEventAnalytics.recent_events}
            />
          </WorkspacePanel>
        </section>

        <section className="grid gap-4 xl:grid-cols-2">
          <WorkspacePanel
            description={t('Choose the monthly posture that best matches expected gateway demand.')}
            title={t('Plan catalog')}
          >
            <div className="grid gap-3">
              {plans.map((plan) => (
                <article
                  key={plan.id}
                  className={catalogCardClassName}
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <div className="flex flex-wrap gap-2">
                        <Badge variant={isRecommendedPlan(plan, recommendation) ? 'success' : 'secondary'}>
                          {plan.name}
                        </Badge>
                        <Badge variant="default">{plan.price_label}</Badge>
                      </div>
                      <strong className="text-lg text-zinc-950 dark:text-zinc-50">
                        {t('{units} included units', {
                          units: formatUnits(plan.included_units),
                        })}
                      </strong>
                      <p className="text-sm text-zinc-600 dark:text-zinc-300">{plan.highlight}</p>
                    </div>
                    <Button
                      type="button"
                      onClick={() =>
                        openCheckout({
                          kind: 'subscription_plan',
                          target: plan,
                        })
                      }
                    >
                      {plan.cta}
                    </Button>
                  </div>
                </article>
              ))}
            </div>
          </WorkspacePanel>

          <WorkspacePanel
            description={t('Use top-ups to restore headroom without changing the base plan.')}
            title={t('Recharge packs')}
          >
            <div className="grid gap-3">
              {packs.map((pack) => (
                <article
                  key={pack.id}
                  className={catalogCardClassName}
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <div className="flex flex-wrap gap-2">
                        <Badge variant={isRecommendedPack(pack, recommendation) ? 'warning' : 'secondary'}>
                          {pack.label}
                        </Badge>
                        <Badge variant="default">{pack.price_label}</Badge>
                      </div>
                      <strong className="text-lg text-zinc-950 dark:text-zinc-50">
                        {t('{units} units', {
                          units: formatUnits(pack.points),
                        })}
                      </strong>
                      <p className="text-sm text-zinc-600 dark:text-zinc-300">{pack.note}</p>
                    </div>
                    <Button
                      type="button"
                      onClick={() =>
                        openCheckout({
                          kind: 'recharge_pack',
                          target: pack,
                        })
                      }
                    >
                      {t('Create checkout')}
                    </Button>
                  </div>
                </article>
              ))}
            </div>
          </WorkspacePanel>
        </section>

        <section>
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant={orderLane === 'all' ? 'default' : 'secondary'}>
                  {t('{count} all orders', { count: orders.length })}
                </Badge>
                <Badge variant={orderLane === 'pending_payment' ? 'default' : 'secondary'}>
                  {t('{count} Pending payment queue', { count: pendingPaymentCount })}
                </Badge>
                <Badge variant={orderLane === 'failed' ? 'warning' : 'secondary'}>
                  {t('{count} Failed payment', { count: failedPaymentCount })}
                </Badge>
                <Badge variant={orderLane === 'timeline' ? 'success' : 'secondary'}>
                  {t('{count} Order timeline', { count: timelineOrderCount })}
                </Badge>
              </div>
            )}
            description={orderWorkbenchCopy}
            title={t('Order workbench')}
          >
            <div className="grid gap-4">
              <DataTable
                columns={[
                  {
                    id: 'offer',
                    header: t('Offer'),
                    cell: (row: PortalCommerceOrder) => row.target_name,
                  },
                  {
                    id: 'kind',
                    header: t('Kind'),
                    cell: (row: PortalCommerceOrder) => targetKindLabel(row.target_kind, t),
                  },
                  {
                    id: 'coupon',
                    header: t('Coupon'),
                    cell: (row: PortalCommerceOrder) => row.applied_coupon_code ?? t('None'),
                  },
                  {
                    id: 'payable',
                    header: t('Payable'),
                    cell: (row: PortalCommerceOrder) => row.payable_price_label,
                  },
                  {
                    id: 'units',
                    header: t('Granted units'),
                    cell: (row: PortalCommerceOrder) => formatUnits(row.granted_units + row.bonus_units),
                  },
                  {
                    id: 'status',
                    header: t('Status'),
                    cell: (row: PortalCommerceOrder) => (
                      <Badge variant={orderStatusTone(row.status)}>
                        {orderStatusLabel(row.status, t)}
                      </Badge>
                    ),
                  },
                  {
                    id: 'time',
                    header: t('Created'),
                    cell: (row: PortalCommerceOrder) => formatDateTime(row.created_at_ms),
                  },
                  {
                    id: 'actions',
                    header: t('Actions'),
                    cell: (row: PortalCommerceOrder) => (
                      <div className="flex flex-wrap gap-2">
                        <Button
                          disabled={checkoutSessionLoading}
                          onClick={() => void loadCheckoutSession(row.order_id)}
                          variant="secondary"
                        >
                          {checkoutSessionLoading && checkoutSessionOrderId === row.order_id
                            ? t('Loading checkout...')
                            : t('Open checkout')}
                        </Button>
                      </div>
                    ),
                  },
                ]}
                emptyState={(
                  <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                    <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                      {orderEmptyTitle}
                    </strong>
                    <p className="text-sm text-zinc-500 dark:text-zinc-400">
                      {orderEmptyDetail}
                    </p>
                  </div>
                )}
                getRowId={(row: PortalCommerceOrder) => row.order_id}
                rows={visibleOrders}
              />
            </div>
          </WorkspacePanel>
        </section>

        <section className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant="secondary">
                  {t('{count} payment timeline rows', { count: formatUnits(paymentHistory.length) })}
                </Badge>
              </div>
            )}
            description={t(
              'Payment history keeps checkout outcomes, payment method evidence, and refund status visible in one billing timeline.',
            )}
            title={t('Payment history')}
          >
            <PaymentHistoryAuditTable
              columns={[
                {
                  id: 'order',
                  header: t('Order'),
                  cell: (row: BillingPaymentHistoryRow) => (
                    <div className="grid gap-1">
                      <strong className="text-zinc-950 dark:text-zinc-50">{row.target_name}</strong>
                      <span className="text-xs text-zinc-500 dark:text-zinc-400">{row.order_id}</span>
                    </div>
                  ),
                },
                {
                  id: 'event',
                  header: t('Event'),
                  cell: (row: BillingPaymentHistoryRow) => (
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="secondary">
                        {paymentHistoryRowKindLabel(row.row_kind, t)}
                      </Badge>
                      <Badge variant={row.event_type === 'refunded' ? 'warning' : 'default'}>
                        {paymentEventTypeLabel(row.event_type, t)}
                      </Badge>
                    </div>
                  ),
                },
                {
                  id: 'rail',
                  header: t('Payment method'),
                  cell: (row: BillingPaymentHistoryRow) => paymentHistoryRailCell(row, t),
                },
                {
                  id: 'provider_event',
                  header: t('Payment update reference'),
                  cell: (row: BillingPaymentHistoryRow) => row.provider_event_id ?? t('Not recorded'),
                },
                {
                  id: 'processing',
                  header: t('Processing'),
                  cell: (row: BillingPaymentHistoryRow) => (
                    <Badge variant={paymentProcessingStatusTone(row.processing_status)}>
                      {paymentProcessingStatusLabel(row.processing_status, t)}
                    </Badge>
                  ),
                },
                {
                  id: 'status_after',
                  header: t('Status'),
                  cell: (row: BillingPaymentHistoryRow) => (
                    <Badge variant={orderStatusTone(row.order_status_after ?? row.order_status)}>
                      {orderStatusLabel(row.order_status_after ?? row.order_status, t)}
                    </Badge>
                  ),
                },
                {
                  id: 'observed',
                  header: t('Observed'),
                  cell: (row: BillingPaymentHistoryRow) => formatDateTime(row.received_at_ms),
                },
              ]}
              emptyState={(
                <EmptyState
                  description={t('No payment lifecycle evidence has been recorded for this workspace yet.')}
                  title={t('No payment history yet')}
                />
              )}
              getRowId={(row: BillingPaymentHistoryRow) => row.id}
              rows={paymentHistory}
            />
          </WorkspacePanel>

          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant="secondary">
                  {t('{count} refund rows', { count: formatUnits(refundHistory.length) })}
                </Badge>
              </div>
            )}
            description={t(
              'Refund history keeps completed refund outcomes, payment method evidence, and the resulting order status visible without reopening each order.',
            )}
            title={t('Refund history')}
          >
            <RefundHistoryAuditTable
              columns={[
                {
                  id: 'order',
                  header: t('Order'),
                  cell: (row: BillingPaymentHistoryRow) => (
                    <div className="grid gap-1">
                      <strong className="text-zinc-950 dark:text-zinc-50">{row.target_name}</strong>
                      <span className="text-xs text-zinc-500 dark:text-zinc-400">{row.order_id}</span>
                    </div>
                  ),
                },
                {
                  id: 'source',
                  header: t('Source'),
                  cell: (row: BillingPaymentHistoryRow) => paymentHistoryRowKindLabel(row.row_kind, t),
                },
                {
                  id: 'rail',
                  header: t('Payment method'),
                  cell: (row: BillingPaymentHistoryRow) => paymentHistoryRailCell(row, t),
                },
                {
                  id: 'reference',
                  header: t('Reference'),
                  cell: (row: BillingPaymentHistoryRow) => row.checkout_reference ?? t('Not recorded'),
                },
                {
                  id: 'status',
                  header: t('Status'),
                  cell: (row: BillingPaymentHistoryRow) => (
                    <Badge variant={orderStatusTone(row.order_status_after ?? row.order_status)}>
                      {orderStatusLabel(row.order_status_after ?? row.order_status, t)}
                    </Badge>
                  ),
                },
                {
                  id: 'observed',
                  header: t('Observed'),
                  cell: (row: BillingPaymentHistoryRow) => formatDateTime(row.received_at_ms),
                },
              ]}
              emptyState={(
                <EmptyState
                  description={t('No refund outcomes have been recorded for this workspace yet.')}
                  title={t('No refund history yet')}
                />
              )}
              getRowId={(row: BillingPaymentHistoryRow) => row.id}
              rows={refundHistory}
            />
          </WorkspacePanel>
        </section>

        <section className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
          <WorkspacePanel
            description={checkoutSessionStatus}
            title={t('Checkout details')}
          >
            {checkoutSession ? (
              <div className="grid gap-4">
                <div className="grid gap-3 md:grid-cols-2 text-sm text-zinc-600 dark:text-zinc-300">
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Reference')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutReference}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Payment method')}</span>
                    <div className="grid justify-items-end gap-1 text-right">
                      <strong className="text-zinc-950 dark:text-zinc-50">
                        {checkoutRailProvider
                          ? checkoutMethodProviderLabel(checkoutRailProvider, t)
                          : t('Not recorded')}
                      </strong>
                      {checkoutPaymentMethodName ? (
                        <span className="text-xs text-zinc-500 dark:text-zinc-400">
                          {checkoutPaymentMethodName}
                        </span>
                      ) : null}
                    </div>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Primary method')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPrimaryRailLabel}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Current status')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPresentationStatusLabelText}
                    </strong>
                  </div>
                </div>

                <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Guidance')}
                  </p>
                  <p className="mt-3 text-sm text-zinc-600 dark:text-zinc-300">
                    {checkoutPresentationGuidanceText(checkoutPresentation, t)}
                  </p>
                </div>

                <div className="grid gap-3">
                  {visibleCheckoutMethods.length ? (
                    visibleCheckoutMethods.map((method) => {
                      const providerLabel = checkoutMethodProviderLabel(method.provider, t);
                      const checkoutLaunchDecision = supportsFormalProviderCheckoutLaunch(method)
                        ? buildBillingCheckoutLaunchDecision({
                            checkout_method: method,
                            payment_attempts: checkoutPaymentAttempts,
                          })
                        : null;

                      return (
                        <article
                          key={method.id}
                          className={catalogCardClassName}
                        >
                          <div className="flex flex-wrap items-start justify-between gap-3">
                            <div className="grid gap-2">
                              <div className="flex flex-wrap gap-2">
                                <Badge variant="default">{method.label}</Badge>
                                <Badge variant="secondary">
                                  {providerLabel}
                                </Badge>
                                <Badge variant="secondary">
                                  {checkoutMethodChannelLabel(method.channel, t)}
                                </Badge>
                                <Badge variant="secondary">
                                  {checkoutMethodSessionKindLabel(method.session_kind, t)}
                                </Badge>
                                {method.recommended ? (
                                  <Badge variant="success">{t('Recommended')}</Badge>
                                ) : null}
                                {method.supports_webhook ? (
                                  <Badge variant="warning">{t('Payment outcomes')}</Badge>
                                ) : null}
                                <Badge variant={checkoutMethodAvailabilityTone(method.availability)}>
                                  {checkoutMethodAvailabilityLabel(method.availability, t)}
                                </Badge>
                              </div>
                              <p className="text-sm text-zinc-600 dark:text-zinc-300">{method.detail}</p>
                              {checkoutLaunchDecision ? (
                                <p className="text-xs text-zinc-500 dark:text-zinc-400">
                                  {providerCheckoutLaunchDecisionDetail(
                                    checkoutLaunchDecision.kind,
                                    providerLabel,
                                    t,
                                  )}
                                </p>
                              ) : null}
                              <div className="grid gap-2 rounded-[20px] border border-dashed border-zinc-300/80 bg-white/70 p-3 text-xs text-zinc-600 dark:border-zinc-700 dark:bg-zinc-950/40 dark:text-zinc-300">
                                <div className="grid gap-1">
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Checkout reference')}</span>
                                    <code className="rounded bg-zinc-950/5 px-2 py-1 text-[11px] text-zinc-700 dark:bg-zinc-100/10 dark:text-zinc-200">
                                      {method.session_reference}
                                    </code>
                                  </div>
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Verification method')}</span>
                                    <code className="rounded bg-zinc-950/5 px-2 py-1 text-[11px] text-zinc-700 dark:bg-zinc-100/10 dark:text-zinc-200">
                                      {checkoutMethodVerificationLabel(method.webhook_verification, t)}
                                    </code>
                                  </div>
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Refund coverage')}</span>
                                    <strong className="text-zinc-950 dark:text-zinc-50">
                                      {method.supports_refund ? t('Available') : t('Unavailable')}
                                    </strong>
                                  </div>
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Partial refunds')}</span>
                                    <strong className="text-zinc-950 dark:text-zinc-50">
                                      {method.supports_partial_refund
                                        ? t('Available')
                                        : t('Unavailable')}
                                    </strong>
                                  </div>
                                </div>
                                {method.qr_code_payload ? (
                                  <div className="grid gap-1">
                                    <span>{t('QR code content')}</span>
                                    <code className="overflow-x-auto rounded bg-zinc-950/5 px-2 py-2 text-[11px] text-zinc-700 dark:bg-zinc-100/10 dark:text-zinc-200">
                                      {method.qr_code_payload}
                                    </code>
                                  </div>
                                ) : null}
                              </div>
                            </div>
                            <div className="grid justify-items-end gap-3">
                              <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                                {checkoutMethodActionLabel(method.action, t)}
                              </strong>
                              {supportsFormalProviderCheckoutLaunch(method) ? (
                                <Button
                                  disabled={providerCheckoutMethodId !== null}
                                  onClick={() => void handleProviderCheckoutLaunch(method)}
                                  variant="primary"
                                >
                                  {providerCheckoutMethodId === method.id
                                    ? t('Opening checkout...')
                                    : providerCheckoutLaunchActionLabel(
                                        checkoutLaunchDecision?.kind ?? 'create_first_attempt',
                                        t,
                                      )}
                                </Button>
                              ) : method.action === 'settle_order' && paymentSimulationEnabled && activeCheckoutOrder ? (
                                <Button
                                  disabled={queueActionOrderId !== null}
                                  onClick={() => void handleQueueAction(activeCheckoutOrder, 'settle')}
                                  variant="primary"
                                >
                                  {queueActionOrderId === activeCheckoutOrder.order_id && queueActionType === 'settle'
                                    ? t('Settling...')
                                    : t('Settle order')}
                                </Button>
                              ) : method.action === 'cancel_order' && activeCheckoutOrder ? (
                                <Button
                                  disabled={queueActionOrderId !== null}
                                  onClick={() => void handleQueueAction(activeCheckoutOrder, 'cancel')}
                                  variant="secondary"
                                >
                                  {queueActionOrderId === activeCheckoutOrder.order_id && queueActionType === 'cancel'
                                    ? t('Canceling...')
                                    : t('Cancel order')}
                                </Button>
                              ) : null}
                            </div>
                          </div>
                        </article>
                      );
                    })
                  ) : (
                      <EmptyState
                        description={t('This checkout is already closed, so there are no remaining payment actions.')}
                        title={t('No checkout methods remain')}
                      />
                  )}
                </div>

                <div className={detailCardClassName}>
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                            {t('Checkout attempts')}
                      </p>
                              <p className="text-sm text-zinc-600 dark:text-zinc-300">
                                {t(
                                  'Checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench.',
                                )}
                              </p>
                    </div>
                    {latestCheckoutPaymentAttemptId ? (
                      <Badge variant="default">{t('Latest attempt')}</Badge>
                    ) : null}
                  </div>
                  {checkoutPaymentAttempts.length ? (
                    <div className="mt-4 grid gap-3">
                      {checkoutPaymentAttempts.map((attempt) => (
                        <article
                          key={attempt.payment_attempt_id}
                          className="rounded-[24px] border border-dashed border-zinc-300/80 bg-white/70 p-4 dark:border-zinc-700 dark:bg-zinc-950/40"
                        >
                          <div className="grid gap-3">
                            <div className="flex flex-wrap items-start justify-between gap-3">
                              <div className="grid gap-2">
                                <div className="flex flex-wrap gap-2">
                                  <Badge variant={paymentAttemptStatusTone(attempt.status)}>
                                    {paymentAttemptStatusLabel(attempt.status, t)}
                                  </Badge>
                                  <Badge variant="secondary">
                                    {t('Attempt #{sequence}', { sequence: attempt.attempt_sequence })}
                                  </Badge>
                                  {latestCheckoutPaymentAttemptId === attempt.payment_attempt_id ? (
                                    <Badge variant="default">{t('Latest attempt')}</Badge>
                                  ) : null}
                                </div>
                                <div className="grid gap-2 text-sm text-zinc-600 dark:text-zinc-300">
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Reference')}</span>
                                    <code className="rounded bg-zinc-950/5 px-2 py-1 text-[11px] text-zinc-700 dark:bg-zinc-100/10 dark:text-zinc-200">
                                      {paymentAttemptReference(attempt)}
                                    </code>
                                  </div>
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Initiated')}</span>
                                    <strong className="text-zinc-950 dark:text-zinc-50">
                                      {formatDateTime(attempt.initiated_at_ms)}
                                    </strong>
                                  </div>
                                  <div className="flex flex-wrap items-center justify-between gap-2">
                                    <span>{t('Updated')}</span>
                                    <strong className="text-zinc-950 dark:text-zinc-50">
                                      {formatDateTime(attempt.updated_at_ms)}
                                    </strong>
                                  </div>
                                </div>
                              </div>
                              <div className="grid justify-items-end gap-1 text-right">
                                <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                                  {checkoutMethodProviderLabel(attempt.provider, t)}
                                </strong>
                                <span className="text-xs text-zinc-500 dark:text-zinc-400">
                                  {titleCaseToken(attempt.channel)}
                                </span>
                              </div>
                            </div>
                            {attempt.error_message ? (
                              <div className="rounded-[18px] border border-amber-200 bg-amber-50/90 px-3 py-2 text-xs text-amber-700 dark:border-amber-900/60 dark:bg-amber-950/30 dark:text-amber-200">
                                {attempt.error_message}
                              </div>
                            ) : null}
                          </div>
                        </article>
                      ))}
                    </div>
                  ) : (
                    <div className="mt-4">
                      <EmptyState
                        description={t('No checkout attempts have been recorded for this order yet.')}
                        title={t('No checkout attempts recorded yet')}
                      />
                    </div>
                  )}
                </div>

                {paymentSimulationEnabled ? (
                  hasProviderHandoff(checkoutMethods) ? (
                    <div className={detailCardClassName}>
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="grid gap-2">
                          <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                            {t('Payment outcome sandbox')}
                          </p>
                          <p className="text-sm text-zinc-600 dark:text-zinc-300">
                            {t(
                              'Apply settlement, failure, or cancellation outcomes for the selected payment method before live payment confirmation is enabled.',
                            )}
                          </p>
                        </div>
                        <Badge variant="warning">{t('Sandbox only')}</Badge>
                      </div>
                      {providerCallbackMethods.length > 1 ? (
                        <div className="mt-4 grid gap-2 md:max-w-sm">
                          <span className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                            {t('Sandbox method')}
                          </span>
                          <Select
                            onValueChange={(value: string) => setProviderCallbackMethodId(value)}
                            value={activeProviderCallbackMethod?.id ?? ''}
                          >
                            <SelectTrigger>
                              <SelectValue placeholder={t('Choose sandbox method')} />
                            </SelectTrigger>
                            <SelectContent>
                              {providerCallbackMethods.map((method) => (
                                <SelectItem key={method.id} value={method.id}>
                                  {t('{provider} / {channel}', {
                                    provider: checkoutMethodProviderLabel(method.provider, t),
                                    channel: checkoutMethodChannelLabel(method.channel, t),
                                  })}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                      ) : null}
                      {activeProviderCallbackMethod ? (
                        <div className="mt-4 rounded-[24px] border border-dashed border-zinc-300/80 bg-white/70 p-4 text-sm text-zinc-600 dark:border-zinc-700 dark:bg-zinc-950/40 dark:text-zinc-300">
                          {t('Payment outcomes will use {provider} on {channel}.', {
                            provider: activeProviderLabel ?? t('Provider'),
                            channel: activeProviderChannelLabel ?? t('Payment channel'),
                          })}
                        </div>
                      ) : null}
                      <div className="mt-4 flex flex-wrap gap-2">
                        <Button
                          disabled={providerEventOrderId !== null || !activeProviderCallbackMethod}
                          onClick={() => void handleProviderEvent('settled', activeProviderCallbackMethod)}
                          variant="primary"
                        >
                          {providerEventOrderId === checkoutSessionOrderId
                          && providerEventType === 'settled'
                            ? t('Applying settlement...')
                            : t('Apply settlement outcome')}
                        </Button>
                        <Button
                          disabled={providerEventOrderId !== null || !activeProviderCallbackMethod}
                          onClick={() => void handleProviderEvent('failed', activeProviderCallbackMethod)}
                          variant="secondary"
                        >
                          {providerEventOrderId === checkoutSessionOrderId
                          && providerEventType === 'failed'
                            ? t('Applying failure...')
                            : t('Apply failure outcome')}
                        </Button>
                        <Button
                          disabled={providerEventOrderId !== null || !activeProviderCallbackMethod}
                          onClick={() => void handleProviderEvent('canceled', activeProviderCallbackMethod)}
                          variant="secondary"
                        >
                          {providerEventOrderId === checkoutSessionOrderId
                          && providerEventType === 'canceled'
                            ? t('Applying cancel...')
                            : t('Apply cancellation outcome')}
                        </Button>
                      </div>
                    </div>
                  ) : null
                ) : null}
              </div>
            ) : (
                    <EmptyState
                      description={t('Open the checkout workbench from Pending payment queue to inspect the selected order.')}
                      title={t('No checkout selected')}
                    />
              )}
          </WorkspacePanel>

          <WorkspacePanel
            description={t('Checkout workbench keeps checkout access, selected reference, and payable price aligned under one payment method.')}
            title={t('Payment method')}
          >
            <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              <div className="flex items-center justify-between gap-3">
                <span>{t('Primary method')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutPrimaryRailLabel}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>{t('Current selected reference')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutPresentation?.reference ?? t('Awaiting pending order')}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>{t('Payable price')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutPresentation?.payable_price_label ?? checkoutSession?.payable_price_label ?? t('n/a')}
                </strong>
              </div>
            </div>
          </WorkspacePanel>
        </section>
      </div>
    </>
  );
}





