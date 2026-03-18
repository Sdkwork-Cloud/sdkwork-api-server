import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal auth entry exposes a launch-cockpit narrative instead of plain auth copy', () => {
  const authComponents = read('packages/sdkwork-router-portal-auth/src/components/index.tsx');
  const authPages = read('packages/sdkwork-router-portal-auth/src/pages/index.tsx');

  assert.match(authComponents, /Start in four moves/);
  assert.match(authComponents, /Why teams trust this portal/);
  assert.match(authPages, /Preview the first launch path/);
});

test('portal shell keeps workspace context in the rail and moves shell settings into the profile dock', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');
  const shellStatus = read('packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx');
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');

  assert.match(sidebar, /Active workspace/);
  assert.match(shellStatus, /Workspace status/);
  assert.match(header, /Portal Workspace/);
  assert.match(header, /WindowControls/);
  assert.doesNotMatch(header, /Config center/);
  assert.doesNotMatch(header, /Workspace shell/);
  assert.match(profileDock, /Settings/);
  assert.match(profileDock, /Sign out/);
  assert.match(routes, /Routing/);
  assert.doesNotMatch(sidebar, /Need help\?/);
});

test('dashboard follows a simplified SaaS overview architecture', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const dashboardRepository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');

  assert.match(dashboardPage, /Traffic overview/);
  assert.match(dashboardPage, /Routing posture/);
  assert.match(dashboardPage, /Quick actions/);
  assert.match(dashboardPage, /Workspace modules/);
  assert.match(dashboardPage, /Recent activity/);
  assert.match(dashboardPage, /Recent requests/);
  assert.match(dashboardPage, /Provider share/);
  assert.match(dashboardPage, /Model demand/);
  assert.match(dashboardRepository, /getPortalRoutingSummary/);
  assert.match(dashboardRepository, /listPortalRoutingDecisionLogs/);
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
  assert.match(dashboardPage, /Quick actions/);
  assert.match(usagePage, /Refine view/);
  assert.match(routingPage, /Save posture/);
  assert.match(apiKeysPage, /Create key/);
  assert.match(billingPage, /Checkout preview/);
  assert.match(creditsPage, /Redeem credits/);
  assert.match(userPage, /Password rotation/);
  assert.match(accountPage, /Financial account/);
});
