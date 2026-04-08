import type {
  PortalAnonymousRouteKey,
  PortalRouteKey,
  PortalRouteManifestEntry,
  PortalRouteModuleId,
  PortalProductModuleManifest,
  PortalTopLevelRouteKey,
} from 'sdkwork-router-portal-types';

import { portalRoutes } from '../../routes';
import { PORTAL_ROUTE_PATHS } from './routePaths';

export const portalProductModules: PortalProductModuleManifest[] = [
  {
    moduleId: 'sdkwork-router-portal-gateway',
    pluginId: 'sdkwork-router-portal-gateway',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-gateway',
    displayName: 'Gateway',
    routeKeys: ['gateway'],
    capabilityTags: ['gateway-posture', 'deployment-modes'],
    requiredPermissions: ['portal.gateway.read'],
    navigation: {
      group: 'operations',
      order: 10,
      sidebar: false,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'none',
      chunkGroup: 'gateway',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-dashboard',
    pluginId: 'sdkwork-router-portal-dashboard',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-dashboard',
    displayName: 'Dashboard',
    routeKeys: ['dashboard'],
    capabilityTags: ['workspace-dashboard', 'traffic-summary', 'spend-summary'],
    requiredPermissions: ['portal.dashboard.read'],
    navigation: {
      group: 'operations',
      order: 20,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'dashboard',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-routing',
    pluginId: 'sdkwork-router-portal-routing',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-routing',
    displayName: 'Routing',
    routeKeys: ['routing'],
    capabilityTags: ['routing-workbench', 'routing-profiles', 'routing-snapshots'],
    requiredPermissions: ['portal.routing.read', 'portal.routing.write'],
    navigation: {
      group: 'operations',
      order: 30,
      sidebar: false,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'none',
      chunkGroup: 'routing',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-api-keys',
    pluginId: 'sdkwork-router-portal-api-keys',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-api-keys',
    displayName: 'API Keys',
    routeKeys: ['api-keys'],
    capabilityTags: ['api-key-governance', 'api-key-groups', 'client-quick-setup'],
    requiredPermissions: ['portal.api_keys.read', 'portal.api_keys.write'],
    navigation: {
      group: 'access',
      order: 40,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'api-keys',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-usage',
    pluginId: 'sdkwork-router-portal-usage',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-usage',
    displayName: 'Usage',
    routeKeys: ['usage'],
    capabilityTags: ['usage-analytics', 'provider-analytics', 'billing-evidence'],
    requiredPermissions: ['portal.usage.read'],
    navigation: {
      group: 'operations',
      order: 50,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'usage',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-user',
    pluginId: 'sdkwork-router-portal-user',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-user',
    displayName: 'User',
    routeKeys: ['user'],
    capabilityTags: ['user-profile', 'security-settings'],
    requiredPermissions: ['portal.user.read', 'portal.user.write'],
    navigation: {
      group: 'access',
      order: 60,
      sidebar: false,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'none',
      chunkGroup: 'user',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-credits',
    pluginId: 'sdkwork-router-portal-credits',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-credits',
    displayName: 'Redeem',
    routeKeys: ['credits'],
    capabilityTags: ['growth-rewards', 'coupon-redeem'],
    requiredPermissions: ['portal.credits.read', 'portal.credits.write'],
    navigation: {
      group: 'revenue',
      order: 70,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'credits',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-recharge',
    pluginId: 'sdkwork-router-portal-recharge',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-recharge',
    displayName: 'Recharge',
    routeKeys: ['recharge'],
    capabilityTags: ['balance-topup', 'recharge-workflow'],
    requiredPermissions: ['portal.recharge.read', 'portal.recharge.write'],
    navigation: {
      group: 'revenue',
      order: 80,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'recharge',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-billing',
    pluginId: 'sdkwork-router-portal-billing',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-billing',
    displayName: 'Billing',
    routeKeys: ['billing'],
    capabilityTags: ['plan-management', 'billing-recovery', 'commerce-history'],
    requiredPermissions: ['portal.billing.read'],
    navigation: {
      group: 'revenue',
      order: 90,
      sidebar: false,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'none',
      chunkGroup: 'billing',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-settlements',
    pluginId: 'sdkwork-router-portal-settlements',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-settlements',
    displayName: 'Settlements',
    routeKeys: ['settlements'],
    capabilityTags: ['settlement-explorer', 'credit-holds', 'pricing-evidence'],
    requiredPermissions: ['portal.settlements.read'],
    navigation: {
      group: 'revenue',
      order: 100,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'settlements',
    },
  },
  {
    moduleId: 'sdkwork-router-portal-account',
    pluginId: 'sdkwork-router-portal-account',
    pluginKind: 'portal-module',
    packageName: 'sdkwork-router-portal-account',
    displayName: 'Account',
    routeKeys: ['account'],
    capabilityTags: ['account-balance', 'payment-posture', 'billing-history'],
    requiredPermissions: ['portal.account.read'],
    navigation: {
      group: 'revenue',
      order: 110,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'account',
    },
  },
];

const portalRouteModulePackages: Record<PortalRouteKey, PortalRouteModuleId> = {
  gateway: 'sdkwork-router-portal-gateway',
  dashboard: 'sdkwork-router-portal-dashboard',
  routing: 'sdkwork-router-portal-routing',
  'api-keys': 'sdkwork-router-portal-api-keys',
  usage: 'sdkwork-router-portal-usage',
  user: 'sdkwork-router-portal-user',
  credits: 'sdkwork-router-portal-credits',
  recharge: 'sdkwork-router-portal-recharge',
  billing: 'sdkwork-router-portal-billing',
  settlements: 'sdkwork-router-portal-settlements',
  account: 'sdkwork-router-portal-account',
};

const portalProductModuleById = Object.fromEntries(
  portalProductModules.map((productModule) => [productModule.moduleId, productModule]),
) as Record<PortalRouteModuleId, PortalProductModuleManifest>;

const portalProductModuleByRouteKey = portalProductModules.reduce(
  (accumulator, productModule) => {
    for (const routeKey of productModule.routeKeys) {
      accumulator[routeKey] = productModule;
    }

    return accumulator;
  },
  {} as Record<PortalRouteKey, PortalProductModuleManifest>,
);

const portalRouteModuleByKey = portalRouteModulePackages;

export const portalRouteManifest: PortalRouteManifestEntry[] = portalRoutes.map((route) => ({
  ...route,
  path: PORTAL_ROUTE_PATHS[route.key],
  moduleId: portalRouteModuleByKey[route.key],
  prefetchGroup: portalProductModuleByRouteKey[route.key].loading.chunkGroup,
  productModule: portalProductModuleByRouteKey[route.key],
}));

export function resolvePortalPath(
  routeKey: PortalAnonymousRouteKey | PortalRouteKey | PortalTopLevelRouteKey,
): string {
  return PORTAL_ROUTE_PATHS[routeKey];
}

export function resolvePortalProductModule(
  moduleId: PortalRouteModuleId,
): PortalProductModuleManifest {
  return portalProductModuleById[moduleId];
}
