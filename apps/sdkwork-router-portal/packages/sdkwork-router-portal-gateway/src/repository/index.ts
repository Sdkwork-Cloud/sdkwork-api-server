import {
  getPortalCommerceMembership,
  getDesktopRuntimeSnapshot,
  getPortalCommerceCatalog,
  getPortalDashboard,
  getPortalGatewayRateLimitSnapshot,
  getProductRuntimeHealthSnapshot,
  restartDesktopRuntime,
  resolveGatewayBaseUrl,
} from 'sdkwork-router-portal-portal-api';

import { buildGatewayCommandCenterSnapshot } from '../services';
import type { GatewayCommandCenterSnapshot } from '../types';

export async function loadGatewayCommandCenterSnapshot(): Promise<GatewayCommandCenterSnapshot> {
  const [
    dashboard,
    gatewayBaseUrl,
    desktopRuntime,
    runtimeHealth,
    commerceCatalog,
    membership,
    rateLimitSnapshot,
  ] = await Promise.all([
    getPortalDashboard(),
    resolveGatewayBaseUrl(),
    getDesktopRuntimeSnapshot(),
    getProductRuntimeHealthSnapshot(),
    getPortalCommerceCatalog(),
    getPortalCommerceMembership(),
    getPortalGatewayRateLimitSnapshot(),
  ]);

  return buildGatewayCommandCenterSnapshot({
    dashboard,
    desktopRuntime,
    runtimeHealth,
    commerceCatalog,
    membership,
    gatewayBaseUrl,
    rateLimitSnapshot,
  });
}

export async function restartGatewayCommandCenterDesktopRuntime(): Promise<GatewayCommandCenterSnapshot> {
  await restartDesktopRuntime();
  return loadGatewayCommandCenterSnapshot();
}
