import type { PortalRouteDefinition } from 'sdkwork-router-portal-types';

export const portalRoutes: PortalRouteDefinition[] = [
  {
    key: 'gateway',
    group: 'operations',
    labelKey: 'Gateway',
    eyebrowKey: 'Platform',
    detailKey: 'Compatibility, deployment modes, and launch posture',
    sidebarVisible: false,
  },
  {
    key: 'dashboard',
    group: 'operations',
    labelKey: 'Dashboard',
    eyebrowKey: 'Overview',
    detailKey: 'Traffic, routing, access, and spend at a glance',
  },
  {
    key: 'routing',
    group: 'operations',
    labelKey: 'Routing',
    eyebrowKey: 'Control',
    detailKey: 'Default strategy, failover posture, and route evidence',
    sidebarVisible: false,
  },
  {
    key: 'api-keys',
    group: 'access',
    labelKey: 'API Keys',
    eyebrowKey: 'Credentials',
    detailKey: 'Issue, inspect, and rotate project keys',
  },
  {
    key: 'usage',
    group: 'operations',
    labelKey: 'Usage',
    eyebrowKey: 'Telemetry',
    detailKey: 'Requests, models, providers, and spend telemetry',
  },
  {
    key: 'user',
    group: 'access',
    labelKey: 'User',
    eyebrowKey: 'Identity',
    detailKey: 'Profile, security, and personal access settings',
    sidebarVisible: false,
  },
  {
    key: 'credits',
    group: 'revenue',
    labelKey: 'Redeem',
    eyebrowKey: 'Growth',
    detailKey: 'Coupons, invites, and activation rewards',
  },
  {
    key: 'billing',
    group: 'revenue',
    labelKey: 'Billing',
    eyebrowKey: 'Commerce',
    detailKey: 'Plans, recharge packs, and billing recovery',
    sidebarVisible: false,
  },
  {
    key: 'recharge',
    group: 'revenue',
    labelKey: 'Recharge',
    eyebrowKey: 'Balance',
    detailKey: 'Top up balance with server-managed recharge options',
  },
  {
    key: 'settlements',
    group: 'revenue',
    labelKey: 'Settlements',
    eyebrowKey: 'Evidence',
    detailKey: 'Canonical holds, request settlements, and pricing posture',
  },
  {
    key: 'account',
    group: 'revenue',
    labelKey: 'Account',
    eyebrowKey: 'Financial',
    detailKey: 'Cash balance, history visibility, and payment posture',
  },
];

export const portalSidebarRoutes: PortalRouteDefinition[] = portalRoutes.filter(
  (route) => route.sidebarVisible !== false,
);
