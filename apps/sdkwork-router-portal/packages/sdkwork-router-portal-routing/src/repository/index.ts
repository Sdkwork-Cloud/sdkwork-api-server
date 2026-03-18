import {
  getPortalRoutingSummary,
  listPortalRoutingDecisionLogs,
  previewPortalRouting,
  savePortalRoutingPreferences,
} from 'sdkwork-router-portal-portal-api';
import type {
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingSummary,
} from 'sdkwork-router-portal-types';

export function loadPortalRoutingSummary(): Promise<PortalRoutingSummary> {
  return getPortalRoutingSummary();
}

export function loadPortalRoutingDecisionLogs(): Promise<PortalRoutingDecisionLog[]> {
  return listPortalRoutingDecisionLogs();
}

export function updatePortalRoutingPreferences(input: {
  preset_id: string;
  strategy: PortalRoutingPreferences['strategy'];
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
}): Promise<PortalRoutingPreferences> {
  return savePortalRoutingPreferences(input);
}

export function runPortalRoutingPreview(input: {
  capability: string;
  model: string;
  requested_region?: string | null;
  selection_seed?: number | null;
}): Promise<PortalRoutingDecision> {
  return previewPortalRouting(input);
}
