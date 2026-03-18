import type { AdminRouteKey } from 'sdkwork-router-admin-types';

export const ADMIN_ROUTE_PATHS = {
  LOGIN: '/login',
  OVERVIEW: '/overview',
  USERS: '/users',
  TENANTS: '/tenants',
  COUPONS: '/coupons',
  CATALOG: '/catalog',
  TRAFFIC: '/traffic',
  OPERATIONS: '/operations',
  SETTINGS: '/settings',
} as const;

export const adminRoutePathByKey: Record<AdminRouteKey, string> = {
  overview: ADMIN_ROUTE_PATHS.OVERVIEW,
  users: ADMIN_ROUTE_PATHS.USERS,
  tenants: ADMIN_ROUTE_PATHS.TENANTS,
  coupons: ADMIN_ROUTE_PATHS.COUPONS,
  catalog: ADMIN_ROUTE_PATHS.CATALOG,
  traffic: ADMIN_ROUTE_PATHS.TRAFFIC,
  operations: ADMIN_ROUTE_PATHS.OPERATIONS,
  settings: ADMIN_ROUTE_PATHS.SETTINGS,
};

export function adminRouteKeyFromPathname(pathname: string): AdminRouteKey | null {
  const normalized = pathname.endsWith('/') && pathname !== '/' ? pathname.slice(0, -1) : pathname;
  const match = Object.entries(adminRoutePathByKey).find(([, path]) => path === normalized);
  return (match?.[0] as AdminRouteKey | undefined) ?? null;
}
