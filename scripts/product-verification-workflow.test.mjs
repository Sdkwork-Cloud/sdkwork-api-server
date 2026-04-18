import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function extractNamedStepBlock(containerText, stepName) {
  const stepPattern = new RegExp(
    String.raw`^\s+- name: ${stepName}\r?\n[\s\S]*?(?=^\s+- name:|\Z)`,
    'im',
  );
  const match = containerText.match(stepPattern);
  assert.ok(match, `missing ${stepName} step`);
  return match[0];
}

function withNode24JavaScriptActionsEnv(workflowText) {
  if (/FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/.test(workflowText)) {
    return workflowText;
  }

  return workflowText.replace(
    /\r?\njobs:\r?\n/,
    `\n\nenv:\n  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'\n\njobs:\n`,
  );
}

test('repository exposes a pull-request product verification workflow with governed installs and strict mode', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'product-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/product-verification.yml');

  const workflow = read('.github/workflows/product-verification.yml');

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/);
  assert.match(workflow, /\.github\/workflows\/release\.yml/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /pnpm\/action-setup@v4/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  assert.match(workflow, /docs\/pnpm-lock\.yaml/);
  assert.match(workflow, /\.github\/workflows\/product-verification\.yml/);
  assert.match(workflow, /apps\/sdkwork-router-admin\/\*\*/);
  assert.match(workflow, /apps\/sdkwork-router-portal\/\*\*/);
  assert.match(workflow, /docs\/\*\*/);
  assert.doesNotMatch(workflow, /console\/\*\*/);
  assert.match(workflow, /README\.md/);
  assert.match(workflow, /README\.zh-CN\.md/);
  assert.match(workflow, /scripts\/check-router-product\.mjs/);
  assert.match(workflow, /scripts\/browser-runtime-smoke\.mjs/);
  assert.match(workflow, /scripts\/browser-runtime-smoke\.test\.mjs/);
  assert.match(workflow, /scripts\/check-portal-browser-runtime\.mjs/);
  assert.match(workflow, /scripts\/check-portal-browser-runtime\.test\.mjs/);
  assert.match(workflow, /scripts\/check-admin-browser-runtime\.mjs/);
  assert.match(workflow, /scripts\/check-admin-browser-runtime\.test\.mjs/);
  assert.match(workflow, /scripts\/run-tauri-cli\.mjs/);
  assert.match(workflow, /scripts\/prepare-router-portal-desktop-runtime\.mjs/);
  assert.match(workflow, /scripts\/prepare-router-portal-desktop-runtime\.test\.mjs/);
  assert.match(workflow, /scripts\/release\/\*\*/);
  assert.match(workflow, /scripts\/release-flow-contract\.test\.mjs/);
  assert.match(workflow, /scripts\/release\/desktop-targets\.mjs/);
  assert.match(workflow, /scripts\/product-verification-workflow-contracts\.mjs/);
  assert.match(workflow, /scripts\/product-verification-workflow\.test\.mjs/);
  assert.match(
    workflow,
    /Run product governance node tests[\s\S]*?node --test scripts\/product-verification-workflow\.test\.mjs scripts\/check-router-product\.test\.mjs scripts\/browser-runtime-smoke\.test\.mjs scripts\/check-admin-browser-runtime\.test\.mjs scripts\/check-portal-browser-runtime\.test\.mjs scripts\/build-router-desktop-assets\.test\.mjs scripts\/check-router-docs-safety\.test\.mjs scripts\/check-router-frontend-budgets\.test\.mjs scripts\/dev\/tests\/pnpm-launch-lib\.test\.mjs scripts\/prepare-router-portal-desktop-runtime\.test\.mjs scripts\/release-flow-contract\.test\.mjs scripts\/release\/tests\/materialize-release-catalog\.test\.mjs scripts\/release\/tests\/release-workflow\.test\.mjs scripts\/release\/tests\/release-attestation-verify\.test\.mjs scripts\/release\/tests\/docs-product-contract\.test\.mjs apps\/sdkwork-router-portal\/tests\/product-entrypoint-scripts\.test\.mjs/,
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*?env:[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile/,
  );
  assert.match(
    workflow,
    /Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile/,
  );
  assert.match(
    workflow,
    /Build docs site[\s\S]*?pnpm --dir docs build/,
  );
  assert.doesNotMatch(workflow, /console\/tests\/sdk-transport-unsafe-integer\.test\.mjs/);
  assert.doesNotMatch(workflow, /console\/pnpm-lock\.yaml/);
  assert.match(
    workflow,
    /Run product verification gate[\s\S]*?env:[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?run:\s*node scripts\/check-router-product\.mjs/,
  );
});

test('product verification workflow defers pnpm version selection to the root packageManager field', () => {
  const rootPackage = JSON.parse(read('package.json'));
  const pnpmSetupStep = extractNamedStepBlock(
    read('.github/workflows/product-verification.yml'),
    'Setup pnpm',
  );

  assert.equal(rootPackage.packageManager, 'pnpm@10.30.2');
  assert.doesNotMatch(
    pnpmSetupStep,
    /^\s+version:/m,
  );
});

test('product verification workflow contract helper rejects workflows that do not opt JavaScript actions into Node 24', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    read('.github/workflows/product-verification.yml').replace(
      /^env:\r?\n\s+FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'\r?\n\r?\n/m,
      '',
    ),
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /node 24|javascript actions/i,
  );
});

test('product verification workflow contract helper rejects workflows without strict frontend install mode', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    withNode24JavaScriptActionsEnv(`
name: product-verification

on:
  pull_request:
    paths:
      - '.github/workflows/product-verification.yml'
      - '.github/workflows/release.yml'
      - 'apps/sdkwork-router-admin/**'
      - 'apps/sdkwork-router-portal/**'
      - 'docs/**'
      - 'README.md'
      - 'README.zh-CN.md'
      - 'scripts/check-router-product.mjs'
      - 'scripts/browser-runtime-smoke.mjs'
      - 'scripts/browser-runtime-smoke.test.mjs'
      - 'scripts/check-portal-browser-runtime.mjs'
      - 'scripts/check-portal-browser-runtime.test.mjs'
      - 'scripts/check-admin-browser-runtime.mjs'
      - 'scripts/check-admin-browser-runtime.test.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.test.mjs'
      - 'scripts/release/**'
      - 'scripts/release-flow-contract.test.mjs'
      - 'scripts/run-tauri-cli.mjs'
      - 'scripts/release/desktop-targets.mjs'
      - 'scripts/product-verification-workflow-contracts.mjs'
      - 'scripts/product-verification-workflow.test.mjs'
  workflow_dispatch:

jobs:
  product-verification:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 10

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        uses: taiki-e/install-action@cargo-audit

      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs

      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile

      - name: Run product governance node tests
        run: node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/browser-runtime-smoke.test.mjs scripts/check-admin-browser-runtime.test.mjs scripts/check-portal-browser-runtime.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/materialize-release-catalog.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-attestation-verify.test.mjs scripts/release/tests/docs-product-contract.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs

      - name: Run product verification gate
        run: node scripts/check-router-product.mjs
`),
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /strict frontend install mode/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not watch the contract module', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    withNode24JavaScriptActionsEnv(`
name: product-verification

on:
  pull_request:
    paths:
      - '.github/workflows/product-verification.yml'
      - '.github/workflows/release.yml'
      - 'apps/sdkwork-router-admin/**'
      - 'apps/sdkwork-router-portal/**'
      - 'docs/**'
      - 'README.md'
      - 'README.zh-CN.md'
      - 'scripts/check-router-product.mjs'
      - 'scripts/browser-runtime-smoke.mjs'
      - 'scripts/browser-runtime-smoke.test.mjs'
      - 'scripts/check-portal-browser-runtime.mjs'
      - 'scripts/check-portal-browser-runtime.test.mjs'
      - 'scripts/check-admin-browser-runtime.mjs'
      - 'scripts/check-admin-browser-runtime.test.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.test.mjs'
      - 'scripts/release/**'
      - 'scripts/release-flow-contract.test.mjs'
      - 'scripts/run-tauri-cli.mjs'
      - 'scripts/release/desktop-targets.mjs'
      - 'scripts/product-verification-workflow.test.mjs'
  workflow_dispatch:

jobs:
  product-verification:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 10

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        uses: taiki-e/install-action@cargo-audit

      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs

      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile

      - name: Run product governance node tests
        run: node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/browser-runtime-smoke.test.mjs scripts/check-admin-browser-runtime.test.mjs scripts/check-portal-browser-runtime.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/materialize-release-catalog.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-attestation-verify.test.mjs scripts/release/tests/docs-product-contract.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs

      - name: Run product verification gate
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs
`),
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /contract module/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not watch product desktop/runtime helper inputs', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    withNode24JavaScriptActionsEnv(`
name: product-verification

on:
  pull_request:
    paths:
      - '.github/workflows/product-verification.yml'
      - '.github/workflows/release.yml'
      - 'apps/sdkwork-router-admin/**'
      - 'apps/sdkwork-router-portal/**'
      - 'docs/**'
      - 'README.md'
      - 'README.zh-CN.md'
      - 'scripts/check-router-product.mjs'
      - 'scripts/browser-runtime-smoke.mjs'
      - 'scripts/browser-runtime-smoke.test.mjs'
      - 'scripts/check-portal-browser-runtime.mjs'
      - 'scripts/check-portal-browser-runtime.test.mjs'
      - 'scripts/check-admin-browser-runtime.mjs'
      - 'scripts/check-admin-browser-runtime.test.mjs'
      - 'scripts/release/**'
      - 'scripts/release-flow-contract.test.mjs'
      - 'scripts/product-verification-workflow-contracts.mjs'
      - 'scripts/product-verification-workflow.test.mjs'
  workflow_dispatch:

jobs:
  product-verification:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 10

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        uses: taiki-e/install-action@cargo-audit

      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs

      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile

      - name: Run product governance node tests
        run: node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/browser-runtime-smoke.test.mjs scripts/check-admin-browser-runtime.test.mjs scripts/check-portal-browser-runtime.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/materialize-release-catalog.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-attestation-verify.test.mjs scripts/release/tests/docs-product-contract.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs

      - name: Run product verification gate
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs
`),
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /desktop runtime helper/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not run the shared pnpm helper tests', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    withNode24JavaScriptActionsEnv(`
name: product-verification

on:
  pull_request:
    paths:
      - '.github/workflows/product-verification.yml'
      - '.github/workflows/release.yml'
      - 'apps/sdkwork-router-admin/**'
      - 'apps/sdkwork-router-portal/**'
      - 'docs/**'
      - 'README.md'
      - 'README.zh-CN.md'
      - 'scripts/check-router-product.mjs'
      - 'scripts/browser-runtime-smoke.mjs'
      - 'scripts/browser-runtime-smoke.test.mjs'
      - 'scripts/check-portal-browser-runtime.mjs'
      - 'scripts/check-portal-browser-runtime.test.mjs'
      - 'scripts/check-admin-browser-runtime.mjs'
      - 'scripts/check-admin-browser-runtime.test.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.test.mjs'
      - 'scripts/release/**'
      - 'scripts/release-flow-contract.test.mjs'
      - 'scripts/run-tauri-cli.mjs'
      - 'scripts/release/desktop-targets.mjs'
      - 'scripts/dev/pnpm-launch-lib.mjs'
      - 'scripts/dev/tests/pnpm-launch-lib.test.mjs'
      - 'scripts/product-verification-workflow-contracts.mjs'
      - 'scripts/product-verification-workflow.test.mjs'
  workflow_dispatch:

jobs:
  product-verification:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 10

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        uses: taiki-e/install-action@cargo-audit

      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile

      - name: Run product governance node tests
        run: node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/browser-runtime-smoke.test.mjs scripts/check-admin-browser-runtime.test.mjs scripts/check-portal-browser-runtime.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/materialize-release-catalog.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-attestation-verify.test.mjs scripts/release/tests/docs-product-contract.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs

      - name: Run product verification gate
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs
`),
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /shared pnpm helper/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not build the docs site', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    withNode24JavaScriptActionsEnv(`
name: product-verification

on:
  pull_request:
    paths:
      - '.github/workflows/product-verification.yml'
      - '.github/workflows/release.yml'
      - 'apps/sdkwork-router-admin/**'
      - 'apps/sdkwork-router-portal/**'
      - 'docs/**'
      - 'README.md'
      - 'README.zh-CN.md'
      - 'scripts/check-router-product.mjs'
      - 'scripts/browser-runtime-smoke.mjs'
      - 'scripts/browser-runtime-smoke.test.mjs'
      - 'scripts/check-portal-browser-runtime.mjs'
      - 'scripts/check-portal-browser-runtime.test.mjs'
      - 'scripts/check-admin-browser-runtime.mjs'
      - 'scripts/check-admin-browser-runtime.test.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.mjs'
      - 'scripts/prepare-router-portal-desktop-runtime.test.mjs'
      - 'scripts/release/**'
      - 'scripts/release-flow-contract.test.mjs'
      - 'scripts/run-tauri-cli.mjs'
      - 'scripts/release/desktop-targets.mjs'
      - 'scripts/dev/pnpm-launch-lib.mjs'
      - 'scripts/dev/tests/pnpm-launch-lib.test.mjs'
      - 'scripts/product-verification-workflow-contracts.mjs'
      - 'scripts/product-verification-workflow.test.mjs'
  workflow_dispatch:

jobs:
  product-verification:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 10

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        uses: taiki-e/install-action@cargo-audit

      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs

      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile

      - name: Run product governance node tests
        run: node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/browser-runtime-smoke.test.mjs scripts/check-admin-browser-runtime.test.mjs scripts/check-portal-browser-runtime.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/materialize-release-catalog.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-attestation-verify.test.mjs scripts/release/tests/docs-product-contract.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs

      - name: Run product verification gate
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs
`),
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /docs site/i,
  );
});
