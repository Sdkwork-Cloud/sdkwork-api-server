import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadRoutingServices() {
  const load = jiti(import.meta.url, {
    moduleCache: false,
    alias: {
      'sdkwork-router-portal-commons/i18n-core': path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'i18n-core.ts',
      ),
    },
  });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-routing',
      'src',
      'services',
      'index.ts',
    ),
  );
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

  assert.match(routingPage, /data-slot="portal-routing-toolbar"/);
  assert.match(routingPage, /ManagementWorkbench/);
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
  assert.doesNotMatch(routingPage, /<SectionHeader/);
  assert.doesNotMatch(routingPage, /<Surface/);
  assert.doesNotMatch(routingPage, /<Tabs/);
  assert.doesNotMatch(routingPage, /Policy editor/);
  assert.doesNotMatch(routingPage, /Recent routing evidence/);
  assert.match(routingServices, /first healthy available provider in your ordered list wins/);
});

test('routing dialogs, actions, and status feedback localize through shared portal i18n', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(routingPage, /usePortalI18n/);
  assert.match(routingPage, /<Dialog open=\{editDialogOpen\}/);
  assert.match(routingPage, /<Dialog open=\{previewDialogOpen\}/);
  assert.match(routingPage, /<DialogTitle>\{t\('Edit routing posture'\)\}<\/DialogTitle>/);
  assert.match(routingPage, /label=\{t\('Routing profile label'\)\}/);
  assert.match(routingPage, /label=\{t\('Requested model'\)\}/);
  assert.match(routingPage, /placeholder=\{t\('Optional deterministic seed'\)\}/);
  assert.match(routingPage, /t\('Saving routing preferences for this project\.\.\.'\)/);
  assert.match(
    routingPage,
    /t\(\s*'Routing posture saved\. The workbench now reflects the updated provider order and guardrails\.'/,
  );
  assert.match(
    routingPage,
    /t\(\s*'Preview updated with the current routing posture and added to the evidence stream\.'/,
  );
  assert.match(routingPage, /data-slot="portal-routing-toolbar"/);
  assert.match(routingPage, /t\('Edit posture'\)/);
  assert.match(routingPage, /t\('Run preview'\)/);
  assert.match(routingPage, /t\('Manage routing profiles'\)/);
  assert.doesNotMatch(routingPage, /title=\{t\('Routing'\)\}/);
  assert.match(routingPage, /title=\{t\('Preparing routing workbench'\)\}/);

  assert.match(commons, /'Edit routing posture'/);
  assert.match(commons, /'Manage routing profiles'/);
  assert.match(commons, /'Routing profile label'/);
  assert.match(commons, /'Optional deterministic seed'/);
  assert.match(commons, /'Saving routing preferences for this project\.\.\.'/);
  assert.match(commons, /'Preparing routing workbench'/);
});

test('routing profile workbench copy localizes reusable-profile inventory and save-from-posture flows', () => {
  const dialog = read(
    'packages/sdkwork-router-portal-routing/src/components/PortalRoutingProfilesDialog.tsx',
  );
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(dialog, /t\('Routing profiles'\)/);
  assert.match(dialog, /t\('Current posture'\)/);
  assert.match(dialog, /t\('Save current posture'\)/);
  assert.match(dialog, /t\('Use as posture'\)/);
  assert.match(dialog, /t\('Save as profile'\)/);
  assert.match(dialog, /t\('No routing profiles yet'\)/);
  assert.match(
    dialog,
    /t\(\s*'Review reusable routing profiles for this workspace and save the current routing posture as a reusable profile for API key groups\.'/,
  );
  assert.match(
    dialog,
    /t\(\s*'The current routing strategy, provider order, and guardrails will be captured into a reusable profile for this workspace\.'/,
  );
  assert.match(commons, /'Current posture'/);
  assert.match(commons, /'Save current posture'/);
  assert.match(commons, /'Use as posture'/);
  assert.match(commons, /'Save as profile'/);
  assert.match(commons, /'No routing profiles yet'/);
});

test('routing snapshot evidence copy localizes compiled snapshot inventory and filter states through shared portal i18n', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const dialog = read(
    'packages/sdkwork-router-portal-routing/src/components/PortalRoutingSnapshotsDialog.tsx',
  );
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(routingPage, /t\('View compiled snapshots'\)/);
  assert.match(dialog, /t\('Compiled snapshots'\)/);
  assert.match(dialog, /t\('Search compiled snapshots'\)/);
  assert.match(dialog, /t\('Applied routing profile'\)/);
  assert.match(dialog, /t\('Bound API key group'\)/);
  assert.match(dialog, /t\('No applied routing profile'\)/);
  assert.match(dialog, /t\('No API key group scope'\)/);
  assert.match(
    dialog,
    /t\(\s*'Inspect the compiled routing evidence for this workspace after policy, project defaults, and API key group profile overlays are combined\.'/,
  );
  assert.match(
    dialog,
    /t\(\s*'No compiled snapshots match the current filter'\)/,
  );
  assert.match(
    dialog,
    /t\(\s*'Broaden the query or refresh the workspace after new routing decisions land\.'/,
  );

  assert.match(commons, /'View compiled snapshots'/);
  assert.match(commons, /'Compiled snapshots'/);
  assert.match(commons, /'Search compiled snapshots'/);
  assert.match(commons, /'Bound API key group'/);
  assert.match(commons, /'No compiled snapshots match the current filter'/);
});

test('routing preview and evidence stream localize fallback and compiled snapshot linkage through shared portal i18n', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(routingPage, /label:\s*t\('Compiled snapshot'\)/);
  assert.match(routingPage, /label:\s*t\('Fallback posture'\)/);
  assert.match(routingPage, /t\('No snapshot captured'\)/);
  assert.match(routingPage, /t\('No fallback used'\)/);
  assert.match(routingPage, /t\('Open snapshot evidence'\)/);
  assert.match(
    routingPage,
    /t\(\s*'Selection evidence is linked to the compiled route state when a snapshot id is available\.'/,
  );
  assert.match(
    routingPage,
    /t\(\s*'Fallback reasoning is preserved so operators can distinguish degraded routing from normal preference selection\.'/,
  );
  assert.match(commons, /'Compiled snapshot'/);
  assert.match(commons, /'Fallback posture'/);
  assert.match(commons, /'No snapshot captured'/);
  assert.match(commons, /'No fallback used'/);
  assert.match(commons, /'Open snapshot evidence'/);
});

test('routing summary panels localize lower workbench insights and preview assessments through shared portal i18n', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(
    routingPage,
    /t\(\s*'Guardrail posture keeps cost, latency, regional preference, and the latest routing signals readable before you publish changes\.'/,
  );
  assert.match(routingPage, /title=\{t\('Guardrail posture'\)\}/);
  assert.match(routingPage, /t\(\s*'Latest routing signals'/);
  assert.match(
    routingPage,
    /t\(\s*'Preview and live traces stay adjacent to guardrails so posture changes remain explainable without secondary tabs\.'/,
  );
  assert.match(routingPage, /title:\s*t\('Preview outcome'\)/);
  assert.match(
    routingPage,
    /t\(\s*'Preview outcome keeps the selected provider, fallback path, and provider assessments visible before traffic posture is saved\.'/,
  );
  assert.match(routingPage, /t\(\s*'Candidate assessments'/);
  assert.match(
    routingPage,
    /t\(\s*'Selection evidence stays operationally readable so support teams can validate health, latency, and policy posture before rollout\.'/,
  );
  assert.match(routingPage, /t\(\s*'No routing signals yet'/);
  assert.match(
    routingPage,
    /t\(\s*'Run a preview or wait for live traffic to collect routing signals\.'/,
  );
  assert.match(routingPage, /t\(\s*'No preview assessments yet'/);
  assert.match(
    routingPage,
    /t\(\s*'Run a preview to inspect provider-level candidate assessments\.'/,
  );
  assert.match(routingPage, /t\(\s*'Preview only'/);
  assert.match(routingPage, /t\(\s*'Degraded fallback'/);
  assert.match(routingPage, /t\(\s*'Guardrails applied'/);

  assert.match(commons, /'Guardrail posture'/);
  assert.match(commons, /'Latest routing signals'/);
  assert.match(commons, /'Preview outcome'/);
  assert.match(commons, /'Candidate assessments'/);
  assert.match(commons, /'No routing signals yet'/);
  assert.match(commons, /'No preview assessments yet'/);
});

test('routing summary cards and workbench row statuses localize through shared portal i18n instead of inline english labels', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const routingServices = read('packages/sdkwork-router-portal-routing/src/services/index.ts');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(routingPage, /label:\s*t\('Active posture'\)/);
  assert.match(routingPage, /label:\s*t\('Default provider'\)/);
  assert.match(routingPage, /value:\s*form\.default_provider_id \?\? t\('Auto fallback'\)/);
  assert.match(routingPage, /label:\s*t\('Preview model'\)/);
  assert.match(routingPage, /label:\s*t\('Evidence entries'\)/);
  assert.match(routingPage, /\{preset\.active \? t\('Active'\) : t\('Available'\)\}/);
  assert.match(routingPage, /\? t\('Degraded'\)/);
  assert.match(routingPage, /\? t\('Guardrailed'\)/);
  assert.match(routingPage, /\? t\('Preview'\)/);
  assert.match(routingPage, /: t\('Live'\)/);
  assert.match(routingPage, /\{log\.matched_policy_id \?\? t\('No matched policy'\)\}/);
  assert.match(routingPage, /\{isDefault \? t\('Default'\) : t\('Ordered'\)\}/);
  assert.match(
    routingPage,
    /t\(\s*'Default provider stays available as the stable fallback when several providers remain eligible\.'/,
  );
  assert.match(
    routingPage,
    /t\(\s*'Ordered providers keep deterministic failover readable for operators and support teams\.'/,
  );
  assert.match(routingPage, /header:\s*t\(workbenchConfig\.actionsLabel\)/);
  assert.match(routingPage, /header:\s*t\(workbenchConfig\.scopeLabel\)/);
  assert.match(routingPage, /header:\s*t\(workbenchConfig\.detailLabel\)/);
  assert.match(routingPage, /cell:\s*\(row\)\s*=>\s*row\.actions/);
  assert.match(routingServices, /translatePortalText/);

  assert.match(commons, /'Active posture'/);
  assert.match(commons, /'Preview model'/);
  assert.match(commons, /'Evidence entries'/);
  assert.match(commons, /'No matched policy'/);
  assert.match(commons, /'Ordered providers keep deterministic failover readable for operators and support teams\.'/);
});

test('routing view model tolerates missing preview assessments from live payloads', () => {
  const { buildPortalRoutingViewModel } = loadRoutingServices();
  const now = Date.now();

  const viewModel = buildPortalRoutingViewModel(
    {
      project_id: 'project-demo',
      preferences: {
        project_id: 'project-demo',
        preset_id: 'balanced',
        strategy: 'geo_affinity',
        ordered_provider_ids: ['provider-a'],
        default_provider_id: 'provider-a',
        max_cost: null,
        max_latency_ms: null,
        require_healthy: true,
        preferred_region: 'us-east',
        updated_at_ms: now,
      },
      latest_model_hint: 'gpt-4o-mini',
      preview: {
        selected_provider_id: 'provider-a',
        candidate_ids: ['provider-a'],
        matched_policy_id: null,
        strategy: 'geo_affinity',
        selection_seed: 7,
        selection_reason: 'provider-a matched the region',
        requested_region: 'us-east',
        slo_applied: false,
        slo_degraded: false,
      },
      provider_options: [
        {
          provider_id: 'provider-a',
          display_name: 'Provider A',
          channel_id: 'openai',
          preferred: true,
          default_provider: true,
        },
      ],
    },
    [
      {
        decision_id: 'decision-1',
        decision_source: 'preview',
        capability: 'chat',
        route_key: 'chat',
        selected_provider_id: 'provider-a',
        strategy: 'geo_affinity',
        selection_seed: 7,
        selection_reason: 'provider-a matched the region',
        requested_region: 'us-east',
        slo_applied: false,
        slo_degraded: false,
        created_at_ms: now,
      },
    ],
  );

  assert.deepEqual(viewModel.preview.assessments, []);
  assert.deepEqual(viewModel.logs[0].assessments, []);
});

test('routing view model preserves compiled snapshot ids and fallback reasons inside evidence rows', () => {
  const { buildPortalRoutingViewModel } = loadRoutingServices();
  const now = Date.now();

  const viewModel = buildPortalRoutingViewModel(
    {
      project_id: 'project-demo',
      preferences: {
        project_id: 'project-demo',
        preset_id: 'balanced',
        strategy: 'geo_affinity',
        ordered_provider_ids: ['provider-a'],
        default_provider_id: 'provider-a',
        max_cost: null,
        max_latency_ms: null,
        require_healthy: true,
        preferred_region: 'us-east',
        updated_at_ms: now,
      },
      latest_model_hint: 'gpt-4o-mini',
      preview: {
        selected_provider_id: 'provider-a',
        candidate_ids: ['provider-a', 'provider-b'],
        matched_policy_id: 'policy-live',
        compiled_routing_snapshot_id: 'snapshot-preview',
        strategy: 'geo_affinity',
        selection_seed: 7,
        selection_reason: 'provider-a matched the region',
        fallback_reason: 'provider-b was retained as regional fallback',
        requested_region: 'us-east',
        slo_applied: false,
        slo_degraded: false,
      },
      provider_options: [
        {
          provider_id: 'provider-a',
          display_name: 'Provider A',
          channel_id: 'openai',
          preferred: true,
          default_provider: true,
        },
      ],
    },
    [
      {
        decision_id: 'decision-1',
        decision_source: 'preview',
        capability: 'chat',
        route_key: 'chat',
        selected_provider_id: 'provider-a',
        matched_policy_id: 'policy-live',
        compiled_routing_snapshot_id: 'snapshot-preview',
        strategy: 'geo_affinity',
        selection_seed: 7,
        selection_reason: 'provider-a matched the region',
        fallback_reason: 'provider-b was retained as regional fallback',
        requested_region: 'us-east',
        slo_applied: false,
        slo_degraded: false,
        created_at_ms: now,
      },
    ],
  );

  assert.equal(viewModel.preview.compiled_routing_snapshot_id, 'snapshot-preview');
  assert.equal(viewModel.preview.fallback_reason, 'provider-b was retained as regional fallback');
  assert.equal(viewModel.evidence[0].snapshot_id, 'snapshot-preview');
  assert.equal(viewModel.evidence[0].fallback_reason, 'provider-b was retained as regional fallback');
});

test('routing view model tolerates missing decision log collections', () => {
  const { buildPortalRoutingViewModel } = loadRoutingServices();
  const now = Date.now();

  const viewModel = buildPortalRoutingViewModel(
    {
      project_id: 'project-demo',
      preferences: {
        project_id: 'project-demo',
        preset_id: 'balanced',
        strategy: 'deterministic_priority',
        ordered_provider_ids: [],
        default_provider_id: null,
        max_cost: null,
        max_latency_ms: null,
        require_healthy: true,
        preferred_region: null,
        updated_at_ms: now,
      },
      latest_model_hint: 'gpt-4o-mini',
      preview: {
        selected_provider_id: 'provider-a',
        candidate_ids: ['provider-a'],
        matched_policy_id: null,
        strategy: 'deterministic_priority',
        selection_seed: null,
        selection_reason: 'provider-a is the default provider',
        requested_region: null,
        slo_applied: false,
        slo_degraded: false,
      },
      provider_options: [],
    },
    undefined,
  );

  assert.deepEqual(viewModel.logs, []);
  assert.deepEqual(viewModel.evidence, []);
});
