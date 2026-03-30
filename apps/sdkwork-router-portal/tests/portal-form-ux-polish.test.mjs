import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('form-heavy portal pages use focused dialogs and workbenches instead of always-expanded forms', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const apiKeyDialogs = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx',
  );
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');

  assert.match(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.match(apiKeysPage, /PortalApiKeyDialogs/);
  assert.match(apiKeyDialogs, /<Dialog/);
  assert.match(apiKeyDialogs, /Create API key/);
  assert.match(apiKeyDialogs, /Lifecycle policy/);
  assert.match(apiKeyDialogs, /How to use this key/);

  assert.match(routingPage, /data-slot="portal-routing-toolbar"/);
  assert.match(routingPage, /data-slot="portal-routing-filter-bar"/);
  assert.match(routingPage, /Edit posture/);
  assert.match(routingPage, /Run preview/);
  assert.match(routingPage, /Evidence stream/);
  assert.match(routingPage, /<Dialog/);
  assert.doesNotMatch(routingPage, /<Tabs/);
  assert.doesNotMatch(routingPage, /Policy editor/);

  assert.match(userPage, /<Tabs/);
  assert.match(userPage, /<Dialog/);
  assert.match(userPage, /Security center/);
});

test('portal api key create form reuses shared shadcn-style form primitives instead of local wrappers', () => {
  const createForm = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx');

  assert.match(createForm, /sdkwork-router-portal-commons/);
  assert.match(createForm, /Button/);
  assert.match(createForm, /Input/);
  assert.match(createForm, /Select/);
  assert.match(createForm, /Textarea/);
  assert.doesNotMatch(createForm, /function TextInput/);
  assert.doesNotMatch(createForm, /function SelectInput/);
  assert.doesNotMatch(createForm, /function TextArea/);
  assert.doesNotMatch(createForm, /<button/);
  assert.doesNotMatch(createForm, /<textarea/);
});

test('portal config center and api key table actions also reuse shared Button primitives', () => {
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');
  const apiKeyTable = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');

  assert.match(configCenter, /Button/);
  assert.match(apiKeyTable, /Button|InlineButton/);
  assert.doesNotMatch(configCenter, /<button/);
  assert.doesNotMatch(apiKeyTable, /<button/);
});
