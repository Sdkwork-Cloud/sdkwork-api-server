import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

export async function assertProductVerificationWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'product-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/product-verification.yml');

  const workflow = read(repoRoot, path.join('.github', 'workflows', 'product-verification.yml'));

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(
    workflow,
    /workflow_dispatch:\s*[\s\S]*?env:\s*[\s\S]*?FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'[\s\S]*?jobs:/,
    'product verification workflow must opt GitHub JavaScript actions into the Node 24 runtime to avoid Node 20 deprecation drift on hosted runners',
  );
  assert.match(
    workflow,
    /\.github\/workflows\/release\.yml/,
    'product verification workflow must watch the release workflow because release packaging contract changes are product-surface changes',
  );
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /pnpm\/action-setup@v4/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  assert.match(workflow, /\.github\/workflows\/product-verification\.yml/);
  assert.match(workflow, /apps\/sdkwork-router-admin\/\*\*/);
  assert.match(workflow, /apps\/sdkwork-router-portal\/\*\*/);
  assert.match(workflow, /docs\/\*\*/);
  assert.doesNotMatch(
    workflow,
    /console\/\*\*/,
    'product verification workflow should not treat the legacy console workspace as an official product verification trigger surface',
  );
  assert.match(workflow, /README\.md/);
  assert.match(workflow, /README\.zh-CN\.md/);
  assert.match(workflow, /scripts\/check-router-product\.mjs/);
  assert.match(
    workflow,
    /scripts\/browser-runtime-smoke\.mjs/,
    'product verification workflow must watch the shared browser runtime smoke helper',
  );
  assert.match(
    workflow,
    /scripts\/browser-runtime-smoke\.test\.mjs/,
    'product verification workflow must watch the shared browser runtime smoke helper test',
  );
  assert.match(
    workflow,
    /scripts\/check-portal-browser-runtime\.mjs/,
    'product verification workflow must watch the portal browser runtime smoke entrypoint',
  );
  assert.match(
    workflow,
    /scripts\/check-portal-browser-runtime\.test\.mjs/,
    'product verification workflow must watch the portal browser runtime smoke test',
  );
  assert.match(
    workflow,
    /scripts\/check-admin-browser-runtime\.mjs/,
    'product verification workflow must watch the admin browser runtime smoke entrypoint',
  );
  assert.match(
    workflow,
    /scripts\/check-admin-browser-runtime\.test\.mjs/,
    'product verification workflow must watch the admin browser runtime smoke test',
  );
  assert.match(
    workflow,
    /scripts\/run-tauri-cli\.mjs/,
    'product verification workflow must watch the shared desktop runtime helper',
  );
  assert.match(
    workflow,
    /scripts\/prepare-router-portal-desktop-runtime\.mjs/,
    'product verification workflow must watch the portal desktop runtime helper staging script',
  );
  assert.match(
    workflow,
    /scripts\/prepare-router-portal-desktop-runtime\.test\.mjs/,
    'product verification workflow must watch the portal desktop runtime helper staging contract test',
  );
  assert.match(
    workflow,
    /scripts\/release\/\*\*/,
    'product verification workflow must watch the shared release-packaging helper subtree',
  );
  assert.match(
    workflow,
    /scripts\/release-flow-contract\.test\.mjs/,
    'product verification workflow must watch the release flow contract test',
  );
  assert.match(
    workflow,
    /scripts\/release\/desktop-targets\.mjs/,
    'product verification workflow must watch the shared desktop target helper',
  );
  assert.match(
    workflow,
    /scripts\/product-verification-workflow-contracts\.mjs/,
    'product verification workflow must watch the contract module',
  );
  assert.match(workflow, /scripts\/product-verification-workflow\.test\.mjs/);
  assert.match(
    workflow,
    /Run product governance node tests[\s\S]*?node --test scripts\/product-verification-workflow\.test\.mjs scripts\/check-router-product\.test\.mjs scripts\/browser-runtime-smoke\.test\.mjs scripts\/check-admin-browser-runtime\.test\.mjs scripts\/check-portal-browser-runtime\.test\.mjs scripts\/build-router-desktop-assets\.test\.mjs scripts\/check-router-docs-safety\.test\.mjs scripts\/check-router-frontend-budgets\.test\.mjs scripts\/dev\/tests\/pnpm-launch-lib\.test\.mjs scripts\/prepare-router-portal-desktop-runtime\.test\.mjs scripts\/release-flow-contract\.test\.mjs scripts\/release\/tests\/materialize-release-catalog\.test\.mjs scripts\/release\/tests\/release-workflow\.test\.mjs scripts\/release\/tests\/release-attestation-verify\.test\.mjs scripts\/release\/tests\/docs-product-contract\.test\.mjs apps\/sdkwork-router-portal\/tests\/product-entrypoint-scripts\.test\.mjs/,
    'product verification workflow must run workflow, packaging, product, and shared pnpm helper tests before the main product gate',
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*?env:[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile/,
    'product verification workflow must materialize only referenced external release dependencies before frozen frontend installs so workspace-linked packages resolve on GitHub runners without cloning unrelated governance-only repositories',
  );
  assert.match(
    workflow,
    /Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile/,
    'product verification workflow must use explicit frozen installs for the official admin and portal workspaces',
  );
  assert.doesNotMatch(
    workflow,
    /console\/tests\/sdk-transport-unsafe-integer\.test\.mjs|console\/pnpm-lock\.yaml/,
    'product verification workflow must not include legacy console-specific test or dependency inputs',
  );
  assert.match(
    workflow,
    /Run product verification gate[\s\S]*?env:[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?run:\s*node scripts\/check-router-product\.mjs/,
    'strict frontend install mode must be exported before the product verification gate runs',
  );
  assert.match(
    workflow,
    /docs\/pnpm-lock\.yaml/,
    'product verification workflow must cache the docs lockfile because docs build is part of the governed public documentation surface',
  );
  assert.match(
    workflow,
    /Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile/,
    'product verification workflow must use an explicit frozen install for the docs workspace before building the public docs site',
  );
  assert.match(
    workflow,
    /Build docs site[\s\S]*?pnpm --dir docs build/,
    'product verification workflow must build the public docs site before the node contract suite runs',
  );
}
