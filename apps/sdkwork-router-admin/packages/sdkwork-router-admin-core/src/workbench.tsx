import {
  createContext,
  startTransition,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react';

import {
  clearAdminSessionToken,
  getBillingEventSummary,
  getAdminMe,
  getBillingSummary,
  getUsageSummary,
  listApiKeys,
  listApiKeyGroups,
  listBillingEvents,
  listCommercePaymentEvents,
  listChannelModels,
  listChannels,
  listCommercialAccountHolds,
  listCommercialAccountLedger,
  listCommercialAccounts,
  listCommercialPricingPlans,
  listCommercialPricingRates,
  listCommercialRequestSettlements,
  listCompiledRoutingSnapshots,
  listCredentials,
  listMarketingCampaignBudgets,
  listMarketingCampaigns,
  listMarketingCouponCodes,
  listMarketingCouponRedemptions,
  listMarketingCouponReservations,
  listMarketingCouponRollbacks,
  listMarketingCouponTemplates,
  listModelPrices,
  listModels,
  listOperatorUsers,
  listRecentCommerceOrders,
  listPortalUsers,
  listProviderModels,
  listProjects,
  listProviderHealthSnapshots,
  listProviders,
  listRateLimitPolicies,
  listRateLimitWindows,
  listRoutingProfiles,
  listRoutingDecisionLogs,
  listRuntimeStatuses,
  listTenants,
  listUsageRecords,
  loginAdminUser,
  persistAdminSessionToken,
  readAdminSessionToken,
} from 'sdkwork-router-admin-admin-api';
import type {
  AdminSessionUser,
  AdminWorkspaceSnapshot,
  CommercialAccountSummary,
  CommercialRequestSettlementRecord,
} from 'sdkwork-router-admin-types';
import {
  createWorkbenchActions,
  type WorkbenchActions,
} from './workbenchActions';
import { buildSnapshot, emptySnapshot } from './workbenchSnapshot';

interface AdminWorkbenchContextValue extends WorkbenchActions {
  authResolved: boolean;
  sessionUser: AdminSessionUser | null;
  snapshot: AdminWorkspaceSnapshot;
  status: string;
  loading: boolean;
  refreshWorkspace: (explicitSessionUser?: AdminSessionUser | null) => Promise<void>;
  handleLogin: (input: { email: string; password: string }) => Promise<void>;
  handleLogout: () => void;
}

const AdminWorkbenchContext = createContext<AdminWorkbenchContextValue | null>(null);

function selectCommercialLedgerAccountIds(
  commercialAccounts: CommercialAccountSummary[],
  commercialRequestSettlements: CommercialRequestSettlementRecord[],
  maxAccounts = 8,
): number[] {
  const selected = new Set<number>();

  const recentSettlementAccountIds = [...commercialRequestSettlements]
    .sort((left, right) =>
      right.updated_at_ms - left.updated_at_ms
      || right.request_settlement_id - left.request_settlement_id,
    )
    .map((settlement) => settlement.account_id);

  for (const accountId of recentSettlementAccountIds) {
    selected.add(accountId);
    if (selected.size >= maxAccounts) {
      return [...selected];
    }
  }

  const priorityAccountIds = commercialAccounts
    .filter((account) =>
      account.held_balance > 0
      || account.account.status !== 'active',
    )
    .map((account) => account.account.account_id);

  for (const accountId of priorityAccountIds) {
    selected.add(accountId);
    if (selected.size >= maxAccounts) {
      return [...selected];
    }
  }

  for (const account of commercialAccounts) {
    selected.add(account.account.account_id);
    if (selected.size >= maxAccounts) {
      break;
    }
  }

  return [...selected];
}

function selectCommercePaymentAuditOrderIds(
  commerceOrders: AdminWorkspaceSnapshot['commerceOrders'],
  maxOrders = 10,
): string[] {
  const selected = new Set<string>();

  const priorityOrders = [...commerceOrders]
    .sort((left, right) =>
      right.updated_at_ms - left.updated_at_ms
      || right.created_at_ms - left.created_at_ms
      || right.order_id.localeCompare(left.order_id),
    )
    .filter((order) =>
      order.status === 'refunded'
      || order.status === 'pending_payment'
      || order.status === 'failed'
      || order.status === 'canceled',
    );

  for (const order of priorityOrders) {
    selected.add(order.order_id);
    if (selected.size >= maxOrders) {
      return [...selected];
    }
  }

  for (const order of commerceOrders) {
    selected.add(order.order_id);
    if (selected.size >= maxOrders) {
      break;
    }
  }

  return [...selected];
}

export function AdminWorkbenchProvider({ children }: { children: ReactNode }) {
  const [authResolved, setAuthResolved] = useState(false);
  const [sessionUser, setSessionUser] = useState<AdminSessionUser | null>(null);
  const [snapshot, setSnapshot] = useState<AdminWorkspaceSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Authenticate to open the super-admin workspace.');
  const [loading, setLoading] = useState(false);

  async function refreshWorkspace(explicitSessionUser = sessionUser) {
    if (!explicitSessionUser) {
      return;
    }

    setLoading(true);
    setStatus('Refreshing live admin data...');

    try {
      const [operatorDirectory, portalDirectory] = await Promise.all([
        listOperatorUsers(),
        listPortalUsers(),
      ]);

      const [
        tenants,
        projects,
        apiKeys,
        apiKeyGroups,
        routingProfiles,
        compiledRoutingSnapshots,
        rateLimitPolicies,
        rateLimitWindows,
        channels,
        providers,
        credentials,
        models,
        channelModels,
        providerModels,
        modelPrices,
        usageRecords,
        usageSummary,
        billingEvents,
        billingEventSummary,
        billingSummary,
        commerceOrders,
        couponTemplates,
        marketingCampaigns,
        campaignBudgets,
        couponCodes,
        couponReservations,
        couponRedemptions,
        couponRollbacks,
        commercialAccounts,
        commercialAccountHolds,
        commercialRequestSettlements,
        commercialPricingPlans,
        commercialPricingRates,
        routingLogs,
        providerHealth,
        runtimeStatuses,
      ] = await Promise.all([
        listTenants(),
        listProjects(),
        listApiKeys(),
        listApiKeyGroups(),
        listRoutingProfiles(),
        listCompiledRoutingSnapshots(),
        listRateLimitPolicies(),
        listRateLimitWindows(),
        listChannels(),
        listProviders(),
        listCredentials(),
        listModels(),
        listChannelModels(),
        listProviderModels(),
        listModelPrices(),
        listUsageRecords(),
        getUsageSummary(),
        listBillingEvents(),
        getBillingEventSummary(),
        getBillingSummary(),
        listRecentCommerceOrders(24),
        listMarketingCouponTemplates(),
        listMarketingCampaigns(),
        listMarketingCampaignBudgets(),
        listMarketingCouponCodes(),
        listMarketingCouponReservations(),
        listMarketingCouponRedemptions(),
        listMarketingCouponRollbacks(),
        listCommercialAccounts(),
        listCommercialAccountHolds(),
        listCommercialRequestSettlements(),
        listCommercialPricingPlans(),
        listCommercialPricingRates(),
        listRoutingDecisionLogs(),
        listProviderHealthSnapshots(),
        listRuntimeStatuses(),
      ]);

      const commercialLedgerAccountIds = selectCommercialLedgerAccountIds(
        commercialAccounts,
        commercialRequestSettlements,
      );
      const commercePaymentAuditOrderIds = selectCommercePaymentAuditOrderIds(
        commerceOrders,
      );
      const commercialAccountLedger = (
        await Promise.all(
          commercialLedgerAccountIds.map((accountId) => listCommercialAccountLedger(accountId)),
        )
      ).flat();
      const commercePaymentEvents = (
        await Promise.all(
          commercePaymentAuditOrderIds.map((orderId) => listCommercePaymentEvents(orderId)),
        )
      ).flat();

      const nextSnapshot = buildSnapshot(explicitSessionUser, {
        operatorDirectory,
        portalDirectory,
        tenants,
        projects,
        apiKeys,
        apiKeyGroups,
        routingProfiles,
        compiledRoutingSnapshots,
        rateLimitPolicies,
        rateLimitWindows,
        channels,
        providers,
        credentials,
        models,
        channelModels,
        providerModels,
        modelPrices,
        usageRecords,
        usageSummary,
        billingEvents,
        billingEventSummary,
        billingSummary,
        commerceOrders,
        commercePaymentEvents,
        couponTemplates,
        marketingCampaigns,
        campaignBudgets,
        couponCodes,
        couponReservations,
        couponRedemptions,
        couponRollbacks,
        commercialAccounts,
        commercialAccountHolds,
        commercialAccountLedger,
        commercialRequestSettlements,
        commercialPricingPlans,
        commercialPricingRates,
        routingLogs,
        providerHealth,
        runtimeStatuses,
      });

      startTransition(() => {
        setSnapshot(nextSnapshot);
        setStatus('Live control-plane data synchronized.');
      });
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : 'Failed to refresh admin workspace.',
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    const token = readAdminSessionToken();

    if (!token) {
      setAuthResolved(true);
      return;
    }

    let cancelled = false;

    void getAdminMe(token)
      .then(async (user) => {
        if (cancelled) {
          return;
        }

        setSessionUser(user);
        await refreshWorkspace(user);
      })
      .catch(() => {
        clearAdminSessionToken();
      })
      .finally(() => {
        if (!cancelled) {
          setAuthResolved(true);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleLogin(input: { email: string; password: string }) {
    setLoading(true);
    setStatus('Establishing operator session...');

    try {
      const session = await loginAdminUser(input);
      persistAdminSessionToken(session.token);
      setSessionUser(session.user);
      setStatus('Operator session established. Loading super-admin workspace...');
      await refreshWorkspace(session.user);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Login failed.');
    } finally {
      setLoading(false);
      setAuthResolved(true);
    }
  }

  function handleLogout() {
    clearAdminSessionToken();
    setSessionUser(null);
    setSnapshot(emptySnapshot);
    setStatus('Signed out of the super-admin workspace.');
    setAuthResolved(true);
  }
  const {
    handleCloneCommercialPricingPlan,
    handlePublishCommercialPricingPlan,
    handleScheduleCommercialPricingPlan,
    handleRetireCommercialPricingPlan,
    handleSynchronizeCommercialPricingLifecycle,
    handleUpdateCommercialPricingPlan,
    handleUpdateCommercialPricingRate,
    ...otherActions
  } = createWorkbenchActions({
    refreshWorkspace: () => refreshWorkspace(),
    setStatus,
  });

  const value: AdminWorkbenchContextValue = {
    authResolved,
    sessionUser,
    snapshot,
    status,
    loading,
    refreshWorkspace,
    handleLogin,
    handleLogout,
    ...otherActions,
    handleCloneCommercialPricingPlan,
    handlePublishCommercialPricingPlan,
    handleScheduleCommercialPricingPlan,
    handleRetireCommercialPricingPlan,
    handleSynchronizeCommercialPricingLifecycle,
    handleUpdateCommercialPricingPlan,
    handleUpdateCommercialPricingRate,
  };

  return (
    <AdminWorkbenchContext.Provider value={value}>
      {children}
    </AdminWorkbenchContext.Provider>
  );
}

export function useAdminWorkbench(): AdminWorkbenchContextValue {
  const context = useContext(AdminWorkbenchContext);

  if (!context) {
    throw new Error('useAdminWorkbench must be used within AdminWorkbenchProvider.');
  }

  return context;
}
