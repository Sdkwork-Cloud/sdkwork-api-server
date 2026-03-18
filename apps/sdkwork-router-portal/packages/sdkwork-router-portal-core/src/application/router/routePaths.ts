import type { PortalAnonymousRouteKey, PortalRouteKey } from 'sdkwork-router-portal-types';

export const PORTAL_ROUTE_PATHS: Record<PortalAnonymousRouteKey | PortalRouteKey, string> = {
  login: '/login',
  register: '/register',
  dashboard: '/dashboard',
  routing: '/routing',
  'api-keys': '/api-keys',
  usage: '/usage',
  user: '/user',
  credits: '/credits',
  billing: '/billing',
  account: '/account',
};

export function toRouteElementPath(pathname: string): string {
  return pathname.replace(/^\//, '');
}
