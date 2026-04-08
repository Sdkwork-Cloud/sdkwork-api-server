import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

const binRoot = path.resolve(import.meta.dirname, '..');

function canSpawnPowerShellFromNode() {
  if (process.platform !== 'win32') {
    return false;
  }

  const probe = spawnSync(
    'powershell.exe',
    ['-NoProfile', '-Command', '$PSVersionTable.PSEdition'],
    { encoding: 'utf8' },
  );

  return !probe.error && probe.status === 0;
}

test('start-dev.ps1 wires in a Windows backend warm-up build with managed cargo environment defaults', () => {
  const startDevScript = readFileSync(path.join(binRoot, 'start-dev.ps1'), 'utf8');
  const commonScript = readFileSync(path.join(binRoot, 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(commonScript, /function Enable-RouterManagedCargoEnv/);
  assert.match(commonScript, /function Invoke-RouterWindowsBackendWarmupBuild/);
  assert.match(commonScript, /function Get-RouterWindowsBackendWarmupCommandDisplay/);
  assert.match(commonScript, /function Write-RouterUtf8File/);
  assert.match(commonScript, /SDKWORK_ROUTER_DEV_HOME/);
  assert.match(commonScript, /\.sdkwork-target-vs2022/);
  assert.match(commonScript, /\$env:CARGO_BUILD_JOBS = '1'/);
  assert.match(commonScript, /'build'/);
  assert.match(commonScript, /'admin-api-service'/);
  assert.match(commonScript, /'gateway-service'/);
  assert.match(commonScript, /'portal-api-service'/);
  assert.match(startDevScript, /Enable-RouterManagedCargoEnv -RepoRoot \$repoRoot/);
  assert.match(startDevScript, /Invoke-RouterWindowsBackendWarmupBuild -RepoRoot \$repoRoot/);
  assert.match(startDevScript, /Write-RouterUtf8File -FilePath \$planFile -Content \$planOutput/);
});

test('start-dev.ps1 dry-run honors SDKWORK_ROUTER_DEV_HOME and refreshes the generated plan file', {
  skip: !canSpawnPowerShellFromNode(),
}, () => {
  const devHome = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-dev-home-'));
  const runDir = path.join(devHome, 'run');
  const planFile = path.join(runDir, 'start-workspace.plan.txt');

  mkdirSync(runDir, { recursive: true });
  writeFileSync(planFile, 'stale-plan', 'utf8');

  try {
    const result = spawnSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(binRoot, 'start-dev.ps1'),
        '-DryRun',
      ],
      {
        cwd: binRoot,
        env: {
          ...process.env,
          SDKWORK_ROUTER_DEV_HOME: devHome,
        },
        encoding: 'utf8',
      },
    );

    const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;
    assert.equal(result.status, 0, output);
    assert.match(output, /CARGO_TARGET_DIR=/);
    assert.match(
      output,
      /backend warm-up: cargo build -p admin-api-service -p gateway-service -p portal-api-service -j 1/,
    );
    assert.ok(existsSync(planFile), 'expected dry-run plan file to be written');

    const planContents = readFileSync(planFile, 'utf8');
    assert.doesNotMatch(planContents, /stale-plan/);
    assert.match(planContents, /\[start-workspace\] unified launch settings/);
    assert.match(planContents, /frontend_mode=preview/);
  } finally {
    rmSync(devHome, { recursive: true, force: true });
  }
});
