import { adminBaseUrl } from 'sdkwork-router-admin-admin-api';
import type {
  AdminAlert,
  AdminSessionUser,
  AdminWorkspaceSnapshot,
  BillingEventSummary,
  BillingSummary,
  CampaignBudgetRecord,
  CouponRecord,
  CouponCodeRecord,
  CouponTemplateRecord,
  ManagedUser,
  MarketingCampaignRecord,
  OperatorUserRecord,
  PortalUserRecord,
  UsageSummary,
} from 'sdkwork-router-admin-types';

const emptyUsageSummary: UsageSummary = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};

const emptyBillingSummary: BillingSummary = {
  total_entries: 0,
  project_count: 0,
  total_units: 0,
  total_amount: 0,
  active_quota_policy_count: 0,
  exhausted_project_count: 0,
  projects: [],
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

export const emptySnapshot: AdminWorkspaceSnapshot = {
  sessionUser: null,
  operatorUsers: [],
  portalUsers: [],
  coupons: [],
  couponTemplates: [],
  marketingCampaigns: [],
  campaignBudgets: [],
  couponCodes: [],
  couponReservations: [],
  couponRedemptions: [],
  couponRollbacks: [],
  tenants: [],
  projects: [],
  apiKeys: [],
  apiKeyGroups: [],
  routingProfiles: [],
  compiledRoutingSnapshots: [],
  rateLimitPolicies: [],
  rateLimitWindows: [],
  channels: [],
  providers: [],
  credentials: [],
  models: [],
  channelModels: [],
  providerModels: [],
  modelPrices: [],
  usageRecords: [],
  usageSummary: emptyUsageSummary,
  billingEvents: [],
  billingEventSummary: emptyBillingEventSummary,
  billingSummary: emptyBillingSummary,
  commerceOrders: [],
  commercePaymentEvents: [],
  commercialAccounts: [],
  commercialAccountHolds: [],
  commercialAccountLedger: [],
  commercialRequestSettlements: [],
  commercialPricingPlans: [],
  commercialPricingRates: [],
  routingLogs: [],
  providerHealth: [],
  runtimeStatuses: [],
  overviewMetrics: [],
  alerts: [],
};

export function buildManagedUsers(
  operatorDirectory: OperatorUserRecord[],
  portalDirectory: PortalUserRecord[],
  usageRecords: AdminWorkspaceSnapshot['usageRecords'],
  usageSummary: UsageSummary,
  billingSummary: BillingSummary,
): { operatorUsers: ManagedUser[]; portalUsers: ManagedUser[] } {
  const requestsByProject = new Map(
    usageSummary.projects.map((project) => [project.project_id, project.request_count]),
  );
  const unitsByProject = new Map(
    billingSummary.projects.map((project) => [project.project_id, project.used_units]),
  );
  const tokensByProject = new Map<string, number>();

  for (const record of usageRecords) {
    tokensByProject.set(
      record.project_id,
      (tokensByProject.get(record.project_id) ?? 0) + record.total_tokens,
    );
  }

  const operatorUsers = operatorDirectory.map<ManagedUser>((user) => ({
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    role: 'operator',
    active: user.active,
    request_count: 0,
    usage_units: 0,
    total_tokens: 0,
    source: 'live',
  }));

  const portalUsers = portalDirectory.map<ManagedUser>((user) => ({
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    role: 'portal',
    active: user.active,
    workspace_tenant_id: user.workspace_tenant_id,
    workspace_project_id: user.workspace_project_id,
    request_count: requestsByProject.get(user.workspace_project_id) ?? 0,
    usage_units: unitsByProject.get(user.workspace_project_id) ?? 0,
    total_tokens: tokensByProject.get(user.workspace_project_id) ?? 0,
    source: 'live',
  }));

  return { operatorUsers, portalUsers };
}

function marketingCampaignPriority(status: MarketingCampaignRecord['status']): number {
  switch (status) {
    case 'active':
      return 5;
    case 'scheduled':
      return 4;
    case 'paused':
      return 3;
    case 'draft':
      return 2;
    case 'ended':
      return 1;
    case 'archived':
      return 0;
    default:
      return -1;
  }
}

function campaignBudgetPriority(status: CampaignBudgetRecord['status']): number {
  switch (status) {
    case 'active':
      return 3;
    case 'exhausted':
      return 2;
    case 'draft':
      return 1;
    case 'closed':
      return 0;
    default:
      return -1;
  }
}

function compareMarketingCampaigns(
  left: MarketingCampaignRecord,
  right: MarketingCampaignRecord,
): number {
  return (
    marketingCampaignPriority(right.status) - marketingCampaignPriority(left.status)
    || right.updated_at_ms - left.updated_at_ms
  );
}

function compareCampaignBudgets(left: CampaignBudgetRecord, right: CampaignBudgetRecord): number {
  return (
    campaignBudgetPriority(right.status) - campaignBudgetPriority(left.status)
    || right.updated_at_ms - left.updated_at_ms
  );
}

function canonicalCouponDiscountLabel(template: CouponTemplateRecord): string {
  switch (template.benefit.benefit_kind) {
    case 'percentage_off':
      return template.benefit.subsidy_percent != null
        ? `${template.benefit.subsidy_percent}% off`
        : template.display_name || template.template_key;
    case 'fixed_amount_off':
      return template.benefit.subsidy_amount_minor != null
        ? `${template.benefit.subsidy_amount_minor} off`
        : template.display_name || template.template_key;
    case 'grant_units':
      return template.benefit.grant_units != null
        ? `${template.benefit.grant_units} units`
        : template.display_name || template.template_key;
    default:
      return template.display_name || template.template_key;
  }
}

function canonicalCouponNote(
  template: CouponTemplateRecord,
  campaign: MarketingCampaignRecord | null,
): string {
  const campaignName = campaign?.display_name?.trim();
  if (campaignName) {
    return campaignName;
  }

  const templateName = template.display_name.trim();
  return templateName || template.template_key;
}

function canonicalCouponExpiry(
  code: CouponCodeRecord,
  campaign: MarketingCampaignRecord | null,
): string {
  const timestamp = code.expires_at_ms ?? campaign?.end_at_ms ?? null;
  if (timestamp == null || !Number.isFinite(timestamp) || timestamp <= 0) {
    return '';
  }

  return new Date(timestamp).toISOString().slice(0, 10);
}

function canonicalCouponRemaining(
  budget: CampaignBudgetRecord | null,
  code: CouponCodeRecord,
): number {
  if (!budget) {
    return code.status === 'available' || code.status === 'reserved' ? 1 : 0;
  }

  return Math.max(
    budget.total_budget_minor - budget.reserved_budget_minor - budget.consumed_budget_minor,
    0,
  );
}

function isCanonicalCouponActive(
  template: CouponTemplateRecord,
  campaign: MarketingCampaignRecord | null,
  budget: CampaignBudgetRecord | null,
  code: CouponCodeRecord,
  nowMs: number,
): boolean {
  const templateActive = template.status === 'active';
  const campaignActive = !campaign || (
    campaign.status === 'active'
    && (campaign.start_at_ms == null || campaign.start_at_ms <= nowMs)
    && (campaign.end_at_ms == null || campaign.end_at_ms >= nowMs)
  );
  const budgetActive = !budget || (
    budget.status === 'active'
    && canonicalCouponRemaining(budget, code) > 0
  );
  const codeActive = (
    (code.status === 'available' || code.status === 'reserved')
    && (code.expires_at_ms == null || code.expires_at_ms >= nowMs)
  );

  return templateActive && campaignActive && budgetActive && codeActive;
}

function deriveCoupons(
  liveData: Pick<
    AdminWorkspaceSnapshot,
    'couponTemplates' | 'marketingCampaigns' | 'campaignBudgets' | 'couponCodes'
  >,
): CouponRecord[] {
  const campaignsByTemplate = new Map<string, MarketingCampaignRecord>();
  for (const campaign of liveData.marketingCampaigns) {
    const existing = campaignsByTemplate.get(campaign.coupon_template_id);
    if (!existing || compareMarketingCampaigns(existing, campaign) > 0) {
      campaignsByTemplate.set(campaign.coupon_template_id, campaign);
    }
  }

  const budgetsByCampaign = new Map<string, CampaignBudgetRecord>();
  for (const budget of liveData.campaignBudgets) {
    const existing = budgetsByCampaign.get(budget.marketing_campaign_id);
    if (!existing || compareCampaignBudgets(existing, budget) > 0) {
      budgetsByCampaign.set(budget.marketing_campaign_id, budget);
    }
  }

  const codesByTemplate = new Map<string, CouponCodeRecord[]>();
  for (const code of liveData.couponCodes) {
    const templateCodes = codesByTemplate.get(code.coupon_template_id) ?? [];
    templateCodes.push(code);
    codesByTemplate.set(code.coupon_template_id, templateCodes);
  }
  for (const templateCodes of codesByTemplate.values()) {
    templateCodes.sort((left, right) => (
      left.code_value.localeCompare(right.code_value)
      || right.updated_at_ms - left.updated_at_ms
    ));
  }

  const nowMs = Date.now();
  const coupons: CouponRecord[] = [];
  for (const template of liveData.couponTemplates) {
    const templateCodes = codesByTemplate.get(template.coupon_template_id);
    if (!templateCodes?.length) {
      continue;
    }

    const campaign = campaignsByTemplate.get(template.coupon_template_id) ?? null;
    const budget = campaign
      ? budgetsByCampaign.get(campaign.marketing_campaign_id) ?? null
      : null;

    for (const code of templateCodes) {
      coupons.push({
        id: code.coupon_code_id,
        code: code.code_value,
        discount_label: canonicalCouponDiscountLabel(template),
        audience: template.restriction.subject_scope,
        remaining: canonicalCouponRemaining(budget, code),
        active: isCanonicalCouponActive(template, campaign, budget, code, nowMs),
        note: canonicalCouponNote(template, campaign),
        expires_on: canonicalCouponExpiry(code, campaign),
      });
    }
  }

  coupons.sort((left, right) => (
    left.code.localeCompare(right.code)
    || left.id.localeCompare(right.id)
  ));
  return coupons;
}

export function buildOverviewMetrics(
  snapshot: Omit<AdminWorkspaceSnapshot, 'overviewMetrics' | 'alerts'>,
) {
  const coveredProviders = new Set(
    snapshot.credentials.map((credential) => credential.provider_id),
  );

  return [
    {
      label: 'Admin API base',
      value: adminBaseUrl(),
      detail: 'Independent admin project talking to the operator control plane.',
    },
    {
      label: 'Managed users',
      value: String(snapshot.operatorUsers.length + snapshot.portalUsers.length),
      detail: 'Combined operator and portal inventory.',
    },
    {
      label: 'Active models',
      value: String(snapshot.models.length),
      detail: 'Models currently exposed through the routing catalog.',
    },
    {
      label: 'Credential coverage',
      value: `${coveredProviders.size}/${snapshot.providers.length}`,
      detail: 'Providers currently backed by at least one upstream credential.',
    },
    {
      label: 'Request volume',
      value: String(snapshot.usageSummary.total_requests),
      detail: 'Total requests recorded by the usage summary.',
    },
  ];
}

export function buildAlerts(
  snapshot: Omit<AdminWorkspaceSnapshot, 'overviewMetrics' | 'alerts'>,
): AdminAlert[] {
  const alerts: AdminAlert[] = [];
  const coveredProviders = new Set(
    snapshot.credentials.map((credential) => credential.provider_id),
  );
  const providersWithoutCredential = snapshot.providers.filter(
    (provider) => !coveredProviders.has(provider.id),
  );

  if (!snapshot.models.length) {
    alerts.push({
      id: 'no-models',
      title: 'No model catalog entries',
      detail: 'The routing layer has no published models. Create or upsert models in Catalog.',
      severity: 'high',
    });
  }

  if (snapshot.billingSummary.exhausted_project_count > 0) {
    alerts.push({
      id: 'quota-exhausted',
      title: 'Projects with exhausted quota',
      detail: `${snapshot.billingSummary.exhausted_project_count} projects have exhausted their quota posture.`,
      severity: 'high',
    });
  }

  if (snapshot.runtimeStatuses.some((runtime) => !runtime.healthy)) {
    alerts.push({
      id: 'runtime-risk',
      title: 'Runtime health degradation detected',
      detail: 'One or more managed runtimes are unhealthy. Review the Operations module.',
      severity: 'medium',
    });
  }

  if (providersWithoutCredential.length > 0) {
    alerts.push({
      id: 'credential-gap',
      title: 'Provider credentials are missing',
      detail: `${providersWithoutCredential.length} providers have no credential coverage. Rotate or create credentials in Catalog before routing live traffic.`,
      severity: 'medium',
    });
  }

  alerts.push({
    id: 'coupon-repository',
    title: 'Coupon workspace is canonical-derived',
    detail: 'Coupon rows now derive from canonical template, campaign, budget, and code governance.',
    severity: 'low',
  });

  return alerts;
}

export function buildSnapshot(
  sessionUser: AdminSessionUser,
  liveData: Omit<
    AdminWorkspaceSnapshot,
    'sessionUser' | 'operatorUsers' | 'portalUsers' | 'coupons' | 'overviewMetrics' | 'alerts'
  > & {
    operatorDirectory: OperatorUserRecord[];
    portalDirectory: PortalUserRecord[];
  },
): AdminWorkspaceSnapshot {
  const { operatorUsers, portalUsers } = buildManagedUsers(
    liveData.operatorDirectory,
    liveData.portalDirectory,
    liveData.usageRecords,
    liveData.usageSummary,
    liveData.billingSummary,
  );
  const {
    operatorDirectory: _operatorDirectory,
    portalDirectory: _portalDirectory,
    ...workspaceData
  } = liveData;

  const base = {
    sessionUser,
    operatorUsers,
    portalUsers,
    ...workspaceData,
    coupons: deriveCoupons(workspaceData),
  };

  return {
    ...base,
    overviewMetrics: buildOverviewMetrics(base),
    alerts: buildAlerts(base),
  };
}
