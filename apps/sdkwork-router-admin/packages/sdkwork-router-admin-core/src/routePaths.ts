import type { AdminRouteKey } from 'sdkwork-router-admin-types';

export const ADMIN_ROUTE_PATHS = {
  ROOT: '/',
  AUTH: '/auth',
  LOGIN: '/login',
  REGISTER: '/register',
  FORGOT_PASSWORD: '/forgot-password',
  OVERVIEW: '/overview',
  USERS: '/users',
  TENANTS: '/tenants',
  COUPONS: '/coupons',
  CATALOG: '/catalog',
  TRAFFIC: '/traffic',
  OPERATIONS: '/operations',
  SETTINGS: '/settings',
} as const;

const AUTH_ROUTE_PATHS = [
  ADMIN_ROUTE_PATHS.AUTH,
  ADMIN_ROUTE_PATHS.LOGIN,
  ADMIN_ROUTE_PATHS.REGISTER,
  ADMIN_ROUTE_PATHS.FORGOT_PASSWORD,
] as const;

function normalizeAdminPathname(pathname: string): string {
  return pathname.endsWith('/') && pathname !== '/' ? pathname.slice(0, -1) : pathname;
}

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
  const normalized = normalizeAdminPathname(pathname);
  const match = Object.entries(adminRoutePathByKey).find(([, path]) => path === normalized);
  return (match?.[0] as AdminRouteKey | undefined) ?? null;
}

export function isAdminAuthPath(pathname: string): boolean {
  return AUTH_ROUTE_PATHS.includes(
    normalizeAdminPathname(pathname) as (typeof AUTH_ROUTE_PATHS)[number],
  );
}
