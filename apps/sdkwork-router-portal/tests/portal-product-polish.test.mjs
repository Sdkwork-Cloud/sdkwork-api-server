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
  assert.match(profileDock, /Settings/);
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

  assert.match(creditsPage, /Redemption guardrails/);
  assert.match(creditsPage, /Recommended offer/);
  assert.match(billingPage, /Estimated runway/);
  assert.match(billingPage, /Recommended bundle/);
});

test('user and account modules are separated into personal identity and financial posture', () => {
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(userPage, /Personal security checklist/);
  assert.match(userPage, /Password rotation/);
  assert.match(userPage, /Profile facts/);

  assert.match(accountPage, /Financial account/);
  assert.match(accountPage, /Cash balance/);
  assert.match(accountPage, /Ledger evidence/);
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
  assert.doesNotMatch(usagePage, /MetricCard/);
  assert.doesNotMatch(routingPage, /MetricCard/);
  assert.doesNotMatch(billingPage, /MetricCard/);
  assert.doesNotMatch(creditsPage, /MetricCard/);
  assert.doesNotMatch(userPage, /MetricCard/);
  assert.doesNotMatch(accountPage, /MetricCard/);
  assert.match(dashboardPage, /Traffic posture/);
  assert.match(usagePage, /Request volume/);
  assert.match(routingPage, /Routing posture presets/);
  assert.match(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.match(billingPage, /Decision support/);
  assert.match(creditsPage, /Recommended offer/);
  assert.match(userPage, /Profile facts/);
  assert.match(accountPage, /Cash balance/);
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
  assert.match(toolbar, /Search API keys/);
  assert.match(toolbar, /All environments/);
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
  assert.match(apiKeysPage, /data-slot="api-router-page"/);
  assert.match(apiKeysPage, /bg-zinc-50 dark:bg-zinc-950/);
  assert.match(toolbar, /rounded-\[28px\] border border-zinc-200\/80 bg-white\/92 p-4 shadow-\[0_18px_48px_rgba\(15,23,42,0\.08\)\] backdrop-blur dark:border-zinc-800\/80 dark:bg-zinc-950\/70/);
  assert.match(table, /rounded-\[32px\] border border-zinc-200\/80 bg-white\/92 shadow-\[0_18px_48px_rgba\(15,23,42,0\.08\)\] backdrop-blur dark:border-zinc-800\/80 dark:bg-zinc-950\/70/);
  assert.match(table, /bg-zinc-50\/90 dark:bg-zinc-900\/80/);
  assert.match(table, /Portal managed/);
  assert.match(createForm, /rounded-\[28px\] border border-zinc-200 bg-zinc-50\/80 p-5 dark:border-zinc-800 dark:bg-zinc-900\/50/);
  assert.doesNotMatch(apiKeysPage, /Global API keys/);
  assert.doesNotMatch(apiKeysPage, /Latest plaintext key/);
  assert.doesNotMatch(apiKeysPage, /MetricCard/);
  assert.doesNotMatch(apiKeysPage, /Rotation checklist/);
  assert.doesNotMatch(apiKeysPage, /Environment strategy/);
  assert.doesNotMatch(apiKeysPage, /TabsTrigger value="coverage"/);
  assert.doesNotMatch(apiKeysPage, /TabsTrigger value="rotation"/);
});
