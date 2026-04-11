import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent } from 'react';
import {
  Card,
  CardContent,
  Input,
  Label,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  CampaignBudgetRecord,
  CampaignBudgetStatus,
  CouponCodeRecord,
  CouponCodeStatus,
  CouponRecord,
  CouponTemplateRecord,
  CouponTemplateStatus,
  MarketingCampaignRecord,
  MarketingCampaignStatus,
} from 'sdkwork-router-admin-types';

import { CouponsDetailDrawer } from './page/CouponsDetailDrawer';
import { CouponsRegistrySection } from './page/CouponsRegistrySection';
import {
  SelectField,
  daysUntilExpiry,
  expiryDetail,
  isCouponAtRisk,
  isCouponExpiringSoon,
  quotaHealth,
  type CouponStatusFilter,
} from './page/shared';

type CouponsPageProps = AdminPageProps & {
  onUpdateMarketingCouponTemplateStatus: (
    couponTemplateId: string,
    status: CouponTemplateStatus,
  ) => Promise<void> | void;
  onUpdateMarketingCampaignStatus: (
    marketingCampaignId: string,
    status: MarketingCampaignStatus,
  ) => Promise<void> | void;
  onUpdateMarketingCampaignBudgetStatus: (
    campaignBudgetId: string,
    status: CampaignBudgetStatus,
  ) => Promise<void> | void;
  onUpdateMarketingCouponCodeStatus: (
    couponCodeId: string,
    status: CouponCodeStatus,
  ) => Promise<void> | void;
};

type CouponGovernance = {
  template: CouponTemplateRecord | null;
  campaign: MarketingCampaignRecord | null;
  budget: CampaignBudgetRecord | null;
  code: CouponCodeRecord | null;
};

export function CouponsPage({
  snapshot,
  onUpdateMarketingCouponTemplateStatus,
  onUpdateMarketingCampaignStatus,
  onUpdateMarketingCampaignBudgetStatus,
  onUpdateMarketingCouponCodeStatus,
}: CouponsPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<CouponStatusFilter>('all');
  const [selectedCouponId, setSelectedCouponId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const activeCoupons = snapshot.coupons.filter((coupon) => coupon.active);
  const archivedCoupons = snapshot.coupons.filter((coupon) => !coupon.active);
  const atRiskCoupons = activeCoupons.filter(isCouponAtRisk);
  const expiringSoonCoupons = activeCoupons.filter(isCouponExpiringSoon);
  const remainingQuota = activeCoupons.reduce(
    (total, coupon) => total + Math.max(coupon.remaining, 0),
    0,
  );
  const coveredAudiences = new Set(
    activeCoupons
      .map((coupon) => coupon.audience.trim().toLowerCase())
      .filter(Boolean),
  );
  const nextExpiringCoupon =
    activeCoupons
      .map((coupon) => ({
        coupon,
        days: daysUntilExpiry(coupon.expires_on),
      }))
      .filter((item) => item.days !== null && item.days >= 0)
      .sort(
        (left, right) =>
          (left.days ?? Number.MAX_SAFE_INTEGER)
          - (right.days ?? Number.MAX_SAFE_INTEGER),
      )[0] ?? null;
  const activeTemplates = snapshot.couponTemplates.filter(
    (template) => template.status === 'active',
  );
  const activeMarketingCampaigns = snapshot.marketingCampaigns.filter(
    (campaign) => campaign.status === 'active',
  );
  const activeCampaignBudgets = snapshot.campaignBudgets.filter(
    (budget) => budget.status === 'active',
  );
  const availableCodeCount = snapshot.couponCodes.filter(
    (code) => code.status === 'available',
  ).length;
  const liveRedemptionCount = snapshot.couponRedemptions.filter(
    (redemption) => redemption.redemption_status === 'redeemed',
  ).length;
  const rollbackTrailCount = snapshot.couponRollbacks.length;

  const filteredCoupons = useMemo(
    () =>
      snapshot.coupons.filter((coupon) => {
        const matchesStatus =
          statusFilter === 'all'
          || (statusFilter === 'active' && coupon.active)
          || (statusFilter === 'archived' && !coupon.active)
          || (statusFilter === 'at_risk' && isCouponAtRisk(coupon));

        if (!matchesStatus) {
          return false;
        }

        const haystack = [
          coupon.code,
          coupon.discount_label,
          coupon.audience,
          coupon.note,
          coupon.expires_on,
        ]
          .join(' ')
          .toLowerCase();

        return haystack.includes(deferredQuery);
      }),
    [deferredQuery, snapshot.coupons, statusFilter],
  );

  useEffect(() => {
    if (selectedCouponId && !filteredCoupons.some((coupon) => coupon.id === selectedCouponId)) {
      setSelectedCouponId(null);
      setIsDetailDrawerOpen(false);
    }
  }, [filteredCoupons, selectedCouponId]);

  const selectedCoupon =
    filteredCoupons.find((coupon) => coupon.id === selectedCouponId) ?? null;
  const selectedCouponGovernance = useMemo(
    () =>
      selectedCoupon
        ? resolveCouponGovernance(snapshot, selectedCoupon)
        : {
            template: null,
            campaign: null,
            budget: null,
            code: null,
          },
    [selectedCoupon, snapshot],
  );

  const columns = useMemo<DataTableColumn<CouponRecord>[]>(
    () => [
      {
        id: 'campaign',
        header: t('Campaign'),
        cell: (coupon) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {coupon.code}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {coupon.note}
            </div>
          </div>
        ),
      },
      {
        id: 'offer',
        header: t('Offer'),
        cell: (coupon) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{coupon.discount_label}</div>
            <div>{coupon.audience}</div>
          </div>
        ),
      },
      {
        id: 'remaining',
        header: t('Remaining quota'),
        align: 'right',
        cell: (coupon) => formatNumber(coupon.remaining),
        width: 140,
      },
      {
        id: 'quota-health',
        header: t('Quota health'),
        cell: (coupon) => {
          const health = quotaHealth(coupon);
          return (
            <div className="space-y-1">
              <StatusBadge showIcon status={health.label} variant={health.variant} />
              <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                {health.detail}
              </div>
            </div>
          );
        },
      },
      {
        id: 'expires',
        header: t('Expiry'),
        cell: (coupon) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{coupon.expires_on}</div>
            <div>{expiryDetail(coupon)}</div>
          </div>
        ),
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (coupon) => (
          <StatusBadge
            showIcon
            status={coupon.active ? 'active' : 'archived'}
            variant={coupon.active ? 'success' : 'secondary'}
          />
        ),
        width: 140,
      },
    ],
    [formatNumber, t],
  );

  function openDetailDrawer(coupon: CouponRecord) {
    setSelectedCouponId(coupon.id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedCouponId(null);
    }
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="coupon-search">
                  {t('Search campaigns')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="coupon-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('code, audience, note')}
                    value={search}
                  />
                </div>
              </div>

              <div className="min-w-[12rem]">
                <SelectField
                  label={t('Campaign state')}
                  labelVisibility="sr-only"
                  onValueChange={setStatusFilter}
                  options={[
                    { label: t('All campaigns'), value: 'all' },
                    { label: t('Active'), value: 'active' },
                    { label: t('At risk'), value: 'at_risk' },
                    { label: t('Archived'), value: 'archived' },
                  ]}
                  placeholder={t('Campaign state')}
                  value={statusFilter}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(filteredCoupons.length) })}
                  {' | '}
                  {t('{count} live', { count: formatNumber(activeCoupons.length) })}
                  {' | '}
                  {t('{count} at risk', { count: formatNumber(atRiskCoupons.length) })}
                </div>
                <div className="rounded-full border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-muted)] px-3 py-1 text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-secondary)]">
                  {t('Canonical marketing derived')}
                </div>
              </div>
            </form>

            <div className="mt-4 grid gap-3 xl:grid-cols-5">
              {[
                {
                  title: t('Template governance'),
                  value: formatNumber(activeTemplates.length),
                  detail: t('{count} active templates', {
                    count: formatNumber(activeTemplates.length),
                  }),
                },
                {
                  title: t('Campaign budgets'),
                  value: formatNumber(activeCampaignBudgets.length),
                  detail: t('{count} active campaigns', {
                    count: formatNumber(activeMarketingCampaigns.length),
                  }),
                },
                {
                  title: t('Code vault'),
                  value: formatNumber(availableCodeCount),
                  detail: t('{count} total codes', {
                    count: formatNumber(snapshot.couponCodes.length),
                  }),
                },
                {
                  title: t('Redemption ledger'),
                  value: formatNumber(liveRedemptionCount),
                  detail: t('{count} tracked redemptions', {
                    count: formatNumber(snapshot.couponRedemptions.length),
                  }),
                },
                {
                  title: t('Rollback trail'),
                  value: formatNumber(rollbackTrailCount),
                  detail: t('{count} recorded rollbacks', {
                    count: formatNumber(rollbackTrailCount),
                  }),
                },
              ].map((item) => (
                <div
                  className="rounded-2xl border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-muted)]/60 px-4 py-3"
                  key={item.title}
                >
                  <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                    {item.title}
                  </div>
                  <div className="mt-2 text-2xl font-semibold text-[var(--sdk-color-text-primary)]">
                    {item.value}
                  </div>
                  <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                    {item.detail}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <CouponsRegistrySection
            activeCoupons={activeCoupons}
            archivedCoupons={archivedCoupons}
            atRiskCoupons={atRiskCoupons}
            columns={columns}
            coveredAudiencesCount={coveredAudiences.size}
            expiringSoonCoupons={expiringSoonCoupons}
            filteredCoupons={filteredCoupons}
            nextExpiringCoupon={nextExpiringCoupon}
            onSelectCoupon={openDetailDrawer}
            remainingQuota={remainingQuota}
            selectedCouponId={selectedCouponId}
          />
        </div>
      </div>

      <CouponsDetailDrawer
        governance={selectedCouponGovernance}
        onOpenChange={handleDetailDrawerOpenChange}
        onUpdateMarketingCampaignBudgetStatus={(campaignBudgetId, status) =>
          void onUpdateMarketingCampaignBudgetStatus(campaignBudgetId, status)
        }
        onUpdateMarketingCampaignStatus={(marketingCampaignId, status) =>
          void onUpdateMarketingCampaignStatus(marketingCampaignId, status)
        }
        onUpdateMarketingCouponCodeStatus={(couponCodeId, status) =>
          void onUpdateMarketingCouponCodeStatus(couponCodeId, status)
        }
        onUpdateMarketingCouponTemplateStatus={(couponTemplateId, status) =>
          void onUpdateMarketingCouponTemplateStatus(couponTemplateId, status)
        }
        open={isDetailDrawerOpen}
        selectedCoupon={selectedCoupon}
      />
    </>
  );
}

function resolveCouponGovernance(
  snapshot: AdminPageProps['snapshot'],
  coupon: CouponRecord,
): CouponGovernance {
  const normalizedCode = coupon.code.trim().toUpperCase();
  const codes = snapshot.couponCodes
    .filter((record) => record.code_value.trim().toUpperCase() === normalizedCode)
    .sort((left, right) => right.updated_at_ms - left.updated_at_ms);
  const code = codes[0] ?? null;
  const template =
    code
      ? snapshot.couponTemplates.find(
          (record) => record.coupon_template_id === code.coupon_template_id,
        ) ?? null
      : null;
  const campaign = template
    ? snapshot.marketingCampaigns
        .filter((record) => record.coupon_template_id === template.coupon_template_id)
        .sort(compareMarketingCampaigns)[0] ?? null
    : null;
  const budget = campaign
    ? snapshot.campaignBudgets
        .filter((record) => record.marketing_campaign_id === campaign.marketing_campaign_id)
        .sort(compareCampaignBudgets)[0] ?? null
    : null;

  return {
    template,
    campaign,
    budget,
    code,
  };
}

function compareMarketingCampaigns(
  left: MarketingCampaignRecord,
  right: MarketingCampaignRecord,
): number {
  return (
    marketingCampaignPriority(right.status) - marketingCampaignPriority(left.status)
    || right.updated_at_ms - left.updated_at_ms
  );
}

function compareCampaignBudgets(
  left: CampaignBudgetRecord,
  right: CampaignBudgetRecord,
): number {
  return (
    campaignBudgetPriority(right.status) - campaignBudgetPriority(left.status)
    || right.updated_at_ms - left.updated_at_ms
  );
}

function marketingCampaignPriority(status: MarketingCampaignStatus): number {
  switch (status) {
    case 'active':
      return 5;
    case 'scheduled':
      return 4;
    case 'paused':
      return 3;
    case 'draft':
      return 2;
    case 'ended':
      return 1;
    case 'archived':
      return 0;
    default:
      return -1;
  }
}

function campaignBudgetPriority(status: CampaignBudgetStatus): number {
  switch (status) {
    case 'active':
      return 3;
    case 'exhausted':
      return 2;
    case 'draft':
      return 1;
    case 'closed':
      return 0;
    default:
      return -1;
  }
}
