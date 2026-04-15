import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function extractMap(source, name) {
  const start = source.indexOf(`const ${name}: Record<string, string> = {`);
  assert.notEqual(start, -1, `missing map ${name}`);

  const open = source.indexOf('{', start);
  const close = source.indexOf('\n};', open);
  assert.notEqual(close, -1, `missing closing brace for ${name}`);

  const body = source.slice(open + 1, close);
  return new Map(
    [...body.matchAll(/\n\s*"([^"]+)":\s*(?:"([^"]*)"|\n\s*"([^"]*)"),/g)].map((match) => [
      match[1],
      match[2] ?? match[3] ?? '',
    ]),
  );
}

function buildTranslationUsagePattern(key) {
  const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`t\\(\\s*'${escapedKey}'\\s*(?:,|\\))`, 's');
}

test('apirouter access and routing shell copy are overridden by a dedicated zh-CN surface slice', () => {
  const sources = [
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyGroupsDialog.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingSnapshotsDialog.tsx'),
  ];
  const joinedSources = sources.join('\n');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');
  const apirouterSurfaceTranslations = extractMap(
    i18n,
    'ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS',
  );

  const expectedKeys = [
    'API key groups',
    'Define reusable policy groups for workspace-scoped key issuance, routing posture, and accounting defaults.',
    'Search groups',
    'Create group',
    'No groups match the current filter',
    'Broaden the query or create a new policy group for this workspace scope.',
    'Pin the workspace boundary for this reusable key governance group.',
    'Edit group',
    'Leave empty to derive a slug from the group name.',
    'Policy defaults',
    'Examples: chat,responses or images,audio.',
    'No accounting override',
    'Bind to an active routing profile inside the same workspace scope when needed.',
    'No routing profile override',
    'Group updated. Review the refreshed policy state on the left.',
    'Group created. Review the refreshed policy state on the left.',
    'Group disabled. New key assignments now require a different policy group.',
    'Group enabled. Keys in this workspace scope can bind to it again.',
    'Group deleted. Review the refreshed policy inventory.',
    'Failed to save API key group.',
    'Failed to update API key group status.',
    'Failed to delete API key group.',
    'Delete API key group',
    'Delete group',
    'Delete {name}. Keys already bound to this group will need a new policy assignment before future updates.',
    'Enable group',
    'Disable group',
    'Save group',
    'Slug',
    'Color',
    'Default scope',
    'Routing profile',
    'Open',
    'No default scope',
    'Accounting mode',
    'Routing profiles',
    'Capture reusable routing posture so API key groups and workspace policy can bind to a named profile instead of repeating provider order, latency, and health rules.',
    'Search routing profiles',
    'Create profile',
    'No routing profiles match the current filter',
    'Broaden the query or create the first reusable routing policy for this workspace.',
    'Creating a new profile from {name}. Adjust the scope, provider order, or routing constraints before saving.',
    'Pin the workspace boundary and profile metadata before defining routing posture.',
    'Leave empty to derive a slug automatically from the profile name.',
    'Keep the new profile immediately selectable by API key groups after creation.',
    'Routing posture',
    'Set the route selection behavior, region preference, and SLO limits that the gateway should inherit from this profile.',
    'Preferred region',
    'No preferred region',
    'Max cost',
    'Max latency (ms)',
    'Require healthy',
    'Only keep healthy providers in the candidate set when this profile is applied.',
    'Provider order',
    'Choose which providers belong to the profile, then arrange the fallback chain explicitly.',
    'Create route providers first, then return to build a reusable routing profile.',
    'Default provider',
    'Default',
    'Move up',
    'Move down',
    'Use as template',
    'Routing profile created. Review the refreshed policy inventory on the left.',
    'Failed to create routing profile.',
    'Save routing profile',
    'Auto',
    'Compiled snapshots',
    'Inspect the compiled routing evidence that the gateway produced after combining policy, project defaults, and API key group routing profile overlays.',
    'All compiled route snapshots currently loaded into the admin workspace.',
    'Applied routing profile',
    'Snapshots that carry an applied routing profile id.',
    'Bound groups',
    'Route keys',
    'Distinct route keys represented across the compiled snapshot evidence set.',
    'Search compiled snapshots',
    'No applied routing profile',
    'All profiles',
    'No API key group scope',
    'No ordered providers',
    'No default provider',
    'No compiled snapshots match the current filter',
    'Broaden the query or refresh the workspace after new routing decisions land.',
  ];

  for (const key of expectedKeys) {
    assert.match(
      joinedSources,
      buildTranslationUsagePattern(key),
      `expected apirouter surface to render ${key} through t(...)`,
    );
    assert.ok(
      apirouterSurfaceTranslations.has(key),
      `expected dedicated apirouter surface translation key ${key}`,
    );
    assert.notEqual(
      apirouterSurfaceTranslations.get(key),
      key,
      `expected apirouter surface translation ${key} to be localized instead of English`,
    );
  }

  assert.match(i18n, /const ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS: Record<string, string> = \{/);
  assert.match(i18nTranslations, /\.\.\.ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS,/);
});
