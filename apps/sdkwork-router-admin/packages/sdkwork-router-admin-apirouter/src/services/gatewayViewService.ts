import type {
  AdminWorkspaceSnapshot,
  ChannelRecord,
  CredentialRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

export interface GatewayModelCatalogOption {
  value: string;
  channel_id: string;
  channel_name: string;
  model_id: string;
  model_name: string;
  label: string;
}

export interface GatewayRouteInventoryRow {
  provider: ProxyProviderRecord;
  channels: ChannelRecord[];
  credentials: CredentialRecord[];
  model_count: number;
  price_count: number;
  primary_channel_name: string;
  primary_channel_id: string;
  healthy: boolean;
  health_status: string;
}

function sortUniqueChannels(channels: ChannelRecord[]): ChannelRecord[] {
  const seen = new Set<string>();
  const result: ChannelRecord[] = [];

  for (const channel of channels) {
    if (seen.has(channel.id)) {
      continue;
    }

    seen.add(channel.id);
    result.push(channel);
  }

  return result.sort((left, right) => left.name.localeCompare(right.name));
}

function providerChannelIds(provider: ProxyProviderRecord): string[] {
  const channelIds = new Set<string>([provider.channel_id]);
  for (const binding of provider.channel_bindings) {
    channelIds.add(binding.channel_id);
  }

  return Array.from(channelIds);
}

export function buildGatewayModelCatalog(
  snapshot: AdminWorkspaceSnapshot,
): GatewayModelCatalogOption[] {
  const channelNameById = new Map(snapshot.channels.map((channel) => [channel.id, channel.name]));

  return snapshot.channelModels
    .map((record) => {
      const channelName = channelNameById.get(record.channel_id) ?? record.channel_id;
      return {
        value: `${record.channel_id}::${record.model_id}`,
        channel_id: record.channel_id,
        channel_name: channelName,
        model_id: record.model_id,
        model_name: record.model_display_name,
        label: `${channelName} / ${record.model_display_name} (${record.model_id})`,
      };
    })
    .sort((left, right) => left.label.localeCompare(right.label));
}

export function buildGatewayRouteInventory(
  snapshot: AdminWorkspaceSnapshot,
): GatewayRouteInventoryRow[] {
  const channelById = new Map(snapshot.channels.map((channel) => [channel.id, channel]));
  const healthByProviderId = new Map(
    snapshot.providerHealth.map((health) => [health.provider_id, health]),
  );

  return snapshot.providers
    .map((provider) => {
      const channelIds = providerChannelIds(provider);
      const channels = sortUniqueChannels(
        channelIds.map((channelId) => channelById.get(channelId)).filter(Boolean) as ChannelRecord[],
      );
      const credentials = snapshot.credentials.filter(
        (credential) => credential.provider_id === provider.id,
      );
      const modelCount = snapshot.channelModels.filter((model) =>
        channelIds.includes(model.channel_id),
      ).length;
      const priceCount = snapshot.modelPrices.filter(
        (price) => price.proxy_provider_id === provider.id,
      ).length;
      const health = healthByProviderId.get(provider.id);
      const primaryChannel = channelById.get(provider.channel_id);

      return {
        provider,
        channels,
        credentials,
        model_count: modelCount,
        price_count: priceCount,
        primary_channel_name: primaryChannel?.name ?? provider.channel_id,
        primary_channel_id: provider.channel_id,
        healthy: health?.healthy ?? true,
        health_status: health?.status ?? 'unknown',
      };
    })
    .sort((left, right) => left.provider.display_name.localeCompare(right.provider.display_name));
}

