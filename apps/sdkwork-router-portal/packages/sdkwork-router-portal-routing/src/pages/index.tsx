import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';

import {
  Checkbox,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  EmptyState,
  FormField,
  InlineButton,
  Input,
  Label,
  Pill,
  Select,
  Surface,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingProviderOption,
  PortalRoutingSummary,
} from 'sdkwork-router-portal-types';

import { RoutingCardGrid } from '../components';
import type { RoutingCardItem } from '../components';
import {
  loadPortalRoutingDecisionLogs,
  loadPortalRoutingSummary,
  runPortalRoutingPreview,
  updatePortalRoutingPreferences,
} from '../repository';
import { buildPortalRoutingViewModel, buildRoutingStrategyLabel } from '../services';
import type { PortalRoutingPageProps } from '../types';

type RoutingFormState = {
  preset_id: string;
  strategy: PortalRoutingPreferences['strategy'];
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost: string;
  max_latency_ms: string;
  require_healthy: boolean;
  preferred_region: string;
};

type RoutingPreviewFormState = {
  capability: string;
  model: string;
  requested_region: string;
  selection_seed: string;
};

type RoutingWorkbenchLane = 'presets' | 'providers' | 'evidence';

type RoutingWorkbenchRow = {
  id: string;
  focus: string;
  subject: ReactNode;
  scope: ReactNode;
  status: ReactNode;
  detail: ReactNode;
  actions: ReactNode;
  searchText: string;
};

type RoutingWorkbenchConfig = {
  laneLabel: string;
  scopeLabel: string;
  detailLabel: string;
  actionsLabel: string;
  detail: string;
  emptyTitle: string;
  emptyDetail: string;
  focusOptions: Array<{ value: string; label: string }>;
};

const WORKBENCH_OPTIONS: Array<{ value: RoutingWorkbenchLane; label: string }> = [
  { value: 'providers', label: 'Provider roster' },
  { value: 'presets', label: 'Preset catalog' },
  { value: 'evidence', label: 'Evidence stream' },
];

function toFormState(summary: PortalRoutingSummary): RoutingFormState {
  return {
    preset_id: summary.preferences.preset_id || 'platform_default',
    strategy: summary.preferences.strategy,
    ordered_provider_ids: summary.provider_options.map((provider) => provider.provider_id),
    default_provider_id: summary.preferences.default_provider_id ?? null,
    max_cost:
      summary.preferences.max_cost === null || summary.preferences.max_cost === undefined
        ? ''
        : String(summary.preferences.max_cost),
    max_latency_ms:
      summary.preferences.max_latency_ms === null || summary.preferences.max_latency_ms === undefined
        ? ''
        : String(summary.preferences.max_latency_ms),
    require_healthy: summary.preferences.require_healthy,
    preferred_region: summary.preferences.preferred_region ?? '',
  };
}

function numericOrNull(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

function integerOrNull(value: string): number | null {
  const parsed = numericOrNull(value);
  if (parsed === null || !Number.isInteger(parsed) || parsed < 0) {
    return null;
  }

  return parsed;
}

function toPreviewFormState(
  summary: PortalRoutingSummary,
  decisionLogs: PortalRoutingDecisionLog[],
): RoutingPreviewFormState {
  const latestLog = [...decisionLogs].sort(
    (left, right) => right.created_at_ms - left.created_at_ms,
  )[0];

  return {
    capability: latestLog?.capability ?? 'chat_completion',
    model: summary.latest_model_hint,
    requested_region: summary.preferences.preferred_region ?? '',
    selection_seed:
      latestLog?.selection_seed === null || latestLog?.selection_seed === undefined
        ? ''
        : String(latestLog.selection_seed),
  };
}

function syncPreviewFormState(
  current: RoutingPreviewFormState | null,
  summary: PortalRoutingSummary,
  decisionLogs: PortalRoutingDecisionLog[],
): RoutingPreviewFormState {
  if (!current) {
    return toPreviewFormState(summary, decisionLogs);
  }

  return {
    capability: current.capability || 'chat_completion',
    model: current.model || summary.latest_model_hint,
    requested_region: current.requested_region,
    selection_seed: current.selection_seed,
  };
}

function reorderProviders(
  providers: PortalRoutingProviderOption[],
  orderedProviderIds: string[],
): PortalRoutingProviderOption[] {
  return [...providers].sort((left, right) => {
    const leftIndex = orderedProviderIds.indexOf(left.provider_id);
    const rightIndex = orderedProviderIds.indexOf(right.provider_id);
    const resolvedLeft = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const resolvedRight = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    return resolvedLeft - resolvedRight;
  });
}

function searchMatches(query: string, values: Array<string | null | undefined>): boolean {
  if (!query) {
    return true;
  }

  return values.filter(Boolean).join(' ').toLowerCase().includes(query);
}

function evidenceStatusTone(log: PortalRoutingDecisionLog) {
  if (log.slo_degraded) {
    return 'warning';
  }

  if (log.slo_applied) {
    return 'positive';
  }

  if (log.decision_source.toLowerCase().includes('preview')) {
    return 'accent';
  }

  return 'default';
}

function evidenceFocus(log: PortalRoutingDecisionLog): string {
  if (log.decision_source.toLowerCase().includes('preview')) {
    return 'preview';
  }

  if (log.slo_applied || log.slo_degraded) {
    return 'guardrailed';
  }

  return 'live';
}

function buildWorkbenchConfig(lane: RoutingWorkbenchLane): RoutingWorkbenchConfig {
  switch (lane) {
    case 'presets':
      return {
        laneLabel: 'Preset catalog',
        scopeLabel: 'Strategy',
        detailLabel: 'Operational detail',
        actionsLabel: 'Actions',
        detail:
          'Preset catalog converts backend routing strategy enums into product choices that an operator can apply without reading implementation details.',
        emptyTitle: 'No routing presets in this slice',
        emptyDetail: 'Adjust the operational focus or search to reveal a different routing preset.',
        focusOptions: [
          { value: 'all', label: 'All presets' },
          { value: 'active', label: 'Active preset' },
          { value: 'available', label: 'Available presets' },
        ],
      };
    case 'evidence':
      return {
        laneLabel: 'Evidence stream',
        scopeLabel: 'Routing signal',
        detailLabel: 'Selection detail',
        actionsLabel: 'Trace',
        detail:
          'Evidence stream keeps preview and live routing traces on one operational table instead of splitting them across tabs.',
        emptyTitle: 'No routing evidence in this slice',
        emptyDetail: 'Run a preview or send live traffic and routing evidence will appear here.',
        focusOptions: [
          { value: 'all', label: 'All evidence' },
          { value: 'preview', label: 'Preview traces' },
          { value: 'live', label: 'Live traces' },
          { value: 'guardrailed', label: 'Guardrailed' },
        ],
      };
    case 'providers':
    default:
      return {
        laneLabel: 'Provider roster',
        scopeLabel: 'Channel and order',
        detailLabel: 'Routing role',
        actionsLabel: 'Actions',
        detail:
          'Provider roster keeps ordered fallback, default provider, and channel coverage inside one workbench so operations can adjust posture without digging through forms.',
        emptyTitle: 'No providers in this slice',
        emptyDetail: 'Routing provider options will appear once the project summary is available.',
        focusOptions: [
          { value: 'all', label: 'All providers' },
          { value: 'default', label: 'Default provider' },
          { value: 'ordered', label: 'Ordered providers' },
        ],
      };
  }
}

function buildPreviewOutcomeCards(preview: PortalRoutingDecision): RoutingCardItem[] {
  return [
    {
      id: 'preview-provider',
      label: 'Selected provider',
      value: preview.selected_provider_id,
      detail: 'The provider chosen by the latest routing preview.',
      tone: 'positive' as const,
    },
    {
      id: 'preview-reason',
      label: 'Selection reason',
      value: preview.selection_reason ?? 'Top-ranked eligible provider',
      detail: 'The current preview explains why the selected provider won the route.',
    },
    {
      id: 'preview-candidates',
      label: 'Candidate path',
      value: preview.candidate_ids.join(' -> ') || 'No candidates',
      detail: 'Candidate order remains visible so fallback posture is explainable.',
    },
    {
      id: 'preview-slo',
      label: 'SLO posture',
      value: preview.slo_degraded
        ? 'Degraded fallback'
        : preview.slo_applied
          ? 'Guardrails applied'
          : 'No active guardrails',
      detail: preview.matched_policy_id
        ? `Matched policy ${preview.matched_policy_id}.`
        : 'No routing policy matched the current preview inputs.',
      tone: preview.slo_degraded ? 'warning' : preview.slo_applied ? 'positive' : 'default',
    },
  ];
}

export function PortalRoutingPage({ onNavigate }: PortalRoutingPageProps) {
  const [summary, setSummary] = useState<PortalRoutingSummary | null>(null);
  const [decisionLogs, setDecisionLogs] = useState<PortalRoutingDecisionLog[]>([]);
  const [preview, setPreview] = useState<PortalRoutingDecision | null>(null);
  const [form, setForm] = useState<RoutingFormState | null>(null);
  const [previewForm, setPreviewForm] = useState<RoutingPreviewFormState | null>(null);
  const [status, setStatus] = useState('Loading routing posture...');
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [previewDialogOpen, setPreviewDialogOpen] = useState(false);
  const [workbenchLane, setWorkbenchLane] = useState<RoutingWorkbenchLane>('providers');
  const [focusFilter, setFocusFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  async function refresh() {
    const [nextSummary, nextLogs] = await Promise.all([
      loadPortalRoutingSummary(),
      loadPortalRoutingDecisionLogs(),
    ]);
    setSummary(nextSummary);
    setPreview(nextSummary.preview);
    setForm(toFormState(nextSummary));
    setDecisionLogs(nextLogs);
    setPreviewForm((current) => syncPreviewFormState(current, nextSummary, nextLogs));
  }

  useEffect(() => {
    let cancelled = false;

    void refresh()
      .then(() => {
        if (!cancelled) {
          setStatus(
            'Routing workbench is synced with the latest project posture, provider order, and decision evidence.',
          );
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const viewModel = useMemo(() => {
    if (!summary) {
      return null;
    }

    return buildPortalRoutingViewModel(summary, decisionLogs, preview);
  }, [decisionLogs, preview, summary]);

  const orderedProviders = useMemo(() => {
    if (!viewModel || !form) {
      return [];
    }

    return reorderProviders(viewModel.provider_options, form.ordered_provider_ids);
  }, [form, viewModel]);

  const summaryCards = useMemo(() => {
    if (!viewModel || !form || !previewForm) {
      return [];
    }

    return [
      {
        id: 'summary-posture',
        label: 'Active posture',
        value: buildRoutingStrategyLabel(form.strategy),
        detail:
          'Routing strategy is translated into user-facing posture language instead of raw enum names.',
        tone: 'positive' as const,
      },
      {
        id: 'summary-default',
        label: 'Default provider',
        value: form.default_provider_id ?? 'Auto fallback',
        detail:
          'Default provider acts as the stable fallback when multiple candidates remain eligible.',
      },
      {
        id: 'summary-model',
        label: 'Preview model',
        value: previewForm.model || viewModel.summary.latest_model_hint,
        detail:
          'The current preview model stays visible so operators always know which workload is being tuned.',
      },
      {
        id: 'summary-evidence',
        label: 'Evidence entries',
        value: String(decisionLogs.length),
        detail:
          'Preview and live routing traces remain close to the posture editor for faster diagnosis.',
      },
    ];
  }, [decisionLogs.length, form, previewForm, viewModel]);

  const workbenchConfig = useMemo(
    () => buildWorkbenchConfig(workbenchLane),
    [workbenchLane],
  );

  const workbenchRows = useMemo<RoutingWorkbenchRow[]>(() => {
    if (!viewModel || !form) {
      return [];
    }

    if (workbenchLane === 'presets') {
      return viewModel.preset_cards.map((preset) => ({
        id: preset.id,
        focus: preset.active ? 'active' : 'available',
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{preset.title}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">{preset.id}</p>
          </div>
        ),
        scope: buildRoutingStrategyLabel(preset.strategy),
        status: (
          <Pill tone={preset.active ? 'positive' : 'default'}>
            {preset.active ? 'Active' : 'Available'}
          </Pill>
        ),
        detail: preset.detail,
        actions: (
          <InlineButton
            onClick={() => {
              if (preset.id === 'predictable') {
                setForm((current) =>
                  current
                    ? {
                        ...current,
                        preset_id: 'predictable',
                        strategy: 'deterministic_priority',
                        require_healthy: false,
                      }
                    : current,
                );
              } else if (preset.id === 'distribution') {
                setForm((current) =>
                  current
                    ? {
                        ...current,
                        preset_id: 'distribution',
                        strategy: 'weighted_random',
                        require_healthy: false,
                      }
                    : current,
                );
              } else if (preset.id === 'reliability') {
                setForm((current) =>
                  current
                    ? {
                        ...current,
                        preset_id: 'reliability',
                        strategy: 'slo_aware',
                        require_healthy: true,
                        max_latency_ms: current.max_latency_ms || '250',
                      }
                    : current,
                );
              } else if (preset.id === 'regional') {
                setForm((current) =>
                  current
                    ? {
                        ...current,
                        preset_id: 'regional',
                        strategy: 'geo_affinity',
                        preferred_region: current.preferred_region || 'us-east',
                      }
                    : current,
                );
              }

              setStatus(
                'Preset applied locally. Save posture when the updated routing shape looks right.',
              );
            }}
            tone={preset.active ? 'secondary' : 'primary'}
          >
            {preset.active ? 'Active preset' : 'Apply preset'}
          </InlineButton>
        ),
        searchText: [preset.title, preset.detail, preset.id, preset.strategy].join(' ').toLowerCase(),
      }));
    }

    if (workbenchLane === 'evidence') {
      return [...viewModel.logs]
        .sort((left, right) => right.created_at_ms - left.created_at_ms)
        .map((log) => ({
          id: log.decision_id,
          focus: evidenceFocus(log),
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">
                {log.route_key} -&gt; {log.selected_provider_id}
              </strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                {new Date(log.created_at_ms).toLocaleString()}
              </p>
            </div>
          ),
          scope: (
            <div className="space-y-1">
              <div>{buildRoutingStrategyLabel(log.strategy)}</div>
              <div className="text-xs text-zinc-500 dark:text-zinc-400">
                {log.capability}
                {log.requested_region ? ` / ${log.requested_region}` : ''}
              </div>
            </div>
          ),
          status: (
            <Pill tone={evidenceStatusTone(log)}>
              {log.slo_degraded
                ? 'Degraded'
                : log.slo_applied
                  ? 'Guardrailed'
                  : log.decision_source.toLowerCase().includes('preview')
                    ? 'Preview'
                    : 'Live'}
            </Pill>
          ),
          detail:
            log.selection_reason
            ?? log.assessments[0]?.reasons[0]
            ?? 'Selection evidence is available from the current routing trace.',
          actions: (
            <div className="space-y-1 text-xs text-zinc-500 dark:text-zinc-400">
              <div>{log.decision_source}</div>
              <div>{log.matched_policy_id ?? 'No matched policy'}</div>
            </div>
          ),
          searchText: [
            log.route_key,
            log.selected_provider_id,
            log.strategy,
            log.decision_source,
            log.selection_reason,
            log.requested_region,
            log.capability,
          ]
            .filter(Boolean)
            .join(' ')
            .toLowerCase(),
        }));
    }

    return orderedProviders.map((provider, index) => {
      const isDefault = form.default_provider_id === provider.provider_id;

      return {
        id: provider.provider_id,
        focus: isDefault ? 'default' : 'ordered',
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{provider.display_name}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">{provider.provider_id}</p>
          </div>
        ),
        scope: (
          <div className="space-y-1">
            <div>{provider.channel_id}</div>
            <div className="text-xs text-zinc-500 dark:text-zinc-400">Priority #{index + 1}</div>
          </div>
        ),
        status: (
          <Pill tone={isDefault ? 'accent' : 'positive'}>
            {isDefault ? 'Default' : 'Ordered'}
          </Pill>
        ),
        detail: isDefault
          ? 'Default provider stays available as the stable fallback when several providers remain eligible.'
          : 'Ordered providers keep deterministic failover readable for operators and support teams.',
        actions: (
          <div className="flex flex-wrap gap-2">
            <InlineButton
              disabled={index === 0}
              onClick={() => {
                setForm((current) => {
                  if (!current) {
                    return current;
                  }

                  const currentIndex = current.ordered_provider_ids.indexOf(provider.provider_id);
                  if (currentIndex <= 0) {
                    return current;
                  }

                  const orderedProviderIds = [...current.ordered_provider_ids];
                  const [moved] = orderedProviderIds.splice(currentIndex, 1);
                  orderedProviderIds.splice(currentIndex - 1, 0, moved);
                  return { ...current, ordered_provider_ids: orderedProviderIds };
                });
                setStatus(
                  'Provider order changed locally. Save posture to publish the new fallback order.',
                );
              }}
              tone="ghost"
            >
              Move up
            </InlineButton>
            <InlineButton
              disabled={index === orderedProviders.length - 1}
              onClick={() => {
                setForm((current) => {
                  if (!current) {
                    return current;
                  }

                  const currentIndex = current.ordered_provider_ids.indexOf(provider.provider_id);
                  if (
                    currentIndex === -1
                    || currentIndex >= current.ordered_provider_ids.length - 1
                  ) {
                    return current;
                  }

                  const orderedProviderIds = [...current.ordered_provider_ids];
                  const [moved] = orderedProviderIds.splice(currentIndex, 1);
                  orderedProviderIds.splice(currentIndex + 1, 0, moved);
                  return { ...current, ordered_provider_ids: orderedProviderIds };
                });
                setStatus(
                  'Provider order changed locally. Save posture to publish the new fallback order.',
                );
              }}
              tone="ghost"
            >
              Move down
            </InlineButton>
            <InlineButton
              onClick={() => {
                setForm((current) =>
                  current ? { ...current, default_provider_id: provider.provider_id } : current,
                );
                setStatus('Default provider updated locally. Save posture to publish the change.');
              }}
              tone={isDefault ? 'secondary' : 'primary'}
            >
              {isDefault ? 'Default provider' : 'Set default'}
            </InlineButton>
          </div>
        ),
        searchText: [
          provider.display_name,
          provider.provider_id,
          provider.channel_id,
          String(index + 1),
          isDefault ? 'default' : 'ordered',
        ]
          .join(' ')
          .toLowerCase(),
      };
    });
  }, [form, orderedProviders, viewModel, workbenchLane]);

  const visibleWorkbenchRows = useMemo(
    () =>
      workbenchRows.filter(
        (row) =>
          (focusFilter === 'all' || row.focus === focusFilter)
          && searchMatches(deferredSearch, [row.searchText]),
      ),
    [deferredSearch, focusFilter, workbenchRows],
  );

  const previewOutcomeCards = useMemo(
    () => (viewModel ? buildPreviewOutcomeCards(viewModel.preview) : []),
    [viewModel],
  );

  async function handleSave(event?: FormEvent<HTMLFormElement>): Promise<void> {
    event?.preventDefault();
    if (!form) {
      return;
    }

    setSaving(true);
    setStatus('Saving routing preferences for this project...');

    try {
      await updatePortalRoutingPreferences({
        preset_id: form.preset_id,
        strategy: form.strategy,
        ordered_provider_ids: form.ordered_provider_ids,
        default_provider_id: form.default_provider_id,
        max_cost: numericOrNull(form.max_cost),
        max_latency_ms: numericOrNull(form.max_latency_ms),
        require_healthy: form.require_healthy,
        preferred_region: form.preferred_region || null,
      });
      await refresh();
      setEditDialogOpen(false);
      setStatus(
        'Routing posture saved. The workbench now reflects the updated provider order and guardrails.',
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSaving(false);
    }
  }

  async function handlePreview(event?: FormEvent<HTMLFormElement>): Promise<void> {
    event?.preventDefault();
    if (!summary || !form || !previewForm) {
      return;
    }

    setPreviewing(true);
    setStatus('Previewing the active route...');

    try {
      const capability = previewForm.capability.trim() || 'chat_completion';
      const model = previewForm.model.trim() || summary.latest_model_hint;
      const requested_region = previewForm.requested_region.trim() || form.preferred_region || null;
      const selection_seed = integerOrNull(previewForm.selection_seed);

      const nextPreview = await runPortalRoutingPreview({
        capability,
        model,
        requested_region,
        selection_seed,
      });

      setPreview(nextPreview);
      const nextLogs = await loadPortalRoutingDecisionLogs();
      setDecisionLogs(nextLogs);
      setPreviewForm((current) => ({
        capability,
        model,
        requested_region: requested_region ?? '',
        selection_seed:
          selection_seed === null ? current?.selection_seed ?? '' : String(selection_seed),
      }));
      setPreviewDialogOpen(false);
      setWorkbenchLane('evidence');
      setFocusFilter('preview');
      setStatus('Preview updated with the current routing posture and added to the evidence stream.');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setPreviewing(false);
    }
  }

  if (!viewModel || !form || !previewForm) {
    return (
      <Surface detail={status} title="Routing">
        <EmptyState
          detail="Routing posture will appear once the portal finishes loading project summary, provider options, and decision evidence."
          title="Preparing routing workbench"
        />
      </Surface>
    );
  }

  const focusLabel =
    workbenchConfig.focusOptions.find((option) => option.value === focusFilter)?.label ?? 'All';
  const guardrailCards: RoutingCardItem[] = viewModel.guardrails.map((item) => {
    const tone: RoutingCardItem['tone'] =
      item.id === 'provider-default'
        ? 'accent'
        : item.value === 'Open'
          ? 'default'
          : 'positive';

    return {
      id: item.id,
      label: item.label,
      value: item.value,
      detail: item.detail,
      tone,
    };
  });
  const latestSignals = viewModel.evidence.slice(0, 3);
  const previewAssessments = viewModel.preview.assessments.slice(0, 4);

  return (
    <>
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit routing posture</DialogTitle>
            <DialogDescription>
              Save posture after adjusting profile label, strategy, regional preference, and reliability guardrails.
            </DialogDescription>
          </DialogHeader>

          <form className="grid gap-4 md:grid-cols-2" onSubmit={(event) => void handleSave(event)}>
            <FormField label="Routing profile label">
              <Input
                onChange={(event) => setForm({ ...form, preset_id: event.target.value })}
                placeholder="predictable"
                value={form.preset_id}
              />
            </FormField>
            <FormField label="Strategy">
              <Select
                onChange={(event) =>
                  setForm({
                    ...form,
                    strategy: event.target.value as PortalRoutingPreferences['strategy'],
                  })
                }
                value={form.strategy}
              >
                <option value="deterministic_priority">Predictable order</option>
                <option value="weighted_random">Traffic distribution</option>
                <option value="slo_aware">Reliability guardrails</option>
                <option value="geo_affinity">Regional preference</option>
              </Select>
            </FormField>
            <FormField label="Max cost">
              <Input
                onChange={(event) => setForm({ ...form, max_cost: event.target.value })}
                placeholder="0.30"
                value={form.max_cost}
              />
            </FormField>
            <FormField label="Max latency ms">
              <Input
                onChange={(event) => setForm({ ...form, max_latency_ms: event.target.value })}
                placeholder="250"
                value={form.max_latency_ms}
              />
            </FormField>
            <FormField label="Preferred region">
              <Select
                onChange={(event) => setForm({ ...form, preferred_region: event.target.value })}
                value={form.preferred_region}
              >
                <option value="">Auto</option>
                <option value="us-east">us-east</option>
                <option value="us-west">us-west</option>
                <option value="eu-west">eu-west</option>
                <option value="ap-southeast">ap-southeast</option>
              </Select>
            </FormField>
            <FormField label="Default provider">
              <Select
                onChange={(event) =>
                  setForm({
                    ...form,
                    default_provider_id: event.target.value || null,
                  })
                }
                value={form.default_provider_id ?? ''}
              >
                <option value="">Auto fallback</option>
                {orderedProviders.map((provider) => (
                  <option key={provider.provider_id} value={provider.provider_id}>
                    {provider.display_name}
                  </option>
                ))}
              </Select>
            </FormField>
            <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60 md:col-span-2">
              <Label className="flex items-center gap-3 text-sm font-medium text-zinc-700 dark:text-zinc-300">
                <Checkbox
                  checked={form.require_healthy}
                  onChange={(event) =>
                    setForm({
                      ...form,
                      require_healthy: event.target.checked,
                    })
                  }
                />
                <span>Require healthy providers</span>
              </Label>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                Reliability guardrails bias routing toward healthy, lower-risk providers before traffic leaves the workspace.
              </p>
            </div>
            <DialogFooter className="md:col-span-2">
              <InlineButton onClick={() => setEditDialogOpen(false)} tone="ghost">
                Cancel
              </InlineButton>
              <InlineButton disabled={saving} tone="primary" type="submit">
                {saving ? 'Saving...' : 'Save posture'}
              </InlineButton>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={previewDialogOpen} onOpenChange={setPreviewDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Preview route</DialogTitle>
            <DialogDescription>
              Preview route inputs are stored separately from the saved posture so operators can test scenarios before traffic shifts.
            </DialogDescription>
          </DialogHeader>

          <form className="grid gap-4 md:grid-cols-2" onSubmit={(event) => void handlePreview(event)}>
            <FormField label="Capability">
              <Input
                onChange={(event) =>
                  setPreviewForm({ ...previewForm, capability: event.target.value })
                }
                placeholder="chat_completion"
                value={previewForm.capability}
              />
            </FormField>
            <FormField label="Requested model">
              <Input
                onChange={(event) =>
                  setPreviewForm({ ...previewForm, model: event.target.value })
                }
                placeholder={viewModel.summary.latest_model_hint}
                value={previewForm.model}
              />
            </FormField>
            <FormField label="Requested region">
              <Select
                onChange={(event) =>
                  setPreviewForm({ ...previewForm, requested_region: event.target.value })
                }
                value={previewForm.requested_region}
              >
                <option value="">Auto</option>
                <option value="us-east">us-east</option>
                <option value="us-west">us-west</option>
                <option value="eu-west">eu-west</option>
                <option value="ap-southeast">ap-southeast</option>
              </Select>
            </FormField>
            <FormField label="Selection seed">
              <Input
                inputMode="numeric"
                onChange={(event) =>
                  setPreviewForm({ ...previewForm, selection_seed: event.target.value })
                }
                placeholder="Optional deterministic seed"
                value={previewForm.selection_seed}
              />
            </FormField>
            <DialogFooter className="md:col-span-2">
              <InlineButton onClick={() => setPreviewDialogOpen(false)} tone="ghost">
                Close
              </InlineButton>
              <InlineButton disabled={previewing} tone="primary" type="submit">
                {previewing ? 'Running preview...' : 'Run preview'}
              </InlineButton>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4">
        <Surface
          actions={(
            <div
              data-slot="portal-routing-toolbar"
              className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap"
            >
              <InlineButton onClick={() => setEditDialogOpen(true)} tone="primary">
                Edit posture
              </InlineButton>
              <InlineButton onClick={() => setPreviewDialogOpen(true)} tone="secondary">
                Run preview
              </InlineButton>
              <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
                Open usage
              </InlineButton>
              <InlineButton onClick={() => onNavigate('api-keys')} tone="ghost">
                Validate with a key
              </InlineButton>
            </div>
          )}
          detail={status}
          title="Routing posture"
        >
          <RoutingCardGrid items={summaryCards} />
        </Surface>

        <Surface detail={workbenchConfig.detail} title="Routing workbench">
          <div className="grid gap-4">
            <div className="flex flex-wrap items-center gap-3 text-sm text-zinc-500 dark:text-zinc-400">
              <Pill tone="seed">{workbenchConfig.laneLabel}</Pill>
              <span>{`${visibleWorkbenchRows.length} of ${workbenchRows.length} rows visible`}</span>
              <span>{`Focus: ${focusLabel}`}</span>
            </div>

            <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
              Routing workbench keeps Provider roster, Preset catalog, and Evidence stream inside
              one operator table while edit and preview actions stay inside focused dialogs.
            </p>

            <ToolbarInline
              data-slot="portal-routing-filter-bar"
            >
              <ToolbarSearchField
                className="min-w-[15rem] flex-[0_1_20rem]"
                label="Search routing evidence"
                onChange={(event) => setSearchQuery(event.target.value)}
                placeholder="Search routing evidence"
                value={searchQuery}
              />
              <ToolbarField label="Workbench lane" className="min-w-[12rem] shrink-0">
                <Select
                  onChange={(event) => {
                    setWorkbenchLane(event.target.value as RoutingWorkbenchLane);
                    setFocusFilter('all');
                  }}
                  value={workbenchLane}
                >
                  {WORKBENCH_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </Select>
              </ToolbarField>

              <ToolbarField
                label="Operational focus"
                className="min-w-[12rem] shrink-0"
              >
                <Select
                  onChange={(event) => setFocusFilter(event.target.value)}
                  value={focusFilter}
                >
                  {workbenchConfig.focusOptions.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </Select>
              </ToolbarField>

              <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
                <InlineButton
                  onClick={() => {
                    setFocusFilter('all');
                    setSearchQuery('');
                  }}
                  tone="secondary"
                >
                  Clear filters
                </InlineButton>
              </div>
            </ToolbarInline>

            <DataTable
              columns={[
                { key: 'subject', label: 'Subject', render: (row) => row.subject },
                { key: 'scope', label: workbenchConfig.scopeLabel, render: (row) => row.scope },
                { key: 'status', label: 'Status', render: (row) => row.status },
                { key: 'detail', label: workbenchConfig.detailLabel, render: (row) => row.detail },
                {
                  key: 'actions',
                  label: workbenchConfig.actionsLabel,
                  render: (row) => row.actions,
                },
              ]}
              empty={(
                <div className="mx-auto flex max-w-[34rem] flex-col items-center gap-2 text-center">
                  <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {workbenchConfig.emptyTitle}
                  </strong>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {workbenchConfig.emptyDetail}
                  </p>
                </div>
              )}
              getKey={(row) => row.id}
              rows={visibleWorkbenchRows}
            />
          </div>
        </Surface>

        <div className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
          <Surface
            detail="Guardrail posture keeps cost, latency, regional preference, and the latest routing signals readable before you publish changes."
            title="Guardrail posture"
          >
            <div className="grid gap-4">
              <RoutingCardGrid columns="xl:grid-cols-2" items={guardrailCards} />

              <section className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      Latest routing signals
                    </strong>
                    <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      Preview and live traces stay adjacent to guardrails so posture changes remain
                      explainable without secondary tabs.
                    </p>
                  </div>
                  <Pill tone="accent">{`${latestSignals.length} signals`}</Pill>
                </div>

                <div className="mt-4 grid gap-3">
                  {latestSignals.length ? (
                    latestSignals.map((item) => (
                      <article
                        key={item.id}
                        className="rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70"
                      >
                        <div className="flex flex-wrap items-start justify-between gap-3">
                          <div className="space-y-1">
                            <strong className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                              {item.title}
                            </strong>
                            <p className="text-xs uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                              {item.timestamp_label}
                            </p>
                          </div>
                        </div>
                        <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                          {item.detail}
                        </p>
                      </article>
                    ))
                  ) : (
                    <EmptyState
                      detail="Run a preview or wait for live traffic to collect routing signals."
                      title="No routing signals yet"
                    />
                  )}
                </div>
              </section>
            </div>
          </Surface>

          <Surface
            detail="Preview outcome keeps the selected provider, fallback path, and provider assessments visible before traffic posture is saved."
            title="Preview outcome"
          >
            <div className="grid gap-4">
              <RoutingCardGrid columns="xl:grid-cols-2" items={previewOutcomeCards} />

              <section className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      Candidate assessments
                    </strong>
                    <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      Selection evidence stays operationally readable so support teams can validate
                      health, latency, and policy posture before rollout.
                    </p>
                  </div>
                  <Pill
                    tone={
                      viewModel.preview.slo_degraded
                        ? 'warning'
                        : viewModel.preview.slo_applied
                          ? 'positive'
                          : 'seed'
                    }
                  >
                    {viewModel.preview.slo_degraded
                      ? 'Degraded fallback'
                      : viewModel.preview.slo_applied
                        ? 'Guardrails applied'
                        : 'Preview only'}
                  </Pill>
                </div>

                <div className="mt-4 grid gap-3">
                  {previewAssessments.length ? (
                    previewAssessments.map((assessment) => (
                      <article
                        key={assessment.provider_id}
                        className="rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70"
                      >
                        <div className="flex flex-wrap items-start justify-between gap-3">
                          <div className="space-y-1">
                            <strong className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                              {assessment.provider_id}
                            </strong>
                            <p className="text-xs text-zinc-500 dark:text-zinc-400">
                              {assessment.region ? `${assessment.region} / ` : ''}
                              {assessment.available ? 'Available' : 'Not available'}
                            </p>
                          </div>
                          <Pill tone={assessment.available ? 'positive' : 'warning'}>
                            {assessment.health}
                          </Pill>
                        </div>

                        <div className="mt-3 flex flex-wrap gap-2 text-xs text-zinc-500 dark:text-zinc-400">
                          <span>{`Rank ${assessment.policy_rank}`}</span>
                          <span>
                            {`Latency ${assessment.latency_ms === null || assessment.latency_ms === undefined ? 'n/a' : `${assessment.latency_ms}ms`}`}
                          </span>
                          <span>
                            {`Cost ${assessment.cost === null || assessment.cost === undefined ? 'n/a' : `$${assessment.cost.toFixed(4)}`}`}
                          </span>
                        </div>

                        <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                          {assessment.reasons[0]
                            ?? assessment.slo_violations[0]
                            ?? 'The preview did not expose additional assessment detail for this provider.'}
                        </p>
                      </article>
                    ))
                  ) : (
                    <EmptyState
                      detail="Run a preview to inspect provider-level candidate assessments."
                      title="No preview assessments yet"
                    />
                  )}
                </div>
              </section>
            </div>
          </Surface>
        </div>
      </div>
    </>
  );
}
