import type {
  AdminAlert,
  AdminWorkspaceSnapshot,
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

function normalizeSnapshot(snapshot: Partial<AdminWorkspaceSnapshot>): AdminWorkspaceSnapshot {
  return {
    sessionUser: snapshot.sessionUser ?? null,
    operatorUsers: safeArray(snapshot.operatorUsers),
    portalUsers: safeArray(snapshot.portalUsers),
    coupons: safeArray(snapshot.coupons),
    tenants: safeArray(snapshot.tenants),
    projects: safeArray(snapshot.projects),
    apiKeys: safeArray(snapshot.apiKeys),
    rateLimitPolicies: safeArray(snapshot.rateLimitPolicies),
    rateLimitWindows: safeArray(snapshot.rateLimitWindows),
    channels: safeArray(snapshot.channels),
    providers: safeArray(snapshot.providers),
    credentials: safeArray(snapshot.credentials),
    models: safeArray(snapshot.models),
    channelModels: safeArray(snapshot.channelModels),
    modelPrices: safeArray(snapshot.modelPrices),
    usageRecords: safeArray(snapshot.usageRecords),
    usageSummary: normalizeUsageSummary(snapshot.usageSummary),
    billingSummary: normalizeBillingSummary(snapshot.billingSummary),
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
