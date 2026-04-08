import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

export async function assertRuntimeToolingContracts({
  repoRoot,
} = {}) {
  const runtimeToolingModulePath = path.join(repoRoot, 'bin', 'lib', 'router-runtime-tooling.mjs');
  const runtimeToolingTestPath = path.join(repoRoot, 'bin', 'tests', 'router-runtime-tooling.test.mjs');

  assert.equal(existsSync(runtimeToolingModulePath), true, 'missing bin/lib/router-runtime-tooling.mjs');
  assert.equal(existsSync(runtimeToolingTestPath), true, 'missing bin/tests/router-runtime-tooling.test.mjs');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'start.sh')), true, 'missing bin/start.sh');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'stop.sh')), true, 'missing bin/stop.sh');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'start.ps1')), true, 'missing bin/start.ps1');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'stop.ps1')), true, 'missing bin/stop.ps1');

  const module = await import(pathToFileURL(runtimeToolingModulePath).href);
  assert.equal(typeof module.createReleaseBuildPlan, 'function');
  assert.equal(typeof module.createInstallPlan, 'function');
  assert.equal(typeof module.renderSystemdUnit, 'function');
  assert.equal(typeof module.renderLaunchdPlist, 'function');
  assert.equal(typeof module.renderWindowsTaskXml, 'function');

  const runtimeToolingTests = read(repoRoot, 'bin/tests/router-runtime-tooling.test.mjs');
  assert.match(runtimeToolingTests, /function canSpawnUnixShellFromNode\(\)/);
  assert.match(
    runtimeToolingTests,
    /test\('unix runtime entrypoints default to the installed home beside the packaged scripts when binaries are colocated'/,
  );
  assert.match(
    runtimeToolingTests,
    /test\('installed unix runtime start\.sh and stop\.sh manage an installed home end-to-end'/,
  );
}
