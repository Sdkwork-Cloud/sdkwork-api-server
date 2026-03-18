import type {
  PortalRouteKey,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

export interface PortalBillingPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface BillingRunway {
  label: string;
  detail: string;
  projected_days: number | null;
  daily_units: number | null;
}

export interface BillingBundleRecommendation {
  title: string;
  detail: string;
}

export interface BillingRecommendation {
  title: string;
  detail: string;
  plan: SubscriptionPlan | null;
  pack: RechargePack | null;
  runway: BillingRunway;
  bundle: BillingBundleRecommendation;
}

export interface BillingPageData {
  summary: ProjectBillingSummary;
  usage_records: UsageRecord[];
}
