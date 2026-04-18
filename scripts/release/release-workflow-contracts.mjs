import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

export async function assertReleaseWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read(repoRoot, path.join('.github', 'workflows', 'release.yml'));

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /permissions:\s*[\s\S]*contents:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*id-token:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*attestations:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*artifact-metadata:\s*write/);

  assert.match(
    workflow,
    /rust-dependency-audit:[\s\S]*?runs-on:\s*ubuntu-latest[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?dtolnay\/rust-toolchain@stable[\s\S]*?Swatinem\/rust-cache@v2[\s\S]*?taiki-e\/install-action@cargo-audit[\s\S]*?node scripts\/check-rust-dependency-audit\.mjs/,
    'release workflow must execute a dedicated Rust dependency audit gate against the exact release ref before any assets are built',
  );

  assert.match(
    workflow,
    /product-verification:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile/,
    'release workflow product verification must materialize only referenced external release dependencies before frozen installs so workspace-linked packages resolve on GitHub runners without cloning unrelated governance-only repositories',
  );
  assert.match(
    workflow,
    /product-verification:[\s\S]*?runs-on:\s*ubuntu-latest[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?pnpm\/action-setup@v4[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?dtolnay\/rust-toolchain@stable[\s\S]*?Swatinem\/rust-cache@v2[\s\S]*?taiki-e\/install-action@cargo-audit[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?Run release product verification[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?node scripts\/check-router-product\.mjs/,
    'release workflow must execute product verification with frozen installs and strict frontend install mode before any assets are built',
  );
  assert.doesNotMatch(
    workflow,
    /console\/pnpm-lock\.yaml/,
    'release workflow should not cache the legacy console lockfile when that workspace is not part of the official release pipeline',
  );
  assert.match(
    workflow,
    /docs\/pnpm-lock\.yaml/,
    'release workflow must cache the docs lockfile because the public docs site is a governed product surface during release verification',
  );
  assert.match(
    workflow,
    /Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile/,
    'release workflow must use an explicit frozen install for the docs workspace before building the public docs site',
  );
  assert.match(
    workflow,
    /Build docs site[\s\S]*?pnpm --dir docs build/,
    'release workflow must build the docs site before running release product verification',
  );

  assert.match(
    workflow,
    /governance-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit[\r\n]+\s*-\s*product-verification/,
    'governance release job must wait for prepare, Rust dependency audit, and product verification gates',
  );
  assert.match(
    workflow,
    /governance-release:[\s\S]*?Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_PATH:\s*docs\/release\/release-sync-audit-latest\.json[\s\S]*?node scripts\/release\/materialize-release-sync-audit\.mjs/,
    'governance release job must seed the sync-audit materializer from the committed governed artifact path unless an explicit JSON override is supplied',
  );
  assert.match(
    workflow,
    /governance-release:[\s\S]*?Materialize release telemetry export[\s\S]*?SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH:\s*docs\/release\/release-telemetry-export-latest\.json[\s\S]*?node scripts\/release\/materialize-release-telemetry-export\.mjs/,
    'governance release job must seed the telemetry-export materializer from the committed governed artifact path unless an explicit control-plane handoff overrides it',
  );
  assert.match(
    workflow,
    /governance-release:[\s\S]*?Materialize external release dependencies[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Materialize release window snapshot[\s\S]*?node scripts\/release\/materialize-release-window-snapshot\.mjs[\s\S]*?Materialize release sync audit[\s\S]*?node scripts\/release\/materialize-release-sync-audit\.mjs[\s\S]*?Materialize release telemetry export[\s\S]*?node scripts\/release\/materialize-release-telemetry-export\.mjs[\s\S]*?Materialize release telemetry snapshot[\s\S]*?node scripts\/release\/materialize-release-telemetry-snapshot\.mjs[\s\S]*?Materialize SLO governance evidence[\s\S]*?node scripts\/release\/materialize-slo-governance-evidence\.mjs[\s\S]*?Materialize release governance bundle[\s\S]*?node scripts\/release\/materialize-release-governance-bundle\.mjs[\s\S]*?Run release governance gate[\s\S]*?node scripts\/release\/run-release-governance-checks\.mjs --format json/,
    'governance release job must materialize governed evidence, assemble the governance bundle, and execute the governance gate',
  );
  assert.match(
    workflow,
    /governance-release:[\s\S]*?Upload release governance bundle artifact[\s\S]*?name:\s*release-governance-bundle[\s\S]*?path:\s*artifacts\/release-governance-bundle\/\*\*\/*/,
    'governance release job must upload the governance bundle as a workflow artifact',
  );

  assert.match(
    workflow,
    /native-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit[\r\n]+\s*-\s*product-verification[\r\n]+\s*-\s*governance-release/,
    'native release job must wait for governance release completion before official assets are built and published',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Build portal desktop release[\s\S]*?node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{\s*matrix\.target\s*\}\}/,
    'native release job must build the portal desktop product',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Build portal desktop release[\s\S]*?Collect native release assets[\s\S]*?node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --target \$\{\{\s*matrix\.target\s*\}\} --output-dir artifacts\/release[\s\S]*?Run installed native runtime smoke on Windows[\s\S]*?Run installed native runtime smoke on Unix/,
    'native release job must package official assets before installed-runtime smoke so smoke verifies the packaged server bundle',
  );
  assert.doesNotMatch(
    workflow,
    /Build admin desktop release|run-desktop-release-build\.mjs --app admin/,
    'native release job must not publish or build a standalone admin desktop product',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload official release assets[\s\S]*?name:\s*release-assets-native-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.tar\.gz[\s\S]*?sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.manifest\.json[\s\S]*?desktop\/portal\/sdkwork-router-portal-desktop-\*/,
    'native release upload step must publish only official server and portal desktop assets',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate native release assets attestation[\s\S]*?sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.tar\.gz[\s\S]*?sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.manifest\.json[\s\S]*?desktop\/portal\/sdkwork-router-portal-desktop-\*/,
    'native release attestation must cover only the official server and portal desktop assets',
  );

  assert.match(
    workflow,
    /publish:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*governance-release[\r\n]+\s*-\s*native-release/,
    'publish job must wait for governance release and native release jobs',
  );
  assert.match(
    workflow,
    /publish:[\s\S]*?Checkout release ref[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}/,
    'publish job must check out the exact release ref before invoking repository-owned publish scripts',
  );
  assert.match(
    workflow,
    /publish:[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22/,
    'publish job must pin the Node runtime before generating repository-owned release metadata',
  );
  assert.match(
    workflow,
    /publish:[\s\S]*?Download packaged release assets[\s\S]*?path:\s*artifacts\/release[\s\S]*?Generate release catalog[\s\S]*?node scripts\/release\/materialize-release-catalog\.mjs --release-tag \$\{\{\s*needs\.prepare\.outputs\.release_tag\s*\}\} --assets-root artifacts\/release --output artifacts\/release\/release-catalog\.json/,
    'publish job must materialize a single release-catalog.json into the canonical artifacts/release tree before publishing',
  );
  assert.match(
    workflow,
    /publish:[\s\S]*?Generate release catalog[\s\S]*?Generate release catalog attestation[\s\S]*?actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release\/release-catalog\.json/,
    'publish job must generate a dedicated attestation for the release-catalog asset after it is materialized',
  );
  assert.match(
    workflow,
    /publish:[\s\S]*?Download packaged release assets[\s\S]*?pattern:\s*release-assets-native-\*[\s\S]*?Publish release assets[\s\S]*?softprops\/action-gh-release@v2[\s\S]*?artifacts\/release\/\*\*\/sdkwork-api-router-product-server-\*\.tar\.gz[\s\S]*?artifacts\/release\/\*\*\/sdkwork-api-router-product-server-\*\.tar\.gz\.sha256\.txt[\s\S]*?artifacts\/release\/\*\*\/sdkwork-api-router-product-server-\*\.manifest\.json[\s\S]*?artifacts\/release\/\*\*\/desktop\/portal\/sdkwork-router-portal-desktop-\*[\s\S]*?artifacts\/release\/release-catalog\.json/,
    'publish job must attach the explicit official server and portal desktop asset globs plus the unified release catalog',
  );
  assert.doesNotMatch(
    workflow,
    /desktop\/portal\/\*\*\/*/,
    'desktop release publication must not expose raw bundle trees; only normalized official desktop asset names are allowed',
  );
  assert.doesNotMatch(
    workflow,
    /files:\s*\|\s*[\s\S]*?artifacts\/release\/\*\*\/\*/,
    'publish job must not use a catch-all artifacts/release glob that could leak non-product files',
  );

  assert.doesNotMatch(workflow, /web-release:/, 'web release job must not exist in the official release workflow');
  assert.doesNotMatch(workflow, /package-release-assets\.mjs web/, 'web release packaging must not be part of the official workflow');
  assert.doesNotMatch(workflow, /release-assets-web/, 'official release workflow must not upload web release assets');

  const helper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );
  const releaseWindowHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    ).href,
  );
  const releaseSyncHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    ).href,
  );
  const telemetryExportHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-export.mjs'),
    ).href,
  );
  const telemetrySnapshotHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    ).href,
  );
  const sloHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );
  const governanceBundleHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-governance-bundle.mjs'),
    ).href,
  );
  const releaseCatalogHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-catalog.mjs'),
    ).href,
  );
  const unixRuntimeSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-unix-installed-runtime-smoke.mjs'),
    ).href,
  );
  const windowsRuntimeSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );
  const linuxDockerComposeSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );
  const linuxHelmRenderSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-helm-render-smoke.mjs'),
    ).href,
  );

  assert.equal(typeof helper.listExternalReleaseDependencySpecs, 'function');
  assert.equal(typeof helper.buildExternalReleaseClonePlan, 'function');
  assert.equal(typeof helper.auditExternalReleaseDependencyCoverage, 'function');
  assert.equal(typeof helper.selectExternalReleaseDependencySpecsForMaterialization, 'function');
  assert.equal(typeof releaseWindowHelper.resolveReleaseWindowSnapshotProducerInput, 'function');
  assert.equal(typeof releaseWindowHelper.materializeReleaseWindowSnapshot, 'function');
  assert.equal(typeof releaseSyncHelper.resolveReleaseSyncAuditProducerInput, 'function');
  assert.equal(typeof releaseSyncHelper.materializeReleaseSyncAudit, 'function');
  assert.equal(typeof telemetryExportHelper.resolveReleaseTelemetryExportProducerInput, 'function');
  assert.equal(typeof telemetryExportHelper.materializeReleaseTelemetryExport, 'function');
  assert.equal(typeof telemetrySnapshotHelper.resolveReleaseTelemetryExportInput, 'function');
  assert.equal(typeof telemetrySnapshotHelper.resolveReleaseTelemetrySnapshotInput, 'function');
  assert.equal(typeof telemetrySnapshotHelper.deriveReleaseTelemetrySnapshotFromExport, 'function');
  assert.equal(typeof telemetrySnapshotHelper.validateReleaseTelemetrySnapshotShape, 'function');
  assert.equal(typeof telemetrySnapshotHelper.materializeReleaseTelemetrySnapshot, 'function');
  assert.equal(typeof sloHelper.resolveSloGovernanceEvidenceInput, 'function');
  assert.equal(typeof sloHelper.validateSloGovernanceEvidenceShape, 'function');
  assert.equal(typeof sloHelper.materializeSloGovernanceEvidence, 'function');
  assert.equal(typeof governanceBundleHelper.listReleaseGovernanceBundleArtifactSpecs, 'function');
  assert.equal(typeof governanceBundleHelper.createReleaseGovernanceBundleManifest, 'function');
  assert.equal(typeof governanceBundleHelper.materializeReleaseGovernanceBundle, 'function');
  assert.equal(typeof releaseCatalogHelper.collectReleaseCatalogEntries, 'function');
  assert.equal(typeof releaseCatalogHelper.createReleaseCatalog, 'function');
  assert.equal(typeof releaseCatalogHelper.materializeReleaseCatalog, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.parseArgs, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.createUnixInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.createUnixInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.createUnixInstalledRuntimeSmokeEvidence, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.parseArgs, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.createWindowsInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.createWindowsInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.createWindowsInstalledRuntimeSmokeEvidence, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.parseArgs, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.createLinuxDockerComposeSmokeOptions, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.createLinuxDockerComposeSmokePlan, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.createLinuxDockerComposeSmokeEvidence, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.parseArgs, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.createLinuxHelmRenderSmokeOptions, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.createLinuxHelmRenderSmokePlan, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.createLinuxHelmRenderSmokeEvidence, 'function');

  const specs = helper.listExternalReleaseDependencySpecs();
  assert.equal(specs.length, 4);
  assert.deepEqual(
    specs.map((spec) => spec.id),
    ['sdkwork-core', 'sdkwork-ui', 'sdkwork-appbase', 'sdkwork-craw-chat-sdk'],
  );

  const coverage = helper.auditExternalReleaseDependencyCoverage();
  assert.equal(coverage.covered, true);
  assert.deepEqual(coverage.uncoveredReferences, []);
  assert.deepEqual(coverage.externalDependencyIds, ['sdkwork-ui']);

  const referencedSpecs = helper.selectExternalReleaseDependencySpecsForMaterialization({
    specs,
    coverage,
    env: {
      SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: 'referenced',
    },
  });
  assert.deepEqual(
    referencedSpecs.map((spec) => spec.id),
    ['sdkwork-ui'],
  );
}
