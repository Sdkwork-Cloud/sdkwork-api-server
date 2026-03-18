import type {
  PortalAnonymousRouteKey,
  PortalRouteDefinition,
  PortalRouteKey,
} from 'sdkwork-router-portal-types';

import { portalRoutes } from '../../routes';
import { PORTAL_ROUTE_PATHS } from './routePaths';

export interface PortalShellRouteDefinition extends PortalRouteDefinition {
  path: string;
}

export const portalRouteManifest: PortalShellRouteDefinition[] = portalRoutes.map((route) => ({
  ...route,
  path: PORTAL_ROUTE_PATHS[route.key],
}));

export function resolvePortalPath(routeKey: PortalAnonymousRouteKey | PortalRouteKey): string {
  return PORTAL_ROUTE_PATHS[routeKey];
}
