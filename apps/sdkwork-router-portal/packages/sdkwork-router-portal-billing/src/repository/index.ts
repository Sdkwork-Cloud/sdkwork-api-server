import {
  getPortalBillingSummary,
  listPortalUsageRecords,
} from 'sdkwork-router-portal-portal-api';

import type { BillingPageData } from '../types';

export async function loadBillingPageData(): Promise<BillingPageData> {
  const [summary, usage_records] = await Promise.all([
    getPortalBillingSummary(),
    listPortalUsageRecords(),
  ]);

  return {
    summary,
    usage_records,
  };
}
