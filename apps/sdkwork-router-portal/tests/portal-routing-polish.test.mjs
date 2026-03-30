import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal API SDK exposes routing summary, preferences, preview, and evidence calls', () => {
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');

  assert.match(portalApi, /getPortalRoutingSummary/);
  assert.match(portalApi, /getPortalRoutingPreferences/);
  assert.match(portalApi, /savePortalRoutingPreferences/);
  assert.match(portalApi, /previewPortalRouting/);
  assert.match(portalApi, /listPortalRoutingDecisionLogs/);
});

test('portal shared types expose routing contracts and expanded route keys', () => {
  const types = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(types, /'routing'/);
  assert.match(types, /'user'/);
  assert.match(types, /interface PortalRoutingSummary/);
  assert.match(types, /interface PortalRoutingPreferences/);
  assert.match(types, /interface PortalRoutingDecision/);
  assert.match(types, /interface PortalRoutingDecisionLog/);
});

test('routing module speaks in user-facing routing posture language', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const routingServices = read('packages/sdkwork-router-portal-routing/src/services/index.ts');

  assert.match(routingPage, /Routing workbench/);
  assert.match(routingPage, /Preset catalog/);
  assert.match(routingPage, /Provider roster/);
  assert.match(routingPage, /Evidence stream/);
  assert.match(routingPage, /Edit routing posture/);
  assert.match(routingPage, /Preview route/);
  assert.match(routingPage, /Routing profile label/);
  assert.match(routingPage, /Capability/);
  assert.match(routingPage, /Requested model/);
  assert.match(routingPage, /Selection seed/);
  assert.match(routingPage, /Search routing evidence/);
  assert.match(routingPage, /Save posture/);
  assert.doesNotMatch(routingPage, /<Tabs/);
  assert.doesNotMatch(routingPage, /Policy editor/);
  assert.doesNotMatch(routingPage, /Recent routing evidence/);
  assert.match(routingServices, /first healthy available provider in your ordered list wins/);
});
