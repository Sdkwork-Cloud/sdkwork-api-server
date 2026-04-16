import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('repository ships Docker and Helm deployment assets aligned to product server runtime', () => {
  const dockerfilePath = path.join(repoRoot, 'deploy', 'docker', 'Dockerfile');
  const composePath = path.join(repoRoot, 'deploy', 'docker', 'docker-compose.yml');
  const envExamplePath = path.join(repoRoot, 'deploy', 'docker', '.env.example');
  const chartPath = path.join(repoRoot, 'deploy', 'helm', 'sdkwork-api-router', 'Chart.yaml');
  const valuesPath = path.join(repoRoot, 'deploy', 'helm', 'sdkwork-api-router', 'values.yaml');
  const deploymentTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'deployment.yaml',
  );
  const serviceTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'service.yaml',
  );
  const secretTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'secret.yaml',
  );
  const ingressTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'ingress.yaml',
  );

  assert.equal(existsSync(dockerfilePath), true, 'missing deploy/docker/Dockerfile');
  assert.equal(existsSync(composePath), true, 'missing deploy/docker/docker-compose.yml');
  assert.equal(existsSync(envExamplePath), true, 'missing deploy/docker/.env.example');
  assert.equal(existsSync(chartPath), true, 'missing deploy/helm/sdkwork-api-router/Chart.yaml');
  assert.equal(existsSync(valuesPath), true, 'missing deploy/helm/sdkwork-api-router/values.yaml');
  assert.equal(existsSync(deploymentTemplatePath), true, 'missing Helm deployment template');
  assert.equal(existsSync(serviceTemplatePath), true, 'missing Helm service template');
  assert.equal(existsSync(secretTemplatePath), true, 'missing Helm secret template');
  assert.equal(existsSync(ingressTemplatePath), true, 'missing Helm ingress template');

  const dockerfile = read('deploy/docker/Dockerfile');
  assert.match(dockerfile, /router-product-service/);
  assert.match(dockerfile, /SDKWORK_BOOTSTRAP_PROFILE=prod/);
  assert.match(dockerfile, /SDKWORK_BOOTSTRAP_DATA_DIR=\/opt\/sdkwork\/data/);
  assert.match(dockerfile, /SDKWORK_ADMIN_SITE_DIR=\/opt\/sdkwork\/sites\/admin\/dist/);
  assert.match(dockerfile, /SDKWORK_PORTAL_SITE_DIR=\/opt\/sdkwork\/sites\/portal\/dist/);
  assert.match(dockerfile, /EXPOSE 3001/);
  assert.match(dockerfile, /HEALTHCHECK[\s\S]*\/api\/v1\/health/);
  assert.match(dockerfile, /USER sdkwork/);

  const compose = read('deploy/docker/docker-compose.yml');
  assert.match(compose, /^services:/m);
  assert.match(compose, /^\s{2}postgres:/m);
  assert.match(compose, /image:\s*docker\.io\/library\/postgres:16-alpine/);
  assert.match(compose, /^\s{2}router:/m);
  assert.match(compose, /dockerfile:\s*deploy\/docker\/Dockerfile/);
  assert.match(compose, /SDKWORK_DATABASE_URL:/);
  assert.match(compose, /SDKWORK_BOOTSTRAP_PROFILE:/);
  assert.match(compose, /SDKWORK_ADMIN_JWT_SIGNING_SECRET:/);
  assert.match(compose, /SDKWORK_PORTAL_JWT_SIGNING_SECRET:/);
  assert.match(compose, /SDKWORK_CREDENTIAL_MASTER_KEY:/);
  assert.match(compose, /SDKWORK_METRICS_BEARER_TOKEN:/);
  assert.match(compose, /3001:3001/);
  assert.match(compose, /\/api\/v1\/health/);

  const envExample = read('deploy/docker/.env.example');
  assert.match(envExample, /^SDKWORK_POSTGRES_DB=/m);
  assert.match(envExample, /^SDKWORK_POSTGRES_USER=/m);
  assert.match(envExample, /^SDKWORK_POSTGRES_PASSWORD=/m);
  assert.match(envExample, /^SDKWORK_ADMIN_JWT_SIGNING_SECRET=/m);
  assert.match(envExample, /^SDKWORK_PORTAL_JWT_SIGNING_SECRET=/m);
  assert.match(envExample, /^SDKWORK_CREDENTIAL_MASTER_KEY=/m);
  assert.match(envExample, /^SDKWORK_METRICS_BEARER_TOKEN=/m);

  const chart = read('deploy/helm/sdkwork-api-router/Chart.yaml');
  assert.match(chart, /^apiVersion:\s*v2$/m);
  assert.match(chart, /^name:\s*sdkwork-api-router$/m);
  assert.match(chart, /^type:\s*application$/m);

  const values = read('deploy/helm/sdkwork-api-router/values.yaml');
  assert.match(values, /^image:/m);
  assert.match(values, /^\s{2}repository:/m);
  assert.match(values, /^service:/m);
  assert.match(values, /^ingress:/m);
  assert.match(values, /^secrets:/m);
  assert.match(values, /bootstrapProfile:\s*prod/);

  const deployment = read('deploy/helm/sdkwork-api-router/templates/deployment.yaml');
  assert.match(deployment, /SDKWORK_DATABASE_URL/);
  assert.match(deployment, /SDKWORK_BOOTSTRAP_PROFILE/);
  assert.match(deployment, /SDKWORK_BOOTSTRAP_DATA_DIR/);
  assert.match(deployment, /SDKWORK_ADMIN_SITE_DIR/);
  assert.match(deployment, /SDKWORK_PORTAL_SITE_DIR/);
  assert.match(deployment, /SDKWORK_ADMIN_JWT_SIGNING_SECRET/);
  assert.match(deployment, /SDKWORK_PORTAL_JWT_SIGNING_SECRET/);
  assert.match(deployment, /SDKWORK_CREDENTIAL_MASTER_KEY/);
  assert.match(deployment, /SDKWORK_METRICS_BEARER_TOKEN/);
  assert.match(deployment, /\/api\/v1\/health/);
  assert.match(deployment, /\/api\/admin\/health/);
  assert.match(deployment, /\/api\/portal\/health/);

  const secretTemplate = read('deploy/helm/sdkwork-api-router/templates/secret.yaml');
  assert.match(secretTemplate, /SDKWORK_DATABASE_URL/);
  assert.match(secretTemplate, /SDKWORK_ADMIN_JWT_SIGNING_SECRET/);
  assert.match(secretTemplate, /SDKWORK_PORTAL_JWT_SIGNING_SECRET/);
  assert.match(secretTemplate, /SDKWORK_CREDENTIAL_MASTER_KEY/);
  assert.match(secretTemplate, /SDKWORK_METRICS_BEARER_TOKEN/);
});

test('native product server release packager exports deployment assets into commercial bundles', async () => {
  const packager = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs')).href,
  );

  assert.equal(
    typeof packager.listNativeProductServerDeploymentAssetRoots,
    'function',
    'expected deployment roots export for product-server bundles',
  );
  assert.deepEqual(
    packager.listNativeProductServerDeploymentAssetRoots(),
    {
      deploy: path.join(repoRoot, 'deploy'),
    },
  );

  const packagerSource = read('scripts/release/package-release-assets.mjs');
  assert.match(packagerSource, /- deploy\/: docker, compose, and helm deployment assets/);
  assert.match(packagerSource, /deploymentAssetRoots/);
  assert.match(packagerSource, /listNativeProductServerDeploymentAssetRoots/);
});
