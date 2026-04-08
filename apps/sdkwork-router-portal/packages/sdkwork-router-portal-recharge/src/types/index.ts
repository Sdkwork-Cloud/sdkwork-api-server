import type {
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalCustomRechargePolicy,
  PortalRechargeOption,
  PortalRouteKey,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalRechargePageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface PortalRechargePageData {
  summary: ProjectBillingSummary;
  rechargeOptions: PortalRechargeOption[];
  customRechargePolicy: PortalCustomRechargePolicy | null;
  orders: PortalCommerceOrder[];
}

export interface PortalRechargeQuoteSnapshot {
  amountLabel: string;
  projectedBalanceLabel: string;
  grantedUnitsLabel: string;
  effectiveRatioLabel: string;
  pricingRuleLabel: string;
}

export type PortalRechargeSelectionMode = 'preset' | 'custom';

export interface PortalRechargeSelection {
  amountCents: number;
  mode: PortalRechargeSelectionMode;
}

export interface PortalRechargePageState {
  quote: PortalCommerceQuote | null;
  selection: PortalRechargeSelection | null;
}
