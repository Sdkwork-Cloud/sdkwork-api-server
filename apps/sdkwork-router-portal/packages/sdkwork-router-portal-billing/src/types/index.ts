import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventGroupSummary,
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
  PaymentMethodRecord,
  PortalCommerceCheckoutSession,
  PortalCommerceCheckoutSessionMethod,
  PortalCommerceCheckoutSessionStatus,
  PortalCommerceMembership,
  PortalCommerceReconciliationSummary,
  PortalCommerceOrder,
  PortalCommercePaymentEventProcessingStatus,
  PortalCommercePaymentEventType,
  PortalCommerceQuote,
  PortalRouteKey,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

export interface PortalBillingPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface BillingRunway {
  label: string;
  detail: string;
  projected_days: number | null;
  daily_units: number | null;
}

export interface BillingBundleRecommendation {
  title: string;
  detail: string;
}

export interface BillingRecommendation {
  title: string;
  detail: string;
  plan: SubscriptionPlan | null;
  pack: RechargePack | null;
  runway: BillingRunway;
  bundle: BillingBundleRecommendation;
}

export type BillingPaymentHistoryRowKind =
  | 'payment_event'
  | 'refunded_order_state';

export interface BillingPaymentHistoryRow {
  row_kind: BillingPaymentHistoryRowKind;
  id: string;
  order_id: string;
  target_name: string;
  target_kind: PortalCommerceOrder['target_kind'];
  payable_price_label: string;
  order_status: PortalCommerceOrder['status'];
  order_status_after?: PortalCommerceOrder['status'] | null;
  provider: string;
  event_type: PortalCommercePaymentEventType;
  payment_event_id?: string | null;
  provider_event_id?: string | null;
  payment_method_name?: string | null;
  processing_status?: PortalCommercePaymentEventProcessingStatus | null;
  processing_message?: string | null;
  checkout_reference?: string | null;
  checkout_session_status?: PortalCommerceCheckoutSessionStatus | null;
  guidance?: string | null;
  received_at_ms: number;
  processed_at_ms?: number | null;
}

export interface BillingPageData {
  summary: ProjectBillingSummary;
  usage_records: UsageRecord[];
  billing_events: BillingEventRecord[];
  billing_event_summary: BillingEventSummary;
  plans: SubscriptionPlan[];
  packs: RechargePack[];
  orders: PortalCommerceOrder[];
  payment_history: BillingPaymentHistoryRow[];
  refund_history: BillingPaymentHistoryRow[];
  payment_simulation_enabled: boolean;
  membership: PortalCommerceMembership | null;
  commercial_reconciliation: PortalCommerceReconciliationSummary | null;
  commercial_account: CommercialAccountSummary | null;
  commercial_balance: CommercialAccountBalanceSnapshot | null;
  commercial_benefit_lots: CommercialAccountBenefitLotRecord[];
  commercial_holds: CommercialAccountHoldRecord[];
  commercial_request_settlements: CommercialRequestSettlementRecord[];
  commercial_pricing_plans: CommercialPricingPlanRecord[];
  commercial_pricing_rates: CommercialPricingRateRecord[];
}

export interface BillingCheckoutDetail {
  order: PortalCommerceOrder;
  checkout_session: PortalCommerceCheckoutSession;
  checkout_methods: PortalCommerceCheckoutSessionMethod[];
  payment_attempts: CommercePaymentAttemptRecord[];
  payment_methods: PaymentMethodRecord[];
  latest_payment_attempt: CommercePaymentAttemptRecord | null;
  selected_payment_method: PaymentMethodRecord | null;
}

export interface BillingEventAnalyticsTotals {
  total_events: number;
  total_request_count: number;
  total_tokens: number;
  total_image_count: number;
  total_audio_seconds: number;
  total_video_seconds: number;
  total_music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingRoutingEvidence {
  events_with_profile: number;
  events_with_compiled_snapshot: number;
  events_with_fallback_reason: number;
}

export interface BillingEventAnalyticsViewModel {
  totals: BillingEventAnalyticsTotals;
  top_capabilities: BillingEventCapabilitySummary[];
  group_chargeback: BillingEventGroupSummary[];
  accounting_mode_mix: BillingEventAccountingModeSummary[];
  recent_events: BillingEventRecord[];
  routing_evidence: BillingRoutingEvidence;
}

export type BillingCheckoutPreview = PortalCommerceQuote;
