import type {
  PortalRouteKey,
  PortalGatewayRateLimitSnapshot,
  PortalRuntimeHealthSnapshot,
  PortalRuntimeServiceHealth,
} from 'sdkwork-router-portal-types';

export interface PortalGatewayPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface GatewayPostureCard {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface GatewayCompatibilityRow {
  id: string;
  tool: string;
  protocol: string;
  routeFamily: string;
  truth: string;
  outcome: string;
}

export interface GatewayModeCard {
  id: string;
  title: string;
  command: string;
  summary: string;
  notes: string[];
}

export interface GatewayTopologyPlaybook {
  id: string;
  title: string;
  command: string;
  topology: string;
  detail: string;
}

export interface GatewayReadinessAction {
  id: string;
  title: string;
  detail: string;
  cta: string;
  route: PortalRouteKey;
  tone?: 'primary' | 'secondary' | 'ghost';
}

export interface GatewayLaunchReadinessSummary {
  score: number;
  status: 'ready' | 'watch' | 'blocked';
  headline: string;
  detail: string;
  blockersHeading: string;
  blockers: string[];
  watchpointsHeading: string;
  watchpoints: string[];
}

export interface GatewayRuntimeControl {
  id: string;
  title: string;
  detail: string;
  cta: string;
  action: 'restart-desktop-runtime';
  enabled: boolean;
  tone?: 'primary' | 'secondary' | 'ghost';
}

export interface GatewayVerificationSnippet {
  id: string;
  title: string;
  routeFamily: string;
  command: string;
}

export interface GatewayServiceHealthCheck extends PortalRuntimeServiceHealth {}

export interface GatewayCommandCenterSnapshot {
  gatewayBaseUrl: string;
  postureCards: GatewayPostureCard[];
  launchReadiness: GatewayLaunchReadinessSummary;
  rateLimitCards: GatewayPostureCard[];
  rateLimitSnapshot: PortalGatewayRateLimitSnapshot;
  runtimeCards: GatewayPostureCard[];
  runtimeHealth: PortalRuntimeHealthSnapshot;
  serviceHealthChecks: GatewayServiceHealthCheck[];
  runtimeControls: GatewayRuntimeControl[];
  compatibilityRows: GatewayCompatibilityRow[];
  modeCards: GatewayModeCard[];
  topologyPlaybooks: GatewayTopologyPlaybook[];
  verificationSnippets: GatewayVerificationSnippet[];
  commerceCatalogCards: GatewayPostureCard[];
  readinessActions: GatewayReadinessAction[];
}
