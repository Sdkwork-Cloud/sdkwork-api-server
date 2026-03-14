export type RuntimeMode = 'server' | 'embedded';

export interface ConsoleSection {
  id: string;
  title: string;
}

export interface TenantRecord {
  id: string;
  name: string;
}

export interface ProjectRecord {
  tenant_id: string;
  id: string;
  name: string;
}

export interface GatewayApiKeyRecord {
  tenant_id: string;
  project_id: string;
  environment: string;
  hashed_key: string;
  active: boolean;
}

export interface CreatedGatewayApiKey {
  plaintext: string;
  hashed: string;
  tenant_id: string;
  project_id: string;
  environment: string;
}

export interface ChannelRecord {
  id: string;
  name: string;
}

export interface ProxyProviderRecord {
  id: string;
  channel_id: string;
  adapter_kind: string;
  base_url: string;
  display_name: string;
}

export interface ModelCatalogRecord {
  external_name: string;
  provider_id: string;
}

export type RoutingCandidateHealth = 'healthy' | 'unhealthy' | 'unknown';

export interface RoutingCandidateAssessment {
  provider_id: string;
  available: boolean;
  health: RoutingCandidateHealth;
  policy_rank: number;
  weight?: number;
  cost?: number;
  latency_ms?: number;
  reasons: string[];
}

export interface RoutingSimulationResult {
  selected_provider_id: string;
  candidate_ids: string[];
  matched_policy_id?: string;
  strategy?: string;
  selection_seed?: number;
  selection_reason?: string;
  assessments: RoutingCandidateAssessment[];
}

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
}

export interface LedgerEntry {
  project_id: string;
  units: number;
  amount: number;
}
