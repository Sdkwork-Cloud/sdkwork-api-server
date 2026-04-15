import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const repoRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('repository exposes a cached package-group Rust verification workflow', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'rust-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/rust-verification.yml');

  const workflow = read('.github/workflows/rust-verification.yml');

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /group:\s*interface-openapi/);
  assert.match(workflow, /group:\s*gateway-service/);
  assert.match(workflow, /group:\s*admin-service/);
  assert.match(workflow, /group:\s*portal-service/);
  assert.match(workflow, /group:\s*dependency-audit/);
  assert.match(workflow, /group:\s*product-runtime/);
  assert.match(workflow, /vendor\/\*\*/);
  assert.match(workflow, /scripts\/check-rust-dependency-audit\.mjs/);
  assert.match(workflow, /scripts\/check-rust-dependency-audit\.test\.mjs/);
  assert.match(workflow, /Install cargo-audit/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  assert.match(workflow, /Run rust governance node tests/);
  assert.match(
    workflow,
    /node --test scripts\/check-rust-dependency-audit\.test\.mjs scripts\/check-rust-verification-matrix\.test\.mjs scripts\/rust-verification-workflow\.test\.mjs/,
  );
  assert.match(
    workflow,
    /node scripts\/check-rust-verification-matrix\.mjs --group \$\{\{ matrix\.group \}\}/,
  );
  assert.match(workflow, /rust-verification-windows-workspace:/);
  assert.match(workflow, /runs-on:\s*windows-latest/);
  assert.match(workflow, /github\.event_name == 'workflow_dispatch'/);
  assert.match(workflow, /github\.event\.inputs\.group == 'workspace'/);
  assert.match(workflow, /node scripts\/check-rust-verification-matrix\.mjs --group workspace/);
});
