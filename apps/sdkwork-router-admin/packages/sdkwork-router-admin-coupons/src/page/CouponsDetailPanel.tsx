import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
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

import { expiryDetail, quotaHealth } from './shared';

type CouponsDetailPanelProps = {
  governance: {
    template: CouponTemplateRecord | null;
    campaign: MarketingCampaignRecord | null;
    budget: CampaignBudgetRecord | null;
    code: CouponCodeRecord | null;
  };
  onUpdateMarketingCampaignBudgetStatus: (
    campaignBudgetId: string,
    status: CampaignBudgetStatus,
  ) => void;
  onUpdateMarketingCampaignStatus: (
    marketingCampaignId: string,
    status: MarketingCampaignStatus,
  ) => void;
  onUpdateMarketingCouponCodeStatus: (
    couponCodeId: string,
    status: CouponCodeStatus,
  ) => void;
  onUpdateMarketingCouponTemplateStatus: (
    couponTemplateId: string,
    status: CouponTemplateStatus,
  ) => void;
  selectedCoupon: CouponRecord;
};

export function CouponsDetailPanel({
  governance,
  onUpdateMarketingCampaignBudgetStatus,
  onUpdateMarketingCampaignStatus,
  onUpdateMarketingCouponCodeStatus,
  onUpdateMarketingCouponTemplateStatus,
  selectedCoupon,
}: CouponsDetailPanelProps) {
  const { formatNumber, t } = useAdminI18n();
  const health = quotaHealth(selectedCoupon);
  const templateAction = governance.template
    ? governance.template.status === 'active'
      ? { label: t('Archive template'), nextStatus: 'archived' as const }
      : { label: t('Activate template'), nextStatus: 'active' as const }
    : null;
  const campaignAction = governance.campaign
    ? governance.campaign.status === 'active'
      ? { label: t('Pause campaign'), nextStatus: 'paused' as const }
      : { label: t('Activate campaign'), nextStatus: 'active' as const }
    : null;
  const budgetAction = governance.budget
    ? governance.budget.status === 'active'
      ? { label: t('Close budget'), nextStatus: 'closed' as const }
      : { label: t('Activate budget'), nextStatus: 'active' as const }
    : null;
  const codeAction = governance.code
    ? governance.code.status === 'available'
      ? { disabled: false, label: t('Disable code'), nextStatus: 'disabled' as const }
      : governance.code.status === 'disabled'
        ? { disabled: false, label: t('Enable code'), nextStatus: 'available' as const }
        : {
            disabled: true,
            label: t('Code locked by lifecycle'),
            nextStatus: governance.code.status,
          }
    : null;

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between gap-3">
            <CardTitle className="text-base">{t('Campaign posture')}</CardTitle>
            <StatusBadge
              showIcon
              status={selectedCoupon.active ? 'active' : 'archived'}
              variant={selectedCoupon.active ? 'success' : 'secondary'}
            />
          </div>
          <CardDescription>{selectedCoupon.note}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <DescriptionList columns={2}>
            <DescriptionItem>
              <DescriptionTerm>{t('Audience')}</DescriptionTerm>
              <DescriptionDetails>{selectedCoupon.audience}</DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Discount')}</DescriptionTerm>
              <DescriptionDetails>{selectedCoupon.discount_label}</DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Remaining quota')}</DescriptionTerm>
              <DescriptionDetails>{formatNumber(selectedCoupon.remaining)}</DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Expiry')}</DescriptionTerm>
              <DescriptionDetails>{selectedCoupon.expires_on}</DescriptionDetails>
            </DescriptionItem>
          </DescriptionList>
          <div className="grid gap-3 md:grid-cols-2">
            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none">
              <CardHeader className="space-y-1">
                <CardTitle className="text-sm">{t('Quota health')}</CardTitle>
                <CardDescription>{health.detail}</CardDescription>
              </CardHeader>
              <CardContent className="pt-0">
                <StatusBadge showIcon status={health.label} variant={health.variant} />
              </CardContent>
            </Card>
            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none">
              <CardHeader className="space-y-1">
                <CardTitle className="text-sm">{t('Expiry window')}</CardTitle>
                <CardDescription>{expiryDetail(selectedCoupon)}</CardDescription>
              </CardHeader>
              <CardContent className="pt-0 text-sm text-[var(--sdk-color-text-secondary)]">
                {t(
                  'Support and campaign operators can use this window to stage renewals or retire the offer.',
                )}
              </CardContent>
            </Card>
            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none md:col-span-2">
              <CardHeader className="space-y-1">
                <CardTitle className="text-sm">{t('Governance controls')}</CardTitle>
                <CardDescription>
                  {t(
                    'Template, campaign, budget, and code status controls let operators stop risk exposure or restore offers without editing the whole record.',
                  )}
                </CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <GovernanceStatusCard
                  actionLabel={templateAction?.label ?? t('No template linked')}
                  disabled={!governance.template || !templateAction}
                  onAction={() => {
                    if (governance.template && templateAction) {
                      onUpdateMarketingCouponTemplateStatus(
                        governance.template.coupon_template_id,
                        templateAction.nextStatus,
                      );
                    }
                  }}
                  statusLabel={governance.template?.status ?? t('missing')}
                  title={t('Template status')}
                />
                <GovernanceStatusCard
                  actionLabel={campaignAction?.label ?? t('No campaign linked')}
                  disabled={!governance.campaign || !campaignAction}
                  onAction={() => {
                    if (governance.campaign && campaignAction) {
                      onUpdateMarketingCampaignStatus(
                        governance.campaign.marketing_campaign_id,
                        campaignAction.nextStatus,
                      );
                    }
                  }}
                  statusLabel={governance.campaign?.status ?? t('missing')}
                  title={t('Campaign status')}
                />
                <GovernanceStatusCard
                  actionLabel={budgetAction?.label ?? t('No budget linked')}
                  disabled={!governance.budget || !budgetAction}
                  onAction={() => {
                    if (governance.budget && budgetAction) {
                      onUpdateMarketingCampaignBudgetStatus(
                        governance.budget.campaign_budget_id,
                        budgetAction.nextStatus,
                      );
                    }
                  }}
                  statusLabel={governance.budget?.status ?? t('missing')}
                  title={t('Budget status')}
                />
                <GovernanceStatusCard
                  actionLabel={codeAction?.label ?? t('No code linked')}
                  disabled={!governance.code || !codeAction || codeAction.disabled}
                  onAction={() => {
                    if (
                      governance.code
                      && codeAction
                      && !codeAction.disabled
                      && typeof codeAction.nextStatus === 'string'
                    ) {
                      onUpdateMarketingCouponCodeStatus(
                        governance.code.coupon_code_id,
                        codeAction.nextStatus,
                      );
                    }
                  }}
                  statusLabel={governance.code?.status ?? t('missing')}
                  title={t('Code status')}
                />
              </CardContent>
            </Card>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function GovernanceStatusCard({
  actionLabel,
  disabled,
  onAction,
  statusLabel,
  title,
}: {
  actionLabel: string;
  disabled: boolean;
  onAction: () => void;
  statusLabel: string;
  title: string;
}) {
  return (
    <Card className="border-[var(--sdk-color-border-subtle)] shadow-none">
      <CardHeader className="space-y-1">
        <CardTitle className="text-sm">{title}</CardTitle>
        <CardDescription>{statusLabel}</CardDescription>
      </CardHeader>
      <CardContent className="pt-0">
        <Button disabled={disabled} onClick={onAction} size="sm" type="button" variant="outline">
          {actionLabel}
        </Button>
      </CardContent>
    </Card>
  );
}
