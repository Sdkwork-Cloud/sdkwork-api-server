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
  API_ROUTER_ROOT: '/api-router',
  API_ROUTER_API_KEYS: '/api-router/api-keys',
  API_ROUTER_RATE_LIMITS: '/api-router/rate-limits',
  API_ROUTER_ROUTE_CONFIG: '/api-router/route-config',
  API_ROUTER_MODEL_MAPPING: '/api-router/model-mapping',
  API_ROUTER_USAGE_RECORDS: '/api-router/usage-records',
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
  'api-keys': ADMIN_ROUTE_PATHS.API_ROUTER_API_KEYS,
  'rate-limits': ADMIN_ROUTE_PATHS.API_ROUTER_RATE_LIMITS,
  'route-config': ADMIN_ROUTE_PATHS.API_ROUTER_ROUTE_CONFIG,
  'model-mapping': ADMIN_ROUTE_PATHS.API_ROUTER_MODEL_MAPPING,
  'usage-records': ADMIN_ROUTE_PATHS.API_ROUTER_USAGE_RECORDS,
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
