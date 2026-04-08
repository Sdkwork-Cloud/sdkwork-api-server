import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal auth entry mirrors claw-studio visual hierarchy instead of the custom launch-cockpit narrative', () => {
  const authPage = read('packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx');

  assert.match(authPage, /qrLogin|QrCode/);
  assert.match(authPage, /welcomeBack|Create workspace|resetPassword|Recover access/);
  assert.match(authPage, /continueWith|GitHub|Google|GitBranch|Globe/);
  assert.doesNotMatch(authPage, /Preview the first launch path/);
  assert.doesNotMatch(authPage, /Start in four moves/);
  assert.doesNotMatch(authPage, /Why teams trust this portal/);
});

test('portal shell keeps grouped business navigation in a claw-style rail and moves shell settings into the user control', () => {
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');

  assert.match(desktopShell, /DesktopShellFrame/);
  assert.doesNotMatch(layout, /ShellStatus/);
  assert.match(desktopShell, /WindowControls/);
  assert.match(layout, /PortalDesktopShell/);
  assert.match(layout, /PortalSettingsCenter/);
  assert.match(desktopShell, /sidebar=\{/);
  assert.doesNotMatch(desktopShell, /navigation=\{/);
  assert.match(navigationRail, /Operations/);
  assert.match(navigationRail, /Access/);
  assert.match(navigationRail, /Revenue/);
  assert.match(navigationRail, /data-slot="sidebar-user-control"/);
  assert.match(navigationRail, /User details/);
  assert.match(navigationRail, /Sign out/);
  assert.doesNotMatch(navigationRail, /Need help\?/);
  assert.doesNotMatch(navigationRail, /<NavigationRail|NavigationRail\s*\}\s*from/);
  assert.match(routes, /Routing/);
  assert.match(routes, /group:\s*'operations'/);
  assert.match(routes, /key:\s*'user'[\s\S]*?sidebarVisible:\s*false/);
});

test('dashboard follows claw-studio analytics workbench architecture adapted to portal telemetry', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const dashboardComponents = read('packages/sdkwork-router-portal-dashboard/src/components/index.tsx');
  const dashboardRepository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');
  const dualColumnSectionCount = (
    dashboardPage.match(/xl:grid-cols-\[1\.35fr_0\.95fr\]/g) ?? []
  ).length;

  assert.match(dashboardComponents, /DashboardSummaryCard/);
  assert.match(dashboardComponents, /StatusBadge/);
  assert.doesNotMatch(dashboardComponents, /DashboardStatusPill/);
  assert.match(dashboardComponents, /DashboardRevenueTrendChart/);
  assert.match(dashboardComponents, /DashboardTokenTrendChart/);
  assert.match(dashboardComponents, /DashboardDistributionRingChart/);
  assert.match(dashboardComponents, /DashboardModelDistributionChart/);
  assert.doesNotMatch(dashboardPage, /SectionHeader/);
  assert.match(dashboardPage, /WorkspacePanel/);
  assert.match(dashboardPage, /ManagementWorkbench/);
  assert.match(dashboardPage, /StatusBadge/);
  assert.doesNotMatch(dashboardPage, /DashboardStatusPill/);
  assert.match(dashboardPage, /DashboardBalanceCard/);
  assert.match(dashboardPage, /DashboardMetricCard/);
  assert.match(dashboardPage, /Balance/);
  assert.match(dashboardPage, /Revenue/);
  assert.match(dashboardPage, /Total requests/);
  assert.match(dashboardPage, /Average booked spend/);
  assert.match(dashboardPage, /Today/);
  assert.match(dashboardPage, /7 days/);
  assert.match(dashboardPage, /This month/);
  assert.doesNotMatch(dashboardPage, /Portal overview/);
  assert.doesNotMatch(dashboardPage, /Workspace command center/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Traffic posture'\)\}/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Cost and quota'\)\}/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Workspace readiness'\)\}/);
  assert.match(dashboardPage, /Analytics workbench/);
  assert.match(dashboardPage, /Routing evidence/);
  assert.match(dashboardPage, /Next actions/);
  assert.match(dashboardPage, /Module posture/);
  assert.match(dashboardPage, /Recent requests/);
  assert.match(dashboardPage, /Provider distribution/);
  assert.match(dashboardPage, /Model distribution/);
  assert.doesNotMatch(dashboardPage, /Commercial highlights/);
  assert.doesNotMatch(dashboardPage, /Leading accounting mode/);
  assert.doesNotMatch(dashboardPage, /Leading capability/);
  assert.doesNotMatch(dashboardPage, /Multimodal demand/);
  assert.doesNotMatch(dashboardComponents, /DashboardSectionHeader/);
  assert.doesNotMatch(dashboardPage, /const surfaceClass =/);
  assert.ok(
    dualColumnSectionCount >= 2,
    'dashboard should repeat the claw-studio dual-column panel rhythm',
  );
  assert.match(dashboardPage, /data-slot="portal-dashboard-workbench-tabs"/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-grid/);
  assert.doesNotMatch(dashboardPage, /ResponsiveContainer/);
  assert.doesNotMatch(dashboardPage, /DashboardSectionHeader/);
  assert.match(dashboardRepository, /getPortalRoutingSummary/);
  assert.match(dashboardRepository, /listPortalRoutingDecisionLogs/);
  assert.doesNotMatch(dashboardPage, /Traffic overview/);
  assert.doesNotMatch(dashboardPage, /Workspace modules/);
  assert.doesNotMatch(dashboardPage, /Recent activity/);
});

test('redeem and billing pages expose coupon activation and payment decision support', () => {
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsComponents = read('packages/sdkwork-router-portal-credits/src/components/index.tsx');
  const billingComponents = read('packages/sdkwork-router-portal-billing/src/components/index.tsx');
  const creditsRepository = read('packages/sdkwork-router-portal-credits/src/repository/index.ts');
  const billingRepository = read('packages/sdkwork-router-portal-billing/src/repository/index.ts');

  assert.match(creditsPage, /portal-redeem-entry-card/);
  assert.match(creditsPage, /portal-redeem-invite-card/);
  assert.match(creditsPage, /portal-redeem-wallet-table/);
  assert.match(creditsPage, /portal-redeem-reward-history-table/);
  assert.match(creditsPage, /Redeem code/);
  assert.match(creditsPage, /My coupons/);
  assert.match(creditsPage, /Reward history/);
  assert.match(creditsPage, /Invite rewards/);
  assert.match(creditsPage, /Copy invite link/);
  assert.match(billingPage, /Active membership/);
  assert.match(billingPage, /Estimated runway/);
  assert.match(billingPage, /Recommended bundle/);
  assert.match(billingPage, /Pending payment queue/);
  assert.match(billingPage, /Checkout details/);
  assert.match(billingPage, /Open checkout/);
  assert.match(billingPage, /Payment method/);
  assert.match(
    billingPage,
    /Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture\./,
  );
  assert.match(billingPage, /Pending payment queue keeps unpaid or unfulfilled orders visible until payment completes or the order leaves the queue\./);
  assert.match(billingPage, /Checkout attempts that closed on the failure path and need a fresh checkout decision\./);
  assert.match(
    billingPage,
    /Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up\./,
  );
  assert.match(
    billingPage,
    /\{reference\} is the current \{provider\} \/ \{channel\} payment reference for this order\./,
  );
  assert.match(
    billingPage,
    /Billing view keeps live quota, checkout progress, and payment history in one place\./,
  );
  assert.match(billingPage, /Payment history keeps checkout outcomes, payment method evidence, and refund status visible in one billing timeline\./);
  assert.match(billingPage, /Refund history keeps completed refund outcomes, payment method evidence, and the resulting order status visible without reopening each order\./);
  assert.match(billingPage, /Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path\./);
  assert.match(billingPage, /Manual settlement/);
  assert.doesNotMatch(billingPage, /Operator settlement/);
  assert.match(billingPage, /Settlement coverage/);
  assert.match(billingPage, /Payment outcome sandbox/);
  assert.match(billingPage, /Sandbox method/);
  assert.match(billingPage, /Primary method/);
  assert.match(billingPage, /Checkout access/);
  assert.match(billingPage, /Checkout attempts/);
  assert.match(billingPage, /Checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench\./);
  assert.match(billingPage, /No checkout attempts recorded yet/);
  assert.match(billingPage, /Checkout workbench keeps checkout access, selected reference, and payable price aligned under one payment method\./);
  assert.match(billingPage, /No checkout guidance is available for this order yet\./);
  assert.match(billingPage, /Payment outcomes/);
  assert.match(billingPage, /Apply settlement, failure, or cancellation outcomes for the selected payment method before live payment confirmation is enabled\./);
  assert.match(billingPage, /Choose sandbox method/);
  assert.match(billingPage, /Payment outcomes will use \{provider\} on \{channel\}\./);
  assert.match(billingPage, /Verification method/);
  assert.match(billingPage, /Checkout reference/);
  assert.match(billingPage, /Manual confirmation/);
  assert.match(billingPage, /Manual step/);
  assert.match(billingPage, /Signed callback check/);
  assert.match(billingPage, /Stripe signature check/);
  assert.match(billingPage, /Alipay RSA-SHA256 check/);
  assert.match(billingPage, /WeChat Pay RSA-SHA256 check/);
  assert.match(billingPage, /QR code content/);
  assert.match(billingPage, /Hosted checkout flow/);
  assert.match(billingPage, /QR checkout flow/);
  assert.match(billingPage, /Applying \{provider\} settlement outcome for \{orderId\}\.\.\./);
  assert.match(billingPage, /Applying \{provider\} failure outcome for \{orderId\}\.\.\./);
  assert.match(billingPage, /Applying \{provider\} cancellation outcome for \{orderId\}\.\.\./);
  assert.match(billingPage, /was settled after the \{provider\} payment confirmation\./);
  assert.match(billingPage, /was marked failed after the \{provider\} payment confirmation\./);
  assert.match(billingPage, /was canceled after the \{provider\} payment confirmation\./);
  assert.match(billingPage, /This checkout is already closed, so there are no remaining payment actions\./);
  assert.match(billingPage, /Apply settlement outcome/);
  assert.match(billingPage, /Apply failure outcome/);
  assert.match(billingPage, /Apply cancellation outcome/);
  assert.match(billingPage, /Opening checkout\.\.\./);
  assert.match(billingPage, /Start checkout/);
  assert.match(billingPage, /Resume checkout/);
  assert.match(billingPage, /fresh checkout attempt/);
  assert.match(billingPage, /created a \{provider\} checkout attempt, but no checkout link was returned\./);
  assert.match(billingPage, /now uses the \{provider\} checkout launch path\./);
  assert.match(billingPage, /Start the first checkout now\./);
  assert.match(billingPage, /Failed payment/);
  assert.match(billingPage, /Settle order/);
  assert.match(billingPage, /Cancel order/);
  assert.match(billingPage, /Order timeline/);
  assert.match(billingPage, /Payment update reference/);
  assert.doesNotMatch(billingPage, /Commercial settlement rail/);
  assert.doesNotMatch(billingPage, /\bPayment rail\b/);
  assert.doesNotMatch(billingPage, /\bPrimary rail\b/);
  assert.doesNotMatch(billingPage, /\bEvent rail\b/);
  assert.doesNotMatch(billingPage, /selected payment rail/);
  assert.doesNotMatch(billingPage, /payment rail evidence/);
  assert.doesNotMatch(billingPage, /sandbox rail/);
  assert.doesNotMatch(billingPage, /Provider callbacks/);
  assert.doesNotMatch(billingPage, /Provider webhooks/);
  assert.doesNotMatch(billingPage, /Simulate provider settlement/);
  assert.doesNotMatch(billingPage, /Simulate provider failure/);
  assert.doesNotMatch(billingPage, /callback rehearsal/);
  assert.doesNotMatch(billingPage, /callback flow/);
  assert.doesNotMatch(billingPage, /provider callback/);
  assert.doesNotMatch(billingPage, /provider handoff/);
  assert.doesNotMatch(billingPage, /Provider events/);
  assert.doesNotMatch(billingPage, /Provider event/);
  assert.doesNotMatch(billingPage, /Launching provider checkout/);
  assert.doesNotMatch(billingPage, /Launch provider checkout/);
  assert.doesNotMatch(billingPage, /Resume provider checkout/);
  assert.doesNotMatch(billingPage, /Launch the first provider checkout now/);
  assert.doesNotMatch(billingPage, /payment attempt/);
  assert.doesNotMatch(billingPage, /payment attempts/);
  assert.doesNotMatch(billingPage, /Replay provider settlement, failure, or cancellation events/);
  assert.doesNotMatch(billingPage, /Replaying \{provider\} settlement/);
  assert.doesNotMatch(billingPage, /Replaying \{provider\} failure/);
  assert.doesNotMatch(billingPage, /Replaying \{provider\} cancellation/);
  assert.doesNotMatch(billingPage, /Replay settlement event/);
  assert.doesNotMatch(billingPage, /Replay failure event/);
  assert.doesNotMatch(billingPage, /Replay cancel event/);
  assert.doesNotMatch(
    billingPage,
    /Formal order-scoped checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Commercial account exposes canonical balance, hold, and account identity state beside the workspace billing posture\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Failed payment isolates checkout attempts that need coupon updates, a different payment method, or a fresh checkout attempt\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline\./,
  );
  assert.doesNotMatch(
    billingPage,
    /\{reference\} anchors the current \{provider\} \/ \{channel\} payment method for this order\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Formal checkout keeps checkout access, selected reference, and payable price aligned under one payment method\./,
  );
  assert.doesNotMatch(billingPage, /No formal guidance is available for this order yet\./);
  assert.doesNotMatch(
    billingPage,
    /\{targetName\} created a formal \{provider\} checkout attempt, but no checkout URL was returned\./,
  );
  assert.doesNotMatch(
    billingPage,
    /\{targetName\} now uses the formal \{provider\} checkout launch path\./,
  );
  assert.doesNotMatch(billingPage, /Payment event sandbox/);
  assert.doesNotMatch(billingPage, /Event target/);
  assert.doesNotMatch(billingPage, /Choose event target/);
  assert.doesNotMatch(billingPage, /active sandbox target/);
  assert.doesNotMatch(billingPage, /Event signature/);
  assert.doesNotMatch(billingPage, /Checkout session/);
  assert.doesNotMatch(billingPage, /Open session/);
  assert.doesNotMatch(billingPage, /Loading session/);
  assert.doesNotMatch(billingPage, /Loading checkout session/);
  assert.doesNotMatch(billingPage, /existing provider session/);
  assert.doesNotMatch(billingPage, /Manual action/);
  assert.doesNotMatch(billingPage, /Hosted checkout session/);
  assert.doesNotMatch(billingPage, /QR code session/);
  assert.doesNotMatch(billingPage, /This checkout session is already closed, so there are no remaining payment actions\./);
  assert.doesNotMatch(billingPage, /Session reference/);
  assert.doesNotMatch(billingPage, /QR payload/);
  assert.doesNotMatch(billingPage, /operator-facing posture/);
  assert.doesNotMatch(billingPage, /workspace settles or cancels them/);
  assert.doesNotMatch(billingPage, /provider callback review/);
  assert.doesNotMatch(billingPage, /operator-facing audit timeline/);
  assert.doesNotMatch(billingPage, /closed-loop refund outcomes/);
  assert.doesNotMatch(billingPage, /refund closure/);
  assert.doesNotMatch(billingPage, /operators can distinguish degraded routing from normal preference selection/);
  assert.doesNotMatch(billingPage, /verify provider, checkout reference, and final order state without reopening each order/);
  assert.match(creditsPage, /Redeem/);
  assert.match(creditsRepository, /validatePortalCoupon/);
  assert.match(creditsRepository, /reservePortalCouponRedemption/);
  assert.match(creditsRepository, /confirmPortalCouponRedemption/);
  assert.match(creditsRepository, /listPortalCommerceOrders/);
  assert.match(creditsRepository, /listPortalMarketingMyCoupons/);
  assert.match(creditsRepository, /listPortalMarketingRewardHistory/);
  assert.match(creditsRepository, /listPortalMarketingRedemptions/);
  assert.match(billingRepository, /getPortalCommerceCatalog/);
  assert.match(billingRepository, /getPortalCommerceOrderCenter/);
  assert.match(billingRepository, /getPortalCommerceOrder/);
  assert.match(billingRepository, /listPortalCommercePaymentMethods/);
  assert.match(billingRepository, /getPortalCommercePaymentAttempt/);
  assert.match(billingRepository, /getBillingCheckoutDetail/);
  assert.match(billingRepository, /getPortalCommercialAccountHistory/);
  assert.match(billingRepository, /previewPortalCommerceQuote/);
  assert.match(billingRepository, /createPortalCommerceOrder/);
  assert.match(billingRepository, /getPortalCommerceCheckoutSession/);
  assert.match(billingRepository, /sendPortalCommercePaymentEvent/);
  assert.match(billingRepository, /settlePortalCommerceOrder/);
  assert.match(billingRepository, /cancelPortalCommerceOrder/);
  assert.match(billingRepository, /membership:\s*order_center\.membership/);
  assert.match(billingRepository, /commercial_reconciliation:\s*order_center\.reconciliation/);
  assert.match(billingPage, /getBillingCheckoutDetail/);
  assert.match(billingPage, /checkoutDetail\?\.latest_payment_attempt/);
  assert.match(billingPage, /checkoutDetail\?\.selected_payment_method/);
  assert.doesNotMatch(creditsComponents, /portalx-/);
  assert.doesNotMatch(billingComponents, /portalx-/);
});

test('product-facing copy avoids internal seam jargon across billing, gateway, and docs', () => {
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const authPage = read('packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const gatewayServices = read('packages/sdkwork-router-portal-gateway/src/services/index.ts');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const readme = read('README.md');

  assert.doesNotMatch(billingPage, /provider callback seam/);
  assert.doesNotMatch(billingPage, /Webhook seam/);
  assert.doesNotMatch(billingPage, /checkout seam/);
  assert.doesNotMatch(billingPage, /checkout seams/);
  assert.doesNotMatch(billingPage, /Server mode seam/);
  assert.doesNotMatch(billingPage, /backend quote service/);
  assert.doesNotMatch(billingPage, /PSP SDK/);
  assert.doesNotMatch(billingPage, /semantics/);
  assert.doesNotMatch(creditsPage, /seeded UI logic/);
  assert.doesNotMatch(authPage, /portal backend/);
  assert.doesNotMatch(gatewayPage, /backend product inventory/);
  assert.doesNotMatch(gatewayPage, /frontend-only launch copy/);
  assert.doesNotMatch(gatewayPage, /placeholder launch copy/);
  assert.doesNotMatch(gatewayServices, /portal backend catalog/);
  assert.doesNotMatch(gatewayServices, /frontend launch placeholders/);
  assert.doesNotMatch(gatewayServices, /backend-backed catalog entries/);
  assert.doesNotMatch(gatewayServices, /launch placeholders/);
  assert.doesNotMatch(gatewayServices, /frontend seed seam/);
  assert.doesNotMatch(routingPage, /backend routing strategy enums/);
  assert.doesNotMatch(readme, /backend commerce catalog flows/);
  assert.doesNotMatch(readme, /coupon redemption seam/);
});

test('gateway command center makes compatibility, deployment modes, and commerce readiness explicit', () => {
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const gatewayRepository = read('packages/sdkwork-router-portal-gateway/src/repository/index.ts');
  const gatewayServices = read('packages/sdkwork-router-portal-gateway/src/services/index.ts');
  const gatewayTypes = read('packages/sdkwork-router-portal-gateway/src/types/index.ts');
  const gatewayComponents = read('packages/sdkwork-router-portal-gateway/src/components/index.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const tauriMain = read('src-tauri/src/main.rs');

  assert.match(appRoutes, /sdkwork-router-portal-console/);
  assert.match(appRoutes, /case 'gateway'/);
  assert.match(gatewayRepository, /getPortalDashboard/);
  assert.match(gatewayRepository, /resolveGatewayBaseUrl/);
  assert.match(gatewayRepository, /getDesktopRuntimeSnapshot/);
  assert.match(gatewayRepository, /getProductRuntimeHealthSnapshot/);
  assert.match(gatewayRepository, /getPortalCommerceCatalog/);
  assert.match(gatewayRepository, /getPortalCommerceMembership/);
  assert.match(gatewayRepository, /restartDesktopRuntime/);
  assert.match(gatewayTypes, /GatewayLaunchReadinessSummary/);
  assert.match(gatewayTypes, /GatewayRuntimeControl/);
  assert.match(gatewayTypes, /GatewayCompatibilityRow/);
  assert.match(gatewayTypes, /GatewayModeCard/);
  assert.match(gatewayTypes, /GatewayServiceHealthCheck/);
  assert.match(gatewayComponents, /GatewayLaunchReadinessPanel/);
  assert.match(gatewayComponents, /GatewayRuntimeControlsGrid/);
  assert.doesNotMatch(gatewayComponents, /GatewayCompatibilityTable/);
  assert.doesNotMatch(gatewayComponents, /GatewayRateLimitPolicyTable/);
  assert.doesNotMatch(gatewayComponents, /GatewayRateLimitWindowTable/);
  assert.doesNotMatch(gatewayComponents, /GatewayServiceHealthGrid/);
  assert.doesNotMatch(gatewayComponents, /GatewayVerificationGrid/);
  assert.match(gatewayServices, /Codex/);
  assert.match(gatewayServices, /Claude Code/);
  assert.match(gatewayServices, /Gemini-compatible clients/);
  assert.match(gatewayServices, /OpenClaw/);
  assert.match(gatewayServices, /desktop mode/i);
  assert.match(gatewayServices, /server mode/i);
  assert.match(gatewayServices, /web, gateway, admin, portal/);
  assert.match(gatewayServices, /Desktop runtime evidence/);
  assert.match(gatewayServices, /Launch readiness/);
  assert.match(gatewayServices, /Critical blockers/);
  assert.match(gatewayServices, /Restart desktop runtime/);
  assert.match(gatewayServices, /Commerce catalog/);
  assert.match(gatewayPage, /Gateway posture/);
  assert.match(gatewayPage, /Launch readiness/);
  assert.match(gatewayPage, /Critical blockers/);
  assert.match(gatewayPage, /Command workbench/);
  assert.match(gatewayPage, /data-slot="portal-gateway-filter-bar"/);
  assert.match(gatewayPage, /Workbench lane/);
  assert.match(gatewayPage, /Operational focus/);
  assert.match(gatewayPage, /Search gateway evidence/);
  assert.match(gatewayPage, /Compatibility routes/);
  assert.match(gatewayPage, /Rate-limit policies/);
  assert.match(gatewayPage, /Rate-limit windows/);
  assert.match(gatewayPage, /Service health/);
  assert.match(gatewayPage, /Verification commands/);
  assert.match(gatewayPage, /Desktop runtime/);
  assert.match(gatewayPage, /Refresh command center/);
  assert.match(gatewayPage, /Refresh service health/);
  assert.match(gatewayPage, /Restart desktop runtime/);
  assert.match(gatewayPage, /Deployment playbooks/);
  assert.match(gatewayPage, /Mode switchboard/);
  assert.match(gatewayPage, /Topology playbooks/);
  assert.match(gatewayPage, /\/api\/v1\/models/);
  assert.match(gatewayPage, /\/v1\/messages/);
  assert.match(gatewayPage, /generateContent/);
  assert.match(gatewayPage, /Commercial runway/);
  assert.match(gatewayPage, /Commerce catalog/);
  assert.match(gatewayPage, /Active membership/);
  assert.match(gatewayPage, /Open API Keys/);
  assert.match(gatewayPage, /Open Routing/);
  assert.match(gatewayPage, /Open Billing/);
  assert.doesNotMatch(gatewayPage, /Compatibility matrix/);
  assert.doesNotMatch(gatewayPage, /Desktop runtime controls/);
  assert.doesNotMatch(gatewayPage, /Readiness and commerce/);
  assert.match(tauriMain, /restart_product_runtime/);
});

test('gateway compatibility copy stays aligned with official Claude and Gemini gateway expectations', () => {
  const gatewayServices = read('packages/sdkwork-router-portal-gateway/src/services/index.ts');
  const quickSetup = read('packages/sdkwork-router-portal-api-keys/src/services/quickSetup.ts');

  assert.match(gatewayServices, /anthropic-version/);
  assert.match(gatewayServices, /anthropic-beta/);
  assert.match(quickSetup, /GOOGLE_GEMINI_BASE_URL/);
  assert.match(quickSetup, /GEMINI_API_KEY_AUTH_MECHANISM/);
});

test('user and account modules are separated into personal identity and financial posture', () => {
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');
  const userComponents = read('packages/sdkwork-router-portal-user/src/components/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const accountRepository = read('packages/sdkwork-router-portal-account/src/repository/index.ts');
  const accountLegacyFactsPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-account',
    'src',
    'components',
    'index.tsx',
  );

  assert.match(userPage, /data-slot="portal-user-toolbar"/);
  assert.match(userPage, /User details/);
  assert.match(userPage, /Profile overview/);
  assert.match(userPage, /Phone binding/);
  assert.match(userPage, /WeChat binding/);
  assert.match(userPage, /Privacy preferences/);
  assert.match(userPage, /Password and authentication/);
  assert.match(userPage, /Change password/);
  assert.doesNotMatch(userPage, /Personal security checklist/);
  assert.doesNotMatch(userPage, /Profile facts/);
  assert.doesNotMatch(userPage, /portalx-summary-card/);
  assert.doesNotMatch(userPage, /portal-shell-info-card/);
  assert.doesNotMatch(userComponents, /portalx-fact-list/);

  assert.match(accountPage, /portal-account-toolbar/);
  assert.match(accountPage, /Search account history/);
  assert.match(accountPage, /Revenue/);
  assert.match(accountPage, /Today/);
  assert.match(accountPage, /This month/);
  assert.doesNotMatch(accountPage, /Account posture/);
  assert.match(accountPage, /Account history/);
  assert.match(accountPage, /TabsTrigger value="all"/);
  assert.match(accountRepository, /getPortalCommerceMembership/);
  assert.match(accountRepository, /getPortalUsageSummary/);
  assert.match(accountRepository, /listPortalUsageRecords/);
  assert.doesNotMatch(accountPage, /Membership posture/);
  assert.doesNotMatch(accountPage, /Remaining units:/);
  assert.equal(existsSync(accountLegacyFactsPath), false);
});

test('portal workspaces remove top section heroes so pages open directly on real content', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.doesNotMatch(dashboardPage, /SectionHero/);
  assert.doesNotMatch(usagePage, /SectionHero/);
  assert.doesNotMatch(routingPage, /SectionHero/);
  assert.doesNotMatch(apiKeysPage, /SectionHero/);
  assert.doesNotMatch(billingPage, /SectionHero/);
  assert.doesNotMatch(creditsPage, /SectionHero/);
  assert.doesNotMatch(userPage, /SectionHero/);
  assert.doesNotMatch(accountPage, /SectionHero/);
  assert.doesNotMatch(dashboardPage, /portalx-status-row/);
  assert.doesNotMatch(usagePage, /portalx-status-row/);
  assert.doesNotMatch(routingPage, /portalx-status-row/);
  assert.doesNotMatch(apiKeysPage, /portalx-status-row/);
  assert.doesNotMatch(billingPage, /portalx-status-row/);
  assert.doesNotMatch(creditsPage, /portalx-status-row/);
  assert.doesNotMatch(userPage, /portalx-status-row/);
  assert.doesNotMatch(accountPage, /portalx-status-row/);
  assert.doesNotMatch(dashboardPage, /StatCard/);
  assert.match(usagePage, /StatCard/);
  assert.doesNotMatch(routingPage, /StatCard/);
  assert.doesNotMatch(billingPage, /StatCard/);
  assert.doesNotMatch(creditsPage, /StatCard/);
  assert.doesNotMatch(userPage, /StatCard/);
  assert.match(accountPage, /AccountMetricCard/);
  assert.match(dashboardPage, /Balance/);
  assert.match(dashboardPage, /Revenue/);
  assert.match(usagePage, /Total requests/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.match(usagePage, /data-slot="portal-usage-table"/);
  assert.match(usagePage, /data-slot="portal-usage-pagination"/);
  assert.doesNotMatch(usagePage, /Manage keys/);
  assert.doesNotMatch(usagePage, /Review billing/);
  assert.doesNotMatch(usagePage, /WorkspacePanel/);
  assert.doesNotMatch(usagePage, /Search usage/);
  assert.match(routingPage, /Routing workbench/);
  assert.match(routingPage, /data-slot="portal-routing-toolbar"/);
  assert.match(routingPage, /data-slot="portal-routing-filter-bar"/);
  assert.match(apiKeysPage, /PortalApiKeyDrawers/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-toolbar"/);
  assert.match(billingPage, /Decision support/);
  assert.match(creditsPage, /Redeem history/);
  assert.match(userPage, /User details/);
  assert.match(accountPage, /Search account history/);
  assert.match(accountPage, /Revenue/);
  assert.doesNotMatch(accountPage, /Account posture/);
  assert.match(accountPage, /Account history/);
  assert.doesNotMatch(accountPage, /Financial posture/);
  assert.doesNotMatch(accountPage, /Remaining units:/);
});

test('portal api key workspace uses a backend-style paginated table and drawer flows', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const components = read('packages/sdkwork-router-portal-api-keys/src/components/index.tsx');
  const createForm = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
  );
  const managedNotice = read(
    'packages/sdkwork-router-portal-api-keys/src/components/ApiKeyManagedNoticeCard.tsx',
  );
  const table = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const drawers = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx');

  assert.match(components, /PortalApiKeyDrawers/);
  assert.match(components, /PortalApiKeyCreateForm/);
  assert.match(components, /ApiKeyManagedNoticeCard/);
  assert.match(components, /buildPortalApiKeyTableConfig/);
  assert.doesNotMatch(components, /PortalApiKeyManagerToolbar/);
  assert.match(apiKeysPage, /PortalApiKeyDrawers/);
  assert.match(apiKeysPage, /Create API key/);
  assert.match(apiKeysPage, /Search API keys/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-toolbar"/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-pagination"/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-table"/);
  assert.match(apiKeysPage, /PortalApiKeyTable/);
  assert.doesNotMatch(apiKeysPage, /data-slot="portal-api-key-status"/);
  assert.doesNotMatch(apiKeysPage, /Open usage/);
  assert.doesNotMatch(apiKeysPage, /Refresh inventory/);
  assert.doesNotMatch(apiKeysPage, /SectionHeader/);
  assert.doesNotMatch(apiKeysPage, /WorkspacePanel/);
  assert.doesNotMatch(apiKeysPage, /CrudWorkbench/);
  assert.doesNotMatch(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.match(table, /View details/);
  assert.match(drawers, /Create API key/);
  assert.match(createForm, /Key label/);
  assert.match(createForm, /Environment boundary/);
  assert.match(createForm, /Gateway key mode/);
  assert.match(createForm, /System generated/);
  assert.match(createForm, /Custom key/);
  assert.match(createForm, /Card/);
  assert.match(createForm, /ApiKeyManagedNoticeCard/);
  assert.doesNotMatch(createForm, /Portal-managed key/);
  assert.match(createForm, /Expires at/);
  assert.match(createForm, /Notes/);
  assert.match(managedNotice, /Portal-managed key/);
  assert.match(drawers, /Drawer/);
  assert.match(drawers, /How to use this key/);
  assert.match(drawers, /Quick setup/);
  assert.match(drawers, /Codex/);
  assert.match(drawers, /Claude Code/);
  assert.match(drawers, /OpenCode/);
  assert.match(drawers, /Gemini/);
  assert.match(drawers, /OpenClaw/);
  assert.match(drawers, /Apply setup/);
  assert.match(apiKeysPage, /data-slot="api-router-page"/);
  assert.match(table, /DataTable/);
  assert.doesNotMatch(table, /if \(!items.length\)/);
  assert.match(table, /Portal managed/);
  assert.match(
    createForm,
    /Card className="border-zinc-200 bg-zinc-50\/80 shadow-none dark:border-zinc-800 dark:bg-zinc-900\/50"/,
  );
  assert.doesNotMatch(apiKeysPage, /Global API keys/);
  assert.doesNotMatch(apiKeysPage, /Latest plaintext key/);
  assert.doesNotMatch(apiKeysPage, /One-time plaintext available/);
  assert.doesNotMatch(apiKeysPage, /MetricCard/);
  assert.doesNotMatch(apiKeysPage, /return null;/);
  assert.doesNotMatch(apiKeysPage, /Environment strategy/);
  assert.doesNotMatch(apiKeysPage, /Rotation checklist/);
  assert.doesNotMatch(apiKeysPage, /Quickstart snippet/);
});

test('portal tauri bridge exposes native Api Key setup commands for quick setup parity', () => {
  const tauriMain = read('src-tauri/src/main.rs');

  assert.match(tauriMain, /install_api_router_client_setup/);
  assert.match(tauriMain, /list_api_key_instances/);
  assert.match(tauriMain, /runtime_desktop_snapshot/);
});

test('portal shell adds i18n infrastructure and collapsible extra filters for table workbenches', () => {
  const providers = read('packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const frameworkForm = read('packages/sdkwork-router-portal-commons/src/framework/form.tsx');
  const settingsCenter = read('packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(providers, /PortalI18nProvider/);
  assert.doesNotMatch(commons, /export \* from '\.\/framework'/);
  assert.doesNotMatch(frameworkForm, /export function ToolbarInline/);
  assert.doesNotMatch(frameworkForm, /export function ToolbarSearchField/);
  assert.doesNotMatch(frameworkForm, /export function ToolbarField/);
  assert.match(frameworkForm, /FilterBar/);
  assert.match(frameworkForm, /FilterBarSection/);
  assert.match(frameworkForm, /FilterBarActions/);
  assert.match(frameworkForm, /FilterField/);
  assert.match(frameworkForm, /SearchInput/);
  assert.match(settingsCenter, /Language/);
  assert.doesNotMatch(settingsCenter, /Theme preview|Shell preview|SettingsSection|SettingsStatCard/);
  assert.match(usagePage, /FilterField/);
  assert.match(usagePage, /FilterBarSection/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.doesNotMatch(usagePage, /ToolbarSearchField/);
  assert.doesNotMatch(creditsPage, /FilterField/);
  assert.match(accountPage, /SearchInput/);
  assert.match(apiKeysPage, /SearchInput/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-toolbar"/);
});

test('redeem page simplifies into entry, invite rewards, and redemption history without ledger mode switching', () => {
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const tableCount = creditsPage.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 2);
  assert.match(creditsPage, /Invite rewards/);
  assert.match(creditsPage, /Redeem code/);
  assert.match(creditsPage, /My coupons/);
  assert.match(creditsPage, /Reward history/);
  assert.match(creditsPage, /portal-redeem-wallet-table/);
  assert.match(creditsPage, /portal-redeem-reward-history-table/);
  assert.doesNotMatch(creditsPage, /More filters|Hide filters/);
  assert.doesNotMatch(creditsPage, /View mode/);
  assert.doesNotMatch(creditsPage, /portal-redeem-toolbar/);
  assert.doesNotMatch(creditsPage, /portal-redeem-invite-table/);
  assert.doesNotMatch(creditsPage, /Search redeem offers/);
});

test('billing workspace uses shared workspace panels instead of page-local card wrappers', () => {
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');

  assert.match(billingPage, /WorkspacePanel/);
  assert.doesNotMatch(billingPage, /function DecisionCard/);
  assert.doesNotMatch(billingPage, /function InfoPanel/);
  assert.doesNotMatch(billingPage, /function CatalogPanel/);
  assert.doesNotMatch(
    billingPage,
    /rounded-\[32px\] border-zinc-200\/80 bg-white\/92 shadow-\[0_18px_48px_rgba\(15,23,42,0.08\)\]/,
  );
  assert.doesNotMatch(billingPage, /CardHeader/);
  assert.doesNotMatch(billingPage, /CardTitle/);
  assert.doesNotMatch(billingPage, /CardDescription/);
  assert.doesNotMatch(billingPage, /CardContent/);
});

test('user workspace delegates card surface styling to components instead of page-local class constants', () => {
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');
  const userComponents = read('packages/sdkwork-router-portal-user/src/components/index.tsx');

  assert.match(userPage, /UserSummaryCard/);
  assert.match(userPage, /UserDetailCard/);
  assert.match(userPage, /UserSectionCard/);
  assert.doesNotMatch(userPage, /const summaryCardClassName =/);
  assert.doesNotMatch(userPage, /const detailCardClassName =/);
  assert.doesNotMatch(userPage, /const detailCardTitleClassName =/);
  assert.doesNotMatch(userPage, /const detailCardCopyClassName =/);
  assert.match(userComponents, /export function UserSummaryCard/);
  assert.match(userComponents, /export function UserDetailCard/);
  assert.match(userComponents, /export function UserSectionCard/);
});

test('api key table actions use shared button variants instead of local class recipes', () => {
  const apiKeyTable = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');

  assert.doesNotMatch(apiKeyTable, /secondaryButtonClassName/);
  assert.doesNotMatch(apiKeyTable, /subtleButtonClassName/);
  assert.doesNotMatch(apiKeyTable, /dangerButtonClassName/);
  assert.match(apiKeyTable, /variant="secondary"/);
  assert.match(apiKeyTable, /variant="danger"/);
});
