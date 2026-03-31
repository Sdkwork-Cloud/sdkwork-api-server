import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal api key creation and usage flows localize user-facing copy through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const createForm = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
  );
  const dialogs = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx',
  );
  const table = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const page = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(createForm, /usePortalI18n/);
  assert.match(createForm, /label=\{t\('Key label'\)\}/);
  assert.match(createForm, /placeholder=\{t\('Production rollout'\)\}/);
  assert.match(createForm, /label=\{t\('Gateway key mode'\)\}/);
  assert.match(createForm, /t\('Creating API key\.\.\.'\)/);

  assert.match(dialogs, /usePortalI18n/);
  assert.match(dialogs, /<DialogTitle>\{t\('Create API key'\)\}<\/DialogTitle>/);
  assert.match(dialogs, /usagePreview\?\.title \? t\(usagePreview\.title\) : t\('Usage method'\)/);
  assert.match(dialogs, /t\('How to use this key'\)/);
  assert.match(dialogs, /t\('Apply setup'\)/);

  assert.match(table, /usePortalI18n/);
  assert.match(table, /label: t\('Name'\)/);
  assert.match(table, /t\('No API keys yet'\)/);
  assert.match(table, /t\('Usage method'\)/);

  assert.match(page, /usePortalI18n/);
  assert.match(page, /t\('Key label is required so credentials remain auditable after creation\.'\)/);
  assert.match(page, /t\('Plaintext key copied to clipboard\.'\)/);
  assert.match(page, /t\('Applying \{label\} setup\.\.\.', \{ label: selectedPlan\.label \}\)/);

  assert.match(commons, /'Key label'/);
  assert.match(commons, /'Gateway key mode'/);
  assert.match(commons, /'Creating API key\.\.\.'/);
  assert.match(commons, /'Usage method'/);
  assert.match(commons, /'No API keys yet'/);
  assert.match(commons, /'Plaintext key copied to clipboard\.'/);
});
