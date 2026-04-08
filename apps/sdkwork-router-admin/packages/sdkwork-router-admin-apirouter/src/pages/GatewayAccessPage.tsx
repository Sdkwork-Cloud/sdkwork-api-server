import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
} from '@sdkwork/ui-pc-react';
import { Plus, Search } from 'lucide-react';
import {
  countCurrentlyEffectiveCommercialPricingPlans,
  commercialPricingChargeUnitLabel,
  commercialPricingDisplayUnit,
  commercialPricingMethodLabel,
  selectPrimaryCommercialPricingPlan,
  selectPrimaryCommercialPricingRate,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { AdminPageProps, CreatedGatewayApiKey } from 'sdkwork-router-admin-types';

import { ConfirmActionDialog } from './shared';
import { GatewayAccessDetailDrawer } from './access/GatewayAccessDetailDrawer';
import {
  GatewayApiKeyCreateDialog,
  GatewayApiKeyEditDialog,
  GatewayApiKeyRouteDialog,
} from './access/GatewayAccessForms';
import { GatewayApiKeyGroupsDialog } from './access/GatewayApiKeyGroupsDialog';
import { GatewayAccessRegistrySection } from './access/GatewayAccessRegistrySection';
import { GatewayApiKeyUsageDialog } from './access/GatewayApiKeyUsageDialog';
import { useGatewayAccessWorkspaceState } from './useGatewayAccessWorkspaceState';

type GatewayAccessPageProps = AdminPageProps & {
  onRefreshWorkspace: () => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    label?: string;
    notes?: string;
    expires_at_ms?: number | null;
    plaintext_key?: string;
    api_key_group_id?: string | null;
  }) => Promise<CreatedGatewayApiKey>;
  onUpdateApiKey: (input: {
    hashed_key: string;
    tenant_id: string;
    project_id: string;
    environment: string;
    label: string;
    notes?: string | null;
    expires_at_ms?: number | null;
    api_key_group_id?: string | null;
  }) => Promise<void>;
  onUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  onDeleteApiKey: (hashedKey: string) => Promise<void>;
  onSaveApiKeyGroup: (input: {
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
  }) => Promise<void>;
  onToggleApiKeyGroup: (groupId: string, active: boolean) => Promise<void>;
  onDeleteApiKeyGroup: (groupId: string) => Promise<void>;
};

export function GatewayAccessPage({
  snapshot,
  onRefreshWorkspace,
  onCreateApiKey,
  onSaveApiKeyGroup,
  onToggleApiKeyGroup,
  onDeleteApiKeyGroup,
  onUpdateApiKey,
  onUpdateApiKeyStatus,
  onDeleteApiKey,
}: GatewayAccessPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const {
    search,
    isDetailDrawerOpen,
    isGroupsDialogOpen,
    isCreateOpen,
    createDraftState,
    editingKey,
    editDraft,
    routeKey,
    routeDraft,
    usageKey,
    pendingDelete,
    gatewayBaseUrl,
    openClawInstances,
    loadingInstances,
    selectedClientId,
    selectedInstanceIds,
    applyingClientId,
    usageStatus,
    modelMappings,
    mappingById,
    providerById,
    groupById,
    filteredKeys,
    selectedKeyRecord,
    availableCreateProjects,
    availableEditGroups,
    usagePlaintext,
    quickSetupPlans,
    usageOverlay,
    totalKeys,
    activeKeys,
    customRouteCount,
    expiringSoonCount,
    deleteDialogDescription,
    openCreateDialog,
    handleRefreshWorkspace,
    handleSearchChange,
    handleCreateDialogOpenChange,
    openEditDialog,
    handleEditDialogOpenChange,
    openRouteDialog,
    handleRouteDialogOpenChange,
    openUsageDialog,
    handleUsageDialogOpenChange,
    handleDeleteDialogOpenChange,
    openDetailDrawer,
    handleDetailDrawerOpenChange,
    handleCreateSubmit,
    handleEditSubmit,
    handleRouteSubmit,
    confirmDelete,
    handleApplySetup,
    handleToggleKeyStatus,
    setCreateDraftState,
    setEditDraft,
    setRouteDraft,
    setSelectedClientId,
    setSelectedInstanceIds,
    setIsGroupsDialogOpen,
    setPendingDelete,
  } = useGatewayAccessWorkspaceState({
    snapshot,
    onRefreshWorkspace,
    onCreateApiKey,
    onUpdateApiKey,
    onUpdateApiKeyStatus,
    onDeleteApiKey,
  });
  const activeCommercialAccounts = snapshot.commercialAccounts.filter(
    (record) => record.account.status === 'active',
  ).length;
  const suspendedCommercialAccounts = snapshot.commercialAccounts.filter(
    (record) => record.account.status === 'suspended',
  ).length;
  const activePricingPlans = countCurrentlyEffectiveCommercialPricingPlans(
    snapshot.commercialPricingPlans,
  );
  const pricedMetrics = new Set(
    snapshot.commercialPricingRates.map((rate) => rate.metric_code),
  ).size;
  const primaryPricingPlan = selectPrimaryCommercialPricingPlan(
    snapshot.commercialPricingPlans,
  );
  const primaryPricingRate = selectPrimaryCommercialPricingRate(
    snapshot.commercialPricingRates,
    primaryPricingPlan,
  );
  const capturedSettlementAmount = snapshot.commercialRequestSettlements.reduce(
    (sum, settlement) => sum + settlement.captured_credit_amount,
    0,
  );
  const commercialGovernanceFacts = [
    {
      label: t('Open holds'),
      value: formatNumber(
        snapshot.commercialAccountHolds.filter((hold) =>
          hold.status === 'held'
          || hold.status === 'captured'
          || hold.status === 'partially_released').length,
      ),
    },
    {
      label: t('Request settlements'),
      value: formatNumber(snapshot.commercialRequestSettlements.length),
    },
    {
      label: t('Captured credits'),
      value: formatNumber(capturedSettlementAmount),
    },
  ];
  const commercialAccountFacts = [
    {
      label: t('Total accounts'),
      value: formatNumber(snapshot.commercialAccounts.length),
    },
    {
      label: t('Active'),
      value: formatNumber(activeCommercialAccounts),
    },
    {
      label: t('Suspended'),
      value: formatNumber(suspendedCommercialAccounts),
    },
  ];
  const pricingPostureFacts = [
    {
      label: t('Plans'),
      value: formatNumber(snapshot.commercialPricingPlans.length),
    },
    {
      label: t('Active plans'),
      value: formatNumber(activePricingPlans),
    },
    {
      label: t('Rates'),
      value: formatNumber(snapshot.commercialPricingRates.length),
    },
    {
      label: t('Priced metrics'),
      value: formatNumber(pricedMetrics),
    },
    {
      label: t('Charge unit'),
      value: commercialPricingChargeUnitLabel(primaryPricingRate?.charge_unit, t),
    },
    {
      label: t('Billing method'),
      value: commercialPricingMethodLabel(primaryPricingRate?.pricing_method, t),
    },
    {
      label: t('Price unit'),
      value: commercialPricingDisplayUnit(primaryPricingRate, t),
    },
  ];

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="gateway-access-search">
                  {t('Search API keys')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="gateway-access-search"
                    onChange={(event) => handleSearchChange(event.target.value)}
                    placeholder={t('label, workspace, hashed key, provider')}
                    value={search}
                  />
                </div>
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(filteredKeys.length) })}
                  {' | '}
                  {t('{count} active', { count: formatNumber(activeKeys) })}
                  {' | '}
                  {t('{count} custom routes', { count: formatNumber(customRouteCount) })}
                </div>
                <Button onClick={openCreateDialog} type="button" variant="primary">
                  <Plus className="w-4 h-4" />
                  {t('Create API key')}
                </Button>
                <Button
                  onClick={() => setIsGroupsDialogOpen(true)}
                  type="button"
                  variant="outline"
                >
                  {t('Manage groups')}
                </Button>
                <Button
                  onClick={() => void handleRefreshWorkspace()}
                  type="button"
                  variant="outline"
                >
                  {t('Refresh workspace')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="grid gap-4 xl:grid-cols-3">
          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                  {t('Commercial governance')}
                </div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Credit holds, settlement capture, and liability posture stay visible while governing API access.')}
                </div>
              </div>
              <div className="space-y-2 text-sm">
                {commercialGovernanceFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <div className="text-[var(--sdk-color-text-secondary)]">{item.label}</div>
                    <div className="font-medium text-[var(--sdk-color-text-primary)]">{item.value}</div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                  {t('Commercial accounts')}
                </div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Operators can confirm that API key issuance is mapped onto live commercial account inventory.')}
                </div>
              </div>
              <div className="space-y-2 text-sm">
                {commercialAccountFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <div className="text-[var(--sdk-color-text-secondary)]">{item.label}</div>
                    <div className="font-medium text-[var(--sdk-color-text-primary)]">{item.value}</div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                  {t('Pricing posture')}
                </div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Pricing plans and rates define the commercial surface that gateway access policies must honor.')}
                </div>
              </div>
              <div className="space-y-2 text-sm">
                {pricingPostureFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <div className="text-[var(--sdk-color-text-secondary)]">{item.label}</div>
                    <div className="font-medium text-[var(--sdk-color-text-primary)]">{item.value}</div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="min-h-0 flex-1">
          <GatewayAccessRegistrySection
            activeKeys={activeKeys}
            customRouteCount={customRouteCount}
            expiringSoonCount={expiringSoonCount}
            filteredKeys={filteredKeys}
            groupById={groupById}
            mappingById={mappingById}
            onDeleteKey={setPendingDelete}
            onOpenEditDialog={openEditDialog}
            onOpenRouteDialog={openRouteDialog}
            onOpenUsageDialog={openUsageDialog}
            onSelectKey={openDetailDrawer}
            providerById={providerById}
            selectedKey={selectedKeyRecord}
            totalKeys={totalKeys}
          />
        </div>
      </div>

      <GatewayAccessDetailDrawer
        groupById={groupById}
        mappingById={mappingById}
        onDelete={() => {
          if (!selectedKeyRecord) {
            return;
          }
          setPendingDelete(selectedKeyRecord);
        }}
        onEdit={() => {
          if (!selectedKeyRecord) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openEditDialog(selectedKeyRecord);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        onOpenRouteDialog={() => {
          if (!selectedKeyRecord) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openRouteDialog(selectedKeyRecord);
        }}
        onOpenUsageDialog={() => {
          if (!selectedKeyRecord) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openUsageDialog(selectedKeyRecord);
        }}
        onToggleStatus={() => {
          if (!selectedKeyRecord) {
            return;
          }
          void handleToggleKeyStatus(selectedKeyRecord);
        }}
        open={isDetailDrawerOpen}
        providerById={providerById}
        selectedKey={selectedKeyRecord}
      />

      <GatewayApiKeyCreateDialog
        availableProjects={availableCreateProjects}
        draft={createDraftState}
        modelMappings={modelMappings}
        onOpenChange={handleCreateDialogOpenChange}
        onSubmit={(event) => void handleCreateSubmit(event)}
        open={isCreateOpen}
        setDraft={setCreateDraftState}
        snapshot={snapshot}
      />

      <GatewayApiKeyEditDialog
        availableGroups={availableEditGroups}
        draft={editDraft}
        editingKey={editingKey}
        onOpenChange={handleEditDialogOpenChange}
        onSubmit={(event) => void handleEditSubmit(event)}
        setDraft={setEditDraft}
      />

      <GatewayApiKeyGroupsDialog
        onDeleteApiKeyGroup={onDeleteApiKeyGroup}
        onOpenChange={setIsGroupsDialogOpen}
        onSaveApiKeyGroup={onSaveApiKeyGroup}
        onToggleApiKeyGroup={onToggleApiKeyGroup}
        open={isGroupsDialogOpen}
        preferredScope={{
          tenant_id: selectedKeyRecord?.tenant_id ?? createDraftState.tenant_id,
          project_id: selectedKeyRecord?.project_id ?? createDraftState.project_id,
          environment: selectedKeyRecord?.environment ?? createDraftState.environment,
        }}
        snapshot={snapshot}
      />

      <GatewayApiKeyRouteDialog
        modelMappings={modelMappings}
        onOpenChange={handleRouteDialogOpenChange}
        onSubmit={(event) => void handleRouteSubmit(event)}
        routeDraft={routeDraft}
        routeKey={routeKey}
        setRouteDraft={setRouteDraft}
        snapshot={snapshot}
      />

      <GatewayApiKeyUsageDialog
        applyingClientId={applyingClientId}
        gatewayBaseUrl={gatewayBaseUrl}
        loadingInstances={loadingInstances}
        mappingById={mappingById}
        onApplySetup={(plan) => void handleApplySetup(plan)}
        onOpenChange={handleUsageDialogOpenChange}
        openClawInstances={openClawInstances}
        providerById={providerById}
        quickSetupPlans={quickSetupPlans}
        selectedClientId={selectedClientId}
        selectedInstanceIds={selectedInstanceIds}
        setSelectedClientId={setSelectedClientId}
        setSelectedInstanceIds={setSelectedInstanceIds}
        usageKey={usageKey}
        usageOverlay={usageOverlay}
        usagePlaintext={usagePlaintext}
        usageStatus={usageStatus}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete key')}
        description={deleteDialogDescription}
        onConfirm={() => void confirmDelete()}
        onOpenChange={handleDeleteDialogOpenChange}
        open={Boolean(pendingDelete)}
        title={t('Delete API key')}
      />
    </>
  );
}
