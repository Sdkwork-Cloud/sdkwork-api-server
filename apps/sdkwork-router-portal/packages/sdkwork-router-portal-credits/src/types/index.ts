import type {
  LedgerEntry,
  PortalCommerceCoupon,
  PortalCommerceQuote,
  PortalRouteKey,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalCreditsPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface CouponImpactPreview {
  coupon: PortalCommerceCoupon;
  quote: PortalCommerceQuote;
  status: string;
}

export interface RecommendedCouponOffer {
  offer: PortalCommerceCoupon;
  rationale: string;
  preview: CouponImpactPreview;
}

export interface CreditsGuardrail {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface CreditsPageData {
  summary: ProjectBillingSummary;
  ledger: LedgerEntry[];
  coupons: PortalCommerceCoupon[];
}
