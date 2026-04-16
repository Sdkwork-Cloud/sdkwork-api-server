import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('linux Helm render smoke script exposes a parseable CLI contract for packaged product bundles', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-helm-render-smoke.mjs'),
    ).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createLinuxHelmRenderSmokeOptions, 'function');
  assert.equal(typeof module.createLinuxHelmRenderSmokePlan, 'function');
  assert.equal(typeof module.createLinuxHelmRenderSmokeEvidence, 'function');
  assert.equal(typeof module.resolveExtractedBundleRoot, 'function');

  const options = module.parseArgs([
    '--platform',
    'linux',
    '--arch',
    'arm64',
    '--bundle-path',
    'artifacts/release/native/linux/arm64/bundles/sdkwork-api-router-product-server-linux-arm64.tar.gz',
    '--evidence-path',
    'artifacts/release-governance/helm-render-smoke-linux-arm64.json',
  ]);

  assert.deepEqual(options, {
    platform: 'linux',
    arch: 'arm64',
    bundlePath: path.resolve(
      repoRoot,
      'artifacts',
      'release',
      'native',
      'linux',
      'arm64',
      'bundles',
      'sdkwork-api-router-product-server-linux-arm64.tar.gz',
    ),
    evidencePath: path.resolve(
      repoRoot,
      'artifacts',
      'release-governance',
      'helm-render-smoke-linux-arm64.json',
    ),
  });

  const plan = module.createLinuxHelmRenderSmokePlan({
    repoRoot,
    ...options,
  });

  assert.equal(plan.bundlePath, options.bundlePath);
  assert.equal(plan.evidencePath, options.evidencePath);
  assert.equal(plan.chartRelativePath, 'deploy/helm/sdkwork-api-router');
  assert.equal(plan.renderedManifestPath, path.resolve(repoRoot, 'artifacts', 'release-smoke', 'helm-render-linux-arm64.yaml'));
  assert.equal(plan.renderedManifestRelativePath, 'artifacts/release-smoke/helm-render-linux-arm64.yaml');
  assert.deepEqual(plan.requiredTemplateKinds, ['Secret', 'Service', 'Deployment', 'Ingress']);
  assert.equal(plan.helmValues.databaseUrl, 'postgresql://sdkwork:sdkwork-release-smoke@postgres:5432/sdkwork_api_router');
  assert.equal(plan.helmValues.adminJwtSigningSecret.length > 0, true);
  assert.equal(plan.helmValues.portalJwtSigningSecret.length > 0, true);
  assert.equal(plan.helmValues.credentialMasterKey.length > 0, true);
  assert.equal(plan.helmValues.metricsBearerToken.length > 0, true);
  assert.equal(plan.helmValues.ingressEnabled, true);

  const successEvidence = module.createLinuxHelmRenderSmokeEvidence({
    repoRoot,
    plan,
    ok: true,
    renderedKinds: ['Secret', 'Service', 'Deployment', 'Ingress'],
  });
  assert.equal(successEvidence.ok, true);
  assert.equal(successEvidence.platform, 'linux');
  assert.equal(successEvidence.arch, 'arm64');
  assert.equal(successEvidence.bundlePath, path.relative(repoRoot, options.bundlePath).replaceAll('\\', '/'));
  assert.equal(successEvidence.evidencePath, path.relative(repoRoot, options.evidencePath).replaceAll('\\', '/'));
  assert.equal(successEvidence.renderedManifestPath, plan.renderedManifestRelativePath);
  assert.deepEqual(successEvidence.renderedKinds, ['Secret', 'Service', 'Deployment', 'Ingress']);

  const failureEvidence = module.createLinuxHelmRenderSmokeEvidence({
    repoRoot,
    plan,
    ok: false,
    failure: new Error('helm render validation failed'),
  });
  assert.equal(failureEvidence.ok, false);
  assert.equal(failureEvidence.failure.message, 'helm render validation failed');
});

test('linux Helm render smoke options reject unsupported non-linux release lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-helm-render-smoke.mjs'),
    ).href,
  );

  assert.throws(
    () => module.createLinuxHelmRenderSmokeOptions({
      repoRoot,
      platform: 'macos',
      arch: 'arm64',
    }),
    /only supports linux release lanes/i,
  );
});

test('linux Helm render smoke resolves the extracted bundle root even when the archive file name changes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-helm-render-smoke.mjs'),
    ).href,
  );

  const extractRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-helm-smoke-extract-'));
  const actualBundleRoot = path.join(extractRoot, 'sdkwork-api-router-product-server-linux-x64');
  mkdirSync(actualBundleRoot);

  assert.equal(
    module.resolveExtractedBundleRoot({
      extractRoot,
      bundlePath: path.join(extractRoot, 'sdkwork-api-router-product-server-linux-x64-local.tar.gz'),
    }),
    actualBundleRoot,
  );
});
