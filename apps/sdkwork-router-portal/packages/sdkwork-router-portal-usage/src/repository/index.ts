import {
  listPortalApiKeys,
  listPortalUsageRecords,
} from 'sdkwork-router-portal-portal-api';
import type { GatewayApiKeyRecord, UsageRecord } from 'sdkwork-router-portal-types';

export async function loadUsageWorkbenchData(): Promise<{
  apiKeys: GatewayApiKeyRecord[];
  records: UsageRecord[];
}> {
  const [apiKeys, records] = await Promise.all([
    listPortalApiKeys(),
    listPortalUsageRecords(),
  ]);

  return { apiKeys, records };
}
