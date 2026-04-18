import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function writeModule(filePath, source) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, source, 'utf8');
}

function writeReleaseWorkflowContractFixture({
  workflowText,
  coverage = {
    covered: true,
    uncoveredReferences: [],
    externalDependencyIds: ['sdkwork-ui'],
  },
} = {}) {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts', 'release'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release.yml'),
    workflowText,
    'utf8',
  );

  writeModule(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    `
export function listExternalReleaseDependencySpecs() {
  return [
    { id: 'sdkwork-core', repository: 'Sdkwork-Cloud/sdkwork-core', envRefKey: 'SDKWORK_CORE_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-ui', repository: 'Sdkwork-Cloud/sdkwork-ui', envRefKey: 'SDKWORK_UI_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-appbase', repository: 'Sdkwork-Cloud/sdkwork-appbase', envRefKey: 'SDKWORK_APPBASE_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-craw-chat-sdk', repository: 'Sdkwork-Cloud/craw-chat', envRefKey: 'SDKWORK_CRAW_CHAT_SDK_GIT_REF', defaultRef: 'main' },
  ];
}
export function buildExternalReleaseClonePlan() {
  return { command: 'git', args: [] };
}
export function auditExternalReleaseDependencyCoverage() {
  return ${JSON.stringify(coverage, null, 2)};
}
`,
  );

  for (const [name, body] of Object.entries({
    'materialize-release-window-snapshot.mjs': `
export function resolveReleaseWindowSnapshotProducerInput() { return { source: 'json', snapshot: {} }; }
export function materializeReleaseWindowSnapshot() { return { outputPath: 'docs/release/release-window-snapshot-latest.json' }; }
`,
    'materialize-release-sync-audit.mjs': `
export function resolveReleaseSyncAuditProducerInput() { return { source: 'json', summary: {} }; }
export function materializeReleaseSyncAudit() { return { outputPath: 'docs/release/release-sync-audit-latest.json' }; }
`,
    'materialize-release-telemetry-export.mjs': `
export function resolveReleaseTelemetryExportProducerInput() { return { source: 'json', payload: {} }; }
export function materializeReleaseTelemetryExport() { return { outputPath: 'docs/release/release-telemetry-export-latest.json' }; }
`,
    'materialize-release-telemetry-snapshot.mjs': `
export function resolveReleaseTelemetryExportInput() { return { source: 'json', payload: {} }; }
export function resolveReleaseTelemetrySnapshotInput() { return { source: 'json', payload: {} }; }
export function deriveReleaseTelemetrySnapshotFromExport() { return { generatedAt: '2026-04-18T10:00:00Z', source: { kind: 'release-telemetry-export' }, targets: {} }; }
export function validateReleaseTelemetrySnapshotShape() { return { snapshotId: 'release-telemetry-snapshot-v1', targetCount: 3 }; }
export function materializeReleaseTelemetrySnapshot() { return { outputPath: 'docs/release/release-telemetry-snapshot-latest.json' }; }
`,
    'materialize-slo-governance-evidence.mjs': `
export function resolveSloGovernanceEvidenceInput() { return { source: 'json', payload: {} }; }
export function validateSloGovernanceEvidenceShape() { return { baselineId: 'release-slo-governance-baseline', targetCount: 3 }; }
export function materializeSloGovernanceEvidence() { return { outputPath: 'docs/release/slo-governance-latest.json' }; }
`,
    'materialize-release-governance-bundle.mjs': `
export function listReleaseGovernanceBundleArtifactSpecs() { return []; }
export function createReleaseGovernanceBundleManifest() { return { version: 1, bundleEntryCount: 0, artifacts: [] }; }
export function materializeReleaseGovernanceBundle() { return { outputDir: 'artifacts/release-governance-bundle', bundleEntryCount: 0, manifestPath: 'artifacts/release-governance-bundle/release-governance-bundle-manifest.json' }; }
`,
    'run-unix-installed-runtime-smoke.mjs': `
export function parseArgs() { return {}; }
export function createUnixInstalledRuntimeSmokeOptions() { return {}; }
export function createUnixInstalledRuntimeSmokePlan() { return {}; }
export function createUnixInstalledRuntimeSmokeEvidence() { return {}; }
`,
    'run-windows-installed-runtime-smoke.mjs': `
export function parseArgs() { return {}; }
export function createWindowsInstalledRuntimeSmokeOptions() { return {}; }
export function createWindowsInstalledRuntimeSmokePlan() { return {}; }
export function createWindowsInstalledRuntimeSmokeEvidence() { return {}; }
`,
    'run-linux-docker-compose-smoke.mjs': `
export function parseArgs() { return {}; }
export function createLinuxDockerComposeSmokeOptions() { return {}; }
export function createLinuxDockerComposeSmokePlan() { return {}; }
export function createLinuxDockerComposeSmokeEvidence() { return {}; }
`,
    'run-linux-helm-render-smoke.mjs': `
export function parseArgs() { return {}; }
export function createLinuxHelmRenderSmokeOptions() { return {}; }
export function createLinuxHelmRenderSmokePlan() { return {}; }
export function createLinuxHelmRenderSmokeEvidence() { return {}; }
`,
    'materialize-release-catalog.mjs': `
export function collectReleaseCatalogEntries() { return []; }
export function createReleaseCatalog() { return { version: 1, products: [] }; }
export function materializeReleaseCatalog() { return { outputPath: 'artifacts/release/release-catalog.json', productCount: 2, variantCount: 2 }; }
`,
  })) {
    writeModule(path.join(fixtureRoot, 'scripts', 'release', name), body);
  }

  return fixtureRoot;
}

test('release workflow publishes only official server and portal desktop products', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');

  assert.match(workflow, /governance-release:/);
  assert.match(workflow, /native-release:/);
  assert.match(workflow, /publish:/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /native-release:[\s\S]*?Collect native release assets[\s\S]*?Run installed native runtime smoke on Windows/);
  assert.match(workflow, /native-release:[\s\S]*?Collect native release assets[\s\S]*?Run installed native runtime smoke on Unix/);
  assert.match(workflow, /node scripts\/release\/materialize-release-catalog\.mjs --release-tag \$\{\{ needs\.prepare\.outputs\.release_tag \}\} --assets-root artifacts\/release --output artifacts\/release\/release-catalog\.json/);
  assert.match(workflow, /Generate release catalog attestation/);
  assert.match(workflow, /subject-path:\s*artifacts\/release\/release-catalog\.json/);
  assert.match(workflow, /publish:[\s\S]*?Checkout release ref/);
  assert.match(workflow, /publish:[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22/);
  assert.match(workflow, /sdkwork-api-router-product-server-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}\.tar\.gz/);
  assert.match(workflow, /sdkwork-api-router-product-server-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}\.manifest\.json/);
  assert.match(workflow, /desktop\/portal\/sdkwork-router-portal-desktop-\*/);
  assert.doesNotMatch(workflow, /console\/pnpm-lock\.yaml/);
  assert.match(workflow, /docs\/pnpm-lock\.yaml/);
  assert.match(
    workflow,
    /product-verification:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile/,
  );
  assert.match(
    workflow,
    /Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile/,
  );
  assert.match(
    workflow,
    /Build docs site[\s\S]*?pnpm --dir docs build/,
  );
  assert.match(
    workflow,
    /governance-release:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs/,
  );
  assert.match(
    workflow,
    /Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_PATH:\s*docs\/release\/release-sync-audit-latest\.json[\s\S]*?node scripts\/release\/materialize-release-sync-audit\.mjs/,
  );
  assert.doesNotMatch(workflow, /run-desktop-release-build\.mjs --app admin/);
  assert.doesNotMatch(workflow, /web-release:/);
  assert.doesNotMatch(workflow, /package-release-assets\.mjs web/);
  assert.doesNotMatch(workflow, /release-assets-web/);
  assert.doesNotMatch(workflow, /release-assets\/\*\*\/\*/);
  assert.doesNotMatch(workflow, /desktop\/portal\/\*\*\/*/);
  assert.match(workflow, /artifacts\/release\/\*\*\/sdkwork-api-router-product-server-\*\.tar\.gz/);
  assert.match(workflow, /artifacts\/release\/\*\*\/sdkwork-api-router-product-server-\*\.tar\.gz\.sha256\.txt/);
  assert.match(workflow, /artifacts\/release\/\*\*\/sdkwork-api-router-product-server-\*\.manifest\.json/);
  assert.match(workflow, /artifacts\/release\/\*\*\/desktop\/portal\/sdkwork-router-portal-desktop-\*/);
  assert.match(workflow, /artifacts\/release\/release-catalog\.json/);
});

test('release workflow contract helper accepts the repository workflow', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  await contracts.assertReleaseWorkflowContracts({
    repoRoot,
  });
});

test('release workflow contract helper rejects workflows that omit the governance release job', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  rust-dependency-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Build docs site
        run: pnpm --dir docs build
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs

  native-release:
    runs-on: ubuntu-latest
    steps:
      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}
      - name: Upload official release assets
        uses: actions/upload-artifact@v4
        with:
          name: release-assets-native-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: |
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/desktop/portal/**/*
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that still publish web release assets', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  rust-dependency-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Build docs site
        run: pnpm --dir docs build
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs

  governance-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
    runs-on: ubuntu-latest
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Materialize release window snapshot
        run: node scripts/release/materialize-release-window-snapshot.mjs
      - name: Materialize release sync audit
        run: node scripts/release/materialize-release-sync-audit.mjs
      - name: Materialize release telemetry export
        run: node scripts/release/materialize-release-telemetry-export.mjs
      - name: Materialize release telemetry snapshot
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs
      - name: Materialize SLO governance evidence
        run: node scripts/release/materialize-slo-governance-evidence.mjs
      - name: Materialize release governance bundle
        run: node scripts/release/materialize-release-governance-bundle.mjs
      - name: Upload release governance bundle artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-bundle
          path: artifacts/release-governance-bundle/**/*
      - name: Run release governance gate
        run: node scripts/release/run-release-governance-checks.mjs --format json

  native-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
      - governance-release
    runs-on: ubuntu-latest
    steps:
      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}
      - name: Package web release assets
        run: node scripts/release/package-release-assets.mjs web --release-tag \${{ needs.prepare.outputs.release_tag }} --output-dir artifacts/release
      - name: Upload web release assets
        uses: actions/upload-artifact@v4
        with:
          name: release-assets-web
          path: artifacts/release/**/*
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that still build the admin desktop product', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  rust-dependency-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Build docs site
        run: pnpm --dir docs build
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs

  governance-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
    runs-on: ubuntu-latest
    steps:
      - name: Materialize external release dependencies
        run: node scripts/release/materialize-external-deps.mjs
      - name: Materialize release window snapshot
        run: node scripts/release/materialize-release-window-snapshot.mjs
      - name: Materialize release sync audit
        run: node scripts/release/materialize-release-sync-audit.mjs
      - name: Materialize release telemetry export
        run: node scripts/release/materialize-release-telemetry-export.mjs
      - name: Materialize release telemetry snapshot
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs
      - name: Materialize SLO governance evidence
        run: node scripts/release/materialize-slo-governance-evidence.mjs
      - name: Materialize release governance bundle
        run: node scripts/release/materialize-release-governance-bundle.mjs
      - name: Upload release governance bundle artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-bundle
          path: artifacts/release-governance-bundle/**/*
      - name: Run release governance gate
        run: node scripts/release/run-release-governance-checks.mjs --format json

  native-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
      - governance-release
    runs-on: ubuntu-latest
    steps:
      - name: Build admin desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app admin --target \${{ matrix.target }}
      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}
      - name: Upload official release assets
        uses: actions/upload-artifact@v4
        with:
          name: release-assets-native-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: |
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/desktop/portal/**/*
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that do not build the docs site before release product verification', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  prepare:
    runs-on: ubuntu-latest
    outputs:
      release_tag: \${{ steps.resolve.outputs.release_tag }}
      git_ref: \${{ steps.resolve.outputs.git_ref }}
    steps:
      - id: resolve
        run: |
          echo "release_tag=release-fixture" >> "$GITHUB_OUTPUT"
          echo "git_ref=refs/tags/release-fixture" >> "$GITHUB_OUTPUT"

  rust-dependency-audit:
    needs: prepare
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    needs:
      - prepare
      - rust-dependency-audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /docs site/i,
  );
});
