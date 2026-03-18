import type {
  PortalRouteKey,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingProviderOption,
  PortalRoutingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalRoutingPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface RoutingPresetCard {
  id: string;
  title: string;
  detail: string;
  strategy: PortalRoutingPreferences['strategy'];
  active: boolean;
}

export interface RoutingGuardrailItem {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface RoutingEvidenceItem {
  id: string;
  title: string;
  detail: string;
  timestamp_label: string;
}

export interface PortalRoutingPageViewModel {
  summary: PortalRoutingSummary;
  preview: PortalRoutingDecision;
  preset_cards: RoutingPresetCard[];
  guardrails: RoutingGuardrailItem[];
  evidence: RoutingEvidenceItem[];
  provider_options: PortalRoutingProviderOption[];
  logs: PortalRoutingDecisionLog[];
}
