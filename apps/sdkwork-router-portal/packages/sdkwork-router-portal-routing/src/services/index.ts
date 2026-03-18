import { formatDateTime } from 'sdkwork-router-portal-commons';
import type {
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingSummary,
  PortalRoutingStrategy,
} from 'sdkwork-router-portal-types';

import type {
  PortalRoutingPageViewModel,
  RoutingEvidenceItem,
  RoutingGuardrailItem,
  RoutingPresetCard,
} from '../types';

export function buildRoutingStrategyLabel(
  strategy?: PortalRoutingStrategy | string | null,
): string {
  switch (strategy) {
    case 'deterministic_priority':
      return 'Predictable order';
    case 'weighted_random':
      return 'Traffic distribution';
    case 'slo_aware':
      return 'Reliability guardrails';
    case 'geo_affinity':
      return 'Regional preference';
    case 'static_fallback':
      return 'Platform fallback';
    default:
      return 'Adaptive routing';
  }
}

function buildPresetCards(
  preferences: PortalRoutingPreferences,
): RoutingPresetCard[] {
  return [
    {
      id: 'predictable',
      title: 'Predictable order',
      detail: 'The first healthy available provider in your ordered list wins, and the next provider becomes the deterministic fallback.',
      strategy: 'deterministic_priority',
      active: preferences.strategy === 'deterministic_priority',
    },
    {
      id: 'distribution',
      title: 'Traffic distribution',
      detail: 'Spread traffic across eligible providers when you want to balance exposure instead of pinning every request to one path.',
      strategy: 'weighted_random',
      active: preferences.strategy === 'weighted_random',
    },
    {
      id: 'reliability',
      title: 'Reliability guardrails',
      detail: 'Bias toward healthy, low-latency, and policy-compliant providers when production confidence matters more than raw spread.',
      strategy: 'slo_aware',
      active: preferences.strategy === 'slo_aware',
    },
    {
      id: 'regional',
      title: 'Regional preference',
      detail: 'Prefer providers that match the target region so routing stays closer to user locality and compliance boundaries.',
      strategy: 'geo_affinity',
      active: preferences.strategy === 'geo_affinity',
    },
  ];
}

function buildGuardrails(
  preferences: PortalRoutingPreferences,
  preview: PortalRoutingDecision,
): RoutingGuardrailItem[] {
  return [
    {
      id: 'provider-default',
      label: 'Default provider',
      value: preferences.default_provider_id ?? 'Auto',
      detail: 'A default provider acts as the stable fallback when multiple candidates remain eligible.',
    },
    {
      id: 'cost',
      label: 'Max cost',
      value: preferences.max_cost === null || preferences.max_cost === undefined
        ? 'Open'
        : `$${preferences.max_cost.toFixed(2)}`,
      detail: 'Keep a cost ceiling visible so route posture reflects commercial intent, not only technical possibility.',
    },
    {
      id: 'latency',
      label: 'Max latency',
      value: preferences.max_latency_ms === null || preferences.max_latency_ms === undefined
        ? 'Open'
        : `${preferences.max_latency_ms}ms`,
      detail: 'Latency guardrails let the workspace make reliability posture explicit before traffic starts flowing.',
    },
    {
      id: 'region',
      label: 'Preferred region',
      value: preview.requested_region ?? preferences.preferred_region ?? 'Auto',
      detail: 'The active route preview should always show the region signal that influenced provider selection.',
    },
  ];
}

function buildEvidence(
  logs: PortalRoutingDecisionLog[],
): RoutingEvidenceItem[] {
  return logs.slice(0, 4).map((log) => ({
    id: log.decision_id,
    title: `${log.route_key} -> ${log.selected_provider_id}`,
    detail: `${log.decision_source} used ${log.strategy}${log.requested_region ? ` in ${log.requested_region}` : ''}.`,
    timestamp_label: formatDateTime(log.created_at_ms),
  }));
}

export function buildPortalRoutingViewModel(
  summary: PortalRoutingSummary,
  logs: PortalRoutingDecisionLog[],
  preview?: PortalRoutingDecision | null,
): PortalRoutingPageViewModel {
  const activePreview = preview ?? summary.preview;

  return {
    summary,
    preview: activePreview,
    preset_cards: buildPresetCards(summary.preferences),
    guardrails: buildGuardrails(summary.preferences, activePreview),
    evidence: buildEvidence(logs),
    provider_options: summary.provider_options,
    logs,
  };
}
