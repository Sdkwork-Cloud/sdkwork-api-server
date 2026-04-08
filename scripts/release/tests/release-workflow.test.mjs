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

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    `
export function listExternalReleaseDependencySpecs() {
  return [
    { id: 'sdkwork-core', repository: 'Sdkwork-Cloud/sdkwork-core', envRefKey: 'SDKWORK_CORE_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-ui', repository: 'Sdkwork-Cloud/sdkwork-ui', envRefKey: 'SDKWORK_UI_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-appbase', repository: 'Sdkwork-Cloud/sdkwork-appbase', envRefKey: 'SDKWORK_APPBASE_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-im-sdk', repository: 'Sdkwork-Cloud/sdkwork-im-sdk', envRefKey: 'SDKWORK_IM_SDK_GIT_REF', defaultRef: 'main' },
  ];
}

export function buildExternalReleaseClonePlan() {
  return { command: 'git', args: [] };
}

export function auditExternalReleaseDependencyCoverage() {
  return ${JSON.stringify(coverage, null, 2)};
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-release-telemetry-export.mjs'),
    `
export function resolveReleaseTelemetryExportProducerInput() {
  return { source: 'json', payload: {} };
}

export function materializeReleaseTelemetryExport() {
  return { outputPath: 'docs/release/release-telemetry-export-latest.json' };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    `
export function resolveReleaseWindowSnapshotProducerInput() {
  return { source: 'json', snapshot: {} };
}

export function materializeReleaseWindowSnapshot() {
  return { outputPath: 'docs/release/release-window-snapshot-latest.json' };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    `
export function resolveReleaseSyncAuditProducerInput() {
  return { source: 'json', summary: {} };
}

export function materializeReleaseSyncAudit() {
  return { outputPath: 'docs/release/release-sync-audit-latest.json' };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    `
export function resolveReleaseTelemetryExportInput() {
  return { source: 'json', payload: {} };
}

export function resolveReleaseTelemetrySnapshotInput() {
  return { source: 'json', payload: {} };
}

export function deriveReleaseTelemetrySnapshotFromExport() {
  return {
    generatedAt: '2026-04-08T10:00:00Z',
    source: { kind: 'release-telemetry-export' },
    targets: {},
  };
}

export function validateReleaseTelemetrySnapshotShape() {
  return { snapshotId: 'release-telemetry-snapshot-v1', targetCount: 14 };
}

export function materializeReleaseTelemetrySnapshot() {
  return { outputPath: 'docs/release/release-telemetry-snapshot-latest.json' };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    `
export function resolveSloGovernanceEvidenceInput() {
  return { source: 'json', payload: {} };
}

export function validateSloGovernanceEvidenceShape() {
  return { baselineId: 'release-slo-governance-baseline-2026-04-08', targetCount: 14 };
}

export function materializeSloGovernanceEvidence() {
  return { outputPath: 'docs/release/slo-governance-latest.json' };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-release-governance-bundle.mjs'),
    `
export function listReleaseGovernanceBundleArtifactSpecs() {
  return [];
}

export function createReleaseGovernanceBundleManifest() {
  return {
    version: 1,
    bundleEntryCount: 0,
    artifacts: [],
    restore: {
      command: 'node scripts/release/restore-release-governance-latest.mjs --artifact-dir <downloaded-dir>',
    },
  };
}

export function materializeReleaseGovernanceBundle() {
  return {
    outputDir: 'artifacts/release-governance-bundle',
    bundleEntryCount: 0,
    manifestPath: 'artifacts/release-governance-bundle/release-governance-bundle-manifest.json',
  };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'run-unix-installed-runtime-smoke.mjs'),
    `
export function parseArgs() {
  return {
    platform: 'linux',
    arch: 'x64',
    target: 'x86_64-unknown-linux-gnu',
    runtimeHome: 'artifacts/release-smoke/linux-x64',
    evidencePath: 'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
  };
}

export function createUnixInstalledRuntimeSmokeOptions() {
  return parseArgs();
}

export function createUnixInstalledRuntimeSmokePlan() {
  return {
    runtimeHome: 'artifacts/release-smoke/linux-x64',
    evidencePath: 'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
    installPlan: { directories: ['artifacts/release-smoke/linux-x64'] },
    startCommand: { command: './bin/start.sh', args: ['--home', 'artifacts/release-smoke/linux-x64', '--wait-seconds', '120'] },
    stopCommand: { command: './bin/stop.sh', args: ['--home', 'artifacts/release-smoke/linux-x64', '--wait-seconds', '120'] },
    healthUrls: [],
    routerEnvContents: '',
  };
}

export function createUnixInstalledRuntimeSmokeEvidence() {
  return {
    ok: true,
    platform: 'linux',
    arch: 'x64',
    target: 'x86_64-unknown-linux-gnu',
    runtimeHome: 'artifacts/release-smoke/linux-x64',
    evidencePath: 'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
    healthUrls: [],
  };
}
`,
    'utf8',
  );

  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    `
export function parseArgs() {
  return {
    platform: 'windows',
    arch: 'x64',
    target: 'x86_64-pc-windows-msvc',
    runtimeHome: 'artifacts/release-smoke/windows-x64',
    evidencePath: 'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
  };
}

export function createWindowsInstalledRuntimeSmokeOptions() {
  return parseArgs();
}

export function createWindowsInstalledRuntimeSmokePlan() {
  return {
    runtimeHome: 'artifacts/release-smoke/windows-x64',
    evidencePath: 'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
    installPlan: { directories: ['artifacts/release-smoke/windows-x64'] },
    startCommand: { command: 'powershell.exe', args: ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'bin/start.ps1', '-Home', 'artifacts/release-smoke/windows-x64', '-WaitSeconds', '120'] },
    stopCommand: { command: 'powershell.exe', args: ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'bin/stop.ps1', '-Home', 'artifacts/release-smoke/windows-x64', '-WaitSeconds', '120'] },
    healthUrls: [],
    routerEnvContents: '',
  };
}

export function createWindowsInstalledRuntimeSmokeEvidence() {
  return {
    ok: true,
    platform: 'windows',
    arch: 'x64',
    target: 'x86_64-pc-windows-msvc',
    runtimeHome: 'artifacts/release-smoke/windows-x64',
    evidencePath: 'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
    healthUrls: [],
  };
}
`,
    'utf8',
  );

  return fixtureRoot;
}

test('repository exposes a multi-platform GitHub release workflow for tagged and manual product releases', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /windows-2022/);
  assert.match(workflow, /windows-11-arm/);
  assert.match(workflow, /ubuntu-22\.04/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
  assert.match(workflow, /macos-15-intel/);
  assert.match(workflow, /macos-14/);
  assert.match(workflow, /arch:\s*x64/);
  assert.match(workflow, /arch:\s*arm64/);
  assert.match(workflow, /target:\s*x86_64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*aarch64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*x86_64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*aarch64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*x86_64-apple-darwin/);
  assert.match(workflow, /target:\s*aarch64-apple-darwin/);
  assert.match(workflow, /cargo build --release --target \$\{\{ matrix\.target \}\} -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app admin --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{ matrix\.target \}\}/);
  assert.match(
    workflow,
    /Run installed native runtime smoke on Windows[\s\S]*if: matrix\.platform == 'windows'[\s\S]*node scripts\/release\/run-windows-installed-runtime-smoke\.mjs --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\} --evidence-path artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}\.json/,
  );
  assert.match(
    workflow,
    /Upload Windows installed runtime smoke evidence[\s\S]*if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform == 'windows'\s*\}\}[\s\S]*uses:\s*actions\/upload-artifact@v4[\s\S]*name:\s*release-governance-windows-installed-runtime-smoke-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}[\s\S]*path:\s*artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}\.json/,
  );
  assert.match(
    workflow,
    /Run installed native runtime smoke on Unix[\s\S]*if: matrix\.platform != 'windows'[\s\S]*node scripts\/release\/run-unix-installed-runtime-smoke\.mjs --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\} --evidence-path artifacts\/release-governance\/unix-installed-runtime-smoke-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}\.json/,
  );
  assert.match(
    workflow,
    /Upload Unix installed runtime smoke evidence[\s\S]*if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform != 'windows'\s*\}\}[\s\S]*uses:\s*actions\/upload-artifact@v4[\s\S]*name:\s*release-governance-unix-installed-runtime-smoke-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}[\s\S]*path:\s*artifacts\/release-governance\/unix-installed-runtime-smoke-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}\.json/,
  );
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /pnpm --dir apps\/sdkwork-router-admin build/);
  assert.match(workflow, /pnpm --dir apps\/sdkwork-router-portal build/);
  assert.match(workflow, /pnpm --dir console build/);
  assert.match(workflow, /pnpm --dir docs build/);
  assert.match(workflow, /release-assets-native-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}/);
  assert.match(workflow, /softprops\/action-gh-release@/);
  assert.match(workflow, /permissions:\s*[\s\S]*contents:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*id-token:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*attestations:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*artifact-metadata:\s*write/);
});

test('release workflow materializes GitHub-backed external sibling dependencies before frontend installs', async () => {
  const workflow = read('.github/workflows/release.yml');

  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*node scripts\/release\/materialize-external-deps\.mjs/,
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Install native workspace dependencies/,
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Install web workspace dependencies/,
  );

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

  assert.equal(typeof helper.listExternalReleaseDependencySpecs, 'function');
  assert.equal(typeof helper.buildExternalReleaseClonePlan, 'function');
  assert.equal(typeof helper.auditExternalReleaseDependencyCoverage, 'function');
  assert.equal(typeof releaseWindowHelper.resolveReleaseWindowSnapshotProducerInput, 'function');
  assert.equal(typeof releaseWindowHelper.materializeReleaseWindowSnapshot, 'function');
  assert.equal(typeof releaseSyncHelper.resolveReleaseSyncAuditProducerInput, 'function');
  assert.equal(typeof releaseSyncHelper.materializeReleaseSyncAudit, 'function');
  assert.equal(typeof telemetryExportHelper.resolveReleaseTelemetryExportProducerInput, 'function');
  assert.equal(typeof telemetryExportHelper.materializeReleaseTelemetryExport, 'function');

  const specs = helper.listExternalReleaseDependencySpecs();
  assert.equal(specs.length, 4);
  assert.deepEqual(
    specs.map((spec) => spec.id),
    ['sdkwork-core', 'sdkwork-ui', 'sdkwork-appbase', 'sdkwork-im-sdk'],
  );

  const [sdkworkCoreSpec, sdkworkUiSpec, sdkworkAppbaseSpec, sdkworkImSdkSpec] = specs;
  assert.equal(sdkworkCoreSpec.id, 'sdkwork-core');
  assert.equal(sdkworkCoreSpec.repository, 'Sdkwork-Cloud/sdkwork-core');
  assert.equal(sdkworkCoreSpec.envRefKey, 'SDKWORK_CORE_GIT_REF');
  assert.equal(sdkworkCoreSpec.defaultRef, 'main');
  assert.deepEqual(
    sdkworkCoreSpec.requiredPaths,
    ['package.json'],
  );
  assert.equal(sdkworkUiSpec.id, 'sdkwork-ui');
  assert.equal(sdkworkUiSpec.repository, 'Sdkwork-Cloud/sdkwork-ui');
  assert.equal(sdkworkUiSpec.envRefKey, 'SDKWORK_UI_GIT_REF');
  assert.equal(sdkworkUiSpec.defaultRef, 'main');
  assert.deepEqual(
    sdkworkUiSpec.requiredPaths,
    ['sdkwork-ui-pc-react/package.json'],
  );
  assert.match(
    sdkworkUiSpec.targetDir.replaceAll('\\', '/'),
    /\/sdkwork-ui$/,
  );
  assert.equal(sdkworkAppbaseSpec.id, 'sdkwork-appbase');
  assert.equal(sdkworkAppbaseSpec.repository, 'Sdkwork-Cloud/sdkwork-appbase');
  assert.equal(sdkworkAppbaseSpec.envRefKey, 'SDKWORK_APPBASE_GIT_REF');
  assert.equal(sdkworkAppbaseSpec.defaultRef, 'main');
  assert.deepEqual(
    sdkworkAppbaseSpec.requiredPaths,
    ['package.json'],
  );
  assert.equal(sdkworkImSdkSpec.id, 'sdkwork-im-sdk');
  assert.equal(sdkworkImSdkSpec.repository, 'Sdkwork-Cloud/sdkwork-im-sdk');
  assert.equal(sdkworkImSdkSpec.envRefKey, 'SDKWORK_IM_SDK_GIT_REF');
  assert.equal(sdkworkImSdkSpec.defaultRef, 'main');
  assert.deepEqual(
    sdkworkImSdkSpec.requiredPaths,
    ['README.md'],
  );

  const plan = helper.buildExternalReleaseClonePlan({
    spec: sdkworkUiSpec,
    env: {},
  });
  assert.equal(plan.command, 'git');
  assert.deepEqual(
    plan.args,
    [
      'clone',
      '--depth',
      '1',
      '--branch',
      'main',
      'https://github.com/Sdkwork-Cloud/sdkwork-ui.git',
      sdkworkUiSpec.targetDir,
    ],
  );

  const coverage = helper.auditExternalReleaseDependencyCoverage();
  assert.equal(coverage.covered, true);
  assert.deepEqual(coverage.uncoveredReferences, []);
  assert.deepEqual(coverage.externalDependencyIds, ['sdkwork-ui']);
  assert.ok(
    coverage.references.some((reference) =>
      reference.sourceFile === 'apps/sdkwork-router-admin/package.json'
      && reference.name === '@sdkwork/ui-pc-react',
    ),
    'expected admin package to contribute an external sibling dependency reference',
  );
  assert.ok(
    coverage.references.some((reference) =>
      reference.sourceFile === 'apps/sdkwork-router-portal/pnpm-workspace.yaml'
      && reference.kind === 'pnpm-workspace',
    ),
    'expected portal workspace config to contribute an external sibling dependency reference',
  );
  assert.ok(
    coverage.references.some((reference) =>
      reference.sourceFile === 'apps/sdkwork-router-portal/tsconfig.json'
      && reference.kind === 'tsconfig-path',
    ),
    'expected portal tsconfig paths to contribute an external sibling dependency reference',
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Run release governance gate[\s\S]*Install native workspace dependencies/,
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Run release governance gate[\s\S]*Install web workspace dependencies/,
  );
  assert.match(
    workflow,
    /Run release governance gate[\s\S]*node scripts\/release\/run-release-governance-checks\.mjs --format json/,
  );
  assert.match(
    workflow,
    /Materialize release window snapshot[\s\S]*node scripts\/release\/materialize-release-window-snapshot\.mjs/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize release window snapshot[\s\S]*?SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON:[\s\S]*?run: node scripts\/release\/materialize-release-window-snapshot\.mjs/,
    'native release job must materialize a governed release-window snapshot artifact before the governance gate',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release window snapshot[\s\S]*?SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON:[\s\S]*?run: node scripts\/release\/materialize-release-window-snapshot\.mjs/,
    'web release job must materialize a governed release-window snapshot artifact before the governance gate',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload release window snapshot governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-window-snapshot-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/release-window-snapshot-latest\.json/,
    'native release job must upload the governed release-window snapshot as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release window snapshot governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-window-snapshot-web[\s\S]*?path:\s*docs\/release\/release-window-snapshot-latest\.json/,
    'web release job must upload the governed release-window snapshot as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /Materialize release sync audit[\s\S]*node scripts\/release\/materialize-release-sync-audit\.mjs/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_JSON:[\s\S]*?SDKWORK_API_ROUTER_GIT_REF:[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_IM_SDK_GIT_REF:[\s\S]*?run: node scripts\/release\/materialize-release-sync-audit\.mjs/,
    'native release job must materialize a governed release-sync audit artifact before the governance gate',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_JSON:[\s\S]*?SDKWORK_API_ROUTER_GIT_REF:[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_IM_SDK_GIT_REF:[\s\S]*?run: node scripts\/release\/materialize-release-sync-audit\.mjs/,
    'web release job must materialize a governed release-sync audit artifact before the governance gate',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload release sync audit governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-sync-audit-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/release-sync-audit-latest\.json/,
    'native release job must upload the governed release-sync audit as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release sync audit governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-sync-audit-web[\s\S]*?path:\s*docs\/release\/release-sync-audit-latest\.json/,
    'web release job must upload the governed release-sync audit as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /Materialize release telemetry export[\s\S]*node scripts\/release\/materialize-release-telemetry-export\.mjs/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize release telemetry export[\s\S]*?SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON:[\s\S]*?run: node scripts\/release\/materialize-release-telemetry-export\.mjs/,
    'native release job must materialize a governed telemetry export artifact before snapshot derivation',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release telemetry export[\s\S]*?SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON:[\s\S]*?run: node scripts\/release\/materialize-release-telemetry-export\.mjs/,
    'web release job must materialize a governed telemetry export artifact before snapshot derivation',
  );
  assert.match(
    workflow,
    /Materialize release telemetry snapshot[\s\S]*node scripts\/release\/materialize-release-telemetry-snapshot\.mjs/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize release telemetry snapshot[\s\S]*?SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH:[\s\S]*?release-telemetry-export-latest\.json[\s\S]*?run: node scripts\/release\/materialize-release-telemetry-snapshot\.mjs/,
    'native release job must wire the governed telemetry export artifact into the snapshot materializer step',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release telemetry snapshot[\s\S]*?SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH:[\s\S]*?release-telemetry-export-latest\.json[\s\S]*?run: node scripts\/release\/materialize-release-telemetry-snapshot\.mjs/,
    'web release job must wire the governed telemetry export artifact into the snapshot materializer step',
  );
  assert.match(
    workflow,
    /Materialize SLO governance evidence[\s\S]*node scripts\/release\/materialize-slo-governance-evidence\.mjs/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload release telemetry export governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-export-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/release-telemetry-export-latest\.json/,
    'native release job must upload the governed telemetry export as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload release telemetry snapshot governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-snapshot-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/release-telemetry-snapshot-latest\.json/,
    'native release job must upload the governed telemetry snapshot as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload SLO governance evidence artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-slo-evidence-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/slo-governance-latest\.json/,
    'native release job must upload governed SLO evidence as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release telemetry export governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-export-web[\s\S]*?path:\s*docs\/release\/release-telemetry-export-latest\.json/,
    'web release job must upload the governed telemetry export as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release telemetry snapshot governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-snapshot-web[\s\S]*?path:\s*docs\/release\/release-telemetry-snapshot-latest\.json/,
    'web release job must upload the governed telemetry snapshot as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload SLO governance evidence artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-slo-evidence-web[\s\S]*?path:\s*docs\/release\/slo-governance-latest\.json/,
    'web release job must upload governed SLO evidence as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize SLO governance evidence[\s\S]*?SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH:[\s\S]*?release-telemetry-snapshot-latest\.json[\s\S]*?run: node scripts\/release\/materialize-slo-governance-evidence\.mjs/,
    'native release job must wire the governed telemetry snapshot artifact into the SLO materializer step',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize SLO governance evidence[\s\S]*?SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH:[\s\S]*?release-telemetry-snapshot-latest\.json[\s\S]*?run: node scripts\/release\/materialize-slo-governance-evidence\.mjs/,
    'web release job must wire the governed telemetry snapshot artifact into the SLO materializer step',
  );
  assert.match(
    workflow,
    /Materialize release telemetry export[\s\S]*Materialize release telemetry snapshot[\s\S]*Materialize SLO governance evidence[\s\S]*Run release governance gate/,
  );
  assert.match(
    workflow,
    /Materialize release telemetry export[\s\S]*Upload release telemetry export governance artifact[\s\S]*Materialize release telemetry snapshot[\s\S]*Upload release telemetry snapshot governance artifact[\s\S]*Materialize SLO governance evidence[\s\S]*Upload SLO governance evidence artifact[\s\S]*Run release governance gate/,
    'workflow must persist telemetry export, telemetry snapshot, and SLO evidence artifacts before the governance gate runs',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate governance evidence attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*\|[\s\S]*?docs\/release\/release-window-snapshot-latest\.json[\s\S]*?docs\/release\/release-sync-audit-latest\.json[\s\S]*?docs\/release\/release-telemetry-export-latest\.json[\s\S]*?docs\/release\/release-telemetry-snapshot-latest\.json[\s\S]*?docs\/release\/slo-governance-latest\.json/,
    'native release job must attest governed window, sync, telemetry export, telemetry snapshot, and SLO evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate Windows smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\)\s*&&\s*matrix\.platform\s*==\s*'windows'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?path:\s*artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Windows release lanes must attest installed-runtime smoke evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate Unix smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\)\s*&&\s*matrix\.platform\s*!=\s*'windows'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?path:\s*artifacts\/release-governance\/unix-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Unix release lanes must attest installed-runtime smoke evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate native release assets attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release\/\*\*\/*/,
    'native release job must attest packaged release assets when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Generate governance evidence attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*\|[\s\S]*?docs\/release\/release-window-snapshot-latest\.json[\s\S]*?docs\/release\/release-sync-audit-latest\.json[\s\S]*?docs\/release\/release-telemetry-export-latest\.json[\s\S]*?docs\/release\/release-telemetry-snapshot-latest\.json[\s\S]*?docs\/release\/slo-governance-latest\.json/,
    'web release job must attest governed window, sync, telemetry export, telemetry snapshot, and SLO evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Generate web release assets attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release\/\*\*\/*/,
    'web release job must attest packaged release assets when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /Build portal desktop release[\s\S]*Run installed native runtime smoke on Windows[\s\S]*Upload Windows installed runtime smoke evidence[\s\S]*Run installed native runtime smoke on Unix[\s\S]*Upload Unix installed runtime smoke evidence[\s\S]*Collect native release assets/,
    'native release workflow must execute Windows and Unix install-asset smoke gates, persist their evidence, and only then package assets',
  );
  assert.match(
    workflow,
    /Build portal desktop release[\s\S]*Run installed native runtime smoke on Unix[\s\S]*Upload Unix installed runtime smoke evidence[\s\S]*Collect native release assets/,
    'native release workflow must execute install-asset smoke, persist its evidence, and only then package assets',
  );
  assert.match(
    workflow,
    /Upload release window snapshot governance artifact[\s\S]*Upload release sync audit governance artifact[\s\S]*Upload release telemetry export governance artifact[\s\S]*Upload release telemetry snapshot governance artifact[\s\S]*Upload SLO governance evidence artifact[\s\S]*Generate governance evidence attestation[\s\S]*Run release governance gate/,
    'governed release evidence must be persisted before it is attested and evaluated',
  );
  assert.match(
    workflow,
    /Collect native release assets[\s\S]*Upload native release assets[\s\S]*Generate native release assets attestation/,
    'native release assets must be packaged and uploaded before attestation generation',
  );
  assert.match(
    workflow,
    /Package web release assets[\s\S]*Upload web release assets[\s\S]*Generate web release assets attestation/,
    'web release assets must be packaged and uploaded before attestation generation',
  );
});

test('release workflow contract helper rejects workflows that omit governed repository ref env wiring', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install web workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that omit the governed SLO evidence materialization step', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install web workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that omit telemetry snapshot governance artifact uploads', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Upload SLO governance evidence artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-slo-evidence-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: docs/release/slo-governance-latest.json

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Upload SLO governance evidence artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-slo-evidence-web
          path: docs/release/slo-governance-latest.json

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install web workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that omit SLO governance evidence artifact uploads', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Upload release telemetry snapshot governance artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-telemetry-snapshot-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: docs/release/release-telemetry-snapshot-latest.json

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Upload release telemetry snapshot governance artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-telemetry-snapshot-web
          path: docs/release/release-telemetry-snapshot-latest.json

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install web workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that omit artifact attestation permissions', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Upload release telemetry snapshot governance artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-telemetry-snapshot-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: docs/release/release-telemetry-snapshot-latest.json

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Upload SLO governance evidence artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-slo-evidence-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: docs/release/slo-governance-latest.json

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

      - name: Collect native release assets
        run: node scripts/release/package-release-assets.mjs native --platform \${{ matrix.platform }} --arch \${{ matrix.arch }} --target \${{ matrix.target }} --output-dir artifacts/release

      - name: Upload native release assets
        uses: actions/upload-artifact@v4
        with:
          name: release-assets-native-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: artifacts/release/**/*

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Upload release telemetry snapshot governance artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-telemetry-snapshot-web
          path: docs/release/release-telemetry-snapshot-latest.json

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Upload SLO governance evidence artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-slo-evidence-web
          path: docs/release/slo-governance-latest.json

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

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

test('release workflow contract helper rejects workflows that omit build provenance attestation steps', async () => {
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
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Upload release telemetry snapshot governance artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-telemetry-snapshot-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: docs/release/release-telemetry-snapshot-latest.json

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Upload SLO governance evidence artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-slo-evidence-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: docs/release/slo-governance-latest.json

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Run installed native runtime smoke on Unix
        if: matrix.platform != 'windows'
        run: node scripts/release/run-unix-installed-runtime-smoke.mjs --platform \${{ matrix.platform }} --arch \${{ matrix.arch }} --target \${{ matrix.target }} --evidence-path artifacts/release-governance/unix-installed-runtime-smoke-\${{ matrix.platform }}-\${{ matrix.arch }}.json

      - name: Upload Unix installed runtime smoke evidence
        if: \${{ always() && matrix.platform != 'windows' }}
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-unix-installed-runtime-smoke-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: artifacts/release-governance/unix-installed-runtime-smoke-\${{ matrix.platform }}-\${{ matrix.arch }}.json

      - name: Collect native release assets
        run: node scripts/release/package-release-assets.mjs native --platform \${{ matrix.platform }} --arch \${{ matrix.arch }} --target \${{ matrix.target }} --output-dir artifacts/release

      - name: Upload native release assets
        uses: actions/upload-artifact@v4
        with:
          name: release-assets-native-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: artifacts/release/**/*

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Upload release telemetry snapshot governance artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-telemetry-snapshot-web
          path: docs/release/release-telemetry-snapshot-latest.json

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Upload SLO governance evidence artifact
        uses: actions/upload-artifact@v4
        with:
          name: release-governance-slo-evidence-web
          path: docs/release/slo-governance-latest.json

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

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

test('release workflow contract helper rejects workflows that omit the Unix installed runtime smoke gate', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

      - name: Build release service binaries on Unix
        if: matrix.platform != 'windows'
        run: cargo build --release --target \${{ matrix.target }} -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service

      - name: Build admin desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app admin --target \${{ matrix.target }}

      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}

      - name: Collect native release assets
        run: node scripts/release/package-release-assets.mjs native --platform \${{ matrix.platform }} --arch \${{ matrix.arch }} --target \${{ matrix.target }} --output-dir artifacts/release

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install web workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that omit the Unix installed runtime smoke evidence artifact wiring', async () => {
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

jobs:
  native-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install native workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile

      - name: Build release service binaries on Unix
        if: matrix.platform != 'windows'
        run: cargo build --release --target \${{ matrix.target }} -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service

      - name: Build admin desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app admin --target \${{ matrix.target }}

      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}

      - name: Run installed native runtime smoke on Unix
        if: matrix.platform != 'windows'
        run: node scripts/release/run-unix-installed-runtime-smoke.mjs --platform \${{ matrix.platform }} --arch \${{ matrix.arch }} --target \${{ matrix.target }} --evidence-path artifacts/release-governance/unix-installed-runtime-smoke-\${{ matrix.platform }}-\${{ matrix.arch }}.json

      - name: Collect native release assets
        run: node scripts/release/package-release-assets.mjs native --platform \${{ matrix.platform }} --arch \${{ matrix.arch }} --target \${{ matrix.target }} --output-dir artifacts/release

  web-release:
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/materialize-external-deps.mjs

      - name: Materialize release telemetry snapshot
        env:
          SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON: \${{ vars.SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON || '' }}
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs

      - name: Materialize SLO governance evidence
        env:
          SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH: docs/release/release-telemetry-snapshot-latest.json
        run: node scripts/release/materialize-slo-governance-evidence.mjs

      - name: Run release governance gate
        env:
          SDKWORK_API_ROUTER_GIT_REF: \${{ needs.prepare.outputs.git_ref }}
          SDKWORK_CORE_GIT_REF: \${{ vars.SDKWORK_CORE_GIT_REF || 'main' }}
          SDKWORK_UI_GIT_REF: \${{ vars.SDKWORK_UI_GIT_REF || 'main' }}
          SDKWORK_APPBASE_GIT_REF: \${{ vars.SDKWORK_APPBASE_GIT_REF || 'main' }}
          SDKWORK_IM_SDK_GIT_REF: \${{ vars.SDKWORK_IM_SDK_GIT_REF || 'main' }}
        run: node scripts/release/run-release-governance-checks.mjs --format json

      - name: Install web workspace dependencies
        run: pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('portal package exposes unified product launchers and all desktop scripts use the shared tauri runner', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const packageJson = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const consolePackage = JSON.parse(read('console/package.json'));
  const runnerPath = path.join(repoRoot, 'scripts', 'run-tauri-cli.mjs');

  assert.equal(existsSync(runnerPath), true, 'missing shared scripts/run-tauri-cli.mjs');
  assert.match(packageJson.scripts['product:start'], /node \.\.\/\.\.\/scripts\/run-router-product\.mjs/);
  assert.match(packageJson.scripts['product:service'], /node \.\.\/\.\.\/scripts\/run-router-product\.mjs service/);
  assert.match(packageJson.scripts['server:start'], /node \.\.\/\.\.\/scripts\/run-router-product-service\.mjs/);
  assert.match(packageJson.scripts['server:plan'], /node \.\.\/\.\.\/scripts\/run-router-product-service\.mjs --dry-run --plan-format json/);
  assert.match(packageJson.scripts['product:check'], /node \.\.\/\.\.\/scripts\/check-router-product\.mjs/);
  assert.match(adminPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(adminPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(packageJson.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(packageJson.scripts['tauri:dev:service'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev -- --service/);
  assert.match(packageJson.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consolePackage.scripts['tauri:dev'], /node \.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(consolePackage.scripts['tauri:build'], /node \.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.doesNotMatch(packageJson.scripts['tauri:dev'], /powershell/i);
  assert.doesNotMatch(packageJson.scripts['tauri:build'], /powershell/i);
});

test('release workflow exposes a single governance bundle artifact for operator restore', async () => {
  const workflow = read('.github/workflows/release.yml');

  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release governance bundle[\s\S]*?run: node scripts\/release\/materialize-release-governance-bundle\.mjs/,
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release governance bundle artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-bundle-web[\s\S]*?path:\s*artifacts\/release-governance-bundle\/\*\*\/*/,
  );

  const helper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-governance-bundle.mjs'),
    ).href,
  );

  assert.equal(typeof helper.listReleaseGovernanceBundleArtifactSpecs, 'function');
  assert.equal(typeof helper.createReleaseGovernanceBundleManifest, 'function');
  assert.equal(typeof helper.materializeReleaseGovernanceBundle, 'function');
});

test('shared tauri runner only injects the Visual Studio generator on Windows and carries explicit release target metadata', async () => {
  const runner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-tauri-cli.mjs'),
    ).href,
  );

  assert.equal(typeof runner.createTauriCliPlan, 'function');
  assert.equal(typeof runner.withSupportedWindowsCmakeGenerator, 'function');

  const windowsPlan = runner.createTauriCliPlan({
    commandName: 'build',
    args: ['--target', 'aarch64-pc-windows-msvc'],
    platform: 'win32',
    env: {},
  });
  const linuxPlan = runner.createTauriCliPlan({
    commandName: 'build',
    args: ['--target', 'x86_64-unknown-linux-gnu'],
    platform: 'linux',
    env: {},
  });
  const backgroundDevPlan = runner.createTauriCliPlan({
    commandName: 'dev',
    args: ['--', '--service'],
    platform: 'linux',
    env: {},
  });

  assert.equal(windowsPlan.command, 'tauri.cmd');
  assert.deepEqual(windowsPlan.args, ['build', '--target', 'aarch64-pc-windows-msvc']);
  assert.equal(windowsPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(windowsPlan.env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(windowsPlan.env.SDKWORK_DESKTOP_TARGET, 'aarch64-pc-windows-msvc');
  assert.equal(windowsPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'windows');
  assert.equal(windowsPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'arm64');
  assert.equal(windowsPlan.windowsHide, true);

  assert.equal(linuxPlan.command, 'tauri');
  assert.deepEqual(linuxPlan.args, ['build', '--target', 'x86_64-unknown-linux-gnu']);
  assert.equal(linuxPlan.env.SDKWORK_DESKTOP_TARGET, 'x86_64-unknown-linux-gnu');
  assert.equal(linuxPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'linux');
  assert.equal(linuxPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'x64');
  assert.equal(Object.hasOwn(linuxPlan.env, 'CMAKE_GENERATOR'), false);
  assert.equal(backgroundDevPlan.detached, true);
  assert.equal(backgroundDevPlan.windowsHide, false);
});

test('shared tauri runner prepends the local cargo bin directory on Windows', async () => {
  if (process.platform !== 'win32') {
    return;
  }

  const runner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-tauri-cli.mjs'),
    ).href,
  );

  const plan = runner.createTauriCliPlan({
    commandName: 'dev',
    platform: 'win32',
    env: {
      USERPROFILE: process.env.USERPROFILE ?? '',
      PATH: '',
    },
  });

  const expectedCargoBin = path.join(process.env.USERPROFILE ?? '', '.cargo', 'bin').toLowerCase();
  assert.ok(
    String(plan.env.PATH ?? '')
      .toLowerCase()
      .startsWith(expectedCargoBin),
    'cargo bin should be the first PATH entry for tauri commands',
  );
  assert.match(
    String(plan.env.CARGO_TARGET_DIR ?? ''),
    /sdkwork-tauri-target/i,
  );
});
