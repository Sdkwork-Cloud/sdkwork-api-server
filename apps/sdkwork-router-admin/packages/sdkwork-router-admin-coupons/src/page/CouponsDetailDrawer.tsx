import {
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
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

import { CouponsDetailPanel } from './CouponsDetailPanel';
import { quotaHealth } from './shared';

type CouponsDetailDrawerProps = {
  governance: {
    template: CouponTemplateRecord | null;
    campaign: MarketingCampaignRecord | null;
    budget: CampaignBudgetRecord | null;
    code: CouponCodeRecord | null;
  };
  onOpenChange: (open: boolean) => void;
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
  open: boolean;
  selectedCoupon: CouponRecord | null;
};

export function CouponsDetailDrawer({
  governance,
  onOpenChange,
  onUpdateMarketingCampaignBudgetStatus,
  onUpdateMarketingCampaignStatus,
  onUpdateMarketingCouponCodeStatus,
  onUpdateMarketingCouponTemplateStatus,
  open,
  selectedCoupon,
}: CouponsDetailDrawerProps) {
  const { t } = useAdminI18n();
  const health = selectedCoupon ? quotaHealth(selectedCoupon) : null;

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {selectedCoupon ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{selectedCoupon.code}</DrawerTitle>
                    <DrawerDescription>{selectedCoupon.discount_label}</DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge
                      showIcon
                      status={selectedCoupon.active ? 'active' : 'archived'}
                      variant={selectedCoupon.active ? 'success' : 'secondary'}
                    />
                    {health ? (
                      <StatusBadge showIcon status={health.label} variant={health.variant} />
                    ) : null}
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <CouponsDetailPanel
                governance={governance}
                onUpdateMarketingCampaignBudgetStatus={onUpdateMarketingCampaignBudgetStatus}
                onUpdateMarketingCampaignStatus={onUpdateMarketingCampaignStatus}
                onUpdateMarketingCouponCodeStatus={onUpdateMarketingCouponCodeStatus}
                onUpdateMarketingCouponTemplateStatus={onUpdateMarketingCouponTemplateStatus}
                selectedCoupon={selectedCoupon}
              />
            </DrawerBody>
          </>
        ) : null}
      </DrawerContent>
    </Drawer>
  );
}
