import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
  StatCard,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Plus, Search } from 'lucide-react';
import {
  buildProviderSaveInput,
  describeProviderIntegration,
  type SaveProviderInput,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { AdminPageProps, ProviderCatalogRecord } from 'sdkwork-router-admin-types';

import { ConfirmActionDialog, SelectField } from './shared';
import { GatewayRoutesDetailDrawer } from './routes/GatewayRoutesDetailDrawer';
import { GatewayProviderDialog } from './routes/GatewayProviderDialog';
import { GatewayRoutingProfilesDialog } from './routes/GatewayRoutingProfilesDialog';
import { GatewayRoutingSnapshotsDialog } from './routes/GatewayRoutingSnapshotsDialog';
import { GatewayRoutesRegistrySection } from './routes/GatewayRoutesRegistrySection';
import {
  buildProviderRoutingImpact,
  buildRoutingSnapshotAnalytics,
} from './routes/routingSnapshotAnalytics';
import {
  emptyProviderDraft,
  formatChannels,
  providerDraftFromRecord,
  statusVariant,
  type HealthFilter,
  type ProviderDraft,
} from './routes/shared';
import {
  buildGatewayRouteInventory,
  type GatewayRouteInventoryRow,
} from '../services/gatewayViewService';

type GatewayRoutesPageProps = AdminPageProps & {
  onCreateRoutingProfile: (input: {
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
  }) => Promise<void>;
  onRefreshWorkspace: () => Promise<void>;
  onSaveProvider: (input: SaveProviderInput) => Promise<void>;
  onDeleteProvider: (providerId: string) => Promise<void>;
};

export function GatewayRoutesPage({
  onCreateRoutingProfile,
  snapshot,
  onRefreshWorkspace,
  onSaveProvider,
  onDeleteProvider,
}: GatewayRoutesPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const defaultChannelId = snapshot.channels[0]?.id ?? '';
  const [search, setSearch] = useState('');
  const [channelFilter, setChannelFilter] = useState('all');
  const [healthFilter, setHealthFilter] = useState<HealthFilter>('all');
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);
  const [editingProvider, setEditingProvider] = useState<ProviderCatalogRecord | null>(null);
  const [providerDraft, setProviderDraft] = useState<ProviderDraft>(() =>
    emptyProviderDraft(defaultChannelId),
  );
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isProviderDialogOpen, setIsProviderDialogOpen] = useState(false);
  const [isRoutingProfilesDialogOpen, setIsRoutingProfilesDialogOpen] = useState(false);
  const [isRoutingSnapshotsDialogOpen, setIsRoutingSnapshotsDialogOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<ProviderCatalogRecord | null>(null);
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const inventory = useMemo(() => buildGatewayRouteInventory(snapshot), [snapshot]);
  const routingSnapshotAnalytics = useMemo(
    () => buildRoutingSnapshotAnalytics(snapshot),
    [snapshot],
  );
  const topRoutingProfiles = routingSnapshotAnalytics.topProfiles.slice(0, 3);
  const degradedCount = inventory.filter((row) => !row.healthy).length;
  const filteredInventory = useMemo(
    () =>
      inventory.filter((row) => {
        if (
          channelFilter !== 'all'
          && !row.channels.some((channel) => channel.id === channelFilter)
        ) {
          return false;
        }

        if (healthFilter === 'healthy' && !row.healthy) {
          return false;
        }

        if (healthFilter === 'degraded' && row.healthy) {
          return false;
        }

        if (!deferredSearch) {
          return true;
        }

        const haystack = [
          row.provider.id,
          row.provider.display_name,
          row.provider.adapter_kind,
          row.provider.protocol_kind,
          row.provider.integration.default_plugin_family ?? '',
          describeProviderIntegration(row.provider),
          row.provider.base_url,
          ...row.channels.map((channel) => `${channel.id} ${channel.name}`),
          ...row.credentials.map((credential) => credential.key_reference),
        ]
          .join(' ')
          .toLowerCase();

        return haystack.includes(deferredSearch);
      }),
    [channelFilter, deferredSearch, healthFilter, inventory],
  );

  useEffect(() => {
    if (!filteredInventory.length) {
      if (selectedProviderId !== null) {
        setSelectedProviderId(null);
      }
      setIsDetailDrawerOpen(false);
      return;
    }

    if (
      selectedProviderId
      && filteredInventory.some((row) => row.provider.id === selectedProviderId)
    ) {
      return;
    }

    setSelectedProviderId(filteredInventory[0]?.provider.id ?? null);
    setIsDetailDrawerOpen(false);
  }, [filteredInventory, selectedProviderId]);

  const selectedRow =
    filteredInventory.find((row) => row.provider.id === selectedProviderId)
    ?? filteredInventory[0]
    ?? null;
  const selectedProviderRoutingImpact = useMemo(
    () => (
      selectedRow
        ? buildProviderRoutingImpact(selectedRow.provider.id, routingSnapshotAnalytics)
        : null
    ),
    [routingSnapshotAnalytics, selectedRow],
  );

  const columns = useMemo<DataTableColumn<GatewayRouteInventoryRow>[]>(
    () => [
      {
        id: 'provider',
        header: t('Provider'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-semibold text-[var(--sdk-color-text-primary)]">
              {row.provider.display_name}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.provider.id} / {describeProviderIntegration(row.provider)}
            </div>
          </div>
        ),
      },
      {
        id: 'channels',
        header: t('Channels'),
        cell: (row) => (
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {formatChannels(row)}
          </div>
        ),
      },
      {
        id: 'base-url',
        header: t('Base URL'),
        cell: (row) => (
          <div className="font-mono text-xs text-[var(--sdk-color-text-secondary)]">
            {row.provider.base_url}
          </div>
        ),
      },
      {
        id: 'coverage',
        header: t('Coverage'),
        cell: (row) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{t('{count} published models', { count: formatNumber(row.model_count) })}</div>
            <div>{t('{count} pricing rows', { count: formatNumber(row.price_count) })}</div>
          </div>
        ),
      },
      {
        id: 'credentials',
        header: t('Credentials'),
        cell: (row) => (
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.credentials.length
              ? row.credentials.map((credential) => credential.key_reference).join(', ')
              : t('No credentials')}
          </div>
        ),
      },
      {
        id: 'health',
        header: t('Health'),
        cell: (row) => (
          <StatusBadge
            label={row.health_status}
            showIcon
            status={row.healthy ? 'active' : 'failed'}
            variant={statusVariant(row)}
          />
        ),
        width: 144,
      },
    ],
    [formatNumber, t],
  );

  function resetProviderDialog(): void {
    setEditingProvider(null);
    setProviderDraft(emptyProviderDraft(defaultChannelId));
    setIsProviderDialogOpen(false);
  }

  function openNewProviderDialog(): void {
    setEditingProvider(null);
    setProviderDraft(emptyProviderDraft(defaultChannelId));
    setIsProviderDialogOpen(true);
  }

  function openEditProviderDialog(provider: ProviderCatalogRecord): void {
    setEditingProvider(provider);
    setProviderDraft(providerDraftFromRecord(provider));
    setIsProviderDialogOpen(true);
  }

  function openDetailDrawer(row: GatewayRouteInventoryRow): void {
    setSelectedProviderId(row.provider.id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean): void {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedProviderId(null);
    }
  }

  async function handleProviderSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();
    await onSaveProvider(buildProviderSaveInput(providerDraft));

    resetProviderDialog();
  }

  async function confirmDelete(): Promise<void> {
    if (!pendingDelete) {
      return;
    }

    await onDeleteProvider(pendingDelete.id);
    if (selectedProviderId === pendingDelete.id) {
      setSelectedProviderId(null);
      setIsDetailDrawerOpen(false);
    }
    setPendingDelete(null);
  }

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
                <Label className="sr-only" htmlFor="gateway-routes-search">
                  {t('Search providers')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="gateway-routes-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('provider, base url, credential, channel')}
                    value={search}
                  />
                </div>
              </div>
              <div className="min-w-[12rem]">
                <SelectField
                  label={t('Channel')}
                  labelVisibility="sr-only"
                  onValueChange={setChannelFilter}
                  options={[
                    { label: t('All channels'), value: 'all' },
                    ...snapshot.channels.map((channel) => ({
                      label: `${channel.name} (${channel.id})`,
                      value: channel.id,
                    })),
                  ]}
                  placeholder={t('Channel')}
                  value={channelFilter}
                />
              </div>
              <div className="min-w-[12rem]">
                <SelectField<HealthFilter>
                  label={t('Health')}
                  labelVisibility="sr-only"
                  onValueChange={setHealthFilter}
                  options={[
                    { label: t('All providers'), value: 'all' },
                    { label: t('Healthy only'), value: 'healthy' },
                    { label: t('Degraded only'), value: 'degraded' },
                  ]}
                  placeholder={t('Health')}
                  value={healthFilter}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(filteredInventory.length) })}
                  {' | '}
                  {t('{count} degraded', { count: formatNumber(degradedCount) })}
                  {' | '}
                  {t('{count} total', { count: formatNumber(inventory.length) })}
                </div>
                <Button onClick={openNewProviderDialog} type="button" variant="primary">
                  <Plus className="w-4 h-4" />
                  {t('New route provider')}
                </Button>
                <Button
                  onClick={() => setIsRoutingProfilesDialogOpen(true)}
                  type="button"
                  variant="outline"
                >
                  {t('Manage routing profiles')}
                </Button>
                <Button
                  onClick={() => void onRefreshWorkspace()}
                  type="button"
                  variant="outline"
                >
                  {t('Refresh workspace')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <div className="mb-4 grid gap-4 xl:grid-cols-4">
            <StatCard
              description={t('Compiled snapshots currently loaded from the routing evidence layer.')}
              label={t('Compiled snapshots')}
              value={formatNumber(routingSnapshotAnalytics.totalCompiledSnapshots)}
            />
            <StatCard
              description={t('API key groups currently bound to reusable routing profiles.')}
              label={t('Bound groups')}
              value={formatNumber(routingSnapshotAnalytics.boundGroupCount)}
            />
            <StatCard
              description={t('Snapshots carrying an applied routing profile id.')}
              label={t('Applied routing profile')}
              value={formatNumber(routingSnapshotAnalytics.profileBackedSnapshotCount)}
            />
            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Snapshot evidence')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Review how routing profiles compile into route-key and capability evidence before changing provider posture.')}
                  </div>
                </div>
                <div className="space-y-2 text-sm">
                  {topRoutingProfiles.length ? (
                    topRoutingProfiles.map((profileImpact) => (
                      <div
                        className="flex items-center justify-between gap-3"
                        key={profileImpact.routingProfile.profile_id}
                      >
                        <div className="truncate text-[var(--sdk-color-text-primary)]">
                          {profileImpact.routingProfile.name}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {t('{snapshots} snapshots', {
                            snapshots: formatNumber(profileImpact.compiledSnapshots.length),
                          })}
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="text-[var(--sdk-color-text-secondary)]">
                      {t('No compiled routing evidence is available yet.')}
                    </div>
                  )}
                </div>
                <Button
                  onClick={() => setIsRoutingSnapshotsDialogOpen(true)}
                  type="button"
                  variant="outline"
                >
                  {t('Snapshot evidence')}
                </Button>
              </CardContent>
            </Card>
          </div>

          <GatewayRoutesRegistrySection
            columns={columns}
            degradedCount={degradedCount}
            filteredInventory={filteredInventory}
            inventory={inventory}
            onDeleteProvider={setPendingDelete}
            onEditProvider={openEditProviderDialog}
            onSelectProvider={openDetailDrawer}
            selectedRow={selectedRow}
          />
        </div>
      </div>

      <GatewayRoutesDetailDrawer
        onDelete={() => {
          if (!selectedRow) {
            return;
          }
          setPendingDelete(selectedRow.provider);
        }}
        onEdit={() => {
          if (!selectedRow) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openEditProviderDialog(selectedRow.provider);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        open={isDetailDrawerOpen}
        providerRoutingImpact={selectedProviderRoutingImpact}
        selectedRow={selectedRow}
      />

      <GatewayProviderDialog
        editingProvider={editingProvider}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsProviderDialogOpen(true) : resetProviderDialog()
        }
        onSubmit={(event) => void handleProviderSubmit(event)}
        open={isProviderDialogOpen}
        providerDraft={providerDraft}
        setProviderDraft={setProviderDraft}
        snapshot={snapshot}
      />

      <GatewayRoutingProfilesDialog
        onCreateRoutingProfile={onCreateRoutingProfile}
        onOpenChange={setIsRoutingProfilesDialogOpen}
        open={isRoutingProfilesDialogOpen}
        snapshot={snapshot}
      />

      <GatewayRoutingSnapshotsDialog
        analytics={routingSnapshotAnalytics}
        onOpenChange={setIsRoutingSnapshotsDialogOpen}
        open={isRoutingSnapshotsDialogOpen}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete provider')}
        description={
          pendingDelete
            ? t(
                'Delete {name}. Review route bindings, pricing, and downstream channel coverage before removing the provider.',
                { name: pendingDelete.display_name },
              )
            : ''
        }
        onConfirm={confirmDelete}
        onOpenChange={(open) => {
          if (!open) {
            setPendingDelete(null);
          }
        }}
        open={Boolean(pendingDelete)}
        title={t('Delete route provider')}
      />
    </>
  );
}
