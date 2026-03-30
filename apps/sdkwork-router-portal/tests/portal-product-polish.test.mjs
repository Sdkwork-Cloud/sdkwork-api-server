import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
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
  assert.match(authPage, /continueWith|Github|Chrome/);
  assert.doesNotMatch(authPage, /Preview the first launch path/);
  assert.doesNotMatch(authPage, /Start in four moves/);
  assert.doesNotMatch(authPage, /Why teams trust this portal/);
});

test('portal shell keeps workspace context in the rail and moves shell settings into the profile dock', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');

  assert.doesNotMatch(sidebar, /Active workspace/);
  assert.doesNotMatch(layout, /ShellStatus/);
  assert.match(header, /WindowControls/);
  assert.doesNotMatch(header, /Portal Workspace/);
  assert.doesNotMatch(header, /Current workspace|Workspace context/);
  assert.doesNotMatch(header, /Config center/);
  assert.doesNotMatch(header, /Workspace shell/);
  assert.match(profileDock, /data-slot="portal-sidebar-footer-settings"/);
  assert.match(profileDock, /data-slot="portal-sidebar-user-control"/);
  assert.doesNotMatch(profileDock, /Active workspace/);
  assert.match(profileDock, /Sign out/);
  assert.match(routes, /Routing/);
  assert.doesNotMatch(sidebar, /Need help\?/);
});

test('dashboard follows claw-studio analytics workbench architecture adapted to portal telemetry', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const dashboardComponents = read('packages/sdkwork-router-portal-dashboard/src/components/index.tsx');
  const dashboardRepository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');
  const dualColumnSectionCount = (
    dashboardPage.match(/xl:grid-cols-\[1\.35fr_0\.95fr\]/g) ?? []
  ).length;

  assert.match(dashboardComponents, /DashboardSummaryCard/);
  assert.match(dashboardComponents, /DashboardSectionHeader/);
  assert.match(dashboardComponents, /DashboardRevenueTrendChart/);
  assert.match(dashboardComponents, /DashboardTokenTrendChart/);
  assert.match(dashboardComponents, /DashboardDistributionRingChart/);
  assert.match(dashboardComponents, /DashboardModelDistributionChart/);
  assert.match(dashboardPage, /Traffic posture/);
  assert.match(dashboardPage, /Cost and quota/);
  assert.match(dashboardPage, /Workspace readiness/);
  assert.match(dashboardPage, /Analytics workbench/);
  assert.match(dashboardPage, /Routing evidence/);
  assert.match(dashboardPage, /Next actions/);
  assert.match(dashboardPage, /Module posture/);
  assert.match(dashboardPage, /Recent requests/);
  assert.match(dashboardPage, /Provider distribution/);
  assert.match(dashboardPage, /Model distribution/);
  assert.match(dashboardPage, /const surfaceClass =/);
  assert.ok(
    dualColumnSectionCount >= 2,
    'dashboard should repeat the claw-studio dual-column panel rhythm',
  );
  assert.match(dashboardPage, /data-slot="portal-dashboard-workbench-tabs"/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-grid/);
  assert.doesNotMatch(dashboardPage, /ResponsiveContainer/);
  assert.doesNotMatch(dashboardPage, /Surface/);
  assert.match(dashboardRepository, /getPortalRoutingSummary/);
  assert.match(dashboardRepository, /listPortalRoutingDecisionLogs/);
  assert.doesNotMatch(dashboardPage, /Traffic overview/);
  assert.doesNotMatch(dashboardPage, /Workspace modules/);
  assert.doesNotMatch(dashboardPage, /Recent activity/);
});

test('credits and billing pages expose runway and guardrail decision support', () => {
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsRepository = read('packages/sdkwork-router-portal-credits/src/repository/index.ts');
  const billingRepository = read('packages/sdkwork-router-portal-billing/src/repository/index.ts');

  assert.match(creditsPage, /portal-credits-toolbar/);
  assert.match(creditsPage, /Eligible offers/);
  assert.match(creditsPage, /Potential bonus units/);
  assert.match(creditsPage, /Quota pressure/);
  assert.match(creditsPage, /Search offers or ledger/);
  assert.match(billingPage, /Active membership/);
  assert.match(billingPage, /Estimated runway/);
  assert.match(billingPage, /Recommended bundle/);
  assert.match(billingPage, /Pending payment queue/);
  assert.match(billingPage, /Checkout session/);
  assert.match(billingPage, /Open session/);
  assert.match(billingPage, /Payment rail/);
  assert.match(billingPage, /Operator settlement/);
  assert.match(billingPage, /Provider callbacks/);
  assert.match(billingPage, /Simulate provider settlement/);
  assert.match(billingPage, /Simulate provider failure/);
  assert.match(billingPage, /Failed payment/);
  assert.match(billingPage, /Settle order/);
  assert.match(billingPage, /Cancel order/);
  assert.match(billingPage, /Order timeline/);
  assert.match(creditsPage, /Redeem now|Loading preview/);
  assert.match(creditsRepository, /previewPortalCommerceQuote/);
  assert.match(creditsRepository, /createPortalCommerceOrder/);
  assert.match(billingRepository, /getPortalCommerceMembership/);
  assert.match(billingRepository, /previewPortalCommerceQuote/);
  assert.match(billingRepository, /createPortalCommerceOrder/);
  assert.match(billingRepository, /getPortalCommerceCheckoutSession/);
  assert.match(billingRepository, /sendPortalCommercePaymentEvent/);
  assert.match(billingRepository, /settlePortalCommerceOrder/);
  assert.match(billingRepository, /cancelPortalCommerceOrder/);
  assert.match(billingRepository, /listPortalCommerceOrders/);
});

test('gateway command center makes compatibility, deployment modes, and commerce readiness explicit', () => {
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const gatewayRepository = read('packages/sdkwork-router-portal-gateway/src/repository/index.ts');
  const gatewayServices = read('packages/sdkwork-router-portal-gateway/src/services/index.ts');
  const gatewayTypes = read('packages/sdkwork-router-portal-gateway/src/types/index.ts');
  const gatewayComponents = read('packages/sdkwork-router-portal-gateway/src/components/index.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const tauriMain = read('src-tauri/src/main.rs');

  assert.match(appRoutes, /sdkwork-router-portal-gateway/);
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
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const accountRepository = read('packages/sdkwork-router-portal-account/src/repository/index.ts');

  assert.match(userPage, /Personal security checklist/);
  assert.match(userPage, /Password rotation/);
  assert.match(userPage, /Profile facts/);

  assert.match(accountPage, /portal-account-toolbar/);
  assert.match(accountPage, /Search ledger/);
  assert.match(accountPage, /Membership posture/);
  assert.match(accountRepository, /getPortalCommerceMembership/);
  assert.doesNotMatch(accountPage, /Remaining units:/);
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
  assert.doesNotMatch(dashboardPage, /MetricCard/);
  assert.match(usagePage, /MetricCard/);
  assert.doesNotMatch(routingPage, /MetricCard/);
  assert.doesNotMatch(billingPage, /MetricCard/);
  assert.match(creditsPage, /MetricCard/);
  assert.doesNotMatch(userPage, /MetricCard/);
  assert.match(accountPage, /MetricCard/);
  assert.match(dashboardPage, /Traffic posture/);
  assert.match(usagePage, /Total requests/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.match(usagePage, /Manage keys/);
  assert.match(usagePage, /Review billing/);
  assert.doesNotMatch(usagePage, /Search usage/);
  assert.match(routingPage, /Routing workbench/);
  assert.match(routingPage, /data-slot="portal-routing-toolbar"/);
  assert.match(routingPage, /data-slot="portal-routing-filter-bar"/);
  assert.match(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.match(billingPage, /Decision support/);
  assert.match(creditsPage, /Search offers or ledger/);
  assert.match(userPage, /Profile facts/);
  assert.match(accountPage, /Search ledger/);
  assert.match(accountPage, /Financial posture/);
  assert.match(accountPage, /Ledger overview/);
  assert.doesNotMatch(accountPage, /Remaining units:/);
});

test('portal api key workspace uses a manager toolbar, filter bar, and usage dialog flow inspired by claw api-router', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const components = read('packages/sdkwork-router-portal-api-keys/src/components/index.tsx');
  const createForm = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
  );
  const toolbar = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx');
  const table = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const dialogs = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx');

  assert.match(components, /PortalApiKeyDialogs/);
  assert.match(components, /PortalApiKeyCreateForm/);
  assert.match(toolbar, /Create API key/);
  assert.match(toolbar, /Open usage/);
  assert.match(toolbar, /Search API keys/);
  assert.doesNotMatch(toolbar, /All environments/);
  assert.match(apiKeysPage, /Usage method/);
  assert.match(dialogs, /Create API key/);
  assert.match(createForm, /Key label/);
  assert.match(createForm, /Environment boundary/);
  assert.match(createForm, /Gateway key mode/);
  assert.match(createForm, /System generated/);
  assert.match(createForm, /Custom key/);
  assert.match(createForm, /Portal-managed key/);
  assert.match(createForm, /Expires at/);
  assert.match(createForm, /Notes/);
  assert.match(dialogs, /How to use this key/);
  assert.match(dialogs, /Quick setup/);
  assert.match(dialogs, /Codex/);
  assert.match(dialogs, /Claude Code/);
  assert.match(dialogs, /OpenCode/);
  assert.match(dialogs, /Gemini/);
  assert.match(dialogs, /OpenClaw/);
  assert.match(dialogs, /Apply setup/);
  assert.match(apiKeysPage, /data-slot="api-router-page"/);
  assert.match(apiKeysPage, /bg-zinc-50 dark:bg-zinc-950/);
  assert.match(toolbar, /rounded-\[28px\] border border-zinc-200\/80 bg-white\/92 p-4 shadow-\[0_18px_48px_rgba\(15,23,42,0\.08\)\] backdrop-blur dark:border-zinc-800\/80 dark:bg-zinc-950\/70/);
  assert.match(table, /DataTable/);
  assert.doesNotMatch(table, /if \(!items.length\)/);
  assert.match(table, /Portal managed/);
  assert.match(createForm, /rounded-\[28px\] border border-zinc-200 bg-zinc-50\/80 p-5 dark:border-zinc-800 dark:bg-zinc-900\/50/);
  assert.doesNotMatch(apiKeysPage, /Global API keys/);
  assert.doesNotMatch(apiKeysPage, /Latest plaintext key/);
  assert.doesNotMatch(apiKeysPage, /One-time plaintext available/);
  assert.doesNotMatch(apiKeysPage, /MetricCard/);
  assert.doesNotMatch(apiKeysPage, /Rotation checklist/);
  assert.doesNotMatch(apiKeysPage, /Environment strategy/);
  assert.doesNotMatch(apiKeysPage, /TabsTrigger value="coverage"/);
  assert.doesNotMatch(apiKeysPage, /TabsTrigger value="rotation"/);
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
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const apiKeyToolbar = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx');

  assert.match(providers, /PortalI18nProvider/);
  assert.match(commons, /ToolbarDisclosure/);
  assert.match(commons, /ToolbarField/);
  assert.match(commons, /ToolbarSearchField/);
  assert.match(configCenter, /Language/);
  assert.doesNotMatch(configCenter, /Theme preview|Shell preview|SettingsSection|SettingsStatCard/);
  assert.match(usagePage, /ToolbarField/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.doesNotMatch(usagePage, /ToolbarDisclosure/);
  assert.doesNotMatch(usagePage, /ToolbarSearchField/);
  assert.match(creditsPage, /ToolbarField/);
  assert.match(accountPage, /ToolbarSearchField/);
  assert.match(apiKeyToolbar, /ToolbarSearchField/);
});

test('credits workbench stays on a single switchable table instead of parallel offers and ledger grids', () => {
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const tableCount = creditsPage.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 1);
  assert.match(creditsPage, /Offer state/);
  assert.doesNotMatch(creditsPage, /ToolbarDisclosure/);
});
