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
  assert.match(workflow, /README\.md/);
  assert.match(workflow, /README\.zh-CN\.md/);
  assert.match(workflow, /scripts\/check-router-product\.mjs/);
  assert.match(
    workflow,
    /scripts\/run-tauri-cli\.mjs/,
    'product verification workflow must watch the shared desktop runtime helper',
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
    /Run product governance node tests[\s\S]*?node --test scripts\/product-verification-workflow\.test\.mjs scripts\/check-router-product\.test\.mjs scripts\/build-router-desktop-assets\.test\.mjs scripts\/check-router-docs-safety\.test\.mjs scripts\/check-router-frontend-budgets\.test\.mjs scripts\/dev\/tests\/pnpm-launch-lib\.test\.mjs apps\/sdkwork-router-portal\/tests\/product-entrypoint-scripts\.test\.mjs/,
    'product verification workflow must run workflow, product, and shared pnpm helper tests before the main product gate',
  );
  assert.match(
    workflow,
    /Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile/,
    'product verification workflow must use explicit frozen installs for admin and portal workspaces',
  );
  assert.match(
    workflow,
    /Run product verification gate[\s\S]*?env:[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?run:\s*node scripts\/check-router-product\.mjs/,
    'strict frontend install mode must be exported before the product verification gate runs',
  );
}
