import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

function readWorkspaceFile(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

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

test('router production docs publish a single deployment entrypoint with operations pages in both locales', () => {
  const requiredFiles = [
    'docs/getting-started/production-deployment.md',
    'docs/zh/getting-started/production-deployment.md',
    'docs/operations/install-layout.md',
    'docs/zh/operations/install-layout.md',
    'docs/operations/service-management.md',
    'docs/zh/operations/service-management.md',
  ];

  for (const relativePath of requiredFiles) {
    assert.equal(existsSync(path.join(workspaceRoot, relativePath)), true, `missing ${relativePath}`);
  }

  const vitepressConfig = readWorkspaceFile('docs/.vitepress/config.mjs');
  assert.match(vitepressConfig, /\/getting-started\/production-deployment/);
  assert.match(vitepressConfig, /\/operations\/install-layout/);
  assert.match(vitepressConfig, /\/operations\/service-management/);
  assert.match(vitepressConfig, /\/zh\/getting-started\/production-deployment/);
  assert.match(vitepressConfig, /\/zh\/operations\/install-layout/);
  assert.match(vitepressConfig, /\/zh\/operations\/service-management/);
});

test('README and getting-started docs align to config-file-first production guidance', () => {
  const readme = readWorkspaceFile('README.md');
  const quickstart = readWorkspaceFile('docs/getting-started/quickstart.md');
  const releaseBuilds = readWorkspaceFile('docs/getting-started/release-builds.md');
  const deployReadme = readWorkspaceFile('deploy/README.md');

  assert.match(
    readme,
    /built-in defaults\s*->\s*environment fallback\s*->\s*config file\s*->\s*CLI/i,
  );
  assert.match(readme, /Production Deployment/i);
  assert.match(readme, /system installs default to PostgreSQL/i);
  assert.match(quickstart, /local development only/i);
  assert.match(quickstart, /Production Deployment/);
  assert.match(releaseBuilds, /build and package generation only/i);
  assert.match(releaseBuilds, /Production Deployment/);
  assert.match(deployReadme, /Docker and Helm asset-specific/i);
  assert.doesNotMatch(deployReadme, /system install/i);
});
