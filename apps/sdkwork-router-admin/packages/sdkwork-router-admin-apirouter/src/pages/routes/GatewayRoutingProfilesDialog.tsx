import { useEffect, useMemo, useState, type ChangeEvent, type FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FormActions,
  FormGrid,
  FormSection,
  Input,
  StatusBadge,
  Textarea,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  AdminWorkspaceSnapshot,
  ProviderCatalogRecord,
  RoutingProfileRecord,
} from 'sdkwork-router-admin-types';

import { DialogField, SelectField } from '../shared';

type RoutingProfileDraft = {
  tenant_id: string;
  project_id: string;
  name: string;
  slug: string;
  description: string;
  active: boolean;
  strategy: string;
  ordered_provider_ids: string[];
  default_provider_id: string;
  max_cost: string;
  max_latency_ms: string;
  require_healthy: boolean;
  preferred_region: string;
};

type GatewayRoutingProfilesDialogProps = {
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
  onOpenChange: (open: boolean) => void;
  open: boolean;
  snapshot: AdminWorkspaceSnapshot;
};

const defaultPreferredRegionOptions = [
  '',
  'auto',
  'us-east',
  'us-west',
  'eu-west',
  'ap-southeast',
];

function resolvePreferredScope(snapshot: AdminWorkspaceSnapshot) {
  const tenantId =
    snapshot.projects[0]?.tenant_id || snapshot.tenants[0]?.id || 'tenant_local_demo';
  const projectId =
    snapshot.projects.find((project) => project.tenant_id === tenantId)?.id
    || snapshot.projects[0]?.id
    || 'project_local_demo';

  return {
    tenant_id: tenantId,
    project_id: projectId,
  };
}

function createRoutingProfileDraft(
  scope: ReturnType<typeof resolvePreferredScope>,
  overrides: Partial<RoutingProfileDraft> = {},
): RoutingProfileDraft {
  return {
    tenant_id: scope.tenant_id,
    project_id: scope.project_id,
    name: '',
    slug: '',
    description: '',
    active: true,
    strategy: 'deterministic_priority',
    ordered_provider_ids: [],
    default_provider_id: '',
    max_cost: '',
    max_latency_ms: '',
    require_healthy: true,
    preferred_region: '',
    ...overrides,
  };
}

function optionalString(value: string): string | undefined {
  const normalized = value.trim();
  return normalized ? normalized : undefined;
}

function optionalNumber(value: string): number | undefined {
  const normalized = value.trim();
  if (!normalized) {
    return undefined;
  }

  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function createDraftFromProfile(profile: RoutingProfileRecord): RoutingProfileDraft {
  return createRoutingProfileDraft(
    {
      tenant_id: profile.tenant_id,
      project_id: profile.project_id,
    },
    {
      name: `${profile.name} copy`,
      slug: '',
      description: profile.description ?? '',
      active: profile.active,
      strategy: profile.strategy,
      ordered_provider_ids: [...profile.ordered_provider_ids],
      default_provider_id:
        profile.default_provider_id ?? profile.ordered_provider_ids[0] ?? '',
      max_cost: profile.max_cost != null ? String(profile.max_cost) : '',
      max_latency_ms:
        profile.max_latency_ms != null ? String(profile.max_latency_ms) : '',
      require_healthy: profile.require_healthy,
      preferred_region: profile.preferred_region ?? '',
    },
  );
}

function sortProfiles(left: RoutingProfileRecord, right: RoutingProfileRecord): number {
  if (left.active !== right.active) {
    return left.active ? -1 : 1;
  }

  return right.updated_at_ms - left.updated_at_ms;
}

export function GatewayRoutingProfilesDialog({
  onCreateRoutingProfile,
  onOpenChange,
  open,
  snapshot,
}: GatewayRoutingProfilesDialogProps) {
  const { t } = useAdminI18n();
  const resolvedScope = useMemo(() => resolvePreferredScope(snapshot), [snapshot]);
  const [search, setSearch] = useState('');
  const [busy, setBusy] = useState(false);
  const [statusMessage, setStatusMessage] = useState('');
  const [templateProfileId, setTemplateProfileId] = useState<string | null>(null);
  const [draft, setDraft] = useState<RoutingProfileDraft>(() =>
    createRoutingProfileDraft(resolvedScope),
  );

  useEffect(() => {
    if (!open) {
      return;
    }

    setSearch('');
    setBusy(false);
    setStatusMessage('');
    setTemplateProfileId(null);
    setDraft(createRoutingProfileDraft(resolvedScope));
  }, [open]);

  const availableProjects = useMemo(
    () => snapshot.projects.filter((project) => project.tenant_id === draft.tenant_id),
    [draft.tenant_id, snapshot.projects],
  );

  useEffect(() => {
    if (!availableProjects.length || availableProjects.some((project) => project.id === draft.project_id)) {
      return;
    }

    setDraft((current) => ({
      ...current,
      project_id: availableProjects[0]?.id ?? current.project_id,
    }));
  }, [availableProjects, draft.project_id]);

  const templateProfile = useMemo(
    () =>
      snapshot.routingProfiles.find((profile) => profile.profile_id === templateProfileId)
      ?? null,
    [snapshot.routingProfiles, templateProfileId],
  );

  useEffect(() => {
    if (templateProfileId && !templateProfile) {
      setTemplateProfileId(null);
    }
  }, [templateProfile, templateProfileId]);

  const filteredProfiles = useMemo(() => {
    const normalizedSearch = search.trim().toLowerCase();

    return [...snapshot.routingProfiles]
      .sort(sortProfiles)
      .filter((profile) => {
        if (!normalizedSearch) {
          return true;
        }

        return [
          profile.name,
          profile.slug,
          profile.profile_id,
          profile.tenant_id,
          profile.project_id,
          profile.strategy,
          profile.preferred_region ?? '',
          profile.default_provider_id ?? '',
          ...profile.ordered_provider_ids,
        ]
          .join(' ')
          .toLowerCase()
          .includes(normalizedSearch);
      });
  }, [search, snapshot.routingProfiles]);

  const providerOptions = useMemo(
    () =>
      [...snapshot.providers].sort((left, right) =>
        left.display_name.localeCompare(right.display_name),
      ),
    [snapshot.providers],
  );

  const orderedProviders = useMemo(
    () =>
      draft.ordered_provider_ids
        .map((providerId) =>
          providerOptions.find((provider) => provider.id === providerId) ?? null,
        )
        .filter((provider): provider is ProviderCatalogRecord => provider !== null),
    [draft.ordered_provider_ids, providerOptions],
  );

  const preferredRegionOptions = useMemo(() => {
    const values = new Set(defaultPreferredRegionOptions);
    if (draft.preferred_region.trim()) {
      values.add(draft.preferred_region.trim());
    }
    for (const profile of snapshot.routingProfiles) {
      if (profile.preferred_region?.trim()) {
        values.add(profile.preferred_region.trim());
      }
    }

    return [...values].map((value) => ({
      label: value || t('No preferred region'),
      value,
    }));
  }, [draft.preferred_region, snapshot.routingProfiles, t]);

  function resetDraftForCreation() {
    setTemplateProfileId(null);
    setDraft(
      createRoutingProfileDraft({
        tenant_id: draft.tenant_id,
        project_id: draft.project_id,
      }),
    );
    setStatusMessage('');
  }

  function handleUseTemplate(profile: RoutingProfileRecord) {
    setTemplateProfileId(profile.profile_id);
    setDraft(createDraftFromProfile(profile));
    setStatusMessage('');
  }

  function toggleProvider(providerId: string, checked: boolean) {
    setDraft((current) => {
      const alreadySelected = current.ordered_provider_ids.includes(providerId);
      const orderedProviderIds = checked
        ? alreadySelected
          ? current.ordered_provider_ids
          : [...current.ordered_provider_ids, providerId]
        : current.ordered_provider_ids.filter((id) => id !== providerId);

      return {
        ...current,
        ordered_provider_ids: orderedProviderIds,
        default_provider_id: orderedProviderIds.includes(current.default_provider_id)
          ? current.default_provider_id
          : orderedProviderIds[0] ?? '',
      };
    });
  }

  function moveProvider(providerId: string, direction: -1 | 1) {
    setDraft((current) => {
      const currentIndex = current.ordered_provider_ids.indexOf(providerId);
      const nextIndex = currentIndex + direction;

      if (
        currentIndex === -1
        || nextIndex < 0
        || nextIndex >= current.ordered_provider_ids.length
      ) {
        return current;
      }

      const orderedProviderIds = [...current.ordered_provider_ids];
      const [provider] = orderedProviderIds.splice(currentIndex, 1);
      orderedProviderIds.splice(nextIndex, 0, provider);

      return {
        ...current,
        ordered_provider_ids: orderedProviderIds,
      };
    });
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);

    const orderedProviderIds = draft.ordered_provider_ids.filter(Boolean);
    const defaultProviderId =
      optionalString(draft.default_provider_id) ?? orderedProviderIds[0];

    try {
      await onCreateRoutingProfile({
        tenant_id: draft.tenant_id.trim(),
        project_id: draft.project_id.trim(),
        name: draft.name.trim(),
        slug: optionalString(draft.slug),
        description: optionalString(draft.description),
        active: draft.active,
        strategy: optionalString(draft.strategy) ?? 'deterministic_priority',
        ordered_provider_ids: orderedProviderIds,
        default_provider_id: defaultProviderId,
        max_cost: optionalNumber(draft.max_cost),
        max_latency_ms: optionalNumber(draft.max_latency_ms),
        require_healthy: draft.require_healthy,
        preferred_region: optionalString(draft.preferred_region),
      });

      setStatusMessage(
        t('Routing profile created. Review the refreshed policy inventory on the left.'),
      );
      setTemplateProfileId(null);
      setDraft(
        createRoutingProfileDraft({
          tenant_id: draft.tenant_id,
          project_id: draft.project_id,
        }),
      );
    } catch (error) {
      setStatusMessage(
        error instanceof Error ? error.message : t('Failed to create routing profile.'),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(96vw,86rem)]">
        <DialogHeader>
          <DialogTitle>{t('Routing profiles')}</DialogTitle>
          <DialogDescription>
            {t('Capture reusable routing posture so API key groups and workspace policy can bind to a named profile instead of repeating provider order, latency, and health rules.')}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-6 lg:grid-cols-[22rem,minmax(0,1fr)]">
          <div className="space-y-4">
            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('Search routing profiles')}
                    value={search}
                  />
                </div>
                <Button onClick={resetDraftForCreation} type="button" variant="outline">
                  {t('Create profile')}
                </Button>
              </CardContent>
            </Card>

            <div className="max-h-[60vh] space-y-3 overflow-y-auto pr-1">
              {filteredProfiles.length ? (
                filteredProfiles.map((profile) => (
                  <Card
                    className={
                      templateProfileId === profile.profile_id
                        ? 'border-[var(--sdk-color-primary-500)] shadow-sm'
                        : undefined
                    }
                    key={profile.profile_id}
                  >
                    <CardContent className="space-y-3 p-4">
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="space-y-1">
                          <div className="font-semibold text-[var(--sdk-color-text-primary)]">
                            {profile.name}
                          </div>
                          <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                            {profile.slug}
                          </div>
                        </div>
                        <StatusBadge
                          label={profile.active ? t('Active') : t('Inactive')}
                          showIcon
                          status={profile.active ? 'active' : 'paused'}
                          variant={profile.active ? 'success' : 'secondary'}
                        />
                      </div>

                      <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
                        <div>
                          {profile.tenant_id} / {profile.project_id}
                        </div>
                        <div>
                          {t('Provider order')}: {profile.ordered_provider_ids.length}
                        </div>
                        <div>
                          {t('Preferred region')}: {profile.preferred_region || t('Auto')}
                        </div>
                        <div>
                          {t('Require healthy')}:{' '}
                          {profile.require_healthy ? t('Yes') : t('No')}
                        </div>
                      </div>

                      <div className="flex flex-wrap gap-2">
                        <StatusBadge
                          label={profile.strategy}
                          showIcon
                          status={profile.strategy}
                          variant="secondary"
                        />
                        <Button
                          onClick={() => handleUseTemplate(profile)}
                          size="sm"
                          type="button"
                          variant={templateProfileId === profile.profile_id ? 'primary' : 'outline'}
                        >
                          {t('Use as template')}
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))
              ) : (
                <Card>
                  <CardContent className="space-y-1 p-4 text-sm text-[var(--sdk-color-text-secondary)]">
                    <div className="font-medium text-[var(--sdk-color-text-primary)]">
                      {t('No routing profiles match the current filter')}
                    </div>
                    <div>
                      {t('Broaden the query or create the first reusable routing policy for this workspace.')}
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>
          </div>

          <form className="space-y-6" onSubmit={(event) => void handleSubmit(event)}>
            {statusMessage ? (
              <div className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
                {statusMessage}
              </div>
            ) : null}

            {templateProfile ? (
              <div className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-primary-300)] bg-[var(--sdk-color-primary-50)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
                {t(
                  'Creating a new profile from {name}. Adjust the scope, provider order, or routing constraints before saving.',
                  { name: templateProfile.name },
                )}
              </div>
            ) : null}

            <FormSection
              description={t('Pin the workspace boundary and profile metadata before defining routing posture.')}
              title={t('Create profile')}
            >
              <FormGrid columns={2}>
                {snapshot.tenants.length ? (
                  <SelectField
                    label={t('Tenant')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        tenant_id: value,
                        project_id:
                          snapshot.projects.find((project) => project.tenant_id === value)?.id
                          ?? current.project_id,
                      }))
                    }
                    options={snapshot.tenants.map((tenant) => ({
                      label: `${tenant.name} (${tenant.id})`,
                      value: tenant.id,
                    }))}
                    value={draft.tenant_id}
                  />
                ) : (
                  <DialogField htmlFor="routing-profile-tenant" label={t('Tenant')}>
                    <Input
                      id="routing-profile-tenant"
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          tenant_id: event.target.value,
                        }))
                      }
                      required
                      value={draft.tenant_id}
                    />
                  </DialogField>
                )}

                {availableProjects.length ? (
                  <SelectField
                    label={t('Project')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        project_id: value,
                      }))
                    }
                    options={availableProjects.map((project) => ({
                      label: `${project.name} (${project.id})`,
                      value: project.id,
                    }))}
                    value={draft.project_id}
                  />
                ) : (
                  <DialogField htmlFor="routing-profile-project" label={t('Project')}>
                    <Input
                      id="routing-profile-project"
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          project_id: event.target.value,
                        }))
                      }
                      required
                      value={draft.project_id}
                    />
                  </DialogField>
                )}

                <DialogField htmlFor="routing-profile-name" label={t('Name')}>
                  <Input
                    id="routing-profile-name"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setDraft((current) => ({
                        ...current,
                        name: event.target.value,
                      }))
                    }
                    required
                    value={draft.name}
                  />
                </DialogField>

                <DialogField
                  description={t('Leave empty to derive a slug automatically from the profile name.')}
                  htmlFor="routing-profile-slug"
                  label={t('Slug')}
                >
                  <Input
                    id="routing-profile-slug"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setDraft((current) => ({
                        ...current,
                        slug: event.target.value,
                      }))
                    }
                    value={draft.slug}
                  />
                </DialogField>

                <DialogField htmlFor="routing-profile-description" label={t('Description')}>
                  <Textarea
                    id="routing-profile-description"
                    onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                      setDraft((current) => ({
                        ...current,
                        description: event.target.value,
                      }))
                    }
                    rows={4}
                    value={draft.description}
                  />
                </DialogField>
              </FormGrid>

              <label className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3">
                <Checkbox
                  checked={draft.active}
                  onCheckedChange={(checked: boolean | 'indeterminate') =>
                    setDraft((current) => ({
                      ...current,
                      active: checked === true,
                    }))
                  }
                />
                <div className="space-y-1">
                  <div className="font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Active')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Keep the new profile immediately selectable by API key groups after creation.')}
                  </div>
                </div>
              </label>
            </FormSection>

            <FormSection
              description={t('Set the route selection behavior, region preference, and SLO limits that the gateway should inherit from this profile.')}
              title={t('Routing posture')}
            >
              <FormGrid columns={2}>
                <SelectField
                  label={t('Strategy')}
                  onValueChange={(value) =>
                    setDraft((current) => ({
                      ...current,
                      strategy: value,
                    }))
                  }
                  options={[
                    { label: 'Deterministic priority', value: 'deterministic_priority' },
                    { label: 'Weighted random', value: 'weighted_random' },
                    { label: 'SLO aware', value: 'slo_aware' },
                    { label: 'Geo affinity', value: 'geo_affinity' },
                  ]}
                  value={draft.strategy}
                />

                <SelectField
                  label={t('Preferred region')}
                  onValueChange={(value) =>
                    setDraft((current) => ({
                      ...current,
                      preferred_region: value,
                    }))
                  }
                  options={preferredRegionOptions}
                  value={draft.preferred_region}
                />

                <DialogField htmlFor="routing-profile-max-cost" label={t('Max cost')}>
                  <Input
                    id="routing-profile-max-cost"
                    min={0}
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setDraft((current) => ({
                        ...current,
                        max_cost: event.target.value,
                      }))
                    }
                    placeholder="0.00"
                    step="0.0001"
                    type="number"
                    value={draft.max_cost}
                  />
                </DialogField>

                <DialogField
                  htmlFor="routing-profile-max-latency"
                  label={t('Max latency (ms)')}
                >
                  <Input
                    id="routing-profile-max-latency"
                    min={0}
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setDraft((current) => ({
                        ...current,
                        max_latency_ms: event.target.value,
                      }))
                    }
                    placeholder="1200"
                    step="1"
                    type="number"
                    value={draft.max_latency_ms}
                  />
                </DialogField>
              </FormGrid>

              <label className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3">
                <Checkbox
                  checked={draft.require_healthy}
                  onCheckedChange={(checked: boolean | 'indeterminate') =>
                    setDraft((current) => ({
                      ...current,
                      require_healthy: checked === true,
                    }))
                  }
                />
                <div className="space-y-1">
                  <div className="font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Require healthy')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Only keep healthy providers in the candidate set when this profile is applied.')}
                  </div>
                </div>
              </label>
            </FormSection>

            <FormSection
              description={t('Choose which providers belong to the profile, then arrange the fallback chain explicitly.')}
              title={t('Provider order')}
            >
              {providerOptions.length ? (
                <div className="grid gap-3 md:grid-cols-2">
                  {providerOptions.map((provider) => {
                    const checked = draft.ordered_provider_ids.includes(provider.id);

                    return (
                      <label
                        className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3"
                        key={provider.id}
                      >
                        <Checkbox
                          checked={checked}
                          onCheckedChange={(nextChecked: boolean | 'indeterminate') =>
                            toggleProvider(provider.id, nextChecked === true)
                          }
                        />
                        <div className="space-y-1">
                          <div className="font-medium text-[var(--sdk-color-text-primary)]">
                            {provider.display_name}
                          </div>
                          <div className="font-mono text-xs text-[var(--sdk-color-text-secondary)]">
                            {provider.id}
                          </div>
                          <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                            {provider.adapter_kind}
                          </div>
                        </div>
                      </label>
                    );
                  })}
                </div>
              ) : (
                <div className="rounded-[var(--sdk-radius-panel)] border border-dashed border-[var(--sdk-color-border-default)] px-4 py-5 text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Create route providers first, then return to build a reusable routing profile.')}
                </div>
              )}

              {orderedProviders.length ? (
                <div className="space-y-3">
                  <SelectField
                    label={t('Default provider')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        default_provider_id: value,
                      }))
                    }
                    options={orderedProviders.map((provider) => ({
                      label: `${provider.display_name} (${provider.id})`,
                      value: provider.id,
                    }))}
                    value={draft.default_provider_id}
                  />

                  <div className="space-y-3">
                    {orderedProviders.map((provider, index) => (
                      <div
                        className="flex flex-wrap items-center justify-between gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] px-4 py-3"
                        key={provider.id}
                      >
                        <div className="space-y-1">
                          <div className="font-medium text-[var(--sdk-color-text-primary)]">
                            {index + 1}. {provider.display_name}
                          </div>
                          <div className="font-mono text-xs text-[var(--sdk-color-text-secondary)]">
                            {provider.id}
                          </div>
                        </div>

                        <div className="flex flex-wrap items-center gap-2">
                          {draft.default_provider_id === provider.id ? (
                            <StatusBadge
                              label={t('Default')}
                              showIcon
                              status="active"
                              variant="secondary"
                            />
                          ) : null}
                          <Button
                            disabled={index === 0}
                            onClick={() => moveProvider(provider.id, -1)}
                            size="sm"
                            type="button"
                            variant="outline"
                          >
                            {t('Move up')}
                          </Button>
                          <Button
                            disabled={index === orderedProviders.length - 1}
                            onClick={() => moveProvider(provider.id, 1)}
                            size="sm"
                            type="button"
                            variant="outline"
                          >
                            {t('Move down')}
                          </Button>
                          <Button
                            onClick={() => toggleProvider(provider.id, false)}
                            size="sm"
                            type="button"
                            variant="ghost"
                          >
                            {t('Remove')}
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}
            </FormSection>

            <FormActions>
              <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
                {t('Close')}
              </Button>
              <Button onClick={resetDraftForCreation} type="button" variant="outline">
                {t('Create profile')}
              </Button>
              <Button disabled={busy} type="submit" variant="primary">
                {t('Save routing profile')}
              </Button>
            </FormActions>
          </form>
        </div>
      </DialogContent>
    </Dialog>
  );
}
