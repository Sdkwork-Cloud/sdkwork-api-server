import {
  cloneCommercialPricingPlan,
  createApiKey,
  createApiKeyGroup,
  createCommercialPricingPlan,
  createCommercialPricingRate,
  createRoutingProfile,
  createRateLimitPolicy,
  deleteApiKey,
  deleteApiKeyGroup,
  deleteChannel,
  deleteChannelModel,
  deleteCredential,
  deleteModel,
  deleteModelPrice,
  deleteOperatorUser,
  deletePortalUser,
  deleteProject,
  deleteProvider,
  deleteTenant,
  reloadExtensionRuntimes,
  scheduleCommercialPricingPlan,
  retireCommercialPricingPlan,
  saveChannel,
  saveChannelModel,
  saveCredential,
  saveModel,
  saveModelPrice,
  updateMarketingCampaignBudgetStatus,
  updateMarketingCampaignStatus,
  updateMarketingCouponCodeStatus,
  updateMarketingCouponTemplateStatus,
  saveOperatorUser,
  savePortalUser,
  saveProject,
  saveProvider,
  saveTenant,
  publishCommercialPricingPlan,
  updateApiKey,
  updateApiKeyGroup,
  updateApiKeyGroupStatus,
  updateCommercialPricingPlan,
  updateCommercialPricingRate,
  updateApiKeyStatus,
  updateOperatorUserStatus,
  updatePortalUserStatus,
  synchronizeCommercialPricingLifecycle,
} from 'sdkwork-router-admin-admin-api';
import type {
  CommercialPricingLifecycleSynchronizationReport,
  CampaignBudgetStatus,
  CommercialPricingPlanCreateInput,
  CommercialPricingPlanRecord,
  CommercialPricingRateCreateInput,
  CouponCodeStatus,
  CouponTemplateStatus,
  CreatedGatewayApiKey,
  MarketingCampaignStatus,
  ModelPriceTier,
  RuntimeReloadReport,
  SaveProviderInput,
} from 'sdkwork-router-admin-types';

export type SaveOperatorUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  active: boolean;
};

export type SavePortalUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

export type SaveModelInput = {
  external_name: string;
  provider_id: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number;
};

export type SaveChannelModelInput = {
  channel_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number | null;
  description?: string;
};

export type SaveModelPriceInput = {
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
};

export type CreateRateLimitPolicyInput = {
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
};

export type SaveApiKeyGroupInput = {
  group_id?: string;
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
};

export type CreateRoutingProfileInput = {
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
};

export interface WorkbenchActions {
  handleSaveOperatorUser: (input: SaveOperatorUserInput) => Promise<void>;
  handleSavePortalUser: (input: SavePortalUserInput) => Promise<void>;
  handleToggleOperatorUser: (userId: string, active: boolean) => Promise<void>;
  handleTogglePortalUser: (userId: string, active: boolean) => Promise<void>;
  handleDeleteOperatorUser: (userId: string) => Promise<void>;
  handleDeletePortalUser: (userId: string) => Promise<void>;
  handleUpdateMarketingCouponTemplateStatus: (
    couponTemplateId: string,
    status: CouponTemplateStatus,
  ) => Promise<void>;
  handleUpdateMarketingCampaignStatus: (
    marketingCampaignId: string,
    status: MarketingCampaignStatus,
  ) => Promise<void>;
  handleUpdateMarketingCampaignBudgetStatus: (
    campaignBudgetId: string,
    status: CampaignBudgetStatus,
  ) => Promise<void>;
  handleUpdateMarketingCouponCodeStatus: (
    couponCodeId: string,
    status: CouponCodeStatus,
  ) => Promise<void>;
  handleSaveTenant: (input: { id: string; name: string }) => Promise<void>;
  handleDeleteTenant: (tenantId: string) => Promise<void>;
  handleSaveProject: (input: {
    tenant_id: string;
    id: string;
    name: string;
  }) => Promise<void>;
  handleCreateRoutingProfile: (input: CreateRoutingProfileInput) => Promise<void>;
  handleSaveApiKeyGroup: (input: SaveApiKeyGroupInput) => Promise<void>;
  handleToggleApiKeyGroup: (groupId: string, active: boolean) => Promise<void>;
  handleDeleteApiKeyGroup: (groupId: string) => Promise<void>;
  handleCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    label?: string;
    notes?: string;
    expires_at_ms?: number | null;
    plaintext_key?: string;
    api_key_group_id?: string | null;
  }) => Promise<CreatedGatewayApiKey>;
  handleCreateCommercialPricingPlan: (
    input: CommercialPricingPlanCreateInput,
  ) => Promise<void>;
  handleCreateCommercialPricingRate: (
    input: CommercialPricingRateCreateInput,
  ) => Promise<void>;
  handleUpdateCommercialPricingPlan: (
    pricingPlanId: number,
    input: CommercialPricingPlanCreateInput,
  ) => Promise<void>;
  handleCloneCommercialPricingPlan: (
    pricingPlanId: number,
    input?: {
      plan_version?: number;
      display_name?: string | null;
      status?: string;
    },
  ) => Promise<CommercialPricingPlanRecord>;
  handlePublishCommercialPricingPlan: (
    pricingPlanId: number,
  ) => Promise<CommercialPricingPlanRecord>;
  handleScheduleCommercialPricingPlan: (
    pricingPlanId: number,
  ) => Promise<CommercialPricingPlanRecord>;
  handleRetireCommercialPricingPlan: (
    pricingPlanId: number,
  ) => Promise<CommercialPricingPlanRecord>;
  handleSynchronizeCommercialPricingLifecycle: () => Promise<CommercialPricingLifecycleSynchronizationReport>;
  handleUpdateCommercialPricingRate: (
    pricingRateId: number,
    input: CommercialPricingRateCreateInput,
  ) => Promise<void>;
  handleUpdateApiKey: (input: {
    hashed_key: string;
    tenant_id: string;
    project_id: string;
    environment: string;
    label: string;
    notes?: string | null;
    expires_at_ms?: number | null;
    api_key_group_id?: string | null;
  }) => Promise<void>;
  handleCreateRateLimitPolicy: (input: CreateRateLimitPolicyInput) => Promise<void>;
  handleUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  handleDeleteApiKey: (hashedKey: string) => Promise<void>;
  handleReloadRuntimes: (input?: {
    extension_id?: string;
    instance_id?: string;
  }) => Promise<RuntimeReloadReport>;
  handleDeleteProject: (projectId: string) => Promise<void>;
  handleSaveChannel: (input: { id: string; name: string }) => Promise<void>;
  handleDeleteChannel: (channelId: string) => Promise<void>;
  handleSaveProvider: (input: SaveProviderInput) => Promise<void>;
  handleDeleteProvider: (providerId: string) => Promise<void>;
  handleSaveModel: (input: SaveModelInput) => Promise<void>;
  handleSaveChannelModel: (input: SaveChannelModelInput) => Promise<void>;
  handleDeleteChannelModel: (channelId: string, modelId: string) => Promise<void>;
  handleSaveModelPrice: (input: SaveModelPriceInput) => Promise<void>;
  handleDeleteModelPrice: (
    channelId: string,
    modelId: string,
    proxyProviderId: string,
  ) => Promise<void>;
  handleSaveCredential: (input: {
    tenant_id: string;
    provider_id: string;
    key_reference: string;
    secret_value: string;
  }) => Promise<void>;
  handleDeleteCredential: (
    tenantId: string,
    providerId: string,
    keyReference: string,
  ) => Promise<void>;
  handleDeleteModel: (externalName: string, providerId: string) => Promise<void>;
}

type WorkbenchActionDeps = {
  refreshWorkspace: () => Promise<void>;
  setStatus: (status: string) => void;
};

type RefreshingStatus<T> = string | ((result: T) => string);

function resolveStatus<T>(status: RefreshingStatus<T>, result: T) {
  return typeof status === 'function' ? status(result) : status;
}

function resolveErrorStatus(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

async function runRefreshingAction<T>({
  action,
  failureStatus,
  refreshWorkspace,
  rethrow = false,
  setStatus,
  startStatus,
  successStatus,
}: {
  action: () => Promise<T>;
  failureStatus: string;
  refreshWorkspace: () => Promise<void>;
  rethrow?: boolean;
  setStatus: (status: string) => void;
  startStatus: string;
  successStatus: RefreshingStatus<T>;
}) {
  setStatus(startStatus);

  try {
    const result = await action();
    await refreshWorkspace();
    setStatus(resolveStatus(successStatus, result));
    return result;
  } catch (error) {
    setStatus(resolveErrorStatus(error, failureStatus));
    if (rethrow) {
      throw error;
    }
  }

  return undefined as T;
}

export function createWorkbenchActions({
  refreshWorkspace,
  setStatus,
}: WorkbenchActionDeps): WorkbenchActions {
  return {
    async handleSaveOperatorUser(input) {
      await runRefreshingAction({
        action: () => saveOperatorUser(input),
        failureStatus: 'Failed to save operator user.',
        refreshWorkspace,
        setStatus,
        startStatus: input.id
          ? 'Updating operator identity...'
          : 'Provisioning operator identity...',
        successStatus: 'Operator user saved.',
      });
    },

    async handleSavePortalUser(input) {
      await runRefreshingAction({
        action: () => savePortalUser(input),
        failureStatus: 'Failed to save portal user.',
        refreshWorkspace,
        setStatus,
        startStatus: input.id
          ? 'Updating portal identity...'
          : 'Provisioning portal identity...',
        successStatus: 'Portal user saved.',
      });
    },

    async handleToggleOperatorUser(userId, active) {
      await runRefreshingAction({
        action: () => updateOperatorUserStatus(userId, active),
        failureStatus: 'Failed to update operator access.',
        refreshWorkspace,
        setStatus,
        startStatus: active
          ? 'Re-activating operator access...'
          : 'Disabling operator access...',
        successStatus: 'Operator access updated.',
      });
    },

    async handleTogglePortalUser(userId, active) {
      await runRefreshingAction({
        action: () => updatePortalUserStatus(userId, active),
        failureStatus: 'Failed to update portal access.',
        refreshWorkspace,
        setStatus,
        startStatus: active
          ? 'Re-activating portal access...'
          : 'Disabling portal access...',
        successStatus: 'Portal access updated.',
      });
    },

    async handleDeleteOperatorUser(userId) {
      await runRefreshingAction({
        action: () => deleteOperatorUser(userId),
        failureStatus: 'Failed to delete operator user.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting operator identity...',
        successStatus: 'Operator user deleted.',
      });
    },

    async handleDeletePortalUser(userId) {
      await runRefreshingAction({
        action: () => deletePortalUser(userId),
        failureStatus: 'Failed to delete portal user.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting portal identity...',
        successStatus: 'Portal user deleted.',
      });
    },

    async handleUpdateMarketingCouponTemplateStatus(couponTemplateId, status) {
      await runRefreshingAction({
        action: () => updateMarketingCouponTemplateStatus(couponTemplateId, status),
        failureStatus: 'Failed to update coupon template status.',
        refreshWorkspace,
        setStatus,
        startStatus: `Updating coupon template ${couponTemplateId} status...`,
        successStatus: 'Coupon template status updated.',
      });
    },

    async handleUpdateMarketingCampaignStatus(marketingCampaignId, status) {
      await runRefreshingAction({
        action: () => updateMarketingCampaignStatus(marketingCampaignId, status),
        failureStatus: 'Failed to update marketing campaign status.',
        refreshWorkspace,
        setStatus,
        startStatus: `Updating marketing campaign ${marketingCampaignId} status...`,
        successStatus: 'Marketing campaign status updated.',
      });
    },

    async handleUpdateMarketingCampaignBudgetStatus(campaignBudgetId, status) {
      await runRefreshingAction({
        action: () => updateMarketingCampaignBudgetStatus(campaignBudgetId, status),
        failureStatus: 'Failed to update campaign budget status.',
        refreshWorkspace,
        setStatus,
        startStatus: `Updating campaign budget ${campaignBudgetId} status...`,
        successStatus: 'Campaign budget status updated.',
      });
    },

    async handleUpdateMarketingCouponCodeStatus(couponCodeId, status) {
      await runRefreshingAction({
        action: () => updateMarketingCouponCodeStatus(couponCodeId, status),
        failureStatus: 'Failed to update coupon code status.',
        refreshWorkspace,
        setStatus,
        startStatus: `Updating coupon code ${couponCodeId} status...`,
        successStatus: 'Coupon code status updated.',
      });
    },

    async handleSaveTenant(input) {
      await runRefreshingAction({
        action: () => saveTenant(input),
        failureStatus: 'Failed to save tenant.',
        refreshWorkspace,
        setStatus,
        startStatus: `Saving tenant ${input.id}...`,
        successStatus: 'Tenant saved.',
      });
    },

    async handleDeleteTenant(tenantId) {
      await runRefreshingAction({
        action: () => deleteTenant(tenantId),
        failureStatus: 'Failed to delete tenant.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting tenant...',
        successStatus: 'Tenant deleted.',
      });
    },

    async handleSaveProject(input) {
      await runRefreshingAction({
        action: () => saveProject(input),
        failureStatus: 'Failed to save project.',
        refreshWorkspace,
        setStatus,
        startStatus: `Saving project ${input.id}...`,
        successStatus: 'Project saved.',
      });
    },

    async handleCreateRoutingProfile(input) {
      await runRefreshingAction({
        action: () => createRoutingProfile(input),
        failureStatus: 'Failed to create routing profile.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Creating routing profile...',
        successStatus: (profile) => `Routing profile ${profile.name} created.`,
      });
    },

    async handleSaveApiKeyGroup(input) {
      await runRefreshingAction({
        action: () =>
          input.group_id
            ? updateApiKeyGroup(input.group_id, {
                tenant_id: input.tenant_id,
                project_id: input.project_id,
                environment: input.environment,
                name: input.name,
                slug: input.slug,
                description: input.description,
                color: input.color,
                default_capability_scope: input.default_capability_scope,
                default_accounting_mode: input.default_accounting_mode,
                default_routing_profile_id: input.default_routing_profile_id,
              })
            : createApiKeyGroup({
                tenant_id: input.tenant_id,
                project_id: input.project_id,
                environment: input.environment,
                name: input.name,
                slug: input.slug,
                description: input.description,
                color: input.color,
                default_capability_scope: input.default_capability_scope,
                default_accounting_mode: input.default_accounting_mode,
                default_routing_profile_id: input.default_routing_profile_id,
              }),
        failureStatus: 'Failed to save API key group.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: input.group_id
          ? `Updating API key group ${input.group_id}...`
          : 'Creating API key group...',
        successStatus: input.group_id
          ? 'API key group updated.'
          : 'API key group created.',
      });
    },

    async handleToggleApiKeyGroup(groupId, active) {
      await runRefreshingAction({
        action: () => updateApiKeyGroupStatus(groupId, active),
        failureStatus: 'Failed to update API key group status.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: active
          ? 'Re-activating API key group...'
          : 'Disabling API key group...',
        successStatus: active
          ? 'API key group restored.'
          : 'API key group disabled.',
      });
    },

    async handleDeleteApiKeyGroup(groupId) {
      await runRefreshingAction({
        action: () => deleteApiKeyGroup(groupId),
        failureStatus: 'Failed to delete API key group.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Deleting API key group...',
        successStatus: 'API key group deleted.',
      });
    },

    async handleCreateApiKey(input) {
      return runRefreshingAction({
        action: () => createApiKey(input),
        failureStatus: 'Failed to issue gateway key.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Issuing gateway key...',
        successStatus: (created) =>
          `Gateway key issued for ${created.project_id} (${created.environment}).`,
      });
    },

    async handleCreateCommercialPricingPlan(input) {
      await runRefreshingAction({
        action: () => createCommercialPricingPlan(input),
        failureStatus: 'Failed to create commercial pricing plan.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Creating commercial pricing plan...',
        successStatus: (plan) => `Commercial pricing plan ${plan.display_name} created.`,
      });
    },

    async handleCreateCommercialPricingRate(input) {
      await runRefreshingAction({
        action: () => createCommercialPricingRate(input),
        failureStatus: 'Failed to create commercial pricing rate.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Creating commercial pricing rate...',
        successStatus: (rate) => `Commercial pricing rate ${rate.metric_code} created.`,
      });
    },

    async handleUpdateCommercialPricingPlan(pricingPlanId, input) {
      await runRefreshingAction({
        action: () => updateCommercialPricingPlan(pricingPlanId, input),
        failureStatus: 'Failed to update commercial pricing plan.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Updating commercial pricing plan ${pricingPlanId}...`,
        successStatus: (plan) => `Commercial pricing plan ${plan.display_name} updated.`,
      });
    },

    async handleCloneCommercialPricingPlan(pricingPlanId, input) {
      return runRefreshingAction({
        action: () => cloneCommercialPricingPlan(pricingPlanId, input),
        failureStatus: 'Failed to clone commercial pricing plan.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Cloning commercial pricing plan ${pricingPlanId}...`,
        successStatus: (plan) => `Commercial pricing plan ${plan.display_name} cloned.`,
      });
    },

    async handlePublishCommercialPricingPlan(pricingPlanId) {
      return runRefreshingAction({
        action: () => publishCommercialPricingPlan(pricingPlanId),
        failureStatus: 'Failed to publish commercial pricing plan.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Publishing commercial pricing plan ${pricingPlanId}...`,
        successStatus: (plan) => `Commercial pricing plan ${plan.display_name} published.`,
      });
    },

    async handleScheduleCommercialPricingPlan(pricingPlanId) {
      return runRefreshingAction({
        action: () => scheduleCommercialPricingPlan(pricingPlanId),
        failureStatus: 'Failed to schedule commercial pricing plan.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Scheduling commercial pricing plan ${pricingPlanId}...`,
        successStatus: (plan) => `Commercial pricing plan ${plan.display_name} scheduled.`,
      });
    },

    async handleRetireCommercialPricingPlan(pricingPlanId) {
      return runRefreshingAction({
        action: () => retireCommercialPricingPlan(pricingPlanId),
        failureStatus: 'Failed to retire commercial pricing plan.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Retiring commercial pricing plan ${pricingPlanId}...`,
        successStatus: (plan) => `Commercial pricing plan ${plan.display_name} retired.`,
      });
    },

    async handleSynchronizeCommercialPricingLifecycle() {
      return runRefreshingAction({
        action: () => synchronizeCommercialPricingLifecycle(),
        failureStatus: 'Failed to synchronize pricing lifecycle.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Synchronizing pricing lifecycle...',
        successStatus: (report) => {
          if (report.changed) {
            return `Pricing lifecycle synchronized. Activated ${report.activated_plan_count} plans and ${report.activated_rate_count} rates.`;
          }
          if (report.skipped_plan_count > 0) {
            return `Pricing lifecycle scan finished. ${report.skipped_plan_count} due versions still require rate rows.`;
          }
          return 'Pricing lifecycle already up to date.';
        },
      });
    },

    async handleUpdateCommercialPricingRate(pricingRateId, input) {
      await runRefreshingAction({
        action: () => updateCommercialPricingRate(pricingRateId, input),
        failureStatus: 'Failed to update commercial pricing rate.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Updating commercial pricing rate ${pricingRateId}...`,
        successStatus: (rate) => `Commercial pricing rate ${rate.metric_code} updated.`,
      });
    },

    async handleCreateRateLimitPolicy(input) {
      await runRefreshingAction({
        action: () => createRateLimitPolicy(input),
        failureStatus: 'Failed to save rate limit policy.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Saving rate limit policy...',
        successStatus: (policy) => `Rate limit policy saved for ${policy.project_id}.`,
      });
    },

    async handleUpdateApiKey(input) {
      await runRefreshingAction({
        action: () => updateApiKey(input),
        failureStatus: 'Failed to update gateway key.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: `Updating gateway key ${input.hashed_key}...`,
        successStatus: 'Gateway key updated.',
      });
    },

    async handleUpdateApiKeyStatus(hashedKey, active) {
      await runRefreshingAction({
        action: () => updateApiKeyStatus(hashedKey, active),
        failureStatus: 'Failed to update gateway key.',
        refreshWorkspace,
        setStatus,
        startStatus: active ? 'Restoring gateway key...' : 'Revoking gateway key...',
        successStatus: active ? 'Gateway key restored.' : 'Gateway key revoked.',
      });
    },

    async handleDeleteApiKey(hashedKey) {
      await runRefreshingAction({
        action: () => deleteApiKey(hashedKey),
        failureStatus: 'Failed to delete gateway key.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting gateway key...',
        successStatus: 'Gateway key deleted.',
      });
    },

    async handleReloadRuntimes(input) {
      return runRefreshingAction({
        action: () => reloadExtensionRuntimes(input),
        failureStatus: 'Failed to reload runtimes.',
        refreshWorkspace,
        rethrow: true,
        setStatus,
        startStatus: 'Reloading extension runtimes...',
        successStatus: (report) =>
          `Runtime reload finished. Active runtimes: ${report.active_runtime_count}.`,
      });
    },

    async handleDeleteProject(projectId) {
      await runRefreshingAction({
        action: () => deleteProject(projectId),
        failureStatus: 'Failed to delete project.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting project...',
        successStatus: 'Project deleted.',
      });
    },

    async handleSaveChannel(input) {
      await saveChannel(input);
      await refreshWorkspace();
    },

    async handleDeleteChannel(channelId) {
      await runRefreshingAction({
        action: () => deleteChannel(channelId),
        failureStatus: 'Failed to delete channel.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting channel...',
        successStatus: 'Channel deleted.',
      });
    },

    async handleSaveProvider(input) {
      await saveProvider(input);
      await refreshWorkspace();
    },

    async handleDeleteProvider(providerId) {
      await runRefreshingAction({
        action: () => deleteProvider(providerId),
        failureStatus: 'Failed to delete provider.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting provider...',
        successStatus: 'Provider deleted.',
      });
    },

    async handleSaveModel(input) {
      await saveModel(input);
      await refreshWorkspace();
    },

    async handleSaveChannelModel(input) {
      await runRefreshingAction({
        action: () => saveChannelModel(input),
        failureStatus: 'Failed to save channel model.',
        refreshWorkspace,
        setStatus,
        startStatus: `Saving channel model ${input.model_id}...`,
        successStatus: 'Channel model saved.',
      });
    },

    async handleDeleteChannelModel(channelId, modelId) {
      await runRefreshingAction({
        action: () => deleteChannelModel(channelId, modelId),
        failureStatus: 'Failed to delete channel model.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting channel model...',
        successStatus: 'Channel model deleted.',
      });
    },

    async handleSaveModelPrice(input) {
      await runRefreshingAction({
        action: () => saveModelPrice(input),
        failureStatus: 'Failed to save model pricing.',
        refreshWorkspace,
        setStatus,
        startStatus: `Saving model pricing for ${input.model_id}...`,
        successStatus: 'Model pricing saved.',
      });
    },

    async handleDeleteModelPrice(channelId, modelId, proxyProviderId) {
      await runRefreshingAction({
        action: () => deleteModelPrice(channelId, modelId, proxyProviderId),
        failureStatus: 'Failed to delete model pricing.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting model pricing...',
        successStatus: 'Model pricing deleted.',
      });
    },

    async handleSaveCredential(input) {
      await runRefreshingAction({
        action: () => saveCredential(input),
        failureStatus: 'Failed to save provider credential.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Saving provider credential...',
        successStatus: 'Provider credential saved.',
      });
    },

    async handleDeleteCredential(tenantId, providerId, keyReference) {
      await runRefreshingAction({
        action: () => deleteCredential(tenantId, providerId, keyReference),
        failureStatus: 'Failed to delete provider credential.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting provider credential...',
        successStatus: 'Provider credential deleted.',
      });
    },

    async handleDeleteModel(externalName, providerId) {
      await runRefreshingAction({
        action: () => deleteModel(externalName, providerId),
        failureStatus: 'Failed to delete model.',
        refreshWorkspace,
        setStatus,
        startStatus: 'Deleting model...',
        successStatus: 'Model deleted.',
      });
    },
  };
}
