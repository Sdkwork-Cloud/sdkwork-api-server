import type { PortalRouteDefinition } from 'sdkwork-router-portal-types';

export const portalRoutes: PortalRouteDefinition[] = [
  {
    key: 'dashboard',
    label: 'Dashboard',
    eyebrow: 'Overview',
    detail: 'Traffic, routing, access, and spend at a glance',
  },
  {
    key: 'routing',
    label: 'Routing',
    eyebrow: 'Control',
    detail: 'Default strategy, failover posture, and route evidence',
  },
  {
    key: 'api-keys',
    label: 'API Keys',
    eyebrow: 'Credentials',
    detail: 'Issue, inspect, and rotate project keys',
  },
  {
    key: 'usage',
    label: 'Usage',
    eyebrow: 'Telemetry',
    detail: 'Requests, models, providers, and spend telemetry',
  },
  {
    key: 'user',
    label: 'User',
    eyebrow: 'Identity',
    detail: 'Profile, security, and personal access settings',
  },
  {
    key: 'credits',
    label: 'Credits',
    eyebrow: 'Points',
    detail: 'Quota posture, redemption, and remaining units',
  },
  {
    key: 'billing',
    label: 'Billing',
    eyebrow: 'Commerce',
    detail: 'Plans, recharge packs, and billing recovery',
  },
  {
    key: 'account',
    label: 'Account',
    eyebrow: 'Financial',
    detail: 'Cash balance, ledger visibility, and payment posture',
  },
];
