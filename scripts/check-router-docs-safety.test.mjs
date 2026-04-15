import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('router product docs stay free of retired fixed bootstrap credentials', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-docs-safety.mjs')).href,
  );

  const findings = module.scanDocsForRetiredBootstrapCredentials({
    workspaceRoot,
  });

  assert.deepEqual(findings, []);
});

test('router docs safety scan only targets product-facing docs trees', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-docs-safety.mjs')).href,
  );

  assert.deepEqual(module.DOC_BOOTSTRAP_SCAN_ROOTS, [
    'docs/getting-started',
    'docs/api-reference',
    'docs/operations',
    'docs/zh/getting-started',
    'docs/zh/api-reference',
  ]);

  assert.deepEqual(module.DOC_BOOTSTRAP_SCAN_FILES, [
    'README.md',
    'README.zh-CN.md',
  ]);
});
