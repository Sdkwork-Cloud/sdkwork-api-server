import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('shared admin commons expose page toolbar, dialog primitives, and form fields for CRUD workbenches', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(commons, /PageToolbar/);
  assert.match(commons, /ToolbarField/);
  assert.match(commons, /ToolbarSearchField/);
  assert.match(commons, /Dialog/);
  assert.match(commons, /DialogContent/);
  assert.match(commons, /DialogFooter/);
  assert.match(commons, /ConfirmDialog/);
  assert.match(commons, /FormField/);
  assert.match(commons, /rounded-\[28px\] border border-zinc-200\/80 bg-white\/92/);
  assert.match(commons, /backdrop-blur-xl/);
  assert.match(commons, /data-slot="table-container"/);
  assert.match(theme, /adminx-toolbar-field/);
  assert.match(theme, /adminx-toolbar-field-label/);
  assert.match(theme, /adminx-dialog-backdrop/);
  assert.match(theme, /adminx-dialog-panel/);
  assert.match(theme, /adminx-confirm-dialog/);
});

test('admin form primitives keep a shadcn-style contract and settings or auth pages consume the shared shells', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const auth = read('packages/sdkwork-router-admin-auth/src/index.tsx');
  const generalSettings = read('packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx');
  const navigationSettings = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const settingsPage = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');

  assert.match(commons, /file:border-0 file:bg-transparent file:text-sm file:font-medium/);
  assert.match(commons, /disabled:cursor-not-allowed disabled:opacity-50/);
  assert.match(commons, /appearance-none/);
  assert.match(commons, /export function SearchInput/);
  assert.match(commons, /paddingLeft:\s*['"]2\.75rem['"]/);
  assert.match(auth, /Label/);
  assert.doesNotMatch(auth, /<label className="adminx-auth-label">/);
  assert.match(generalSettings, /FormField/);
  assert.doesNotMatch(generalSettings, /<label className="grid gap-2">/);
  assert.match(navigationSettings, /Label/);
  assert.match(settingsPage, /SearchInput/);
  assert.doesNotMatch(settingsPage, /<Search className="absolute left-3 top-1\/2/);
});

test('users workbench separates create and edit flows into dedicated dialogs', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(users, /PageToolbar/);
  assert.match(users, /<ToolbarInline className="adminx-toolbar-inline-users">[\s\S]*?<ToolbarSearchField[\s\S]*?<ToolbarDisclosure>/);
  assert.match(users, /<ToolbarInline className="adminx-toolbar-inline-users-filters">[\s\S]*?<ToolbarField label="User type">[\s\S]*?<ToolbarField label="Status">/);
  assert.doesNotMatch(users, /<ToolbarDisclosure>[\s\S]*?adminx-form-grid/);
  assert.match(commons, /ml-auto flex shrink-0 items-center justify-end gap-2\.5 whitespace-nowrap/);
  assert.match(theme, /\.adminx-toolbar-inline-users > \.adminx-toolbar-field-search\s*\{/);
  assert.match(theme, /\.adminx-toolbar-inline-users-filters > \.adminx-toolbar-field\s*\{/);
  assert.match(users, /Dialog/);
  assert.match(users, /DialogContent/);
  assert.match(users, /New operator/);
  assert.match(users, /New portal user/);
  assert.match(users, /ConfirmDialog/);
  assert.match(users, /Edit operator/);
  assert.match(users, /Edit portal user/);
  assert.match(users, /pendingDelete/);
});

test('users workbench stays on a single directory table without split operator and portal sections', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const tableCount = users.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 1);
  assert.doesNotMatch(users, /Operator roster/);
  assert.doesNotMatch(users, /Portal roster/);
});

test('tenants workbench promotes tenant, project, and key issuance into independent actions and dialogs', () => {
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');

  assert.match(tenants, /PageToolbar/);
  assert.match(tenants, /Dialog/);
  assert.match(tenants, /DialogContent/);
  assert.match(tenants, /New tenant/);
  assert.match(tenants, /New project/);
  assert.match(tenants, /Issue gateway key/);
  assert.match(tenants, /ConfirmDialog/);
  assert.match(tenants, /revealedApiKey/);
});

test('tenants workbench stays on a single tenant table instead of stacked tenant, project, and key registries', () => {
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const tableCount = tenants.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 1);
  assert.doesNotMatch(tenants, /Tenant registry/);
  assert.doesNotMatch(tenants, /Project registry/);
  assert.doesNotMatch(tenants, /Gateway key inventory/);
});

test('coupon workbench uses a focused campaign dialog instead of inline form editing', () => {
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');

  assert.match(coupons, /PageToolbar/);
  assert.match(coupons, /Dialog/);
  assert.match(coupons, /DialogContent/);
  assert.match(coupons, /New coupon/);
  assert.match(coupons, /ConfirmDialog/);
  assert.match(coupons, /Edit coupon campaign/);
  assert.match(coupons, /pendingDeleteCoupon/);
});

test('coupon workbench exposes campaign posture, audience coverage, and risk signals while staying table-first', () => {
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const tableCount = coupons.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 1);
  assert.match(coupons, /adminx-stat-grid/);
  assert.match(coupons, /Campaign posture/);
  assert.match(coupons, /Audience coverage/);
  assert.match(coupons, /Remaining coupon quota/);
  assert.match(coupons, /Expiring soon/);
  assert.match(coupons, /Quota health/);
  assert.match(coupons, /ToolbarField/);
});

test('catalog workbench keeps registries primary and moves maintenance into dialogs', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');

  assert.match(catalog, /PageToolbar/);
  assert.match(catalog, /Dialog/);
  assert.match(catalog, /DialogContent/);
  assert.match(catalog, /ConfirmDialog/);
  assert.match(catalog, /New channel/);
  assert.match(catalog, /New provider/);
  assert.match(catalog, /Rotate credential/);
  assert.match(catalog, /New model/);
  assert.doesNotMatch(catalog, /Channel maintenance/);
  assert.doesNotMatch(catalog, /Provider maintenance/);
});

test('catalog pricing workbench exposes standardized price units and billing-dimension guidance for commercial operations', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');

  assert.match(catalog, /Manage pricing/);
  assert.match(catalog, /Default large-model billing unit/);
  assert.match(catalog, /Million tokens/);
  assert.match(catalog, /Thousand tokens/);
  assert.match(catalog, /Image generated/);
  assert.match(catalog, /Audio second/);
  assert.match(catalog, /Video minute/);
  assert.match(catalog, /Music track/);
  assert.match(catalog, /Charge dimensions/);
  assert.match(catalog, /Input, output, cache, and per-request charges can be mixed/);
});

test('catalog workbench collapses registry sprawl into a single switchable directory table', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const tableCount = catalog.match(/<DataTable/g)?.length ?? 0;

  assert.match(catalog, /Catalog lane/);
  assert.match(catalog, /Channel focus/);
  assert.match(catalog, /Catalog workbench/);
  assert.match(catalog, /Manage channel models/);
  assert.match(catalog, /ToolbarField/);
  assert.equal(tableCount, 1);
  assert.match(catalog, /Channel model roster/);
  assert.match(catalog, /Pricing roster/);
  assert.doesNotMatch(catalog, /No channel models available\./);
  assert.doesNotMatch(catalog, /No pricing rows available\./);
  assert.doesNotMatch(catalog, /Channel registry/);
  assert.doesNotMatch(catalog, /Proxy provider registry/);
  assert.doesNotMatch(catalog, /Credential inventory/);
  assert.doesNotMatch(catalog, /Provider variant inventory/);
});

test('operations workbench uses targeted dialogs and preserves runtime status as the primary surface', () => {
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  assert.match(operations, /PageToolbar/);
  assert.match(operations, /Dialog/);
  assert.match(operations, /DialogContent/);
  assert.match(operations, /Targeted reload/);
  assert.match(operations, /Reload runtimes/);
});

test('admin pages remove top section heroes so real workspace content starts immediately', () => {
  const overview = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const apiRouter = read('packages/sdkwork-router-admin-apirouter/src/index.ts');
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  assert.doesNotMatch(overview, /SectionHero/);
  assert.doesNotMatch(users, /SectionHero/);
  assert.doesNotMatch(tenants, /SectionHero/);
  assert.doesNotMatch(coupons, /SectionHero/);
  assert.doesNotMatch(apiRouter, /SectionHero/);
  assert.doesNotMatch(catalog, /SectionHero/);
  assert.doesNotMatch(traffic, /SectionHero/);
  assert.doesNotMatch(operations, /SectionHero/);
  assert.match(overview, /adminx-stat-grid/);
  assert.doesNotMatch(overview, /Data-source posture/);
});

test('gateway parity package exposes the four migrated claw surfaces through a shared package root', () => {
  const apiRouter = read('packages/sdkwork-router-admin-apirouter/src/index.ts');

  assert.match(apiRouter, /GatewayAccessPage/);
  assert.match(apiRouter, /GatewayRoutesPage/);
  assert.match(apiRouter, /GatewayModelMappingsPage/);
  assert.match(apiRouter, /GatewayUsagePage/);
  assert.match(apiRouter, /Api Key/);
  assert.match(apiRouter, /Route Config/);
  assert.match(apiRouter, /Model Mapping/);
  assert.match(apiRouter, /Usage Records/);
});

test('api router workbenches stay table-first and avoid extra registry layers above the grid', () => {
  const accessPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const routesPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx');
  const mappingsPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx');
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const mappingsTableCount = mappingsPage.match(/<DataTable/g)?.length ?? 0;

  assert.doesNotMatch(accessPage, /All statuses|All tenants|All projects/);
  assert.doesNotMatch(routesPage, /Credential registry/);
  assert.doesNotMatch(routesPage, /Rotate credential/);
  assert.doesNotMatch(routesPage, /All channels|Health posture/);
  assert.doesNotMatch(mappingsPage, /Rule preview:/);
  assert.doesNotMatch(mappingsPage, /All statuses/);
  assert.equal(mappingsTableCount, 1);
  assert.match(mappingsPage, /ToolbarField/);
  assert.match(mappingsPage, /Mapping status/);
  assert.match(mappingsPage, /All mappings/);
  assert.match(mappingsPage, /Active mappings/);
  assert.match(mappingsPage, /Disabled mappings/);
  assert.doesNotMatch(mappingsPage, /No mapping rules available\./);
  assert.doesNotMatch(
    usagePage,
    /All providers|All models|<span>Sort by<\/span>|<span>Page size<\/span>|<span>Start date<\/span>|<span>End date<\/span>/,
  );
  assert.match(mappingsPage, /Mapping rules:/);
  assert.match(accessPage, /Create Api key/);
  assert.match(accessPage, /Usage method/);
  assert.match(accessPage, /Quick setup/);
  assert.match(accessPage, /Codex/);
  assert.match(accessPage, /Claude Code/);
  assert.match(accessPage, /OpenCode/);
  assert.match(accessPage, /Gemini/);
  assert.match(accessPage, /OpenClaw/);
  assert.match(accessPage, /Apply setup/);
});

test('settings center copy frames shell continuity instead of a standalone preferences page', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');

  assert.match(settings, /control plane|settings center|workspace/i);
  assert.match(workspace, /right canvas|content region|shell posture/i);
});

test('admin shell adds i18n infrastructure and collapsible extra filters for dense table workbenches', () => {
  const providers = read('packages/sdkwork-router-admin-shell/src/application/providers/AppProviders.tsx');
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const generalSettings = read('packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx');
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const trafficPage = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const usersPage = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const tenantsPage = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const routesPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx');

  assert.match(providers, /AdminI18nProvider/);
  assert.match(commons, /ToolbarDisclosure/);
  assert.match(commons, /ToolbarField/);
  assert.match(commons, /ToolbarSearchField/);
  assert.match(generalSettings, /Language/);
  assert.match(usagePage, /ToolbarDisclosure/);
  assert.match(trafficPage, /ToolbarDisclosure/);
  assert.match(usersPage, /ToolbarSearchField/);
  assert.match(tenantsPage, /ToolbarSearchField/);
  assert.match(routesPage, /ToolbarSearchField/);
});

test('admin disclosure filters stay on a single compact row instead of falling back to form grids', () => {
  const stylesheet = read('packages/sdkwork-router-admin-shell/src/styles/index.css');
  const trafficPage = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const operationsPage = read('packages/sdkwork-router-admin-operations/src/index.tsx');
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');

  assert.match(stylesheet, /\.adminx-toolbar-inline-disclosure\s*\{/);
  assert.match(stylesheet, /\.adminx-toolbar-inline-disclosure > \.adminx-toolbar-field\s*\{/);

  assert.match(
    trafficPage,
    /<ToolbarDisclosure>[\s\S]*?<ToolbarInline className="adminx-toolbar-inline-disclosure">[\s\S]*?<ToolbarField label=\{t\('View mode'\)\}>[\s\S]*?<ToolbarField label=\{t\('Recent window'\)\}>/,
  );
  assert.doesNotMatch(trafficPage, /<ToolbarDisclosure>[\s\S]*?adminx-form-grid/);

  assert.match(
    operationsPage,
    /<ToolbarDisclosure>[\s\S]*?<ToolbarInline className="adminx-toolbar-inline-disclosure">[\s\S]*?<ToolbarField label=\{t\('View mode'\)\}>/,
  );
  assert.doesNotMatch(operationsPage, /<ToolbarDisclosure>[\s\S]*?adminx-form-grid/);

  assert.match(
    usagePage,
    /<ToolbarDisclosure>[\s\S]*?<ToolbarInline className="adminx-toolbar-inline-disclosure">[\s\S]*?<ToolbarField label=\{t\('Api key'\)\}>[\s\S]*?<ToolbarField label=\{t\('Time range'\)\}>/,
  );
  assert.doesNotMatch(usagePage, /<ToolbarDisclosure>[\s\S]*?adminx-form-grid/);
});

test('traffic workbench stays on a single switchable table instead of stacked analytics grids', () => {
  const trafficPage = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const tableCount = trafficPage.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 1);
  assert.doesNotMatch(trafficPage, /adminx-users-grid/);
});

test('operations workbench stays on a single switchable table instead of separate provider and runtime grids', () => {
  const operationsPage = read('packages/sdkwork-router-admin-operations/src/index.tsx');
  const tableCount = operationsPage.match(/<DataTable/g)?.length ?? 0;

  assert.equal(tableCount, 1);
  assert.match(operationsPage, /ToolbarDisclosure/);
});
