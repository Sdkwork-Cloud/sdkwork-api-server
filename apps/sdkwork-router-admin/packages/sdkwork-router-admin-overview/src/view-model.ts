import type {
  AdminAlert,
  AdminWorkspaceSnapshot,
  BillingEventSummary,
  BillingSummary,
  ManagedUser,
  OverviewMetric,
  ProjectRecord,
  UsageSummary,
} from 'sdkwork-router-admin-types';

type RankedProject = ProjectRecord & {
  request_count: number;
  total_tokens: number;
  booked_amount: number;
};

export interface AdminOverviewViewModel {
  snapshot: AdminWorkspaceSnapshot;
  metrics: OverviewMetric[];
  alerts: AdminAlert[];
  rankedUsers: ManagedUser[];
  rankedProjects: RankedProject[];
}

function safeArray<T>(value: T[] | null | undefined): T[] {
  return Array.isArray(value) ? value : [];
}

function normalizeUsageSummary(summary?: Partial<UsageSummary> | null): UsageSummary {
  return {
    total_requests: summary?.total_requests ?? 0,
    project_count: summary?.project_count ?? 0,
    model_count: summary?.model_count ?? 0,
    provider_count: summary?.provider_count ?? 0,
    projects: safeArray(summary?.projects),
    providers: safeArray(summary?.providers),
    models: safeArray(summary?.models),
  };
}

function normalizeBillingSummary(summary?: Partial<BillingSummary> | null): BillingSummary {
  return {
    total_entries: summary?.total_entries ?? 0,
    project_count: summary?.project_count ?? 0,
    total_units: summary?.total_units ?? 0,
    total_amount: summary?.total_amount ?? 0,
    active_quota_policy_count: summary?.active_quota_policy_count ?? 0,
    exhausted_project_count: summary?.exhausted_project_count ?? 0,
    projects: safeArray(summary?.projects),
  };
}

function normalizeBillingEventSummary(
  summary?: Partial<BillingEventSummary> | null,
): BillingEventSummary {
  return {
    total_events: summary?.total_events ?? 0,
    project_count: summary?.project_count ?? 0,
    group_count: summary?.group_count ?? 0,
    capability_count: summary?.capability_count ?? 0,
    total_request_count: summary?.total_request_count ?? 0,
    total_units: summary?.total_units ?? 0,
    total_input_tokens: summary?.total_input_tokens ?? 0,
    total_output_tokens: summary?.total_output_tokens ?? 0,
    total_tokens: summary?.total_tokens ?? 0,
    total_image_count: summary?.total_image_count ?? 0,
    total_audio_seconds: summary?.total_audio_seconds ?? 0,
    total_video_seconds: summary?.total_video_seconds ?? 0,
    total_music_seconds: summary?.total_music_seconds ?? 0,
    total_upstream_cost: summary?.total_upstream_cost ?? 0,
    total_customer_charge: summary?.total_customer_charge ?? 0,
    projects: safeArray(summary?.projects),
    groups: safeArray(summary?.groups),
    capabilities: safeArray(summary?.capabilities),
    accounting_modes: safeArray(summary?.accounting_modes),
  };
}

function normalizeSnapshot(snapshot: Partial<AdminWorkspaceSnapshot>): AdminWorkspaceSnapshot {
  return {
    sessionUser: snapshot.sessionUser ?? null,
    operatorUsers: safeArray(snapshot.operatorUsers),
    portalUsers: safeArray(snapshot.portalUsers),
    coupons: safeArray(snapshot.coupons),
    couponTemplates: safeArray(snapshot.couponTemplates),
    marketingCampaigns: safeArray(snapshot.marketingCampaigns),
    campaignBudgets: safeArray(snapshot.campaignBudgets),
    couponCodes: safeArray(snapshot.couponCodes),
    couponReservations: safeArray(snapshot.couponReservations),
    couponRedemptions: safeArray(snapshot.couponRedemptions),
    couponRollbacks: safeArray(snapshot.couponRollbacks),
    tenants: safeArray(snapshot.tenants),
    projects: safeArray(snapshot.projects),
    apiKeys: safeArray(snapshot.apiKeys),
    apiKeyGroups: safeArray(snapshot.apiKeyGroups),
    routingProfiles: safeArray(snapshot.routingProfiles),
    compiledRoutingSnapshots: safeArray(snapshot.compiledRoutingSnapshots),
    rateLimitPolicies: safeArray(snapshot.rateLimitPolicies),
    rateLimitWindows: safeArray(snapshot.rateLimitWindows),
    channels: safeArray(snapshot.channels),
    providers: safeArray(snapshot.providers),
    credentials: safeArray(snapshot.credentials),
    models: safeArray(snapshot.models),
    channelModels: safeArray(snapshot.channelModels),
    providerModels: safeArray(snapshot.providerModels),
    modelPrices: safeArray(snapshot.modelPrices),
    usageRecords: safeArray(snapshot.usageRecords),
    usageSummary: normalizeUsageSummary(snapshot.usageSummary),
    billingEvents: safeArray(snapshot.billingEvents),
    billingEventSummary: normalizeBillingEventSummary(snapshot.billingEventSummary),
    billingSummary: normalizeBillingSummary(snapshot.billingSummary),
    commerceOrders: safeArray(snapshot.commerceOrders),
    commercePaymentEvents: safeArray(snapshot.commercePaymentEvents),
    commercialAccounts: safeArray(snapshot.commercialAccounts),
    commercialAccountLedger: safeArray(snapshot.commercialAccountLedger),
    commercialAccountHolds: safeArray(snapshot.commercialAccountHolds),
    commercialRequestSettlements: safeArray(snapshot.commercialRequestSettlements),
    commercialPricingPlans: safeArray(snapshot.commercialPricingPlans),
    commercialPricingRates: safeArray(snapshot.commercialPricingRates),
    routingLogs: safeArray(snapshot.routingLogs),
    providerHealth: safeArray(snapshot.providerHealth),
    runtimeStatuses: safeArray(snapshot.runtimeStatuses),
    overviewMetrics: safeArray(snapshot.overviewMetrics),
    alerts: safeArray(snapshot.alerts),
  };
}

function topPortalUsers(snapshot: AdminWorkspaceSnapshot): ManagedUser[] {
  return [...snapshot.portalUsers]
    .sort((left, right) => (
      right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || right.usage_units - left.usage_units
    ))
    .slice(0, 5);
}

function hottestProjects(snapshot: AdminWorkspaceSnapshot): RankedProject[] {
  const tokensByProject = new Map<string, number>();
  for (const record of snapshot.usageRecords) {
    tokensByProject.set(
      record.project_id,
      (tokensByProject.get(record.project_id) ?? 0) + record.total_tokens,
    );
  }

  return snapshot.projects
    .map((project) => {
      const traffic = snapshot.usageSummary.projects.find(
        (summary) => summary.project_id === project.id,
      );
      const billing = snapshot.billingSummary.projects.find(
        (summary) => summary.project_id === project.id,
      );

      return {
        ...project,
        request_count: traffic?.request_count ?? 0,
        total_tokens: tokensByProject.get(project.id) ?? 0,
        booked_amount: billing?.booked_amount ?? 0,
      };
    })
    .sort((left, right) => (
      right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || right.booked_amount - left.booked_amount
    ))
    .slice(0, 5);
}

export function buildAdminOverviewViewModel(
  snapshot: Partial<AdminWorkspaceSnapshot>,
): AdminOverviewViewModel {
  const normalizedSnapshot = normalizeSnapshot(snapshot);

  return {
    snapshot: normalizedSnapshot,
    metrics: normalizedSnapshot.overviewMetrics,
    alerts: normalizedSnapshot.alerts,
    rankedUsers: topPortalUsers(normalizedSnapshot),
    rankedProjects: hottestProjects(normalizedSnapshot),
  };
}
