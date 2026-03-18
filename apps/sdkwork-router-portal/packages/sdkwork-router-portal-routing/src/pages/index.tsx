import { useEffect, useMemo, useState } from 'react';
import {
  EmptyState,
  InlineButton,
  MetricCard,
  Pill,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingProviderOption,
  PortalRoutingSummary,
} from 'sdkwork-router-portal-types';

import {
  RoutingEvidenceList,
  RoutingPresetGrid,
  RoutingProviderList,
} from '../components';
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

function toFormState(summary: PortalRoutingSummary): RoutingFormState {
  return {
    preset_id: summary.preferences.preset_id || 'platform_default',
    strategy: summary.preferences.strategy,
    ordered_provider_ids: summary.provider_options.map((provider) => provider.provider_id),
    default_provider_id: summary.preferences.default_provider_id ?? null,
    max_cost: summary.preferences.max_cost === null || summary.preferences.max_cost === undefined
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

export function PortalRoutingPage({ onNavigate }: PortalRoutingPageProps) {
  const [summary, setSummary] = useState<PortalRoutingSummary | null>(null);
  const [decisionLogs, setDecisionLogs] = useState<PortalRoutingDecisionLog[]>([]);
  const [preview, setPreview] = useState<PortalRoutingDecision | null>(null);
  const [form, setForm] = useState<RoutingFormState | null>(null);
  const [previewForm, setPreviewForm] = useState<RoutingPreviewFormState | null>(null);
  const [status, setStatus] = useState('Loading routing posture...');
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);

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
          setStatus('Routing posture is synced with the latest project evidence.');
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

  function applyPreset(presetId: string) {
    if (!form) {
      return;
    }

    if (presetId === 'predictable') {
      setForm({ ...form, preset_id: 'predictable', strategy: 'deterministic_priority', require_healthy: false });
    } else if (presetId === 'distribution') {
      setForm({ ...form, preset_id: 'distribution', strategy: 'weighted_random', require_healthy: false });
    } else if (presetId === 'reliability') {
      setForm({
        ...form,
        preset_id: 'reliability',
        strategy: 'slo_aware',
        require_healthy: true,
        max_latency_ms: form.max_latency_ms || '250',
      });
    } else if (presetId === 'regional') {
      setForm({
        ...form,
        preset_id: 'regional',
        strategy: 'geo_affinity',
        preferred_region: form.preferred_region || 'us-east',
      });
    }

    setStatus('Preset applied locally. Save the routing posture when it looks right.');
  }

  function moveProvider(providerId: string, direction: 'up' | 'down') {
    if (!form) {
      return;
    }

    const currentIndex = form.ordered_provider_ids.indexOf(providerId);
    if (currentIndex === -1) {
      return;
    }

    const nextIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
    if (nextIndex < 0 || nextIndex >= form.ordered_provider_ids.length) {
      return;
    }

    const orderedProviderIds = [...form.ordered_provider_ids];
    const [moved] = orderedProviderIds.splice(currentIndex, 1);
    orderedProviderIds.splice(nextIndex, 0, moved);
    setForm({ ...form, ordered_provider_ids: orderedProviderIds });
  }

  function setDefaultProvider(providerId: string) {
    if (!form) {
      return;
    }

    setForm({ ...form, default_provider_id: providerId });
  }

  async function handleSave() {
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
      setStatus('Routing preferences saved. The next preview now reflects the updated posture.');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSaving(false);
    }
  }

  async function handlePreview() {
    if (!summary || !form) {
      return;
    }

    setPreviewing(true);
    setStatus('Previewing the active route...');

    try {
      const capability = previewForm?.capability.trim() || 'chat_completion';
      const model = previewForm?.model.trim() || summary.latest_model_hint;
      const requested_region =
        previewForm?.requested_region.trim() || form.preferred_region || null;
      const selection_seed = integerOrNull(previewForm?.selection_seed ?? '');

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
        selection_seed: selection_seed === null ? current?.selection_seed ?? '' : String(selection_seed),
      }));
      setStatus('Preview updated with the current routing posture and project evidence.');
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
          detail="Routing posture will appear once the portal finishes loading project summary, provider options, and recent evidence."
          title="Preparing route control center"
        />
      </Surface>
    );
  }

  const orderedProviders = reorderProviders(viewModel.provider_options, form.ordered_provider_ids);

  return (
    <>
      <div className="portalx-status-row">
        <Pill tone="accent">Model: {viewModel.summary.latest_model_hint}</Pill>
        <Pill tone="positive">Provider: {viewModel.preview.selected_provider_id}</Pill>
        <Pill tone={viewModel.preview.slo_degraded ? 'warning' : 'default'}>
          Strategy: {buildRoutingStrategyLabel(viewModel.preview.strategy ?? viewModel.summary.preferences.strategy)}
        </Pill>
        <InlineButton onClick={handlePreview} tone="secondary">
          {previewing ? 'Previewing...' : 'Preview route'}
        </InlineButton>
        <InlineButton onClick={handleSave} tone="primary">
          {saving ? 'Saving...' : 'Save posture'}
        </InlineButton>
      </div>

      <div className="portalx-summary-grid">
        <MetricCard
          detail="The current strategy label translated into product language instead of raw enum values."
          label="Active posture"
          value={buildRoutingStrategyLabel(form.strategy)}
        />
        <MetricCard
          detail="The provider that the latest preview selected for the current model."
          label="Selected provider"
          value={viewModel.preview.selected_provider_id}
        />
        <MetricCard
          detail="The requested model currently loaded into the preview workbench."
          label="Preview model"
          value={previewForm.model || viewModel.summary.latest_model_hint}
        />
        <MetricCard
          detail="Recent preview and live evidence stays close to the route editor."
          label="Evidence entries"
          value={String(decisionLogs.length)}
        />
      </div>

      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="policy">Policy editor</TabsTrigger>
          <TabsTrigger value="evidence">Evidence stream</TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <div className="portalx-split-grid portalx-split-grid-wide">
            <Surface
              detail="Predictable order, traffic distribution, reliability guardrails, and regional preference translate backend strategy names into routing posture that a customer can actually choose from."
              title="Routing posture presets"
            >
              <RoutingPresetGrid onApply={applyPreset} presets={viewModel.preset_cards} />
            </Surface>

            <Surface
              detail="Use provider order and a default fallback to decide which providers lead the route when multiple candidates are eligible. The first healthy available provider in your ordered list wins."
              title="Provider order"
            >
              <RoutingProviderList
                onDefault={setDefaultProvider}
                onMove={moveProvider}
                providers={orderedProviders.map((provider) => ({
                  ...provider,
                  preferred: form.ordered_provider_ids.includes(provider.provider_id),
                  default_provider: form.default_provider_id === provider.provider_id,
                }))}
              />
            </Surface>
          </div>
        </TabsContent>

        <TabsContent value="policy">
          <div className="portalx-split-grid portalx-split-grid-wide">
            <Surface
              detail={status}
              title="Reliability guardrails"
            >
              <div className="portalx-form portalx-form-card">
                <label className="portalx-field">
                  <span>Routing profile label</span>
                  <input
                    onChange={(event) => setForm({ ...form, preset_id: event.target.value })}
                    placeholder="predictable"
                    value={form.preset_id}
                  />
                </label>
                <label className="portalx-field">
                  <span>Strategy</span>
                  <select
                    onChange={(event) => setForm({
                      ...form,
                      strategy: event.target.value as PortalRoutingPreferences['strategy'],
                    })}
                    value={form.strategy}
                  >
                    <option value="deterministic_priority">Predictable order</option>
                    <option value="weighted_random">Traffic distribution</option>
                    <option value="slo_aware">Reliability guardrails</option>
                    <option value="geo_affinity">Regional preference</option>
                  </select>
                </label>
                <label className="portalx-field">
                  <span>Max cost</span>
                  <input
                    onChange={(event) => setForm({ ...form, max_cost: event.target.value })}
                    placeholder="0.30"
                    value={form.max_cost}
                  />
                </label>
                <label className="portalx-field">
                  <span>Max latency ms</span>
                  <input
                    onChange={(event) => setForm({ ...form, max_latency_ms: event.target.value })}
                    placeholder="250"
                    value={form.max_latency_ms}
                  />
                </label>
                <label className="portalx-field">
                  <span>Preferred region</span>
                  <select
                    onChange={(event) => setForm({ ...form, preferred_region: event.target.value })}
                    value={form.preferred_region}
                  >
                    <option value="">Auto</option>
                    <option value="us-east">us-east</option>
                    <option value="us-west">us-west</option>
                    <option value="eu-west">eu-west</option>
                    <option value="ap-southeast">ap-southeast</option>
                  </select>
                </label>
                <label className="portalx-status-row">
                  <input
                    checked={form.require_healthy}
                    onChange={(event) => setForm({ ...form, require_healthy: event.target.checked })}
                    type="checkbox"
                  />
                  <span>Require healthy providers</span>
                </label>
              </div>
              <div className="portalx-summary-grid">
                {viewModel.guardrails.map((guardrail) => (
                  <MetricCard
                    detail={guardrail.detail}
                    key={guardrail.id}
                    label={guardrail.label}
                    value={guardrail.value}
                  />
                ))}
              </div>
            </Surface>

            <Surface
              detail="Preview the active route before live traffic hits the gateway. The preview uses the current project posture and recent portal evidence."
              title="Preview the active route"
            >
              <div className="portalx-form portalx-form-card">
                <label className="portalx-field">
                  <span>Capability</span>
                  <input
                    onChange={(event) => setPreviewForm({ ...previewForm, capability: event.target.value })}
                    placeholder="chat_completion"
                    value={previewForm.capability}
                  />
                </label>
                <label className="portalx-field">
                  <span>Requested model</span>
                  <input
                    onChange={(event) => setPreviewForm({ ...previewForm, model: event.target.value })}
                    placeholder={viewModel.summary.latest_model_hint}
                    value={previewForm.model}
                  />
                </label>
                <label className="portalx-field">
                  <span>Requested region</span>
                  <select
                    onChange={(event) => setPreviewForm({ ...previewForm, requested_region: event.target.value })}
                    value={previewForm.requested_region}
                  >
                    <option value="">Auto</option>
                    <option value="us-east">us-east</option>
                    <option value="us-west">us-west</option>
                    <option value="eu-west">eu-west</option>
                    <option value="ap-southeast">ap-southeast</option>
                  </select>
                </label>
                <label className="portalx-field">
                  <span>Selection seed</span>
                  <input
                    inputMode="numeric"
                    onChange={(event) => setPreviewForm({ ...previewForm, selection_seed: event.target.value })}
                    placeholder="Optional deterministic seed"
                    value={previewForm.selection_seed}
                  />
                </label>
              </div>
              <div className="portalx-checklist-grid">
                <article className="portalx-checklist-card">
                  <strong>Selected provider</strong>
                  <p>{viewModel.preview.selected_provider_id}</p>
                </article>
                <article className="portalx-checklist-card">
                  <strong>Selection reason</strong>
                  <p>{viewModel.preview.selection_reason ?? 'The current preview uses the top-ranked eligible provider.'}</p>
                </article>
                <article className="portalx-checklist-card">
                  <strong>Candidate path</strong>
                  <p>{viewModel.preview.candidate_ids.join(' -> ') || 'No candidate list available yet.'}</p>
                </article>
                <article className="portalx-checklist-card">
                  <strong>SLO posture</strong>
                  <p>{viewModel.preview.slo_degraded ? 'Degraded fallback' : viewModel.preview.slo_applied ? 'Guardrails applied' : 'No active guardrails'}</p>
                </article>
              </div>
              <div className="portalx-form-actions">
                <InlineButton onClick={handlePreview} tone="secondary">
                  {previewing ? 'Previewing...' : 'Run preview with current inputs'}
                </InlineButton>
                <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
                  Compare with usage
                </InlineButton>
                <InlineButton onClick={() => onNavigate('api-keys')} tone="ghost">
                  Validate with a key
                </InlineButton>
              </div>
            </Surface>
          </div>
        </TabsContent>

        <TabsContent value="evidence">
          <Surface
            detail="Recent routing evidence turns preview into an explainable operational workflow instead of a black box."
            title="Recent routing evidence"
          >
            {viewModel.evidence.length ? (
              <RoutingEvidenceList evidence={viewModel.evidence} logs={viewModel.logs} />
            ) : (
              <EmptyState
                detail="Run a preview or send live traffic and the portal will capture provider evidence here."
                title="No routing evidence yet"
              />
            )}
          </Surface>
        </TabsContent>
      </Tabs>
    </>
  );
}
