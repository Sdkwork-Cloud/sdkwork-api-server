import type {
  AdminAuthSession,
  AdminSessionUser,
  ApiKeyGroupRecord,
  BillingEventRecord,
  BillingEventSummary,
  BillingSummary,
  CampaignBudgetLifecycleAuditRecord,
  CampaignBudgetMutationResult,
  CampaignBudgetRecord,
  ChannelRecord,
  ChannelModelRecord,
  CampaignBudgetStatus,
  CommercialAccountBalanceSnapshot,
  CommercialAccountBenefitLotRecord,
  CommercialAccountLedgerHistoryEntry,
  CommercialAccountHoldRecord,
  CommercialPricingLifecycleSynchronizationReport,
  CommercialPricingPlanCreateInput,
  CommercialAccountSummary,
  CommercialPricingPlanRecord,
  CommercialPricingRateCreateInput,
  CommercialPricingRateRecord,
  CommercialRequestSettlementRecord,
  CompiledRoutingSnapshotRecord,
  CouponCodeLifecycleAuditRecord,
  CouponCodeMutationResult,
  CouponCodeRecord,
  CouponCodeStatus,
  CouponRedemptionRecord,
  CouponReservationRecord,
  CouponRollbackRecord,
  CouponTemplateComparisonResult,
  CouponTemplateLifecycleAuditRecord,
  CouponTemplateRecord,
  CouponTemplateMutationResult,
  CouponTemplateStatus,
  CreatedGatewayApiKey,
  CredentialRecord,
  GatewayApiKeyRecord,
  MarketingCampaignComparisonResult,
  MarketingCampaignLifecycleAuditRecord,
  MarketingCampaignRecord,
  MarketingCampaignMutationResult,
  MarketingCampaignStatus,
  ModelCatalogRecord,
  ModelPriceRecord,
  ModelPriceTier,
  OperatorUserRecord,
  PortalUserRecord,
  ProjectRecord,
  ProviderHealthSnapshot,
  ProviderCatalogRecord,
  ProviderModelRecord,
  ProxyProviderRecord,
  ProviderRecordWithIntegration,
  RateLimitPolicyRecord,
  RateLimitWindowRecord,
  RoutingDecisionLogRecord,
  RoutingProfileRecord,
  RuntimeReloadReport,
  RuntimeStatusRecord,
  SaveProviderInput,
  TenantRecord,
  UsageRecord,
  UsageSummary,
} from 'sdkwork-router-admin-types';
import {
  deleteEmpty,
  getJson,
  patchJson,
  postJson,
  putJson,
  requiredToken,
} from './transport';

export * from './commerce';
export {
  AdminApiError,
  adminBaseUrl,
  clearAdminSessionToken,
  persistAdminSessionToken,
  readAdminSessionToken,
} from './transport';

export function loginAdminUser(input: {
  email: string;
  password: string;
}): Promise<AdminAuthSession> {
  return postJson<typeof input, AdminAuthSession>('/auth/login', input);
}

export function getAdminMe(token?: string): Promise<AdminSessionUser> {
  return getJson<AdminSessionUser>('/auth/me', token);
}

export function listOperatorUsers(token?: string): Promise<OperatorUserRecord[]> {
  return getJson<OperatorUserRecord[]>('/users/operators', token);
}

export function saveOperatorUser(input: {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  active: boolean;
}): Promise<OperatorUserRecord> {
  return postJson<typeof input, OperatorUserRecord>('/users/operators', input, requiredToken());
}

export function updateOperatorUserStatus(
  userId: string,
  active: boolean,
): Promise<OperatorUserRecord> {
  return postJson<{ active: boolean }, OperatorUserRecord>(
    `/users/operators/${userId}/status`,
    { active },
    requiredToken(),
  );
}

export function resetOperatorUserPassword(
  userId: string,
  newPassword: string,
): Promise<OperatorUserRecord> {
  return postJson<{ new_password: string }, OperatorUserRecord>(
    `/users/operators/${userId}/password`,
    { new_password: newPassword },
    requiredToken(),
  );
}

export function deleteOperatorUser(userId: string): Promise<void> {
  return deleteEmpty(`/users/operators/${encodeURIComponent(userId)}`, requiredToken());
}

export function listPortalUsers(token?: string): Promise<PortalUserRecord[]> {
  return getJson<PortalUserRecord[]>('/users/portal', token);
}

export function listMarketingCouponTemplates(token?: string): Promise<CouponTemplateRecord[]> {
  return getJson<CouponTemplateRecord[]>('/marketing/coupon-templates', requiredToken(token));
}

export function saveMarketingCouponTemplate(
  input: CouponTemplateRecord,
): Promise<CouponTemplateRecord> {
  return postJson<CouponTemplateRecord, CouponTemplateRecord>(
    '/marketing/coupon-templates',
    input,
    requiredToken(),
  );
}

export function updateMarketingCouponTemplateStatus(
  couponTemplateId: string,
  status: CouponTemplateStatus,
): Promise<CouponTemplateRecord> {
  return postJson<{ status: CouponTemplateStatus }, CouponTemplateRecord>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/status`,
    { status },
    requiredToken(),
  );
}

export function cloneMarketingCouponTemplate(
  couponTemplateId: string,
  input: {
    coupon_template_id: string;
    template_key: string;
    display_name?: string | null;
    reason: string;
  },
): Promise<CouponTemplateMutationResult> {
  return postJson<typeof input, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/clone`,
    input,
    requiredToken(),
  );
}

export function compareMarketingCouponTemplates(
  couponTemplateId: string,
  targetCouponTemplateId: string,
): Promise<CouponTemplateComparisonResult> {
  return postJson<
    { target_coupon_template_id: string },
    CouponTemplateComparisonResult
  >(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/compare`,
    { target_coupon_template_id: targetCouponTemplateId },
    requiredToken(),
  );
}

export function submitMarketingCouponTemplateForApproval(
  couponTemplateId: string,
  reason: string,
): Promise<CouponTemplateMutationResult> {
  return postJson<{ reason: string }, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/submit-for-approval`,
    { reason },
    requiredToken(),
  );
}

export function approveMarketingCouponTemplate(
  couponTemplateId: string,
  reason: string,
): Promise<CouponTemplateMutationResult> {
  return postJson<{ reason: string }, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/approve`,
    { reason },
    requiredToken(),
  );
}

export function rejectMarketingCouponTemplate(
  couponTemplateId: string,
  reason: string,
): Promise<CouponTemplateMutationResult> {
  return postJson<{ reason: string }, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/reject`,
    { reason },
    requiredToken(),
  );
}

export function publishMarketingCouponTemplate(
  couponTemplateId: string,
  reason: string,
): Promise<CouponTemplateMutationResult> {
  return postJson<{ reason: string }, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/publish`,
    { reason },
    requiredToken(),
  );
}

export function scheduleMarketingCouponTemplate(
  couponTemplateId: string,
  reason: string,
): Promise<CouponTemplateMutationResult> {
  return postJson<{ reason: string }, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/schedule`,
    { reason },
    requiredToken(),
  );
}

export function retireMarketingCouponTemplate(
  couponTemplateId: string,
  reason: string,
): Promise<CouponTemplateMutationResult> {
  return postJson<{ reason: string }, CouponTemplateMutationResult>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/retire`,
    { reason },
    requiredToken(),
  );
}

export function listMarketingCouponTemplateLifecycleAudits(
  couponTemplateId: string,
  token?: string,
): Promise<CouponTemplateLifecycleAuditRecord[]> {
  return getJson<CouponTemplateLifecycleAuditRecord[]>(
    `/marketing/coupon-templates/${encodeURIComponent(couponTemplateId)}/lifecycle-audits`,
    requiredToken(token),
  );
}

export function listMarketingCampaigns(token?: string): Promise<MarketingCampaignRecord[]> {
  return getJson<MarketingCampaignRecord[]>('/marketing/campaigns', requiredToken(token));
}

export function saveMarketingCampaign(
  input: MarketingCampaignRecord,
): Promise<MarketingCampaignRecord> {
  return postJson<MarketingCampaignRecord, MarketingCampaignRecord>(
    '/marketing/campaigns',
    input,
    requiredToken(),
  );
}

export function updateMarketingCampaignStatus(
  marketingCampaignId: string,
  status: MarketingCampaignStatus,
): Promise<MarketingCampaignRecord> {
  return postJson<{ status: MarketingCampaignStatus }, MarketingCampaignRecord>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/status`,
    { status },
    requiredToken(),
  );
}

export function cloneMarketingCampaign(
  marketingCampaignId: string,
  input: {
    marketing_campaign_id: string;
    display_name?: string | null;
    reason: string;
  },
): Promise<MarketingCampaignMutationResult> {
  return postJson<typeof input, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/clone`,
    input,
    requiredToken(),
  );
}

export function compareMarketingCampaigns(
  marketingCampaignId: string,
  targetMarketingCampaignId: string,
): Promise<MarketingCampaignComparisonResult> {
  return postJson<
    { target_marketing_campaign_id: string },
    MarketingCampaignComparisonResult
  >(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/compare`,
    { target_marketing_campaign_id: targetMarketingCampaignId },
    requiredToken(),
  );
}

export function submitMarketingCampaignForApproval(
  marketingCampaignId: string,
  reason: string,
): Promise<MarketingCampaignMutationResult> {
  return postJson<{ reason: string }, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/submit-for-approval`,
    { reason },
    requiredToken(),
  );
}

export function approveMarketingCampaign(
  marketingCampaignId: string,
  reason: string,
): Promise<MarketingCampaignMutationResult> {
  return postJson<{ reason: string }, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/approve`,
    { reason },
    requiredToken(),
  );
}

export function rejectMarketingCampaign(
  marketingCampaignId: string,
  reason: string,
): Promise<MarketingCampaignMutationResult> {
  return postJson<{ reason: string }, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/reject`,
    { reason },
    requiredToken(),
  );
}

export function publishMarketingCampaign(
  marketingCampaignId: string,
  reason: string,
): Promise<MarketingCampaignMutationResult> {
  return postJson<{ reason: string }, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/publish`,
    { reason },
    requiredToken(),
  );
}

export function scheduleMarketingCampaign(
  marketingCampaignId: string,
  reason: string,
): Promise<MarketingCampaignMutationResult> {
  return postJson<{ reason: string }, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/schedule`,
    { reason },
    requiredToken(),
  );
}

export function retireMarketingCampaign(
  marketingCampaignId: string,
  reason: string,
): Promise<MarketingCampaignMutationResult> {
  return postJson<{ reason: string }, MarketingCampaignMutationResult>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/retire`,
    { reason },
    requiredToken(),
  );
}

export function listMarketingCampaignLifecycleAudits(
  marketingCampaignId: string,
  token?: string,
): Promise<MarketingCampaignLifecycleAuditRecord[]> {
  return getJson<MarketingCampaignLifecycleAuditRecord[]>(
    `/marketing/campaigns/${encodeURIComponent(marketingCampaignId)}/lifecycle-audits`,
    requiredToken(token),
  );
}

export function listMarketingCampaignBudgets(token?: string): Promise<CampaignBudgetRecord[]> {
  return getJson<CampaignBudgetRecord[]>('/marketing/budgets', requiredToken(token));
}

export function saveMarketingCampaignBudget(
  input: CampaignBudgetRecord,
): Promise<CampaignBudgetRecord> {
  return postJson<CampaignBudgetRecord, CampaignBudgetRecord>(
    '/marketing/budgets',
    input,
    requiredToken(),
  );
}

export function updateMarketingCampaignBudgetStatus(
  campaignBudgetId: string,
  status: CampaignBudgetStatus,
): Promise<CampaignBudgetRecord> {
  return postJson<{ status: CampaignBudgetStatus }, CampaignBudgetRecord>(
    `/marketing/budgets/${encodeURIComponent(campaignBudgetId)}/status`,
    { status },
    requiredToken(),
  );
}

export function activateMarketingCampaignBudget(
  campaignBudgetId: string,
  reason: string,
): Promise<CampaignBudgetMutationResult> {
  return postJson<{ reason: string }, CampaignBudgetMutationResult>(
    `/marketing/budgets/${encodeURIComponent(campaignBudgetId)}/activate`,
    { reason },
    requiredToken(),
  );
}

export function closeMarketingCampaignBudget(
  campaignBudgetId: string,
  reason: string,
): Promise<CampaignBudgetMutationResult> {
  return postJson<{ reason: string }, CampaignBudgetMutationResult>(
    `/marketing/budgets/${encodeURIComponent(campaignBudgetId)}/close`,
    { reason },
    requiredToken(),
  );
}

export function listMarketingCampaignBudgetLifecycleAudits(
  campaignBudgetId: string,
  token?: string,
): Promise<CampaignBudgetLifecycleAuditRecord[]> {
  return getJson<CampaignBudgetLifecycleAuditRecord[]>(
    `/marketing/budgets/${encodeURIComponent(campaignBudgetId)}/lifecycle-audits`,
    requiredToken(token),
  );
}

export function listMarketingCouponCodes(token?: string): Promise<CouponCodeRecord[]> {
  return getJson<CouponCodeRecord[]>('/marketing/codes', requiredToken(token));
}

export function saveMarketingCouponCode(input: CouponCodeRecord): Promise<CouponCodeRecord> {
  return postJson<CouponCodeRecord, CouponCodeRecord>(
    '/marketing/codes',
    input,
    requiredToken(),
  );
}

export function updateMarketingCouponCodeStatus(
  couponCodeId: string,
  status: CouponCodeStatus,
): Promise<CouponCodeRecord> {
  return postJson<{ status: CouponCodeStatus }, CouponCodeRecord>(
    `/marketing/codes/${encodeURIComponent(couponCodeId)}/status`,
    { status },
    requiredToken(),
  );
}

export function disableMarketingCouponCode(
  couponCodeId: string,
  reason: string,
): Promise<CouponCodeMutationResult> {
  return postJson<{ reason: string }, CouponCodeMutationResult>(
    `/marketing/codes/${encodeURIComponent(couponCodeId)}/disable`,
    { reason },
    requiredToken(),
  );
}

export function restoreMarketingCouponCode(
  couponCodeId: string,
  reason: string,
): Promise<CouponCodeMutationResult> {
  return postJson<{ reason: string }, CouponCodeMutationResult>(
    `/marketing/codes/${encodeURIComponent(couponCodeId)}/restore`,
    { reason },
    requiredToken(),
  );
}

export function listMarketingCouponCodeLifecycleAudits(
  couponCodeId: string,
  token?: string,
): Promise<CouponCodeLifecycleAuditRecord[]> {
  return getJson<CouponCodeLifecycleAuditRecord[]>(
    `/marketing/codes/${encodeURIComponent(couponCodeId)}/lifecycle-audits`,
    requiredToken(token),
  );
}

export function listMarketingCouponReservations(
  token?: string,
): Promise<CouponReservationRecord[]> {
  return getJson<CouponReservationRecord[]>('/marketing/reservations', requiredToken(token));
}

export function listMarketingCouponRedemptions(
  token?: string,
): Promise<CouponRedemptionRecord[]> {
  return getJson<CouponRedemptionRecord[]>('/marketing/redemptions', requiredToken(token));
}

export function listMarketingCouponRollbacks(token?: string): Promise<CouponRollbackRecord[]> {
  return getJson<CouponRollbackRecord[]>('/marketing/rollbacks', requiredToken(token));
}

export function savePortalUser(input: {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
}): Promise<PortalUserRecord> {
  return postJson<typeof input, PortalUserRecord>('/users/portal', input, requiredToken());
}

export function updatePortalUserStatus(
  userId: string,
  active: boolean,
): Promise<PortalUserRecord> {
  return postJson<{ active: boolean }, PortalUserRecord>(
    `/users/portal/${userId}/status`,
    { active },
    requiredToken(),
  );
}

export function resetPortalUserPassword(
  userId: string,
  newPassword: string,
): Promise<PortalUserRecord> {
  return postJson<{ new_password: string }, PortalUserRecord>(
    `/users/portal/${userId}/password`,
    { new_password: newPassword },
    requiredToken(),
  );
}

export function deletePortalUser(userId: string): Promise<void> {
  return deleteEmpty(`/users/portal/${encodeURIComponent(userId)}`, requiredToken());
}

export function listTenants(token?: string): Promise<TenantRecord[]> {
  return getJson<TenantRecord[]>('/tenants', token);
}

export function saveTenant(input: {
  id: string;
  name: string;
}): Promise<TenantRecord> {
  return postJson<typeof input, TenantRecord>('/tenants', input, requiredToken());
}

export function deleteTenant(tenantId: string): Promise<void> {
  return deleteEmpty(`/tenants/${encodeURIComponent(tenantId)}`, requiredToken());
}

export function listProjects(token?: string): Promise<ProjectRecord[]> {
  return getJson<ProjectRecord[]>('/projects', token);
}

export function saveProject(input: {
  tenant_id: string;
  id: string;
  name: string;
}): Promise<ProjectRecord> {
  return postJson<typeof input, ProjectRecord>('/projects', input, requiredToken());
}

export function deleteProject(projectId: string): Promise<void> {
  return deleteEmpty(`/projects/${encodeURIComponent(projectId)}`, requiredToken());
}

export function listApiKeys(token?: string): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys', token);
}

export function listApiKeyGroups(token?: string): Promise<ApiKeyGroupRecord[]> {
  return getJson<ApiKeyGroupRecord[]>('/api-key-groups', token);
}

export function createApiKeyGroup(input: {
  tenant_id: string;
  project_id: string;
  environment: string;
  name: string;
  slug?: string | null;
  description?: string | null;
  color?: string | null;
  default_capability_scope?: string | null;
  default_accounting_mode?: string | null;
  default_routing_profile_id?: string | null;
}): Promise<ApiKeyGroupRecord> {
  return postJson<typeof input, ApiKeyGroupRecord>(
    '/api-key-groups',
    input,
    requiredToken(),
  );
}

export function updateApiKeyGroup(
  groupId: string,
  input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    name: string;
    slug?: string | null;
    description?: string | null;
    color?: string | null;
    default_capability_scope?: string | null;
    default_accounting_mode?: string | null;
    default_routing_profile_id?: string | null;
  },
): Promise<ApiKeyGroupRecord> {
  return patchJson<typeof input, ApiKeyGroupRecord>(
    `/api-key-groups/${encodeURIComponent(groupId)}`,
    input,
    requiredToken(),
  );
}

export function updateApiKeyGroupStatus(
  groupId: string,
  active: boolean,
): Promise<ApiKeyGroupRecord> {
  return postJson<{ active: boolean }, ApiKeyGroupRecord>(
    `/api-key-groups/${encodeURIComponent(groupId)}/status`,
    { active },
    requiredToken(),
  );
}

export function deleteApiKeyGroup(groupId: string): Promise<void> {
  return deleteEmpty(`/api-key-groups/${encodeURIComponent(groupId)}`, requiredToken());
}

export function listRoutingProfiles(token?: string): Promise<RoutingProfileRecord[]> {
  return getJson<RoutingProfileRecord[]>('/routing/profiles', token);
}

export function createRoutingProfile(input: {
  profile_id?: string;
  tenant_id: string;
  project_id: string;
  name: string;
  slug?: string | null;
  description?: string | null;
  active?: boolean;
  strategy?: string;
  ordered_provider_ids?: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy?: boolean;
  preferred_region?: string | null;
}): Promise<RoutingProfileRecord> {
  return postJson<typeof input, RoutingProfileRecord>(
    '/routing/profiles',
    input,
    requiredToken(),
  );
}

export function listCompiledRoutingSnapshots(
  token?: string,
): Promise<CompiledRoutingSnapshotRecord[]> {
  return getJson<CompiledRoutingSnapshotRecord[]>('/routing/snapshots', token);
}

export function createApiKey(input: {
  tenant_id: string;
  project_id: string;
  environment: string;
  label?: string;
  notes?: string;
  expires_at_ms?: number | null;
  plaintext_key?: string;
  api_key_group_id?: string | null;
}): Promise<CreatedGatewayApiKey> {
  return postJson<typeof input, CreatedGatewayApiKey>('/api-keys', input, requiredToken());
}

export function updateApiKey(input: {
  hashed_key: string;
  tenant_id: string;
  project_id: string;
  environment: string;
  label: string;
  notes?: string | null;
  expires_at_ms?: number | null;
  api_key_group_id?: string | null;
}): Promise<GatewayApiKeyRecord> {
  return putJson<
    Omit<typeof input, 'hashed_key'>,
    GatewayApiKeyRecord
  >(
    `/api-keys/${encodeURIComponent(input.hashed_key)}`,
    {
      tenant_id: input.tenant_id,
      project_id: input.project_id,
      environment: input.environment,
      label: input.label,
      notes: input.notes,
      expires_at_ms: input.expires_at_ms,
      api_key_group_id: input.api_key_group_id,
    },
    requiredToken(),
  );
}

export function updateApiKeyStatus(
  hashedKey: string,
  active: boolean,
): Promise<GatewayApiKeyRecord> {
  return postJson<{ active: boolean }, GatewayApiKeyRecord>(
    `/api-keys/${encodeURIComponent(hashedKey)}/status`,
    { active },
    requiredToken(),
  );
}

export function deleteApiKey(hashedKey: string): Promise<void> {
  return deleteEmpty(`/api-keys/${encodeURIComponent(hashedKey)}`, requiredToken());
}

export function listChannels(token?: string): Promise<ChannelRecord[]> {
  return getJson<ChannelRecord[]>('/channels', token);
}

export function saveChannel(input: {
  id: string;
  name: string;
}): Promise<ChannelRecord> {
  return postJson<typeof input, ChannelRecord>('/channels', input, requiredToken());
}

export function deleteChannel(channelId: string): Promise<void> {
  return deleteEmpty(`/channels/${encodeURIComponent(channelId)}`, requiredToken());
}

export function listProviders(token?: string): Promise<ProviderCatalogRecord[]> {
  return getJson<ProviderCatalogRecord[]>('/providers', token);
}

export function saveProvider(input: SaveProviderInput): Promise<ProviderRecordWithIntegration> {
  return postJson<SaveProviderInput, ProviderRecordWithIntegration>(
    '/providers',
    input,
    requiredToken(),
  );
}

export function deleteProvider(providerId: string): Promise<void> {
  return deleteEmpty(`/providers/${encodeURIComponent(providerId)}`, requiredToken());
}

export function listCredentials(token?: string): Promise<CredentialRecord[]> {
  return getJson<CredentialRecord[]>('/credentials', token);
}

export function saveCredential(input: {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_value: string;
}): Promise<CredentialRecord> {
  return postJson<typeof input, CredentialRecord>('/credentials', input, requiredToken());
}

export function deleteCredential(
  tenantId: string,
  providerId: string,
  keyReference: string,
): Promise<void> {
  return deleteEmpty(
    `/credentials/${encodeURIComponent(tenantId)}/providers/${encodeURIComponent(providerId)}/keys/${encodeURIComponent(keyReference)}`,
    requiredToken(),
  );
}

export function listModels(token?: string): Promise<ModelCatalogRecord[]> {
  return getJson<ModelCatalogRecord[]>('/models', token);
}

export function listChannelModels(token?: string): Promise<ChannelModelRecord[]> {
  return getJson<ChannelModelRecord[]>('/channel-models', token);
}

export function listProviderModels(token?: string): Promise<ProviderModelRecord[]> {
  return getJson<ProviderModelRecord[]>('/provider-models', token);
}

export function saveChannelModel(input: {
  channel_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number | null;
  description?: string;
}): Promise<ChannelModelRecord> {
  return postJson<typeof input, ChannelModelRecord>('/channel-models', input, requiredToken());
}

export function deleteChannelModel(channelId: string, modelId: string): Promise<void> {
  return deleteEmpty(
    `/channel-models/${encodeURIComponent(channelId)}/models/${encodeURIComponent(modelId)}`,
    requiredToken(),
  );
}

export function saveProviderModel(input: {
  proxy_provider_id: string;
  channel_id: string;
  model_id: string;
  provider_model_id?: string | null;
  provider_model_family?: string | null;
  capabilities: string[];
  streaming?: boolean | null;
  context_window?: number | null;
  max_output_tokens?: number | null;
  supports_prompt_caching?: boolean;
  supports_reasoning_usage?: boolean;
  supports_tool_usage_metrics?: boolean;
  is_default_route?: boolean;
  is_active?: boolean;
}): Promise<ProviderModelRecord> {
  return postJson<typeof input, ProviderModelRecord>(
    '/provider-models',
    input,
    requiredToken(),
  );
}

export function deleteProviderModel(
  proxyProviderId: string,
  channelId: string,
  modelId: string,
): Promise<void> {
  return deleteEmpty(
    `/provider-models/${encodeURIComponent(proxyProviderId)}/channels/${encodeURIComponent(channelId)}/models/${encodeURIComponent(modelId)}`,
    requiredToken(),
  );
}

export function saveModel(input: {
  external_name: string;
  provider_id: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number;
}): Promise<ModelCatalogRecord> {
  return postJson<typeof input, ModelCatalogRecord>('/models', input, requiredToken());
}

export function deleteModel(externalName: string, providerId: string): Promise<void> {
  return deleteEmpty(
    `/models/${encodeURIComponent(externalName)}/providers/${encodeURIComponent(providerId)}`,
    requiredToken(),
  );
}

export function listModelPrices(token?: string): Promise<ModelPriceRecord[]> {
  return getJson<ModelPriceRecord[]>('/model-prices', token);
}

export function saveModelPrice(input: {
  channel_id: string;
  model_id: string;
  proxy_provider_id: string;
  currency_code: string;
  price_unit: string;
  input_price: number;
  output_price: number;
  cache_read_price: number;
  cache_write_price: number;
  request_price: number;
  price_source_kind: string;
  billing_notes?: string | null;
  pricing_tiers: ModelPriceTier[];
  is_active: boolean;
}): Promise<ModelPriceRecord> {
  return postJson<typeof input, ModelPriceRecord>('/model-prices', input, requiredToken());
}

export function deleteModelPrice(
  channelId: string,
  modelId: string,
  proxyProviderId: string,
): Promise<void> {
  return deleteEmpty(
    `/model-prices/${encodeURIComponent(channelId)}/models/${encodeURIComponent(modelId)}/providers/${encodeURIComponent(proxyProviderId)}`,
    requiredToken(),
  );
}

export function listUsageRecords(token?: string): Promise<UsageRecord[]> {
  return getJson<UsageRecord[]>('/usage/records', token);
}

export function getUsageSummary(token?: string): Promise<UsageSummary> {
  return getJson<UsageSummary>('/usage/summary', token);
}

export function getBillingSummary(token?: string): Promise<BillingSummary> {
  return getJson<BillingSummary>('/billing/summary', token);
}

export function listCommercialAccounts(
  token?: string,
): Promise<CommercialAccountSummary[]> {
  return getJson<CommercialAccountSummary[]>('/billing/accounts', token);
}

export function getCommercialAccountBalance(
  accountId: number,
  token?: string,
): Promise<CommercialAccountBalanceSnapshot> {
  return getJson<CommercialAccountBalanceSnapshot>(
    `/billing/accounts/${encodeURIComponent(String(accountId))}/balance`,
    token,
  );
}

export function listCommercialAccountBenefitLots(
  accountId: number,
  token?: string,
): Promise<CommercialAccountBenefitLotRecord[]> {
  return getJson<CommercialAccountBenefitLotRecord[]>(
    `/billing/accounts/${encodeURIComponent(String(accountId))}/benefit-lots`,
    token,
  );
}

export function listCommercialAccountLedger(
  accountId: number,
  token?: string,
): Promise<CommercialAccountLedgerHistoryEntry[]> {
  return getJson<CommercialAccountLedgerHistoryEntry[]>(
    `/billing/accounts/${encodeURIComponent(String(accountId))}/ledger`,
    token,
  );
}

export function listCommercialAccountHolds(
  token?: string,
): Promise<CommercialAccountHoldRecord[]> {
  return getJson<CommercialAccountHoldRecord[]>('/billing/account-holds', token);
}

export function listCommercialRequestSettlements(
  token?: string,
): Promise<CommercialRequestSettlementRecord[]> {
  return getJson<CommercialRequestSettlementRecord[]>(
    '/billing/request-settlements',
    token,
  );
}

export function listCommercialPricingPlans(
  token?: string,
): Promise<CommercialPricingPlanRecord[]> {
  return getJson<CommercialPricingPlanRecord[]>('/billing/pricing-plans', token);
}

export function createCommercialPricingPlan(
  input: CommercialPricingPlanCreateInput,
): Promise<CommercialPricingPlanRecord> {
  return postJson<CommercialPricingPlanCreateInput, CommercialPricingPlanRecord>(
    '/billing/pricing-plans',
    input,
    requiredToken(),
  );
}

export function updateCommercialPricingPlan(
  pricingPlanId: number,
  input: CommercialPricingPlanCreateInput,
): Promise<CommercialPricingPlanRecord> {
  return putJson<CommercialPricingPlanCreateInput, CommercialPricingPlanRecord>(
    `/billing/pricing-plans/${encodeURIComponent(String(pricingPlanId))}`,
    input,
    requiredToken(),
  );
}

export function cloneCommercialPricingPlan(
  pricingPlanId: number,
  input?: {
    plan_version?: number;
    display_name?: string | null;
    status?: string;
  },
): Promise<CommercialPricingPlanRecord> {
  return postJson<
    { plan_version?: number; display_name?: string | null; status?: string },
    CommercialPricingPlanRecord
  >(
    `/billing/pricing-plans/${encodeURIComponent(String(pricingPlanId))}/clone`,
    input ?? {},
    requiredToken(),
  );
}

export function publishCommercialPricingPlan(
  pricingPlanId: number,
): Promise<CommercialPricingPlanRecord> {
  return postJson<Record<string, never>, CommercialPricingPlanRecord>(
    `/billing/pricing-plans/${encodeURIComponent(String(pricingPlanId))}/publish`,
    {},
    requiredToken(),
  );
}

export function scheduleCommercialPricingPlan(
  pricingPlanId: number,
): Promise<CommercialPricingPlanRecord> {
  return postJson<Record<string, never>, CommercialPricingPlanRecord>(
    `/billing/pricing-plans/${encodeURIComponent(String(pricingPlanId))}/schedule`,
    {},
    requiredToken(),
  );
}

export function retireCommercialPricingPlan(
  pricingPlanId: number,
): Promise<CommercialPricingPlanRecord> {
  return postJson<Record<string, never>, CommercialPricingPlanRecord>(
    `/billing/pricing-plans/${encodeURIComponent(String(pricingPlanId))}/retire`,
    {},
    requiredToken(),
  );
}

export function synchronizeCommercialPricingLifecycle(): Promise<CommercialPricingLifecycleSynchronizationReport> {
  return postJson<Record<string, never>, CommercialPricingLifecycleSynchronizationReport>(
    '/billing/pricing-lifecycle/synchronize',
    {},
    requiredToken(),
  );
}

export function listCommercialPricingRates(
  token?: string,
): Promise<CommercialPricingRateRecord[]> {
  return getJson<CommercialPricingRateRecord[]>('/billing/pricing-rates', token);
}

export function createCommercialPricingRate(
  input: CommercialPricingRateCreateInput,
): Promise<CommercialPricingRateRecord> {
  return postJson<CommercialPricingRateCreateInput, CommercialPricingRateRecord>(
    '/billing/pricing-rates',
    input,
    requiredToken(),
  );
}

export function updateCommercialPricingRate(
  pricingRateId: number,
  input: CommercialPricingRateCreateInput,
): Promise<CommercialPricingRateRecord> {
  return putJson<CommercialPricingRateCreateInput, CommercialPricingRateRecord>(
    `/billing/pricing-rates/${encodeURIComponent(String(pricingRateId))}`,
    input,
    requiredToken(),
  );
}

export function listBillingEvents(token?: string): Promise<BillingEventRecord[]> {
  return getJson<BillingEventRecord[]>('/billing/events', token);
}

export function getBillingEventSummary(token?: string): Promise<BillingEventSummary> {
  return getJson<BillingEventSummary>('/billing/events/summary', token);
}

export function listRoutingDecisionLogs(token?: string): Promise<RoutingDecisionLogRecord[]> {
  return getJson<RoutingDecisionLogRecord[]>('/routing/decision-logs', token);
}

export function listRateLimitPolicies(token?: string): Promise<RateLimitPolicyRecord[]> {
  return getJson<RateLimitPolicyRecord[]>('/gateway/rate-limit-policies', token);
}

export function createRateLimitPolicy(input: {
  policy_id: string;
  project_id: string;
  requests_per_window: number;
  window_seconds: number;
  burst_requests: number;
  enabled: boolean;
  route_key?: string | null;
  api_key_hash?: string | null;
  model_name?: string | null;
  notes?: string | null;
}): Promise<RateLimitPolicyRecord> {
  return postJson<typeof input, RateLimitPolicyRecord>(
    '/gateway/rate-limit-policies',
    input,
    requiredToken(),
  );
}

export function listRateLimitWindows(token?: string): Promise<RateLimitWindowRecord[]> {
  return getJson<RateLimitWindowRecord[]>('/gateway/rate-limit-windows', token);
}

export function listProviderHealthSnapshots(
  token?: string,
): Promise<ProviderHealthSnapshot[]> {
  return getJson<ProviderHealthSnapshot[]>('/routing/health-snapshots', token);
}

export function listRuntimeStatuses(token?: string): Promise<RuntimeStatusRecord[]> {
  return getJson<RuntimeStatusRecord[]>('/extensions/runtime-statuses', token);
}

export function reloadExtensionRuntimes(input?: {
  extension_id?: string;
  instance_id?: string;
}): Promise<RuntimeReloadReport> {
  return postJson<typeof input, RuntimeReloadReport>(
    '/extensions/runtime-reloads',
    input ?? {},
    requiredToken(),
  );
}
