import {
  emptyProviderDraft as createEmptyProviderDraft,
  providerDraftFromRecord as createProviderDraftFromRecord,
  type ProviderDraft,
} from 'sdkwork-router-admin-core';
import type { ProviderCatalogRecord } from 'sdkwork-router-admin-types';

import type { GatewayRouteInventoryRow } from '../../services/gatewayViewService';

export type { ProviderDraft } from 'sdkwork-router-admin-core';

export type HealthFilter = 'all' | 'healthy' | 'degraded';

export function emptyProviderDraft(defaultChannelId: string): ProviderDraft {
  return createEmptyProviderDraft(defaultChannelId);
}

export function providerDraftFromRecord(
  provider: ProviderCatalogRecord,
): ProviderDraft {
  return createProviderDraftFromRecord(provider);
}

export function formatChannels(row: GatewayRouteInventoryRow): string {
  return (
    row.channels.map((channel) => channel.name).join(', ')
    || row.primary_channel_name
  );
}

export function statusVariant(row: GatewayRouteInventoryRow) {
  return row.healthy ? 'success' : 'danger';
}
