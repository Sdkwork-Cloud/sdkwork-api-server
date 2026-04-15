import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

export async function assertReleaseGovernanceWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release-governance.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release-governance.yml');

  const workflow = read(repoRoot, '.github/workflows/release-governance.yml');

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /\.github\/workflows\/release-governance\.yml/, 'release-governance workflow must watch its own workflow file');
  assert.match(workflow, /scripts\/release\/\*\*/);
  assert.match(
    workflow,
    /scripts\/release-governance-workflow-contracts\.mjs/,
    'release-governance workflow must watch the contract module',
  );
  assert.match(workflow, /scripts\/release-governance-workflow\.test\.mjs/);
  assert.match(
    workflow,
    /docs\/架构\/135-可观测性与SLO治理设计-2026-04-07\.md/,
    'release-governance workflow must watch the governed SLO architecture baseline',
  );
  assert.match(
    workflow,
    /docs\/架构\/143-全局架构对齐与收口计划-2026-04-08\.md/,
    'release-governance workflow must watch the global architecture closure baseline',
  );
  assert.match(
    workflow,
    /run:\s*node --test --experimental-test-isolation=none scripts\/release\/tests\/release-governance-runner\.test\.mjs/,
    'release-governance workflow must execute the runner self-test directly',
  );
  assert.match(workflow, /run:\s*node scripts\/release\/run-release-governance-checks\.mjs --profile preflight --format json/);
}
