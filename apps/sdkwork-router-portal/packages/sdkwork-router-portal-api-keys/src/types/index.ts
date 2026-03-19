import type { CreatedGatewayApiKey, GatewayApiKeyRecord, PortalRouteKey } from 'sdkwork-router-portal-types';

export interface PortalApiKeysPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export type PortalApiKeyCreateMode = 'system-generated' | 'custom';

export interface ApiKeyEnvironmentSummary {
  environment: string;
  total: number;
  active: number;
}

export interface ApiKeyEnvironmentStrategyItem {
  environment: string;
  status: string;
  detail: string;
  recommended: boolean;
}

export interface ApiKeyRotationStep {
  id: string;
  title: string;
  detail: string;
}

export interface ApiKeyGuardrail {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface PortalApiKeyEnvironmentOption {
  value: string;
  label: string;
  detail: string;
}

export interface PortalApiKeyFilterState {
  searchQuery: string;
  environment: string;
}

export interface PortalApiKeyCreateFormState {
  label: string;
  keyMode: PortalApiKeyCreateMode;
  customKey: string;
  environment: string;
  customEnvironment: string;
  expiresAt: string;
  notes: string;
}

export interface PortalApiKeyUsagePreview {
  title: string;
  detail: string;
  curlExample: string | null;
  authorizationHeader: string | null;
}

export interface PortalApiKeysPageViewModel {
  keys: GatewayApiKeyRecord[];
  filtered_keys: GatewayApiKeyRecord[];
  environment_summaries: ApiKeyEnvironmentSummary[];
  environment_options: PortalApiKeyEnvironmentOption[];
  environment_strategy: ApiKeyEnvironmentStrategyItem[];
  rotation_checklist: ApiKeyRotationStep[];
  guardrails: ApiKeyGuardrail[];
  created_key: CreatedGatewayApiKey | null;
  quickstart_snippet: string | null;
}
