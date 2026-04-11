import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin coupon workspace pulls canonical marketing governance records into types, workbench snapshot, and coupon surfaces', () => {
  const adminTypes = read('packages/sdkwork-router-admin-types/src/index.ts');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const workbenchActions = read('packages/sdkwork-router-admin-core/src/workbenchActions.ts');
  const snapshot = read('packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts');
  const couponsPage = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const detailPanel = read('packages/sdkwork-router-admin-coupons/src/page/CouponsDetailPanel.tsx');

  assert.match(adminTypes, /couponTemplates: CouponTemplateRecord\[];/);
  assert.match(adminTypes, /marketingCampaigns: MarketingCampaignRecord\[];/);
  assert.match(adminTypes, /campaignBudgets: CampaignBudgetRecord\[];/);
  assert.match(adminTypes, /couponCodes: CouponCodeRecord\[];/);
  assert.match(adminTypes, /couponReservations: CouponReservationRecord\[];/);
  assert.match(adminTypes, /couponRedemptions: CouponRedemptionRecord\[];/);
  assert.match(adminTypes, /couponRollbacks: CouponRollbackRecord\[];/);

  assert.match(workbench, /listMarketingCouponTemplates/);
  assert.match(workbench, /listMarketingCampaigns/);
  assert.match(workbench, /listMarketingCampaignBudgets/);
  assert.match(workbench, /listMarketingCouponCodes/);
  assert.match(workbench, /listMarketingCouponReservations/);
  assert.match(workbench, /listMarketingCouponRedemptions/);
  assert.match(workbench, /listMarketingCouponRollbacks/);
  assert.match(adminApi, /updateMarketingCouponTemplateStatus/);
  assert.match(adminApi, /publishMarketingCouponTemplate/);
  assert.match(adminApi, /scheduleMarketingCouponTemplate/);
  assert.match(adminApi, /retireMarketingCouponTemplate/);
  assert.match(adminApi, /listMarketingCouponTemplateLifecycleAudits/);
  assert.match(adminApi, /updateMarketingCampaignStatus/);
  assert.match(adminApi, /publishMarketingCampaign/);
  assert.match(adminApi, /scheduleMarketingCampaign/);
  assert.match(adminApi, /retireMarketingCampaign/);
  assert.match(adminApi, /listMarketingCampaignLifecycleAudits/);
  assert.match(adminApi, /updateMarketingCampaignBudgetStatus/);
  assert.match(adminApi, /updateMarketingCouponCodeStatus/);
  assert.match(workbenchActions, /handleUpdateMarketingCouponTemplateStatus/);
  assert.match(workbenchActions, /handleUpdateMarketingCampaignStatus/);
  assert.match(workbenchActions, /handleUpdateMarketingCampaignBudgetStatus/);
  assert.match(workbenchActions, /handleUpdateMarketingCouponCodeStatus/);

  assert.match(snapshot, /couponTemplates:/);
  assert.match(snapshot, /marketingCampaigns:/);
  assert.match(snapshot, /campaignBudgets:/);
  assert.match(snapshot, /couponCodes:/);
  assert.match(snapshot, /couponReservations:/);
  assert.match(snapshot, /couponRedemptions:/);
  assert.match(snapshot, /couponRollbacks:/);

  assert.match(couponsPage, /Template governance/);
  assert.match(couponsPage, /Campaign budgets/);
  assert.match(couponsPage, /Code vault/);
  assert.match(couponsPage, /Redemption ledger/);
  assert.match(couponsPage, /Rollback trail/);
  assert.match(couponsPage, /onUpdateMarketingCouponTemplateStatus/);
  assert.match(couponsPage, /onUpdateMarketingCampaignStatus/);
  assert.match(couponsPage, /onUpdateMarketingCampaignBudgetStatus/);
  assert.match(couponsPage, /onUpdateMarketingCouponCodeStatus/);
  assert.doesNotMatch(detailPanel, /Legacy coupon compatibility/);
  assert.match(detailPanel, /Governance controls/);
  assert.match(detailPanel, /Template status/);
  assert.match(detailPanel, /Campaign status/);
  assert.match(detailPanel, /Budget status/);
  assert.match(detailPanel, /Code status/);
});
