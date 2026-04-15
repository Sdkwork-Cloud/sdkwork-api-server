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

test('repository exposes a pull-request release governance workflow that watches its contract surface', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release-governance.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release-governance.yml');

  const workflow = read('.github/workflows/release-governance.yml');

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /\.github\/workflows\/release-governance\.yml/);
  assert.match(workflow, /scripts\/release\/\*\*/);
  assert.match(workflow, /scripts\/release-governance-workflow-contracts\.mjs/);
  assert.match(workflow, /scripts\/release-governance-workflow\.test\.mjs/);
  assert.match(workflow, /docs\/架构\/135-可观测性与SLO治理设计-2026-04-07\.md/);
  assert.match(workflow, /docs\/架构\/143-全局架构对齐与收口计划-2026-04-08\.md/);
  assert.match(
    workflow,
    /run:\s*node --test --experimental-test-isolation=none scripts\/release\/tests\/release-governance-runner\.test\.mjs/,
  );
  assert.match(
    workflow,
    /run:\s*node scripts\/release\/run-release-governance-checks\.mjs --profile preflight --format json/,
  );
});

test('release governance workflow contract helper rejects workflows that do not watch the contract module', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    `
name: release-governance

on:
  pull_request:
    paths:
      - '.github/workflows/release.yml'
      - '.github/workflows/release-governance.yml'
      - 'scripts/release/**'
      - 'scripts/release-governance-workflow.test.mjs'
      - 'bin/**'
      - 'docs/release/**'
  workflow_dispatch:

jobs:
  release-governance:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Run release governance checks
        run: node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /contract module/i,
  );
});

test('release governance workflow contract helper rejects workflows that do not watch the governed SLO architecture docs', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    `
name: release-governance

on:
  pull_request:
    paths:
      - '.github/workflows/release.yml'
      - '.github/workflows/release-governance.yml'
      - 'scripts/release/**'
      - 'scripts/release-governance-workflow-contracts.mjs'
      - 'scripts/release-governance-workflow.test.mjs'
      - 'bin/**'
      - 'docs/release/**'
  workflow_dispatch:

jobs:
  release-governance:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Run release governance checks
        run: node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /slo architecture baseline/i,
  );
});

test('release governance workflow contract helper rejects workflows that do not execute the runner self-test directly', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    `
name: release-governance

on:
  pull_request:
    paths:
      - '.github/workflows/release.yml'
      - '.github/workflows/release-governance.yml'
      - 'scripts/release/**'
      - 'scripts/release-governance-workflow-contracts.mjs'
      - 'scripts/release-governance-workflow.test.mjs'
      - 'bin/**'
      - 'docs/架构/135-可观测性与SLO治理设计-2026-04-07.md'
      - 'docs/架构/143-全局架构对齐与收口计划-2026-04-08.md'
      - 'docs/release/**'
  workflow_dispatch:

jobs:
  release-governance:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22

      - name: Run release governance checks
        run: node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /runner self-test/i,
  );
});
