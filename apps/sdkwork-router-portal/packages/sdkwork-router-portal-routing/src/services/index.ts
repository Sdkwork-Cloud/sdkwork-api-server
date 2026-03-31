import type {
  PortalRoutingAssessment,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingSummary,
  PortalRoutingStrategy,
} from 'sdkwork-router-portal-types';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';

import type {
  PortalRoutingPageViewModel,
  RoutingEvidenceItem,
  RoutingGuardrailItem,
  RoutingPresetCard,
} from '../types';

function formatRoutingDateTime(timestamp: number): string {
  if (!timestamp) {
    return translatePortalText('Pending');
  }

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp));
}

function normalizeRoutingAssessment(
  assessment: Partial<PortalRoutingAssessment> | null | undefined,
): PortalRoutingAssessment {
  return {
    provider_id: assessment?.provider_id ?? translatePortalText('Unknown provider'),
    available: assessment?.available ?? false,
    health: assessment?.health ?? 'unknown',
    policy_rank: assessment?.policy_rank ?? 0,
    weight: assessment?.weight ?? null,
    cost: assessment?.cost ?? null,
    latency_ms: assessment?.latency_ms ?? null,
    region: assessment?.region ?? null,
    region_match: assessment?.region_match ?? null,
    slo_eligible: assessment?.slo_eligible ?? null,
    slo_violations: Array.isArray(assessment?.slo_violations) ? assessment.slo_violations : [],
    reasons: Array.isArray(assessment?.reasons) ? assessment.reasons : [],
  };
}

function normalizeRoutingDecision(
  decision: Partial<PortalRoutingDecision> | null | undefined,
): PortalRoutingDecision {
  return {
    selected_provider_id: decision?.selected_provider_id ?? translatePortalText('Unavailable'),
    candidate_ids: Array.isArray(decision?.candidate_ids) ? decision.candidate_ids : [],
    matched_policy_id: decision?.matched_policy_id ?? null,
    strategy: decision?.strategy ?? null,
    selection_seed: decision?.selection_seed ?? null,
    selection_reason: decision?.selection_reason ?? null,
    requested_region: decision?.requested_region ?? null,
    slo_applied: decision?.slo_applied ?? false,
    slo_degraded: decision?.slo_degraded ?? false,
    assessments: Array.isArray(decision?.assessments)
      ? decision.assessments.map((assessment) => normalizeRoutingAssessment(assessment))
      : [],
  };
}

function normalizeRoutingDecisionLog(
  log: Partial<PortalRoutingDecisionLog>,
): PortalRoutingDecisionLog {
  return {
    decision_id: log.decision_id ?? 'unknown-decision',
    decision_source: log.decision_source ?? 'unknown',
    tenant_id: log.tenant_id ?? null,
    project_id: log.project_id ?? null,
    capability: log.capability ?? 'unknown',
    route_key: log.route_key ?? 'unknown',
    selected_provider_id: log.selected_provider_id ?? translatePortalText('Unavailable'),
    matched_policy_id: log.matched_policy_id ?? null,
    strategy: log.strategy ?? 'unknown',
    selection_seed: log.selection_seed ?? null,
    selection_reason: log.selection_reason ?? null,
    requested_region: log.requested_region ?? null,
    slo_applied: log.slo_applied ?? false,
    slo_degraded: log.slo_degraded ?? false,
    created_at_ms: log.created_at_ms ?? 0,
    assessments: Array.isArray(log.assessments)
      ? log.assessments.map((assessment) => normalizeRoutingAssessment(assessment))
      : [],
  };
}

export function buildRoutingStrategyLabel(
  strategy?: PortalRoutingStrategy | string | null,
): string {
  switch (strategy) {
    case 'deterministic_priority':
      return translatePortalText('Predictable order');
    case 'weighted_random':
      return translatePortalText('Traffic distribution');
    case 'slo_aware':
      return translatePortalText('Reliability guardrails');
    case 'geo_affinity':
      return translatePortalText('Regional preference');
    case 'static_fallback':
      return translatePortalText('Platform fallback');
    default:
      return translatePortalText('Adaptive routing');
  }
}

function buildPresetCards(
  preferences: PortalRoutingPreferences,
): RoutingPresetCard[] {
  return [
    {
      id: 'predictable',
      title: translatePortalText('Predictable order'),
      detail: translatePortalText(
        'The first healthy available provider in your ordered list wins, and the next provider becomes the deterministic fallback.',
      ),
      strategy: 'deterministic_priority',
      active: preferences.strategy === 'deterministic_priority',
    },
    {
      id: 'distribution',
      title: translatePortalText('Traffic distribution'),
      detail: translatePortalText(
        'Spread traffic across eligible providers when you want to balance exposure instead of pinning every request to one path.',
      ),
      strategy: 'weighted_random',
      active: preferences.strategy === 'weighted_random',
    },
    {
      id: 'reliability',
      title: translatePortalText('Reliability guardrails'),
      detail: translatePortalText(
        'Bias toward healthy, low-latency, and policy-compliant providers when production confidence matters more than raw spread.',
      ),
      strategy: 'slo_aware',
      active: preferences.strategy === 'slo_aware',
    },
    {
      id: 'regional',
      title: translatePortalText('Regional preference'),
      detail: translatePortalText(
        'Prefer providers that match the target region so routing stays closer to user locality and compliance boundaries.',
      ),
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
      label: translatePortalText('Default provider'),
      value: preferences.default_provider_id ?? 'Auto',
      detail: translatePortalText(
        'A default provider acts as the stable fallback when multiple candidates remain eligible.',
      ),
    },
    {
      id: 'cost',
      label: translatePortalText('Max cost'),
      value: preferences.max_cost === null || preferences.max_cost === undefined
        ? 'Open'
        : `$${preferences.max_cost.toFixed(2)}`,
      detail: translatePortalText(
        'Keep a cost ceiling visible so route posture reflects commercial intent, not only technical possibility.',
      ),
    },
    {
      id: 'latency',
      label: translatePortalText('Max latency'),
      value: preferences.max_latency_ms === null || preferences.max_latency_ms === undefined
        ? 'Open'
        : `${preferences.max_latency_ms}ms`,
      detail: translatePortalText(
        'Latency guardrails let the workspace make reliability posture explicit before traffic starts flowing.',
      ),
    },
    {
      id: 'region',
      label: translatePortalText('Preferred region'),
      value: preview.requested_region ?? preferences.preferred_region ?? 'Auto',
      detail: translatePortalText(
        'The active route preview should always show the region signal that influenced provider selection.',
      ),
    },
  ];
}

function buildEvidence(
  logs: PortalRoutingDecisionLog[],
): RoutingEvidenceItem[] {
  return logs.slice(0, 4).map((log) => ({
    id: log.decision_id,
    title: `${log.route_key} -> ${log.selected_provider_id}`,
    detail: translatePortalText('{source} used {strategy}{regionSuffix}.', {
      source: log.decision_source,
      strategy: buildRoutingStrategyLabel(log.strategy),
      regionSuffix: log.requested_region
        ? translatePortalText(' in {region}', { region: log.requested_region })
        : '',
    }),
    timestamp_label: formatRoutingDateTime(log.created_at_ms),
  }));
}

export function buildPortalRoutingViewModel(
  summary: PortalRoutingSummary,
  logs: PortalRoutingDecisionLog[],
  preview?: PortalRoutingDecision | null,
): PortalRoutingPageViewModel {
  const normalizedLogs = Array.isArray(logs)
    ? logs.map((log) => normalizeRoutingDecisionLog(log))
    : [];
  const normalizedSummaryPreview = normalizeRoutingDecision(summary.preview);
  const activePreview = normalizeRoutingDecision(preview ?? normalizedSummaryPreview);

  return {
    summary,
    preview: activePreview,
    preset_cards: buildPresetCards(summary.preferences),
    guardrails: buildGuardrails(summary.preferences, activePreview),
    evidence: buildEvidence(normalizedLogs),
    provider_options: summary.provider_options,
    logs: normalizedLogs,
  };
}
