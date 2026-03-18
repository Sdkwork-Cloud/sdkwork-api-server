import type { AdminRouteDefinition } from 'sdkwork-router-admin-types';

export const adminRoutes: AdminRouteDefinition[] = [
  {
    key: 'overview',
    label: 'Overview',
    eyebrow: 'Control',
    detail: 'Global health, alerts, and operator shortcuts',
    group: 'Control Plane',
  },
  {
    key: 'users',
    label: 'Users',
    eyebrow: 'Identity',
    detail: 'Operator and portal user management',
    group: 'Workspace Ops',
  },
  {
    key: 'tenants',
    label: 'Tenants',
    eyebrow: 'Workspace',
    detail: 'Tenants, projects, and gateway keys',
    group: 'Workspace Ops',
  },
  {
    key: 'coupons',
    label: 'Coupons',
    eyebrow: 'Growth',
    detail: 'Campaign and discount code operations',
    group: 'Workspace Ops',
  },
  {
    key: 'catalog',
    label: 'Catalog',
    eyebrow: 'Mesh',
    detail: 'Channels, providers, and model exposure',
    group: 'Routing Mesh',
  },
  {
    key: 'traffic',
    label: 'Traffic',
    eyebrow: 'Audit',
    detail: 'Usage, billing, and request-log visibility',
    group: 'Routing Mesh',
  },
  {
    key: 'operations',
    label: 'Operations',
    eyebrow: 'Runtime',
    detail: 'Health snapshots, reloads, and runtime posture',
    group: 'System',
  },
  {
    key: 'settings',
    label: 'Settings',
    eyebrow: 'Preferences',
    detail: 'Theme mode, theme color, and sidebar preferences',
    group: 'System',
  },
];
