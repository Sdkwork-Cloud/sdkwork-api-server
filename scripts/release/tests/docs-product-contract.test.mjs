import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { readFileSync } from 'node:fs';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('official packaging docs describe only the server and portal desktop products', () => {
  const buildAndPackaging = read('docs/getting-started/build-and-packaging.md');
  const releaseBuilds = read('docs/getting-started/release-builds.md');
  const productionDeployment = read('docs/getting-started/production-deployment.md');
  const installation = read('docs/getting-started/installation.md');
  const buildAndTooling = read('docs/reference/build-and-tooling.md');
  const scriptLifecycle = read('docs/getting-started/script-lifecycle.md');
  const runtimeModes = read('docs/architecture/runtime-modes.md');
  const architecture = read('docs/architecture/software-architecture.md');
  const functionalModules = read('docs/architecture/functional-modules.md');
  const index = read('docs/index.md');

  assert.match(buildAndPackaging, /sdkwork-api-router-product-server/);
  assert.match(buildAndPackaging, /sdkwork-router-portal-desktop/);
  assert.match(buildAndPackaging, /prepare-router-portal-desktop-runtime\.mjs/);
  assert.match(buildAndPackaging, /pnpm --dir apps\/sdkwork-router-portal tauri:build/);
  assert.match(
    buildAndPackaging,
    /cargo build --release -p router-product-service -p gateway-service -p admin-api-service -p portal-api-service -p router-web-service/,
  );
  assert.match(buildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz/i);
  assert.match(buildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz\.sha256\.txt/i);
  assert.match(buildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/i);
  assert.match(buildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.<ext>\.sha256\.txt/);
  assert.match(buildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.doesNotMatch(buildAndPackaging, /pnpm --dir console/);
  assert.doesNotMatch(buildAndPackaging, /console\/dist/);
  assert.match(releaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>/i);
  assert.match(releaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/i);
  assert.match(releaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>/);
  assert.match(releaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.match(releaseBuilds, /build\.sh --verify-release/);
  assert.match(releaseBuilds, /--skip-docs cannot be combined with --verify-release/);
  assert.match(releaseBuilds, /verify-release[\s\S]*docs site/i);
  assert.match(releaseBuilds, /verify-release[\s\S]*release governance preflight/i);
  assert.match(releaseBuilds, /release-catalog\.json/);
  assert.match(releaseBuilds, /artifacts\/release\/release-catalog\.json/);
  assert.match(releaseBuilds, /generatedAt/);
  assert.match(releaseBuilds, /variantKind/);
  assert.match(releaseBuilds, /primaryFileSizeBytes/);
  assert.match(releaseBuilds, /checksumAlgorithm/);
  assert.match(releaseBuilds, /router-product\/data\//);
  assert.match(releaseBuilds, /router-product\/release-manifest\.json/);
  assert.match(releaseBuilds, /router-product\/README\.txt/);
  assert.match(productionDeployment, /release-catalog\.json/);
  assert.doesNotMatch(releaseBuilds, /desktop\/portal\/\*\*\/*/);

  assert.doesNotMatch(installation, /pnpm --dir console/);
  assert.doesNotMatch(installation, /browser\/Tauri operator console under `console\/`/);
  assert.doesNotMatch(installation, /standalone admin app under `apps\/sdkwork-router-admin\/`/);
  assert.match(installation, /development-only admin browser app under `apps\/sdkwork-router-admin\/`/);
  assert.match(installation, /not an official release product/i);

  assert.doesNotMatch(buildAndTooling, /browser-only/);
  assert.match(buildAndTooling, /build\.sh --verify-release/);
  assert.match(buildAndTooling, /official local release verification/i);
  assert.match(buildAndTooling, /release governance preflight/i);
  assert.match(buildAndTooling, /--skip-docs/);
  assert.match(
    buildAndTooling,
    /cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service/,
  );
  assert.match(buildAndTooling, /start-portal\.mjs` \| raw source dev \| start the portal app only \| browser or Tauri \|/);
  assert.doesNotMatch(scriptLifecycle, /optional docs and console browser assets/);

  assert.doesNotMatch(runtimeModes, /console\/src-tauri/);
  assert.match(runtimeModes, /apps\/sdkwork-router-portal\/src-tauri/);

  assert.doesNotMatch(architecture, /the admin and portal Tauri hosts both embed the shared product runtime/);
  assert.doesNotMatch(functionalModules, /\| console \| browser and desktop UI shell \| `console\/` \|/);
  assert.match(
    index,
    /cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service/,
  );
  assert.match(index, /\| apps\/sdkwork-router-portal \| browser or Tauri \| standalone developer self-service portal \|/);
});

test('repository landing docs keep the official product line and installed validation entrypoints aligned', () => {
  const readme = read('README.md');
  const zhReadme = read('README.zh-CN.md');

  for (const content of [readme, zhReadme]) {
    assert.match(content, /sdkwork-api-router-product-server/);
    assert.match(content, /sdkwork-router-portal-desktop/);
    assert.match(content, /current\/bin\/validate-config\.sh --home/);
    assert.match(content, /current\\bin\\validate-config\.ps1 -Home/);
    assert.doesNotMatch(content, /<install-root>\/bin\/validate-config\.sh/);
    assert.doesNotMatch(content, /<install-root>\\bin\\validate-config\.ps1/);
  }
});

test('localized product docs follow the same official packaging contract', () => {
  const zhIndex = read('docs/zh/index.md');
  const zhInstallation = read('docs/zh/getting-started/installation.md');
  const zhBuildAndPackaging = read('docs/zh/getting-started/build-and-packaging.md');
  const zhReleaseBuilds = read('docs/zh/getting-started/release-builds.md');
  const zhProductionDeployment = read('docs/zh/getting-started/production-deployment.md');
  const zhBuildAndTooling = read('docs/zh/reference/build-and-tooling.md');
  const zhRuntimeModes = read('docs/zh/architecture/runtime-modes.md');
  const zhArchitecture = read('docs/zh/architecture/software-architecture.md');
  const zhFunctionalModules = read('docs/zh/architecture/functional-modules.md');
  const zhRepositoryLayout = read('docs/zh/reference/repository-layout.md');

  assert.match(zhIndex, /OpenAI 兼容网关/);
  assert.match(zhIndex, /sdkwork-router-portal-desktop/);
  assert.match(
    zhIndex,
    /cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service/,
  );

  assert.match(zhInstallation, /^# 安装准备/m);
  assert.doesNotMatch(zhInstallation, /pnpm --dir console/);
  assert.match(zhInstallation, /apps\/sdkwork-router-admin\//);
  assert.doesNotMatch(zhInstallation, /独立 admin 应用/);
  assert.match(zhInstallation, /不是正式发布产品/);

  assert.match(zhBuildAndPackaging, /^# 编译与打包/m);
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server/);
  assert.match(zhBuildAndPackaging, /sdkwork-router-portal-desktop/);
  assert.match(
    zhBuildAndPackaging,
    /cargo build --release -p router-product-service -p gateway-service -p admin-api-service -p portal-api-service -p router-web-service/,
  );
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz/);
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz\.sha256\.txt/);
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/);
  assert.match(zhBuildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.<ext>\.sha256\.txt/);
  assert.match(zhBuildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.doesNotMatch(zhBuildAndPackaging, /pnpm --dir console/);
  assert.match(zhReleaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>/);
  assert.match(zhReleaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/);
  assert.match(zhReleaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>/);
  assert.match(zhReleaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.match(zhReleaseBuilds, /build\.sh --verify-release/);
  assert.match(zhReleaseBuilds, /--skip-docs .* --verify-release|--verify-release .* --skip-docs/);
  assert.match(zhReleaseBuilds, /verify-release[\s\S]*docs/);
  assert.match(zhReleaseBuilds, /verify-release[\s\S]*governance/i);
  assert.match(zhReleaseBuilds, /release-catalog\.json/);
  assert.match(zhReleaseBuilds, /artifacts\/release\/release-catalog\.json/);
  assert.match(zhReleaseBuilds, /generatedAt/);
  assert.match(zhReleaseBuilds, /variantKind/);
  assert.match(zhReleaseBuilds, /primaryFileSizeBytes/);
  assert.match(zhReleaseBuilds, /checksumAlgorithm/);
  assert.match(zhReleaseBuilds, /router-product\/data\//);
  assert.match(zhReleaseBuilds, /router-product\/release-manifest\.json/);
  assert.match(zhReleaseBuilds, /router-product\/README\.txt/);
  assert.match(zhProductionDeployment, /release-catalog\.json/);
  assert.doesNotMatch(zhReleaseBuilds, /desktop\/portal\/\*\*\/*/);

  assert.match(zhBuildAndTooling, /^# 构建与工具链/m);
  assert.match(zhBuildAndTooling, /node scripts\/check-router-product\.mjs/);
  assert.match(zhBuildAndTooling, /build\.sh --verify-release/);
  assert.match(zhBuildAndTooling, /正式本地 release 验证/i);
  assert.match(zhBuildAndTooling, /release governance preflight/i);
  assert.match(zhBuildAndTooling, /--skip-docs/);
  assert.match(
    zhBuildAndTooling,
    /cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service/,
  );
  assert.match(zhBuildAndTooling, /product gate/i);
  assert.match(zhBuildAndTooling, /浏览器或 Tauri/);
  assert.doesNotMatch(zhBuildAndTooling, /仅浏览器/);

  assert.match(zhRuntimeModes, /^# 运行模式详解/m);
  assert.doesNotMatch(zhRuntimeModes, /console\/src-tauri/);
  assert.match(zhRuntimeModes, /apps\/sdkwork-router-portal\/src-tauri/);

  assert.match(zhArchitecture, /^# 软件架构/m);
  assert.doesNotMatch(zhFunctionalModules, /\| console \|/);
  assert.doesNotMatch(zhRepositoryLayout, /\|-- console\//);
  assert.doesNotMatch(zhRepositoryLayout, /继续参与发布打包/);
});

test('script lifecycle docs no longer describe console as a release build input', () => {
  const scriptLifecycle = read('docs/getting-started/script-lifecycle.md');
  const zhScriptLifecycle = read('docs/zh/getting-started/script-lifecycle.md');

  assert.doesNotMatch(scriptLifecycle, /console/);
  assert.doesNotMatch(zhScriptLifecycle, /console/);
});

test('supplemental architecture notes no longer treat console as an official product surface', () => {
  const architectureReadme = read('docs/架构/README.md');
  const productScope = read('docs/架构/01-产品设计与需求范围.md');
  const architectureStandard = read('docs/架构/02-架构标准与总体设计.md');
  const moduleBoundaries = read('docs/架构/03-模块规划与边界.md');
  const marketMatrix = read('docs/架构/130-API-Router-行业对标与终局能力矩阵-2026-04-07.md');
  const controlPlane = read('docs/架构/133-控制平面与运营后台设计-2026-04-07.md');

  assert.doesNotMatch(architectureReadme, /console\//);
  assert.doesNotMatch(productScope, /console\//);
  assert.doesNotMatch(architectureStandard, /console\//);
  assert.doesNotMatch(moduleBoundaries, /console\//);
  assert.doesNotMatch(marketMatrix, /console\//);
  assert.doesNotMatch(controlPlane, /console\//);
});

test('install and deployment docs describe the bundle-driven server install contract', () => {
  const installLayout = read('docs/operations/install-layout.md');
  const serviceManagement = read('docs/operations/service-management.md');
  const productionDeployment = read('docs/getting-started/production-deployment.md');
  const zhInstallLayout = read('docs/zh/operations/install-layout.md');
  const zhServiceManagement = read('docs/zh/operations/service-management.md');
  const zhProductionDeployment = read('docs/zh/getting-started/production-deployment.md');

  for (const content of [installLayout, zhInstallLayout]) {
    assert.match(content, /artifacts\/release\/native\/<platform>\/<arch>\/bundles\//);
    assert.match(content, /release-catalog\.json/);
    assert.match(content, /release-manifest\.json/);
    assert.match(content, /README\.txt/);
    assert.match(content, /deploy\//);
  }

  for (const content of [serviceManagement, zhServiceManagement, productionDeployment, zhProductionDeployment]) {
    assert.match(content, /installed-runtime smoke/i);
    assert.match(content, /packaged server bundle/i);
  }

  for (const content of [productionDeployment, zhProductionDeployment]) {
    assert.match(content, /build\.sh --verify-release/);
    assert.match(content, /release governance preflight/i);
  }
});

test('product Docker runtime uses the hosted Linux ARM release baseline', () => {
  const dockerfile = read('deploy/docker/Dockerfile');

  assert.match(dockerfile, /^FROM ubuntu:24\.04$/m);
});
