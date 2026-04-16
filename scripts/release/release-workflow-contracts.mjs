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

  const workflow = read(repoRoot, '.github/workflows/release.yml');

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /permissions:\s*[\s\S]*contents:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*id-token:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*attestations:\s*write/);
  assert.match(workflow, /permissions:\s*[\s\S]*artifact-metadata:\s*write/);
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*node scripts\/release\/materialize-external-deps\.mjs/,
  );
  assert.match(
    workflow,
    /rust-dependency-audit:[\s\S]*?runs-on:\s*ubuntu-latest[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?dtolnay\/rust-toolchain@stable[\s\S]*?Swatinem\/rust-cache@v2[\s\S]*?taiki-e\/install-action@cargo-audit[\s\S]*?node scripts\/check-rust-dependency-audit\.mjs/,
    'release workflow must execute a dedicated Rust dependency audit gate against the exact release ref before any assets are built',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit/,
    'native release job must wait for the dedicated Rust dependency audit gate',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit/,
    'web release job must wait for the dedicated Rust dependency audit gate',
  );
  assert.match(
    workflow,
    /product-verification:[\s\S]*?runs-on:\s*ubuntu-latest[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?pnpm\/action-setup@v4[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?dtolnay\/rust-toolchain@stable[\s\S]*?Swatinem\/rust-cache@v2[\s\S]*?taiki-e\/install-action@cargo-audit[\s\S]*?node scripts\/check-router-product\.mjs/,
    'release workflow must execute a dedicated product verification gate against the exact release ref before any assets are built',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit[\r\n]+\s*-\s*product-verification/,
    'native release job must wait for the dedicated product verification gate',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit[\r\n]+\s*-\s*product-verification/,
    'web release job must wait for the dedicated product verification gate',
  );
  assert.match(
    workflow,
    /product-verification:[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?Run release product verification[\s\S]*?node scripts\/check-router-product\.mjs/,
    'product verification frozen install discipline must be explicit before release product verification runs',
  );
  assert.match(
    workflow,
    /product-verification:[\s\S]*?Run release product verification[\s\S]*?env:[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?run:\s*node scripts\/check-router-product\.mjs/,
    'strict frontend install mode must be exported before release product verification runs',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_CRAW_CHAT_SDK_GIT_REF:[\s\S]*?run: node scripts\/release\/materialize-external-deps\.mjs/,
    'native release job must wire all governed sibling refs into external dependency materialization',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_CRAW_CHAT_SDK_GIT_REF:[\s\S]*?run: node scripts\/release\/materialize-external-deps\.mjs/,
    'web release job must wire all governed sibling refs into external dependency materialization',
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Install native workspace dependencies/,
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Install web workspace dependencies/,
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
    /native-release:[\s\S]*?Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_JSON:[\s\S]*?SDKWORK_API_ROUTER_GIT_REF:[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_CRAW_CHAT_SDK_GIT_REF:[\s\S]*?run: node scripts\/release\/materialize-release-sync-audit\.mjs/,
    'native release job must materialize a governed release-sync audit artifact before the governance gate',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_JSON:[\s\S]*?SDKWORK_API_ROUTER_GIT_REF:[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_CRAW_CHAT_SDK_GIT_REF:[\s\S]*?run: node scripts\/release\/materialize-release-sync-audit\.mjs/,
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
    /native-release:[\s\S]*?Upload release telemetry export governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-export-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/release-telemetry-export-latest\.json/,
    'native release job must upload the governed telemetry export as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release telemetry export governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-export-web[\s\S]*?path:\s*docs\/release\/release-telemetry-export-latest\.json/,
    'web release job must upload the governed telemetry export as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload release telemetry snapshot governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-snapshot-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/release-telemetry-snapshot-latest\.json/,
    'native release job must upload the governed telemetry snapshot as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release telemetry snapshot governance artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-telemetry-snapshot-web[\s\S]*?path:\s*docs\/release\/release-telemetry-snapshot-latest\.json/,
    'web release job must upload the governed telemetry snapshot as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /Materialize SLO governance evidence[\s\S]*node scripts\/release\/materialize-slo-governance-evidence\.mjs/,
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
    /native-release:[\s\S]*?Upload SLO governance evidence artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-slo-evidence-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*docs\/release\/slo-governance-latest\.json/,
    'native release job must upload governed SLO evidence as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload SLO governance evidence artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-slo-evidence-web[\s\S]*?path:\s*docs\/release\/slo-governance-latest\.json/,
    'web release job must upload governed SLO evidence as a dedicated release-governance artifact',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Materialize release governance bundle[\s\S]*?run: node scripts\/release\/materialize-release-governance-bundle\.mjs/,
    'web release job must materialize a single governance bundle for restore operators after the governed latest artifacts exist',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Upload release governance bundle artifact[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-bundle-web[\s\S]*?path:\s*artifacts\/release-governance-bundle\/\*\*\/*/,
    'web release job must publish the restore-oriented governance bundle as a dedicated artifact',
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Materialize release window snapshot[\s\S]*Upload release window snapshot governance artifact[\s\S]*Materialize release sync audit[\s\S]*Upload release sync audit governance artifact[\s\S]*Materialize release telemetry export[\s\S]*Upload release telemetry export governance artifact[\s\S]*Materialize release telemetry snapshot[\s\S]*Upload release telemetry snapshot governance artifact[\s\S]*Materialize SLO governance evidence[\s\S]*Upload SLO governance evidence artifact[\s\S]*Run release governance gate[\s\S]*Install native workspace dependencies/,
  );
  assert.match(
    workflow,
    /Materialize external release dependencies[\s\S]*Materialize release window snapshot[\s\S]*Upload release window snapshot governance artifact[\s\S]*Materialize release sync audit[\s\S]*Upload release sync audit governance artifact[\s\S]*Materialize release telemetry export[\s\S]*Upload release telemetry export governance artifact[\s\S]*Materialize release telemetry snapshot[\s\S]*Upload release telemetry snapshot governance artifact[\s\S]*Materialize SLO governance evidence[\s\S]*Upload SLO governance evidence artifact[\s\S]*Materialize release governance bundle[\s\S]*Upload release governance bundle artifact[\s\S]*Run release governance gate[\s\S]*Install web workspace dependencies/,
  );
  assert.match(
    workflow,
    /Run release governance gate[\s\S]*node scripts\/release\/run-release-governance-checks\.mjs --format json/,
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Run release governance gate[\s\S]*?SDKWORK_API_ROUTER_GIT_REF:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_CRAW_CHAT_SDK_GIT_REF:[\s\S]*?SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH:[\s\S]*?release-window-snapshot-latest\.json[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_PATH:[\s\S]*?release-sync-audit-latest\.json[\s\S]*?run: node scripts\/release\/run-release-governance-checks\.mjs --format json/,
    'native release job must wire the main and sibling repository refs plus governed release-window and release-sync artifacts into the governance gate',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate governance evidence attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*\|[\s\S]*?docs\/release\/release-window-snapshot-latest\.json[\s\S]*?docs\/release\/release-sync-audit-latest\.json[\s\S]*?docs\/release\/release-telemetry-export-latest\.json[\s\S]*?docs\/release\/release-telemetry-snapshot-latest\.json[\s\S]*?docs\/release\/slo-governance-latest\.json/,
    'native release job must attest governed window, sync, telemetry export, telemetry snapshot, and SLO evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate Windows smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\)\s*&&\s*matrix\.platform\s*==\s*'windows'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Windows release lanes must attest installed-runtime smoke evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate Unix smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\)\s*&&\s*matrix\.platform\s*!=\s*'windows'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/unix-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Unix release lanes must attest installed-runtime smoke evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Run Linux Docker Compose packaged product smoke[\s\S]*?if:\s*matrix\.platform == 'linux'[\s\S]*?run:\s*node scripts\/release\/run-linux-docker-compose-smoke\.mjs --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --bundle-path artifacts\/release\/native\/\$\{\{\s*matrix\.platform\s*\}\}\/\$\{\{\s*matrix\.arch\s*\}\}\/bundles\/sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.tar\.gz --evidence-path artifacts\/release-governance\/docker-compose-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Linux release lanes must run a packaged Docker Compose smoke gate against the bundled product-server archive before asset upload',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload Linux Docker Compose smoke evidence[\s\S]*?if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform == 'linux'\s*\}\}[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-docker-compose-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/docker-compose-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Linux release lanes must upload packaged Docker Compose smoke evidence as a dedicated governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate Linux Docker Compose smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\)\s*&&\s*matrix\.platform\s*==\s*'linux'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/docker-compose-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Linux release lanes must attest packaged Docker Compose smoke evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Run Linux Helm render packaged smoke[\s\S]*?if:\s*matrix\.platform == 'linux'[\s\S]*?run:\s*node scripts\/release\/run-linux-helm-render-smoke\.mjs --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --bundle-path artifacts\/release\/native\/\$\{\{\s*matrix\.platform\s*\}\}\/\$\{\{\s*matrix\.arch\s*\}\}\/bundles\/sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.tar\.gz --evidence-path artifacts\/release-governance\/helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Linux release lanes must run a packaged Helm render smoke gate against the bundled product-server archive before asset upload',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload Linux Helm render smoke evidence[\s\S]*?if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform == 'linux'\s*\}\}[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Linux release lanes must upload packaged Helm render smoke evidence as a dedicated governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate Linux Helm render smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\)\s*&&\s*matrix\.platform\s*==\s*'linux'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native Linux release lanes must attest packaged Helm render smoke evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Run installed native runtime smoke on Windows[\s\S]*?if:\s*matrix\.platform == 'windows'[\s\S]*?run:\s*node scripts\/release\/run-windows-installed-runtime-smoke\.mjs --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --target \$\{\{\s*matrix\.target\s*\}\} --evidence-path artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native release job must run a dedicated Windows installed-runtime smoke gate with an explicit evidence artifact path before packaging release assets',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload Windows installed runtime smoke evidence[\s\S]*?if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform == 'windows'\s*\}\}[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native release job must upload Windows installed-runtime smoke evidence as a dedicated governance artifact',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Run installed native runtime smoke on Unix[\s\S]*?if:\s*matrix\.platform != 'windows'[\s\S]*?run:\s*node scripts\/release\/run-unix-installed-runtime-smoke\.mjs --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --target \$\{\{\s*matrix\.target\s*\}\} --evidence-path artifacts\/release-governance\/unix-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native release job must run a dedicated Unix installed-runtime smoke gate with an explicit evidence artifact path before packaging release assets',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Upload Unix installed runtime smoke evidence[\s\S]*?if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform != 'windows'\s*\}\}[\s\S]*?uses:\s*actions\/upload-artifact@v4[\s\S]*?name:\s*release-governance-unix-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/unix-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json/,
    'native release job must upload Unix installed-runtime smoke evidence as a dedicated governance artifact',
  );
  assert.match(
    workflow,
    /Build portal desktop release[\s\S]*Run installed native runtime smoke on Windows[\s\S]*Upload Windows installed runtime smoke evidence[\s\S]*Run installed native runtime smoke on Unix[\s\S]*Upload Unix installed runtime smoke evidence[\s\S]*Collect native release assets[\s\S]*Run Linux Docker Compose packaged product smoke[\s\S]*Upload Linux Docker Compose smoke evidence[\s\S]*Run Linux Helm render packaged smoke[\s\S]*Upload Linux Helm render smoke evidence[\s\S]*Upload native release assets/,
    'native release workflow must place installed-runtime smokes before packaging, then execute Linux packaged bundle smokes before asset upload',
  );
  assert.match(
    workflow,
    /Build portal desktop release[\s\S]*Run installed native runtime smoke on Unix[\s\S]*Upload Unix installed runtime smoke evidence[\s\S]*Collect native release assets[\s\S]*Run Linux Docker Compose packaged product smoke[\s\S]*Upload Linux Docker Compose smoke evidence[\s\S]*Run Linux Helm render packaged smoke[\s\S]*Upload Linux Helm render smoke evidence[\s\S]*Upload native release assets/,
    'native release workflow must place the Unix installed-runtime smoke gate before packaging and Linux packaged bundle smokes before asset upload',
  );
  assert.match(
    workflow,
    /Upload release window snapshot governance artifact[\s\S]*Upload release sync audit governance artifact[\s\S]*Upload release telemetry export governance artifact[\s\S]*Upload release telemetry snapshot governance artifact[\s\S]*Upload SLO governance evidence artifact[\s\S]*Materialize release governance bundle[\s\S]*Upload release governance bundle artifact[\s\S]*Generate governance evidence attestation[\s\S]*Run release governance gate/,
    'governed release evidence must be persisted, bundled for restore, and then evaluated',
  );
  assert.match(
    workflow,
    /Collect native release assets[\s\S]*Run Linux Docker Compose packaged product smoke[\s\S]*Upload Linux Docker Compose smoke evidence[\s\S]*Generate Linux Docker Compose smoke evidence attestation[\s\S]*Run Linux Helm render packaged smoke[\s\S]*Upload Linux Helm render smoke evidence[\s\S]*Generate Linux Helm render smoke evidence attestation[\s\S]*Upload native release assets[\s\S]*Generate native release assets attestation/,
    'native release assets must follow Linux packaged bundle smoke evidence upload and attestation before the final asset attestation runs',
  );
  assert.match(
    workflow,
    /native-release:[\s\S]*?Generate native release assets attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release\/\*\*\/*/,
    'native release job must attest packaged release assets when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Run release governance gate[\s\S]*?SDKWORK_API_ROUTER_GIT_REF:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?SDKWORK_CORE_GIT_REF:[\s\S]*?SDKWORK_UI_GIT_REF:[\s\S]*?SDKWORK_APPBASE_GIT_REF:[\s\S]*?SDKWORK_CRAW_CHAT_SDK_GIT_REF:[\s\S]*?SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH:[\s\S]*?release-window-snapshot-latest\.json[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_PATH:[\s\S]*?release-sync-audit-latest\.json[\s\S]*?run: node scripts\/release\/run-release-governance-checks\.mjs --format json/,
    'web release job must wire the main and sibling repository refs plus governed release-window and release-sync artifacts into the governance gate',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Generate governance evidence attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*\|[\s\S]*?docs\/release\/release-window-snapshot-latest\.json[\s\S]*?docs\/release\/release-sync-audit-latest\.json[\s\S]*?docs\/release\/release-telemetry-export-latest\.json[\s\S]*?docs\/release\/release-telemetry-snapshot-latest\.json[\s\S]*?docs\/release\/slo-governance-latest\.json/,
    'web release job must attest governed window, sync, telemetry export, telemetry snapshot, and SLO evidence when artifact attestations are supported',
  );
  assert.match(
    workflow,
    /Package web release assets[\s\S]*Upload web release assets[\s\S]*Generate web release assets attestation/,
    'web release assets must be packaged and uploaded before attestation generation',
  );
  assert.match(
    workflow,
    /web-release:[\s\S]*?Generate web release assets attestation[\s\S]*?if:\s*\$\{\{\s*!github\.event\.repository\.private\s*\|\|\s*vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED\s*==\s*'true'\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release\/\*\*\/*/,
    'web release job must attest packaged release assets when artifact attestations are supported',
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
  assert.equal(specs[0].repository, 'Sdkwork-Cloud/sdkwork-core');
  assert.equal(specs[0].envRefKey, 'SDKWORK_CORE_GIT_REF');
  assert.equal(specs[0].defaultRef, 'main');
  assert.equal(specs[1].repository, 'Sdkwork-Cloud/sdkwork-ui');
  assert.equal(specs[1].envRefKey, 'SDKWORK_UI_GIT_REF');
  assert.equal(specs[1].defaultRef, 'main');
  assert.equal(specs[2].repository, 'Sdkwork-Cloud/sdkwork-appbase');
  assert.equal(specs[2].envRefKey, 'SDKWORK_APPBASE_GIT_REF');
  assert.equal(specs[2].defaultRef, 'main');
  assert.equal(specs[3].repository, 'Sdkwork-Cloud/craw-chat');
  assert.equal(specs[3].envRefKey, 'SDKWORK_CRAW_CHAT_SDK_GIT_REF');
  assert.equal(specs[3].defaultRef, 'main');

  const coverage = helper.auditExternalReleaseDependencyCoverage();
  assert.equal(coverage.covered, true);
  assert.deepEqual(coverage.uncoveredReferences, []);
  assert.deepEqual(coverage.externalDependencyIds, ['sdkwork-ui']);
}
