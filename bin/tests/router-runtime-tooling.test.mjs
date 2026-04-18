import assert from 'node:assert/strict';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:net';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..');
const testRuntimeRoot = path.join(repoRoot, 'artifacts', 'test-runtime');

function createTempRuntimeHome(prefix) {
  mkdirSync(testRuntimeRoot, { recursive: true });
  return mkdtempSync(path.join(testRuntimeRoot, prefix));
}

function removeTempRuntimeHome(runtimeHome) {
  rmSync(runtimeHome, { recursive: true, force: true });
}

function createTempDir(prefix) {
  mkdirSync(testRuntimeRoot, { recursive: true });
  return mkdtempSync(path.join(testRuntimeRoot, prefix));
}

function detectTarFlavorForFixture() {
  if (process.platform !== 'win32') {
    return 'default';
  }

  const result = spawnSync('tar', ['--version'], {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: false,
  });
  if (result.error || result.status !== 0) {
    return 'unknown';
  }

  const versionOutput = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.toLowerCase();
  if (versionOutput.includes('gnu tar')) {
    return 'gnu';
  }
  if (versionOutput.includes('bsdtar') || versionOutput.includes('libarchive')) {
    return 'bsd';
  }

  return 'unknown';
}

function createTarGzArchive(sourceParentDir, entryName, archivePath) {
  const args = [];
  if (process.platform === 'win32' && detectTarFlavorForFixture() === 'gnu') {
    args.push('--force-local');
  }
  args.push('-czf', archivePath, '-C', sourceParentDir, entryName);

  const result = spawnSync('tar', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: process.platform === 'win32',
  });

  assert.equal(
    result.status,
    0,
    result.error?.message || result.stderr || result.stdout || `failed to create tarball ${archivePath}`,
  );
}

function createOfficialServerBundleFixture({
  releaseOutputDir,
  platform = 'linux',
  arch = 'x64',
} = {}) {
  const bundleDir = path.join(releaseOutputDir, 'native', platform, arch, 'bundles');
  const baseName = `sdkwork-api-router-product-server-${platform}-${arch}`;
  const bundleSourceRoot = path.join(releaseOutputDir, 'bundle-source');
  const bundleRoot = path.join(bundleSourceRoot, baseName);
  const bundlePath = path.join(bundleDir, `${baseName}.tar.gz`);
  const bundleManifestPath = path.join(bundleDir, `${baseName}.manifest.json`);
  const bundleChecksumPath = path.join(bundleDir, `${baseName}.tar.gz.sha256.txt`);
  const releaseCatalogPath = path.join(releaseOutputDir, 'release-catalog.json');

  mkdirSync(path.join(bundleRoot, 'bin'), { recursive: true });
  mkdirSync(path.join(bundleRoot, 'sites', 'admin', 'dist'), { recursive: true });
  mkdirSync(path.join(bundleRoot, 'sites', 'portal', 'dist'), { recursive: true });
  mkdirSync(path.join(bundleRoot, 'data'), { recursive: true });
  mkdirSync(path.join(bundleRoot, 'deploy', 'docker'), { recursive: true });
  mkdirSync(bundleDir, { recursive: true });

  const bundleBinaryName = platform === 'win32'
    ? 'router-product-service.exe'
    : 'router-product-service';
  writeFileSync(
    path.join(bundleRoot, 'bin', bundleBinaryName),
    platform === 'win32'
      ? '@echo off\r\necho bundled router-product-service\r\n'
      : '#!/usr/bin/env sh\nprintf \'%s\\n\' "bundled router-product-service"\n',
    'utf8',
  );
  writeFileSync(path.join(bundleRoot, 'sites', 'admin', 'dist', 'index.html'), '<html>bundled admin</html>\n', 'utf8');
  writeFileSync(path.join(bundleRoot, 'sites', 'portal', 'dist', 'index.html'), '<html>bundled portal</html>\n', 'utf8');
  writeFileSync(path.join(bundleRoot, 'data', 'seed.json'), '{"seed":"bundled"}\n', 'utf8');
  writeFileSync(path.join(bundleRoot, 'deploy', 'docker', 'docker-compose.yml'), 'services:\n  router:\n    image: sdkwork\n', 'utf8');
  writeFileSync(
    path.join(bundleRoot, 'release-manifest.json'),
    `${JSON.stringify({
      bundleOrigin: 'test-fixture',
      bundleVersion: '0.1.0',
    }, null, 2)}\n`,
    'utf8',
  );
  writeFileSync(path.join(bundleRoot, 'README.txt'), 'Official bundled readme\n', 'utf8');

  createTarGzArchive(bundleSourceRoot, baseName, bundlePath);
  writeFileSync(
    bundleManifestPath,
    `${JSON.stringify({
      type: 'product-server-archive',
      productId: 'sdkwork-api-router-product-server',
      archiveFile: path.basename(bundlePath),
      checksumFile: path.basename(bundleChecksumPath),
      sites: ['admin', 'portal'],
      bootstrapDataRoots: ['data'],
      deploymentAssetRoots: ['deploy'],
    }, null, 2)}\n`,
    'utf8',
  );
  writeFileSync(bundleChecksumPath, `sha256  ${path.basename(bundlePath)}\n`, 'utf8');
  writeFileSync(
    releaseCatalogPath,
    `${JSON.stringify({
      version: 1,
      type: 'sdkwork-release-catalog',
      releaseTag: 'fixture-release',
      generatedAt: '2026-04-18T00:00:00.000Z',
      productCount: 1,
      variantCount: 1,
      products: [
        {
          productId: 'sdkwork-api-router-product-server',
          variants: [
            {
              platform,
              arch,
              outputDirectory: toPortablePath(path.relative(releaseOutputDir, bundleDir)),
              variantKind: 'server-archive',
              primaryFile: path.basename(bundlePath),
              primaryFileSizeBytes: 0,
              checksumFile: path.basename(bundleChecksumPath),
              checksumAlgorithm: 'sha256',
              manifestFile: path.basename(bundleManifestPath),
              sha256: 'sha256',
              manifest: {
                type: 'product-server-archive',
                productId: 'sdkwork-api-router-product-server',
                platform,
                arch,
              },
            },
          ],
        },
      ],
    }, null, 2)}\n`,
    'utf8',
  );

  return {
    bundlePath,
    bundleManifestPath,
    bundleChecksumPath,
    releaseCatalogPath,
  };
}

async function withOfficialServerBundleFixtureContext({
  releasePlatform = 'linux',
  arch = 'x64',
} = {}, callback) {
  const fixtureRoot = createTempDir(`install-bundle-${releasePlatform}-${arch}-`);
  const installRoot = path.join(fixtureRoot, 'install');
  const releaseOutputDir = path.join(fixtureRoot, 'release');

  createOfficialServerBundleFixture({
    releaseOutputDir,
    platform: releasePlatform,
    arch,
  });

  try {
    await callback({
      fixtureRoot,
      installRoot,
      releaseOutputDir,
    });
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
}

function toPortablePath(value) {
  return value.replaceAll('\\', '/');
}

function toWslPath(value) {
  const normalized = toPortablePath(value);
  const driveMatch = normalized.match(/^([A-Za-z]):\/(.*)$/);
  if (!driveMatch) {
    return normalized;
  }

  return `/mnt/${driveMatch[1].toLowerCase()}/${driveMatch[2]}`;
}

function quoteForBash(value) {
  return `'${String(value).replaceAll("'", "'\"'\"'")}'`;
}

function quoteForPowerShellSingleQuotedString(value) {
  return String(value).replaceAll("'", "''");
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function runPowerShellCommand(command) {
  return spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-Command',
      command,
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );
}

function canSpawnPowerShellFromNode() {
  if (process.platform !== 'win32') {
    return false;
  }

  const result = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-Command',
      'exit 0',
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  return !result.error && result.status === 0;
}

async function withTcpListener(callback) {
  const server = createServer();
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });

  try {
    const address = server.address();
    assert.ok(address && typeof address === 'object', 'expected a bound TCP address');
    await callback({ port: address.port });
  } finally {
    await new Promise((resolve, reject) => {
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }

        resolve();
      });
    });
  }
}

function runPowerShellStartDryRun(runtimeHome) {
  return spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(repoRoot, 'bin', 'start.ps1'),
      '-DryRun',
      '-Home',
      runtimeHome,
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );
}

function runPowerShellStopDryRun(runtimeHome) {
  return spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(repoRoot, 'bin', 'stop.ps1'),
      '-DryRun',
      '-Home',
      runtimeHome,
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );
}

function hasWslDistro(name) {
  if (process.platform !== 'win32') {
    return false;
  }

  const result = spawnSync('wsl.exe', ['-l', '-q'], {
    cwd: repoRoot,
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    return false;
  }

  return result.stdout
    .replaceAll('\u0000', '')
    .split(/\r?\n/)
    .map((line) => line.trim())
    .includes(name);
}

function resolveGitBashExecutable() {
  const candidates = [
    'C:/Program Files/Git/bin/bash.exe',
    'C:/Program Files/Git/usr/bin/bash.exe',
  ];

  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

function hasUnixShellRuntime() {
  if (process.platform === 'win32') {
    return resolveGitBashExecutable() != null;
  }

  return true;
}

function canSpawnUnixShellFromNode() {
  if (!hasUnixShellRuntime()) {
    return false;
  }

  if (process.platform === 'win32') {
    const bash = resolveGitBashExecutable();
    const result = spawnSync(
      bash,
      ['-lc', 'exit 0'],
      {
        cwd: repoRoot,
        encoding: 'utf8',
      },
    );

    return !result.error && result.status === 0;
  }

  const result = spawnSync(
    'sh',
    ['-lc', 'exit 0'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  return !result.error && result.status === 0;
}

function toGitBashPath(value) {
  const normalized = toPortablePath(value);
  const driveMatch = normalized.match(/^([A-Za-z]):\/(.*)$/);
  if (!driveMatch) {
    return normalized;
  }

  return `/${driveMatch[1].toLowerCase()}/${driveMatch[2]}`;
}

function runUnixShellCommand(command, {
  cwd = repoRoot,
  env = process.env,
  pathPrefix = [],
} = {}) {
  if (process.platform === 'win32') {
    const bash = resolveGitBashExecutable();
    if (!bash) {
      throw new Error('Git Bash is required to run unix shell smoke tests on Windows.');
    }

    const exportedPathPrefix = pathPrefix
      .map((entry) => quoteForBash(toGitBashPath(entry)))
      .join(':');
    const script = [
      exportedPathPrefix ? `export PATH=${exportedPathPrefix}:$PATH` : '',
      `cd ${quoteForBash(toGitBashPath(cwd))}`,
      command,
    ].filter(Boolean).join(' && ');

    return spawnSync(
      bash,
      ['-lc', script],
      {
        cwd,
        env,
        encoding: 'utf8',
      },
    );
  }

  const script = [
    pathPrefix.length > 0 ? `export PATH=${pathPrefix.map((entry) => quoteForBash(entry)).join(':')}:$PATH` : '',
    command,
  ].filter(Boolean).join(' && ');

  return spawnSync(
    'sh',
    ['-lc', script],
    {
      cwd,
      env,
      encoding: 'utf8',
    },
  );
}

function runWslStartDryRun(runtimeHome, distro = 'Ubuntu-22.04') {
  const repoRootWsl = toWslPath(repoRoot);
  const runtimeHomeWsl = toWslPath(runtimeHome);
  const command = [
    `cd ${quoteForBash(repoRootWsl)}`,
    `bash bin/start.sh --dry-run --home ${quoteForBash(runtimeHomeWsl)}`,
  ].join(' && ');

  return spawnSync(
    'wsl.exe',
    ['-d', distro, '--', '/bin/bash', '-lc', command],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );
}

function runWslStopDryRun(runtimeHome, distro = 'Ubuntu-22.04') {
  const repoRootWsl = toWslPath(repoRoot);
  const runtimeHomeWsl = toWslPath(runtimeHome);
  const command = [
    `cd ${quoteForBash(repoRootWsl)}`,
    `bash bin/stop.sh --dry-run --home ${quoteForBash(runtimeHomeWsl)}`,
  ].join(' && ');

  return spawnSync(
    'wsl.exe',
    ['-d', distro, '--', '/bin/bash', '-lc', command],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );
}

async function loadModule() {
  return import(
    pathToFileURL(path.join(repoRoot, 'bin', 'lib', 'router-runtime-tooling.mjs')).href
  );
}

async function loadRouterOpsModule() {
  return import(
    pathToFileURL(path.join(repoRoot, 'bin', 'router-ops.mjs')).href
  );
}

async function loadPackageReleaseAssetsModule() {
  return import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs')).href
  );
}

function installUnixRuntimeSmokeFixture(runtimeHome) {
  const runtimeBinDir = path.join(runtimeHome, 'bin');
  const runtimeLibDir = path.join(runtimeBinDir, 'lib');
  const adminSiteDir = path.join(runtimeHome, 'sites', 'admin', 'dist');
  const portalSiteDir = path.join(runtimeHome, 'sites', 'portal', 'dist');
  const configDir = path.join(runtimeHome, 'config');
  const fakeBinDir = path.join(runtimeHome, 'test-bin');
  const curlLogFile = path.join(runtimeHome, 'var', 'log', 'curl.log');
  const runtimeBinaryPath = path.join(runtimeBinDir, 'router-product-service');

  mkdirSync(runtimeLibDir, { recursive: true });
  mkdirSync(adminSiteDir, { recursive: true });
  mkdirSync(portalSiteDir, { recursive: true });
  mkdirSync(configDir, { recursive: true });
  mkdirSync(fakeBinDir, { recursive: true });
  mkdirSync(path.join(runtimeHome, 'var', 'log'), { recursive: true });

  writeFileSync(
    path.join(runtimeBinDir, 'start.sh'),
    readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8'),
    'utf8',
  );
  writeFileSync(
    path.join(runtimeBinDir, 'stop.sh'),
    readFileSync(path.join(repoRoot, 'bin', 'stop.sh'), 'utf8'),
    'utf8',
  );
  writeFileSync(
    path.join(runtimeLibDir, 'runtime-common.sh'),
    readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8'),
    'utf8',
  );
  writeFileSync(path.join(adminSiteDir, 'index.html'), '<html>admin</html>\n', 'utf8');
  writeFileSync(path.join(portalSiteDir, 'index.html'), '<html>portal</html>\n', 'utf8');
  writeFileSync(
    path.join(configDir, 'router.env'),
    [
      'SDKWORK_WEB_BIND="127.0.0.1:19483"',
      'SDKWORK_GATEWAY_BIND="127.0.0.1:19480"',
      'SDKWORK_ADMIN_BIND="127.0.0.1:19481"',
      'SDKWORK_PORTAL_BIND="127.0.0.1:19482"',
      '',
    ].join('\n'),
    'utf8',
  );
  writeFileSync(
    runtimeBinaryPath,
    [
      '#!/usr/bin/env sh',
      '',
      'if [ "$#" -ge 2 ] && [ "$1" = "--dry-run" ] && [ "$2" = "--plan-format" ]; then',
      '  cat <<EOF',
      '{',
      '  "mode": "dry-run",',
      '  "plan_format": "json",',
      '  "public_web_bind": "127.0.0.1:19483",',
      '  "database_url": "sqlite:///tmp/sdkwork-api-router.db",',
      '  "config_dir": "/tmp/sdkwork-config",',
      '  "config_file": null,',
      '  "node_id_prefix": null,',
      '  "binds": {',
      '    "gateway": "127.0.0.1:19480",',
      '    "admin": "127.0.0.1:19481",',
      '    "portal": "127.0.0.1:19482"',
      '  },',
      '  "site_dirs": {',
      '    "admin": "/tmp/admin",',
      '    "portal": "/tmp/portal"',
      '  },',
      '  "upstreams": {',
      '    "gateway": null,',
      '    "admin": null,',
      '    "portal": null',
      '  }',
      '}',
      'EOF',
      '  exit 0',
      'fi',
      '',
      "trap 'exit 0' TERM INT",
      'while :; do',
      '  sleep 1',
      'done',
      '',
    ].join('\n'),
    'utf8',
  );
  writeFileSync(
    path.join(fakeBinDir, 'uname'),
    [
      '#!/usr/bin/env sh',
      'if [ "${1:-}" = "-m" ]; then',
      "  printf '%s\\n' 'x86_64'",
      '  exit 0',
      'fi',
      "printf '%s\\n' 'Linux'",
      '',
    ].join('\n'),
    'utf8',
  );
  writeFileSync(
    path.join(fakeBinDir, 'curl'),
    [
      '#!/usr/bin/env sh',
      'printf \'%s\\n\' "$*" >> "$CURL_LOG_FILE"',
      'exit 0',
      '',
    ].join('\n'),
    'utf8',
  );

  chmodSync(path.join(runtimeBinDir, 'start.sh'), 0o755);
  chmodSync(path.join(runtimeBinDir, 'stop.sh'), 0o755);
  chmodSync(path.join(runtimeLibDir, 'runtime-common.sh'), 0o755);
  chmodSync(runtimeBinaryPath, 0o755);
  chmodSync(path.join(fakeBinDir, 'uname'), 0o755);
  chmodSync(path.join(fakeBinDir, 'curl'), 0o755);

  return {
    curlLogFile,
    fakeBinDir,
  };
}

test('createReleaseBuildPlan builds the official server and portal desktop release products', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'linux',
    arch: 'x64',
    installDependencies: false,
    includeDocs: true,
  });

  assert.equal(plan.target.targetTriple, 'x86_64-unknown-linux-gnu');
  assert.equal(plan.steps[0].label, 'cargo release build');
  assert.deepEqual(plan.steps[0].args.slice(-14), [
    'build',
    '--release',
    '--target',
    'x86_64-unknown-linux-gnu',
    '-p',
    'admin-api-service',
    '-p',
    'gateway-service',
    '-p',
    'portal-api-service',
    '-p',
    'router-web-service',
    '-p',
    'router-product-service',
  ]);
  assert.equal(plan.steps.some((step) => step.label === 'admin app build'), true);
  assert.equal(plan.steps.some((step) => step.label === 'portal app build'), true);
  assert.equal(plan.steps.some((step) => step.label === 'console build'), false);
  assert.equal(plan.steps.some((step) => step.label === 'docs build'), true);
  assert.deepEqual(
    plan.steps
      .filter((step) => step.label.endsWith('desktop release build'))
      .map((step) => step.label),
    ['portal desktop release build'],
  );
  assert.equal(
    plan.steps.some((step) => step.label === 'admin desktop release build'),
    false,
  );
  assert.equal(plan.steps.at(-1).label, 'native release package');
  assert.deepEqual(plan.steps.at(-1).args, [
    path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    'native',
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--target',
    'x86_64-unknown-linux-gnu',
    '--output-dir',
    path.join(repoRoot, 'artifacts', 'release'),
  ]);
});

test('createReleaseBuildPlan appends packaged release verification steps after native packaging', async () => {
  const module = await loadModule();
  const releaseOutputDir = path.join(repoRoot, 'artifacts', 'release-fixture');

  const linuxPlan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'linux',
    arch: 'x64',
    installDependencies: false,
    includeDocs: false,
    verifyRelease: true,
    releaseOutputDir,
  });

  assert.equal(
    linuxPlan.steps.some((step) => step.label === 'docs build'),
    true,
    'verify-release plans must always build the governed docs site even when includeDocs is false',
  );
  assert.deepEqual(
    linuxPlan.steps.slice(-5).map((step) => step.label),
    [
      'native release package',
      'unix installed runtime smoke',
      'linux docker compose smoke',
      'linux helm render smoke',
      'release governance preflight',
    ],
  );
  assert.deepEqual(linuxPlan.steps.at(-4).args, [
    path.join(repoRoot, 'scripts', 'release', 'run-unix-installed-runtime-smoke.mjs'),
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--target',
    'x86_64-unknown-linux-gnu',
    '--release-output-dir',
    releaseOutputDir,
    '--evidence-path',
    path.join(repoRoot, 'artifacts', 'release-governance', 'unix-installed-runtime-smoke-linux-x64.json'),
  ]);
  assert.deepEqual(linuxPlan.steps.at(-3).args, [
    path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--bundle-path',
    path.join(
      releaseOutputDir,
      'native',
      'linux',
      'x64',
      'bundles',
      'sdkwork-api-router-product-server-linux-x64.tar.gz',
    ),
    '--evidence-path',
    path.join(repoRoot, 'artifacts', 'release-governance', 'docker-compose-smoke-linux-x64.json'),
  ]);
  assert.deepEqual(linuxPlan.steps.at(-2).args, [
    path.join(repoRoot, 'scripts', 'release', 'run-linux-helm-render-smoke.mjs'),
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--bundle-path',
    path.join(
      releaseOutputDir,
      'native',
      'linux',
      'x64',
      'bundles',
      'sdkwork-api-router-product-server-linux-x64.tar.gz',
    ),
    '--evidence-path',
    path.join(repoRoot, 'artifacts', 'release-governance', 'helm-render-smoke-linux-x64.json'),
  ]);
  assert.deepEqual(linuxPlan.steps.at(-1).args, [
    path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    '--profile',
    'preflight',
  ]);

  const windowsPlan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    installDependencies: false,
    includeDocs: false,
    verifyRelease: true,
    releaseOutputDir,
  });

  assert.deepEqual(
    windowsPlan.steps.slice(-3).map((step) => step.label),
    [
      'native release package',
      'windows installed runtime smoke',
      'release governance preflight',
    ],
  );
  assert.deepEqual(windowsPlan.steps.at(-2).args, [
    path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    '--platform',
    'windows',
    '--arch',
    'x64',
    '--target',
    'x86_64-pc-windows-msvc',
    '--release-output-dir',
    releaseOutputDir,
    '--evidence-path',
    path.join(repoRoot, 'artifacts', 'release-governance', 'windows-installed-runtime-smoke-windows-x64.json'),
  ]);
  assert.deepEqual(windowsPlan.steps.at(-1).args, [
    path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
    '--profile',
    'preflight',
  ]);
});

test('createReleaseBuildPlan schedules pnpm install when frontend plugin packages are missing from an existing node_modules tree', async () => {
  const module = await loadModule();
  const fixtureRoot = createTempDir('release-build-plan-frontend-health-');
  const appRoots = {
    admin: path.join(fixtureRoot, 'apps', 'sdkwork-router-admin'),
    portal: path.join(fixtureRoot, 'apps', 'sdkwork-router-portal'),
    docs: path.join(fixtureRoot, 'docs'),
  };

  try {
    for (const appRoot of Object.values(appRoots)) {
      mkdirSync(appRoot, { recursive: true });
    }

    for (const appKey of ['admin', 'portal']) {
      const nodeModulesRoot = path.join(appRoots[appKey], 'node_modules');
      const binRoot = path.join(nodeModulesRoot, '.bin');
      mkdirSync(binRoot, { recursive: true });
      writeFileSync(path.join(nodeModulesRoot, '.modules.yaml'), 'layoutVersion: 5\n', 'utf8');

      for (const packageName of ['typescript', 'vite']) {
        mkdirSync(path.join(nodeModulesRoot, packageName), { recursive: true });
        writeFileSync(
          path.join(nodeModulesRoot, packageName, 'package.json'),
          `{"name":"${packageName}"}\n`,
          'utf8',
        );
      }

      for (const commandName of ['tsc.cmd', 'vite.cmd']) {
        writeFileSync(path.join(binRoot, commandName), '', 'utf8');
      }
    }

    const plan = module.createReleaseBuildPlan({
      repoRoot: fixtureRoot,
      platform: 'win32',
      arch: 'x64',
      env: {
        USERPROFILE: 'C:/Users/admin',
        TEMP: 'C:/Temp',
      },
      includeDocs: false,
    });

    for (const label of ['admin app install', 'portal app install']) {
      const installStep = plan.steps.find((step) => step.label === label);
      assert.ok(installStep, `expected ${label} when required plugin packages are missing`);
      assert.match(installStep.args[4], /--dir/);
      assert.match(installStep.args[4], /\binstall\b/);
    }
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('createReleaseBuildPlan rejects the legacy console workspace toggle for programmatic callers', async () => {
  const module = await loadModule();

  assert.throws(
    () => module.createReleaseBuildPlan({
      repoRoot,
      includeConsole: true,
    }),
    /includeConsole is no longer supported/,
  );
});

test('createReleaseBuildPlan normalizes broken Windows CMake generator defaults for release cargo builds', async () => {
  const module = await loadModule();
  const workspaceTargetDirModule = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'workspace-target-dir.mjs')).href,
  );
  const managedEnv = {
    USERPROFILE: 'C:/Users/admin',
    TEMP: 'C:/Temp',
    CMAKE_GENERATOR: 'Visual Studio 18 2026',
  };
  const defaultTargetDir = workspaceTargetDirModule.resolveWorkspaceTargetDir({
    workspaceRoot: repoRoot,
    env: managedEnv,
    platform: 'win32',
  });
  const defaultTempDir = workspaceTargetDirModule.resolveWorkspaceTempDir({
    workspaceRoot: repoRoot,
    env: managedEnv,
    platform: 'win32',
  });

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: managedEnv,
    includeDocs: false,
  });

  assert.equal(plan.steps[0].env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(plan.steps[0].env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(
    String(plan.steps[0].env.CARGO_TARGET_DIR ?? '').replaceAll('\\', '/'),
    defaultTargetDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(plan.steps[0].env.TEMP ?? '').replaceAll('\\', '/'),
    defaultTempDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(plan.steps[0].env.TMP ?? '').replaceAll('\\', '/'),
    defaultTempDir.replaceAll('\\', '/'),
  );
});

test('createReleaseBuildPlan strips inherited Windows-only CMake generators on non-Windows release cargo builds', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'linux',
    arch: 'x64',
    env: {
      HOME: '/tmp/sdkwork',
      CMAKE_GENERATOR: 'Visual Studio 17 2022',
      HOST_CMAKE_GENERATOR: 'Visual Studio 17 2022',
    },
    includeDocs: false,
  });

  assert.equal(Object.hasOwn(plan.steps[0].env, 'CMAKE_GENERATOR'), false);
  assert.equal(Object.hasOwn(plan.steps[0].env, 'HOST_CMAKE_GENERATOR'), false);
  assert.equal(plan.target.targetTriple, 'x86_64-unknown-linux-gnu');
});

test('createReleaseBuildPlan defaults Windows release cargo builds to a single job and propagates that to downstream steps', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
    includeDocs: false,
  });

  const jobIndex = plan.steps[0].args.indexOf('-j');
  assert.notEqual(jobIndex, -1, 'expected cargo build to pin an explicit job count');
  assert.equal(plan.steps[0].args[jobIndex + 1], '1');
  assert.equal(plan.steps[0].env.CARGO_BUILD_JOBS, '1');
  assert.equal(
    plan.steps.find((step) => step.label === 'portal desktop release build')?.env.CARGO_BUILD_JOBS,
    '1',
  );
});

test('createReleaseBuildPlan respects an explicit Windows cargo job override', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
      CARGO_BUILD_JOBS: '4',
    },
    includeDocs: false,
  });

  const jobIndex = plan.steps[0].args.indexOf('-j');
  assert.notEqual(jobIndex, -1, 'expected cargo build to keep an explicit job count');
  assert.equal(plan.steps[0].args[jobIndex + 1], '4');
  assert.equal(plan.steps[0].env.CARGO_BUILD_JOBS, '4');
});

test('createReleaseBuildPlan uses the shared Windows-safe pnpm launcher for frontend release steps', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
    includeDocs: true,
  });

  const adminBuildStep = plan.steps.find((step) => step.label === 'admin app build');
  assert.ok(adminBuildStep, 'expected an admin app build step');
  assert.equal(adminBuildStep.command, 'powershell.exe');
  assert.deepEqual(adminBuildStep.args.slice(0, 4), [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
  ]);
  assert.match(adminBuildStep.args[4], /pnpm\.cjs/);
  assert.match(adminBuildStep.args[4], /sdkwork-router-admin/);
  assert.match(adminBuildStep.args[4], /\bbuild\b/);
  assert.equal(adminBuildStep.shell, false);
  assert.equal(adminBuildStep.windowsHide, true);
  assert.match(adminBuildStep.env.NODE_OPTIONS ?? '', /vite-windows-realpath-preload\.mjs/);
});

test('createReleaseBuildPlan injects the Windows cc-rs reproducible-build workaround for release cargo builds', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
    includeDocs: false,
  });

  assert.equal(plan.steps[0].env.SDKWORK_CC_DISABLE_BREPRO, '1');
  assert.equal(
    plan.steps.find((step) => step.label === 'portal desktop release build')?.env.SDKWORK_CC_DISABLE_BREPRO,
    '1',
  );
});

test('packageProductServerBundle falls back to repository cargo target roots when managed Windows target roots only contain desktop outputs', async () => {
  const module = await loadPackageReleaseAssetsModule();
  const fixtureRoot = createTempDir('package-product-server-service-root-');
  const outputDir = path.join(fixtureRoot, 'release-output');
  const managedTargetRoot = path.join(fixtureRoot, 'managed', 'x86_64-pc-windows-msvc', 'release');
  const repositoryTargetRoot = path.join(fixtureRoot, 'target', 'x86_64-pc-windows-msvc', 'release');
  const adminSiteDir = path.join(fixtureRoot, 'sites', 'admin', 'dist');
  const portalSiteDir = path.join(fixtureRoot, 'sites', 'portal', 'dist');
  const bootstrapDataDir = path.join(fixtureRoot, 'data');
  const deploymentAssetDir = path.join(fixtureRoot, 'deploy');

  try {
    mkdirSync(managedTargetRoot, { recursive: true });
    writeFileSync(path.join(managedTargetRoot, 'desktop-only.txt'), 'desktop build residue\n', 'utf8');
    mkdirSync(repositoryTargetRoot, { recursive: true });
    for (const binaryName of module.listNativeServiceBinaryNames()) {
      writeFileSync(
        path.join(repositoryTargetRoot, `${binaryName}.exe`),
        `${binaryName}\n`,
        'utf8',
      );
    }

    mkdirSync(adminSiteDir, { recursive: true });
    mkdirSync(portalSiteDir, { recursive: true });
    mkdirSync(bootstrapDataDir, { recursive: true });
    mkdirSync(path.join(deploymentAssetDir, 'docker'), { recursive: true });
    writeFileSync(path.join(adminSiteDir, 'index.html'), '<html>admin</html>\n', 'utf8');
    writeFileSync(path.join(portalSiteDir, 'index.html'), '<html>portal</html>\n', 'utf8');
    writeFileSync(path.join(bootstrapDataDir, 'seed.json'), '{"seed":"fixture"}\n', 'utf8');
    writeFileSync(path.join(deploymentAssetDir, 'docker', 'docker-compose.yml'), 'services: {}\n', 'utf8');

    const result = module.packageProductServerBundle({
      platformId: 'windows',
      archId: 'x64',
      targetTriple: 'x86_64-pc-windows-msvc',
      outputDir,
      resolveServiceRootCandidates: () => [managedTargetRoot, repositoryTargetRoot],
      siteAssetRoots: {
        admin: adminSiteDir,
        portal: portalSiteDir,
      },
      bootstrapDataRoots: {
        data: bootstrapDataDir,
      },
      deploymentAssetRoots: {
        deploy: deploymentAssetDir,
      },
      runTar: (archivePath) => {
        writeFileSync(archivePath, 'bundle archive\n', 'utf8');
      },
    });

    assert.equal(result.productId, 'sdkwork-api-router-product-server');
    assert.equal(
      existsSync(path.join(outputDir, 'native', 'windows', 'x64', 'bundles', result.fileName)),
      true,
    );
    const manifest = JSON.parse(
      readFileSync(path.join(outputDir, 'native', 'windows', 'x64', 'bundles', result.manifestFileName), 'utf8'),
    );
    assert.deepEqual(manifest.services, module.listNativeServiceBinaryNames());
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('resolveAvailableServiceReleaseRoot prefers repository cargo target roots when managed Windows target roots miss service binaries', async () => {
  const module = await loadPackageReleaseAssetsModule();
  const fixtureRoot = createTempDir('resolve-service-release-root-');
  const managedTargetRoot = path.join(fixtureRoot, 'managed', 'x86_64-pc-windows-msvc', 'release');
  const repositoryTargetRoot = path.join(fixtureRoot, 'target', 'x86_64-pc-windows-msvc', 'release');

  try {
    assert.equal(typeof module.resolveAvailableServiceReleaseRoot, 'function');

    mkdirSync(managedTargetRoot, { recursive: true });
    writeFileSync(path.join(managedTargetRoot, 'desktop-only.txt'), 'desktop build residue\n', 'utf8');
    mkdirSync(repositoryTargetRoot, { recursive: true });
    for (const binaryName of module.listNativeServiceBinaryNames()) {
      writeFileSync(
        path.join(repositoryTargetRoot, `${binaryName}.exe`),
        `${binaryName}\n`,
        'utf8',
      );
    }

    const resolvedRoot = module.resolveAvailableServiceReleaseRoot({
      platform: 'windows',
      targetTriple: 'x86_64-pc-windows-msvc',
      serviceReleaseRoots: [managedTargetRoot, repositoryTargetRoot],
    });

    assert.equal(resolvedRoot, repositoryTargetRoot);
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('createInstallPlan separates stable current control assets from versioned release payloads', async () => {
  const module = await loadModule();
  await withOfficialServerBundleFixtureContext({
    releasePlatform: 'macos',
    arch: 'x64',
  }, async ({ installRoot, releaseOutputDir }) => {
    const currentRoot = path.join(installRoot, 'current');
    const releaseRoot = path.join(installRoot, 'releases', '0.1.0');

    const plan = module.createInstallPlan({
      repoRoot,
      installRoot,
      platform: 'darwin',
      releaseOutputDir,
    });

    assert.equal(toPortablePath(plan.installRoot), toPortablePath(installRoot));
    assert.equal(plan.directories.includes(currentRoot), true);
    assert.equal(plan.directories.includes(path.join(currentRoot, 'bin')), true);
    assert.equal(plan.directories.includes(path.join(currentRoot, 'bin', 'lib')), true);
    assert.equal(plan.directories.includes(path.join(currentRoot, 'service', 'systemd')), true);
    assert.equal(plan.directories.includes(path.join(currentRoot, 'service', 'launchd')), true);
    assert.equal(plan.directories.includes(path.join(currentRoot, 'service', 'windows-task')), true);
    assert.equal(plan.directories.includes(path.join(releaseRoot, 'bin')), true);
    assert.equal(plan.directories.includes(path.join(releaseRoot, 'data')), true);
    assert.equal(plan.directories.includes(path.join(releaseRoot, 'sites', 'admin')), true);
    assert.equal(plan.directories.includes(path.join(releaseRoot, 'sites', 'portal')), true);
    assert.equal(plan.directories.includes(path.join(installRoot, 'config')), true);
    assert.equal(plan.directories.includes(path.join(installRoot, 'data')), true);
    assert.equal(plan.directories.includes(path.join(installRoot, 'log')), true);
    assert.equal(plan.directories.includes(path.join(installRoot, 'run')), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'bin', 'start.sh'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'bin', 'validate-config.sh'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'bin', 'stop.ps1'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'bin', 'validate-config.ps1'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'launchd', 'com.sdkwork.api-router.plist'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'systemd', 'sdkwork-api-router.service'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'windows-task', 'sdkwork-api-router.xml'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'systemd', 'install-service.sh'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'systemd', 'uninstall-service.sh'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'launchd', 'install-service.sh'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'launchd', 'uninstall-service.sh'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'windows-task', 'install-service.ps1'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'service', 'windows-task', 'uninstall-service.ps1'))), true);
    assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('current', 'release-manifest.json'))), true);
    assert.equal(
      plan.files.some((file) =>
        file.type === 'bundle-directory'
        && file.bundleEntryPath === 'data'
        && file.targetPath === path.join(releaseRoot, 'data')),
      true,
    );
    assert.equal(
      plan.files.some((file) =>
        file.type === 'bundle-directory'
        && file.bundleEntryPath === toPortablePath(path.join('sites', 'admin', 'dist'))
        && file.targetPath.endsWith(path.join('releases', '0.1.0', 'sites', 'admin', 'dist'))),
      true,
    );
    assert.equal(
      plan.files.some((file) =>
        file.type === 'bundle-directory'
        && file.bundleEntryPath === toPortablePath(path.join('sites', 'portal', 'dist'))
        && file.targetPath.endsWith(path.join('releases', '0.1.0', 'sites', 'portal', 'dist'))),
      true,
    );
    assert.equal(
      plan.files.some((file) =>
        file.type === 'bundle-directory'
        && file.bundleEntryPath === 'deploy'
        && file.targetPath.endsWith(path.join('releases', '0.1.0', 'deploy'))),
      true,
    );
    assert.equal(
      plan.files.some((file) =>
        file.type === 'bundle-file'
        && file.bundleEntryPath === 'release-manifest.json'
        && file.targetPath.endsWith(path.join('releases', '0.1.0', 'release-manifest.json'))),
      true,
    );
    assert.equal(
      plan.files.some((file) =>
        file.type === 'bundle-file'
        && file.bundleEntryPath === 'README.txt'
        && file.targetPath.endsWith(path.join('releases', '0.1.0', 'README.txt'))),
      true,
    );
    assert.equal(
      plan.files
        .filter((file) => file.targetPath.startsWith(releaseRoot))
        .every((file) => file.type === 'bundle-file' || file.type === 'bundle-directory'),
      true,
    );
  });
});

test('createInstallPlan keeps portable mode on the repo-local install root', async () => {
  const module = await loadModule();
  await withOfficialServerBundleFixtureContext({
    releasePlatform: 'linux',
    arch: 'x64',
  }, async ({ releaseOutputDir }) => {
    const plan = module.createInstallPlan({
      repoRoot,
      mode: 'portable',
      platform: 'linux',
      releaseOutputDir,
    });

    assert.equal(plan.mode, 'portable');
    assert.equal(
      toPortablePath(plan.installRoot),
      `${toPortablePath(repoRoot)}/artifacts/install/sdkwork-api-router`,
    );
    assert.equal(
      plan.files.some((file) => toPortablePath(file.targetPath) === `${toPortablePath(plan.installRoot)}/config/router.env`),
      true,
    );
    assert.equal(
      plan.files.some((file) => toPortablePath(file.targetPath) === `${toPortablePath(plan.installRoot)}/current/release-manifest.json`),
      true,
    );
  });
});

test('createInstallPlan system mode emits linux standard program, config, and state directories', async () => {
  const module = await loadModule();
  await withOfficialServerBundleFixtureContext({
    releasePlatform: 'linux',
    arch: 'x64',
  }, async ({ releaseOutputDir }) => {
    const plan = module.createInstallPlan({
      repoRoot,
      mode: 'system',
      platform: 'linux',
      releaseOutputDir,
    });
    const directories = plan.directories.map((directoryPath) => toPortablePath(directoryPath));
    const targetFiles = plan.files.map((file) => toPortablePath(file.targetPath));

    assert.equal(plan.mode, 'system');
    assert.equal(directories.includes('/opt/sdkwork-api-router'), true);
    assert.equal(directories.includes('/opt/sdkwork-api-router/current'), true);
    assert.equal(directories.includes('/opt/sdkwork-api-router/releases/0.1.0'), true);
    assert.equal(directories.includes('/etc/sdkwork-api-router'), true);
    assert.equal(directories.includes('/etc/sdkwork-api-router/conf.d'), true);
    assert.equal(directories.includes('/var/lib/sdkwork-api-router'), true);
    assert.equal(directories.includes('/var/log/sdkwork-api-router'), true);
    assert.equal(directories.includes('/run/sdkwork-api-router'), true);
    assert.equal(targetFiles.includes('/etc/sdkwork-api-router/router.yaml'), true);
    assert.equal(targetFiles.includes('/etc/sdkwork-api-router/router.env'), true);
    assert.equal(targetFiles.includes('/etc/sdkwork-api-router/router.env.example'), true);
    assert.equal(targetFiles.includes('/opt/sdkwork-api-router/current/release-manifest.json'), true);

    const routerConfigFile = plan.files.find(
      (file) => toPortablePath(file.targetPath) === '/etc/sdkwork-api-router/router.yaml',
    );
    assert.ok(routerConfigFile, 'expected system install plan to render router.yaml');
    assert.equal(routerConfigFile.type, 'text');
    assert.match(routerConfigFile.contents, /web_bind: "0\.0\.0\.0:3001"/);
  });
});

test('createInstallPlan system mode publishes formal service assets alongside windows-task compatibility assets', async () => {
  const module = await loadModule();
  await withOfficialServerBundleFixtureContext({
    releasePlatform: 'windows',
    arch: 'x64',
  }, async ({ releaseOutputDir }) => {
    const plan = module.createInstallPlan({
      repoRoot,
      mode: 'system',
      platform: 'win32',
      releaseOutputDir,
      env: {
        ProgramFiles: 'C:/Program Files',
        ProgramData: 'C:/ProgramData',
        USERPROFILE: 'C:/Users/admin',
        TEMP: 'C:/Temp',
      },
    });
    const targetFiles = plan.files.map((file) => toPortablePath(file.targetPath));

    assert.equal(targetFiles.some((targetPath) => targetPath.endsWith('/service/systemd/sdkwork-api-router.service')), true);
    assert.equal(targetFiles.some((targetPath) => targetPath.endsWith('/service/launchd/com.sdkwork.api-router.plist')), true);
    assert.equal(targetFiles.some((targetPath) => targetPath.endsWith('/service/windows-service/install-service.ps1')), true);
    assert.equal(targetFiles.some((targetPath) => targetPath.endsWith('/service/windows-service/uninstall-service.ps1')), true);
    assert.equal(targetFiles.some((targetPath) => targetPath.endsWith('/service/windows-service/run-service.ps1')), true);
    assert.equal(targetFiles.some((targetPath) => targetPath.endsWith('/service/windows-task/sdkwork-api-router.xml')), true);
  });
});

test('createInstallPlan resolves the packaged Windows server bundle as the install payload source', async () => {
  const module = await loadModule();
  await withOfficialServerBundleFixtureContext({
    releasePlatform: 'windows',
    arch: 'x64',
  }, async ({ installRoot, releaseOutputDir }) => {
    const plan = module.createInstallPlan({
      repoRoot,
      installRoot,
      platform: 'win32',
      releaseOutputDir,
      env: {
        USERPROFILE: 'C:/Users/admin',
        TEMP: 'C:/Temp',
      },
    });

    const binaryCopy = plan.files.find((file) =>
      file.type === 'bundle-directory'
      && file.bundleEntryPath === 'bin'
      && file.targetPath.endsWith(path.join('releases', '0.1.0', 'bin')));
    assert.ok(binaryCopy, 'expected packaged bundle bin entry');
    assert.equal(
      toPortablePath(binaryCopy.bundlePath),
      `${toPortablePath(releaseOutputDir)}/native/windows/x64/bundles/sdkwork-api-router-product-server-windows-x64.tar.gz`,
    );
    assert.equal(
      toPortablePath(binaryCopy.bundleManifestPath),
      `${toPortablePath(releaseOutputDir)}/native/windows/x64/bundles/sdkwork-api-router-product-server-windows-x64.manifest.json`,
    );
    assert.equal(
      toPortablePath(binaryCopy.releaseCatalogPath),
      `${toPortablePath(releaseOutputDir)}/release-catalog.json`,
    );
  });
});

test('createInstallPlan treats normalized windows platform ids as packaged Windows bundle layouts', async () => {
  const module = await loadModule();
  await withOfficialServerBundleFixtureContext({
    releasePlatform: 'windows',
    arch: 'x64',
  }, async ({ installRoot, releaseOutputDir }) => {
    const plan = module.createInstallPlan({
      repoRoot,
      installRoot,
      platform: 'windows',
      arch: 'x64',
      releaseOutputDir,
      env: {
        USERPROFILE: 'C:/Users/admin',
        TEMP: 'C:/Temp',
      },
    });

    const binaryCopy = plan.files.find((file) =>
      file.type === 'bundle-directory'
      && file.bundleEntryPath === 'bin'
      && file.targetPath.endsWith(path.join('releases', '0.1.0', 'bin')));
    assert.ok(binaryCopy, 'expected packaged bundle bin entry for normalized windows platform ids');
    assert.equal(
      toPortablePath(binaryCopy.bundlePath),
      `${toPortablePath(releaseOutputDir)}/native/windows/x64/bundles/sdkwork-api-router-product-server-windows-x64.tar.gz`,
    );
  });
});

test('createInstallPlan requires a matching release-catalog entry to resolve the official server bundle', async () => {
  const module = await loadModule();
  const fixtureRoot = createTempDir('install-plan-catalog-resolve-');
  const installRoot = path.join(fixtureRoot, 'install');
  const releaseOutputDir = path.join(fixtureRoot, 'release');
  const releaseCatalogPath = path.join(releaseOutputDir, 'release-catalog.json');

  try {
    createOfficialServerBundleFixture({
      releaseOutputDir,
      platform: 'linux',
      arch: 'x64',
    });

    writeFileSync(
      releaseCatalogPath,
      `${JSON.stringify({
        version: 1,
        type: 'sdkwork-release-catalog',
        releaseTag: 'fixture-release',
        generatedAt: '2026-04-18T00:00:00.000Z',
        productCount: 1,
        variantCount: 1,
        products: [
          {
            productId: 'sdkwork-router-portal-desktop',
            variants: [
              {
                platform: 'linux',
                arch: 'x64',
                outputDirectory: 'native/linux/x64/desktop/portal',
                variantKind: 'desktop-installer',
                primaryFile: 'sdkwork-router-portal-desktop-linux-x64.AppImage',
                checksumFile: 'sdkwork-router-portal-desktop-linux-x64.AppImage.sha256.txt',
                checksumAlgorithm: 'sha256',
                manifestFile: 'sdkwork-router-portal-desktop-linux-x64.manifest.json',
                sha256: 'desktopdigest',
                manifest: {
                  type: 'portal-desktop-installer',
                  productId: 'sdkwork-router-portal-desktop',
                  platform: 'linux',
                  arch: 'x64',
                },
              },
            ],
          },
        ],
      }, null, 2)}\n`,
      'utf8',
    );

    assert.throws(
      () => module.createInstallPlan({
        repoRoot,
        installRoot,
        platform: 'linux',
        arch: 'x64',
        releaseOutputDir,
      }),
      /Missing release-catalog variant for sdkwork-api-router-product-server\/server-archive\/linux\/x64:/,
    );
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('assertInstallInputsExist requires the packaged bundle checksum for bundle-backed install plans', async () => {
  const module = await loadModule();
  const fixtureRoot = createTempDir('install-bundle-inputs-');
  const installRoot = path.join(fixtureRoot, 'install');
  const releaseOutputDir = path.join(fixtureRoot, 'release');
  const { bundleChecksumPath } = createOfficialServerBundleFixture({
    releaseOutputDir,
    platform: 'linux',
    arch: 'x64',
  });

  try {
    const plan = module.createInstallPlan({
      repoRoot,
      installRoot,
      platform: 'linux',
      arch: 'x64',
      releaseOutputDir,
    });

    assert.doesNotThrow(() => module.assertInstallInputsExist(plan));

    rmSync(bundleChecksumPath, { force: true });
    assert.throws(
      () => module.assertInstallInputsExist(plan),
      /Missing install bundle checksum file:/,
    );
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('assertInstallInputsExist requires a matching release-catalog entry for bundle-backed install plans', async () => {
  const module = await loadModule();
  const fixtureRoot = createTempDir('install-bundle-catalog-');
  const installRoot = path.join(fixtureRoot, 'install');
  const releaseOutputDir = path.join(fixtureRoot, 'release');
  const { releaseCatalogPath } = createOfficialServerBundleFixture({
    releaseOutputDir,
    platform: 'linux',
    arch: 'x64',
  });

  try {
    const plan = module.createInstallPlan({
      repoRoot,
      installRoot,
      platform: 'linux',
      arch: 'x64',
      releaseOutputDir,
    });

    assert.doesNotThrow(() => module.assertInstallInputsExist(plan));

    writeFileSync(
      releaseCatalogPath,
      `${JSON.stringify({
        version: 1,
        type: 'sdkwork-release-catalog',
        releaseTag: 'fixture-release',
        generatedAt: '2026-04-18T00:00:00.000Z',
        productCount: 1,
        variantCount: 1,
        products: [
          {
            productId: 'sdkwork-api-router-product-server',
            variants: [
              {
                platform: 'linux',
                arch: 'x64',
                outputDirectory: 'native/linux/x64/bundles',
                variantKind: 'server-archive',
                primaryFile: 'wrong-server-bundle.tar.gz',
                checksumFile: 'wrong-server-bundle.tar.gz.sha256.txt',
                checksumAlgorithm: 'sha256',
                manifestFile: 'wrong-server-bundle.manifest.json',
                sha256: 'sha256',
                manifest: {
                  type: 'product-server-archive',
                  productId: 'sdkwork-api-router-product-server',
                  platform: 'linux',
                  arch: 'x64',
                },
              },
            ],
          },
        ],
      }, null, 2)}\n`,
      'utf8',
    );

    assert.throws(
      () => module.assertInstallInputsExist(plan),
      /Missing install bundle release-catalog entry:/,
    );
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('applyInstallPlan materializes the versioned payload from the packaged server bundle', async () => {
  const module = await loadModule();
  const fixtureRoot = createTempDir('install-bundle-apply-');
  const installRoot = path.join(fixtureRoot, 'install');
  const releaseOutputDir = path.join(fixtureRoot, 'release');

  try {
    createOfficialServerBundleFixture({
      releaseOutputDir,
      platform: 'linux',
      arch: 'x64',
    });

    const plan = module.createInstallPlan({
      repoRoot,
      installRoot,
      platform: 'linux',
      arch: 'x64',
      releaseOutputDir,
    });

    module.assertInstallInputsExist(plan);
    module.applyInstallPlan(plan, { force: true });

    const releaseRoot = path.join(installRoot, 'releases', '0.1.0');
    const currentRoot = path.join(installRoot, 'current');
    const currentManifestPath = path.join(currentRoot, 'release-manifest.json');
    const releasePayloadManifestPath = path.join(releaseRoot, 'release-manifest.json');
    const releasePayloadReadmePath = path.join(releaseRoot, 'README.txt');
    const currentManifest = JSON.parse(readFileSync(currentManifestPath, 'utf8'));
    const releasePayloadManifest = JSON.parse(readFileSync(releasePayloadManifestPath, 'utf8'));

    assert.equal(existsSync(path.join(currentRoot, 'bin', 'start.sh')), true);
    assert.equal(
      readFileSync(path.join(releaseRoot, 'sites', 'admin', 'dist', 'index.html'), 'utf8'),
      '<html>bundled admin</html>\n',
    );
    assert.equal(
      readFileSync(path.join(releaseRoot, 'sites', 'portal', 'dist', 'index.html'), 'utf8'),
      '<html>bundled portal</html>\n',
    );
    assert.equal(
      readFileSync(path.join(releaseRoot, 'data', 'seed.json'), 'utf8'),
      '{"seed":"bundled"}\n',
    );
    assert.equal(
      readFileSync(path.join(releaseRoot, 'deploy', 'docker', 'docker-compose.yml'), 'utf8'),
      'services:\n  router:\n    image: sdkwork\n',
    );
    assert.equal(readFileSync(releasePayloadReadmePath, 'utf8'), 'Official bundled readme\n');
    assert.equal(releasePayloadManifest.bundleOrigin, 'test-fixture');
    assert.equal(
      toPortablePath(currentManifest.releasePayloadManifest),
      toPortablePath(releasePayloadManifestPath),
    );
    assert.equal(
      toPortablePath(currentManifest.releasePayloadReadmeFile),
      toPortablePath(releasePayloadReadmePath),
    );
    assert.equal(
      toPortablePath(currentManifest.deploymentAssetRoot),
      toPortablePath(path.join(releaseRoot, 'deploy')),
    );
    assert.equal(
      toPortablePath(currentManifest.bootstrapDataRoot),
      toPortablePath(path.join(releaseRoot, 'data')),
    );
  } finally {
    removeTempRuntimeHome(fixtureRoot);
  }
});

test('renderRuntimeEnvTemplate defaults release runtime to writable local data and product server-mode ports', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork-api-router';

  const envFile = module.renderRuntimeEnvTemplate({
    installRoot,
    platform: 'linux',
  });

  assert.match(envFile, /SDKWORK_CONFIG_DIR="\/opt\/sdkwork-api-router\/config"/);
  assert.match(envFile, /SDKWORK_CONFIG_FILE="\/opt\/sdkwork-api-router\/config\/router\.yaml"/);
  assert.match(envFile, /SDKWORK_DATABASE_URL="sqlite:\/\/\/opt\/sdkwork-api-router\/data\/sdkwork-api-router\.db"/);
  assert.match(envFile, /SDKWORK_WEB_BIND="0\.0\.0\.0:3001"/);
  assert.match(envFile, /SDKWORK_GATEWAY_BIND="127\.0\.0\.1:8080"/);
  assert.match(envFile, /SDKWORK_ADMIN_BIND="127\.0\.0\.1:8081"/);
  assert.match(envFile, /SDKWORK_PORTAL_BIND="127\.0\.0\.1:8082"/);
  assert.doesNotMatch(envFile, /SDKWORK_ADMIN_SITE_DIR=/);
  assert.doesNotMatch(envFile, /SDKWORK_PORTAL_SITE_DIR=/);
  assert.doesNotMatch(envFile, /SDKWORK_ROUTER_BINARY=/);
});

test('renderRuntimeEnvTemplate system mode prefers config-file discovery and PostgreSQL placeholders', async () => {
  const module = await loadModule();
  const envFile = module.renderRuntimeEnvTemplate({
    installRoot: '/opt/sdkwork-api-router',
    mode: 'system',
    platform: 'linux',
  });

  assert.match(envFile, /SDKWORK_CONFIG_DIR="\/etc\/sdkwork-api-router"/);
  assert.match(envFile, /SDKWORK_CONFIG_FILE="\/etc\/sdkwork-api-router\/router\.yaml"/);
  assert.match(envFile, /SDKWORK_DATABASE_URL="postgresql:\/\/sdkwork:change-me@127\.0\.0\.1:5432\/sdkwork_api_router"/);
  assert.doesNotMatch(envFile, /SDKWORK_ADMIN_SITE_DIR=/);
  assert.doesNotMatch(envFile, /SDKWORK_PORTAL_SITE_DIR=/);
  assert.doesNotMatch(envFile, /SDKWORK_ROUTER_BINARY=/);
  assert.doesNotMatch(envFile, /sqlite:\/\/\/opt\/sdkwork-api-router\/current\/var\/data\/sdkwork-api-router\.db/);
});

test('production start scripts default to product server-mode binds instead of managed dev preview binds', () => {
  const startPs1 = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');

  assert.match(startPs1, /\$env:SDKWORK_WEB_BIND = '0\.0\.0\.0:3001'/);
  assert.match(startPs1, /\$env:SDKWORK_GATEWAY_BIND = '127\.0\.0\.1:8080'/);
  assert.match(startPs1, /\$env:SDKWORK_ADMIN_BIND = '127\.0\.0\.1:8081'/);
  assert.match(startPs1, /\$env:SDKWORK_PORTAL_BIND = '127\.0\.0\.1:8082'/);

  assert.match(startSh, /^SDKWORK_WEB_BIND=\$\{SDKWORK_WEB_BIND:-"0\.0\.0\.0:3001"\}$/m);
  assert.match(startSh, /^SDKWORK_GATEWAY_BIND=\$\{SDKWORK_GATEWAY_BIND:-"127\.0\.0\.1:8080"\}$/m);
  assert.match(startSh, /^SDKWORK_ADMIN_BIND=\$\{SDKWORK_ADMIN_BIND:-"127\.0\.0\.1:8081"\}$/m);
  assert.match(startSh, /^SDKWORK_PORTAL_BIND=\$\{SDKWORK_PORTAL_BIND:-"127\.0\.0\.1:8082"\}$/m);
});

test('service descriptors start the production runtime in foreground mode from the installed home', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork-api-router';

  const systemdUnit = module.renderSystemdUnit({
    installRoot,
    serviceName: 'sdkwork-api-router',
  });
  const launchdPlist = module.renderLaunchdPlist({
    installRoot,
    serviceName: 'com.sdkwork.api-router',
  });
  const windowsTaskXml = module.renderWindowsTaskXml({
    installRoot: 'C:/sdkwork/api-router',
    taskName: 'sdkwork-api-router',
  });

  assert.match(systemdUnit, /ExecStart="\/opt\/sdkwork-api-router\/current\/bin\/start\.sh" --foreground --home "\/opt\/sdkwork-api-router\/current"/);
  assert.match(systemdUnit, /EnvironmentFile=-\/opt\/sdkwork-api-router\/config\/router\.env/);
  assert.match(systemdUnit, /WorkingDirectory=\/opt\/sdkwork-api-router\/current/);

  assert.match(launchdPlist, /<string>\/opt\/sdkwork-api-router\/current\/bin\/start\.sh<\/string>/);
  assert.match(launchdPlist, /<string>--foreground<\/string>/);
  assert.match(launchdPlist, /<string>\/opt\/sdkwork-api-router\/current<\/string>/);
  assert.match(launchdPlist, /<key>KeepAlive<\/key>/);

  assert.match(windowsTaskXml, /powershell\.exe/);
  assert.match(windowsTaskXml, /start\.ps1/);
  assert.match(windowsTaskXml, /-Foreground/);
  assert.match(windowsTaskXml, /sdkwork-api-router/);
});

test('rendered runtime env and systemd unit safely handle install roots with spaces', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork router';

  const envFile = module.renderRuntimeEnvTemplate({
    installRoot,
    platform: 'linux',
  });
  const systemdUnit = module.renderSystemdUnit({
    installRoot,
    serviceName: 'sdkwork-api-router',
  });

  assert.match(envFile, /^SDKWORK_CONFIG_DIR="\/opt\/sdkwork router\/config"$/m);
  assert.match(envFile, /^SDKWORK_DATABASE_URL="sqlite:\/\/\/opt\/sdkwork router\/data\/sdkwork-api-router\.db"$/m);
  assert.doesNotMatch(envFile, /^SDKWORK_ADMIN_SITE_DIR=/m);
  assert.doesNotMatch(envFile, /^SDKWORK_ROUTER_BINARY=/m);

  assert.match(systemdUnit, /WorkingDirectory=\/opt\/sdkwork\\ router\/current/);
  assert.match(systemdUnit, /EnvironmentFile=-\/opt\/sdkwork\\ router\/config\/router\.env/);
  assert.match(systemdUnit, /ExecStart="\/opt\/sdkwork router\/current\/bin\/start\.sh" --foreground --home "\/opt\/sdkwork router\/current"/);
});

test('generated systemd service helper scripts execute against stubbed tools in a writable directory', { skip: process.platform === 'win32' }, async () => {
  const module = await loadModule();
  const tempRoot = createTempDir('service-systemd-');
  const serviceDir = path.join(tempRoot, 'service', 'systemd');
  const fakeBinDir = path.join(tempRoot, 'fake-bin');
  const targetDir = path.join(tempRoot, 'systemd-target');
  const logFile = path.join(tempRoot, 'systemctl.log');

  mkdirSync(serviceDir, { recursive: true });
  mkdirSync(fakeBinDir, { recursive: true });
  mkdirSync(targetDir, { recursive: true });

  try {
    writeFileSync(
      path.join(serviceDir, 'sdkwork-api-router.service'),
      module.renderSystemdUnit({ installRoot: '/opt/sdkwork-api-router/current' }),
      'utf8',
    );
    writeFileSync(
      path.join(serviceDir, 'install-service.sh'),
      module.renderSystemdInstallScript(),
      'utf8',
    );
    writeFileSync(
      path.join(serviceDir, 'uninstall-service.sh'),
      module.renderSystemdUninstallScript(),
      'utf8',
    );
    writeFileSync(path.join(fakeBinDir, 'sudo'), '#!/usr/bin/env sh\nexec "$@"\n', 'utf8');
    writeFileSync(
      path.join(fakeBinDir, 'systemctl'),
      '#!/usr/bin/env sh\nprintf "%s\\n" "$*" >> "$LOG_FILE"\n',
      'utf8',
    );
    chmodSync(path.join(fakeBinDir, 'sudo'), 0o755);
    chmodSync(path.join(fakeBinDir, 'systemctl'), 0o755);

    const env = {
      ...process.env,
      PATH: `${fakeBinDir}:${process.env.PATH ?? ''}`,
      SYSTEMD_DIR: targetDir,
      SYSTEMCTL_BIN: 'systemctl',
      LOG_FILE: logFile,
    };

    const installResult = spawnSync('sh', ['install-service.sh'], {
      cwd: serviceDir,
      env,
      encoding: 'utf8',
    });
    assert.equal(installResult.status, 0, installResult.stderr || installResult.stdout);
    assert.equal(existsSync(path.join(targetDir, 'sdkwork-api-router.service')), true);

    const uninstallResult = spawnSync('sh', ['uninstall-service.sh'], {
      cwd: serviceDir,
      env,
      encoding: 'utf8',
    });
    assert.equal(uninstallResult.status, 0, uninstallResult.stderr || uninstallResult.stdout);
    assert.equal(existsSync(path.join(targetDir, 'sdkwork-api-router.service')), false);

    const log = readFileSync(logFile, 'utf8');
    assert.match(log, /daemon-reload/);
    assert.match(log, /enable --now sdkwork-api-router\.service/);
    assert.match(log, /disable --now sdkwork-api-router\.service/);
    assert.match(log, /reset-failed sdkwork-api-router\.service/);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
});

test('generated launchd helper scripts execute against stubbed tools in a writable directory', { skip: process.platform === 'win32' }, async () => {
  const module = await loadModule();
  const tempRoot = createTempDir('service-launchd-');
  const serviceDir = path.join(tempRoot, 'service', 'launchd');
  const fakeBinDir = path.join(tempRoot, 'fake-bin');
  const targetDir = path.join(tempRoot, 'launchd-target');
  const logFile = path.join(tempRoot, 'launchctl.log');

  mkdirSync(serviceDir, { recursive: true });
  mkdirSync(fakeBinDir, { recursive: true });
  mkdirSync(targetDir, { recursive: true });

  try {
    writeFileSync(
      path.join(serviceDir, 'com.sdkwork.api-router.plist'),
      module.renderLaunchdPlist({
        installRoot: '/opt/sdkwork-api-router/current',
        serviceName: 'com.sdkwork.api-router',
      }),
      'utf8',
    );
    writeFileSync(
      path.join(serviceDir, 'install-service.sh'),
      module.renderLaunchdInstallScript(),
      'utf8',
    );
    writeFileSync(
      path.join(serviceDir, 'uninstall-service.sh'),
      module.renderLaunchdUninstallScript(),
      'utf8',
    );
    writeFileSync(path.join(fakeBinDir, 'sudo'), '#!/usr/bin/env sh\nexec "$@"\n', 'utf8');
    writeFileSync(
      path.join(fakeBinDir, 'launchctl'),
      '#!/usr/bin/env sh\nprintf "%s\\n" "$*" >> "$LOG_FILE"\n',
      'utf8',
    );
    chmodSync(path.join(fakeBinDir, 'sudo'), 0o755);
    chmodSync(path.join(fakeBinDir, 'launchctl'), 0o755);

    const env = {
      ...process.env,
      PATH: `${fakeBinDir}:${process.env.PATH ?? ''}`,
      LAUNCHD_TARGET_DIR: targetDir,
      LAUNCHCTL_BIN: 'launchctl',
      LOG_FILE: logFile,
    };

    const installResult = spawnSync('sh', ['install-service.sh'], {
      cwd: serviceDir,
      env,
      encoding: 'utf8',
    });
    assert.equal(installResult.status, 0, installResult.stderr || installResult.stdout);
    assert.equal(existsSync(path.join(targetDir, 'com.sdkwork.api-router.plist')), true);

    const uninstallResult = spawnSync('sh', ['uninstall-service.sh'], {
      cwd: serviceDir,
      env,
      encoding: 'utf8',
    });
    assert.equal(uninstallResult.status, 0, uninstallResult.stderr || uninstallResult.stdout);
    assert.equal(existsSync(path.join(targetDir, 'com.sdkwork.api-router.plist')), false);

    const log = readFileSync(logFile, 'utf8');
    assert.match(log, /bootstrap system/);
    assert.match(log, /enable system\/com\.sdkwork\.api-router/);
    assert.match(log, /bootout system\/com\.sdkwork\.api-router/);
    assert.match(log, /disable system\/com\.sdkwork\.api-router/);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
});

test('generated windows task helper scripts support stubbed schtasks execution for smoke testing', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, async () => {
  const module = await loadModule();
  const tempRoot = createTempDir('service-wintask-');
  const serviceDir = path.join(tempRoot, 'service', 'windows-task');
  const fakeSchTasks = path.join(tempRoot, 'fake-schtasks.cmd');
  const logFile = path.join(tempRoot, 'schtasks.log');

  mkdirSync(serviceDir, { recursive: true });

  try {
    writeFileSync(
      path.join(serviceDir, 'sdkwork-api-router.xml'),
      module.renderWindowsTaskXml({
        installRoot: 'C:/sdkwork/api-router/current',
        taskName: 'sdkwork-api-router',
      }),
      'utf8',
    );
    writeFileSync(
      path.join(serviceDir, 'install-service.ps1'),
      module.renderWindowsTaskInstallScript(),
      'utf8',
    );
    writeFileSync(
      path.join(serviceDir, 'uninstall-service.ps1'),
      module.renderWindowsTaskUninstallScript(),
      'utf8',
    );
    writeFileSync(
      fakeSchTasks,
      [
        '@echo off',
        'echo %*>> "%SCHTASKS_LOG_FILE%"',
        'if /I "%1"=="/Query" exit /b 0',
        'exit /b 0',
        '',
      ].join('\r\n'),
      'utf8',
    );

    const env = {
      ...process.env,
      SCHTASKS_LOG_FILE: logFile,
    };

    const installResult = spawnSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(serviceDir, 'install-service.ps1'),
        '-StartNow',
        '-SkipAdminCheck',
        '-SchTasksBin',
        fakeSchTasks,
      ],
      {
        cwd: serviceDir,
        env,
        encoding: 'utf8',
      },
    );
    assert.equal(installResult.status, 0, installResult.stderr || installResult.stdout);

    const uninstallResult = spawnSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(serviceDir, 'uninstall-service.ps1'),
        '-SkipAdminCheck',
        '-SchTasksBin',
        fakeSchTasks,
      ],
      {
        cwd: serviceDir,
        env,
        encoding: 'utf8',
      },
    );
    assert.equal(uninstallResult.status, 0, uninstallResult.stderr || uninstallResult.stdout);

    const installScript = readFileSync(path.join(serviceDir, 'install-service.ps1'), 'utf8');
    const uninstallScript = readFileSync(path.join(serviceDir, 'uninstall-service.ps1'), 'utf8');
    assert.match(installScript, /\[string\]\$SchTasksBin = \$env:SCHTASKS_BIN/);
    assert.match(installScript, /\[switch\]\$SkipAdminCheck/);
    assert.match(uninstallScript, /\[string\]\$SchTasksBin = \$env:SCHTASKS_BIN/);
    assert.match(uninstallScript, /\[switch\]\$SkipAdminCheck/);

    const log = readFileSync(logFile, 'utf8');
    assert.match(log, /\/Create \/TN sdkwork-api-router \/XML/);
    assert.match(log, /\/Run \/TN sdkwork-api-router/);
    assert.match(log, /\/Query \/TN sdkwork-api-router/);
    assert.match(log, /\/Delete \/TN sdkwork-api-router \/F/);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
});

test('router-ops install rejects --home without a following value', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--home']),
      /--home requires a value/,
    );
  });
});

test('router-ops build defaults to the official release inputs without the legacy console workspace', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    const options = parseArgs(['build']);

    assert.equal(options.command, 'build');
    assert.equal(options.includeDocs, true);
    assert.equal(Object.hasOwn(options, 'includeConsole'), false);
  });
});

test('router-ops build parses --verify-release as an official local release smoke toggle', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    const options = parseArgs(['build', '--verify-release']);

    assert.equal(options.command, 'build');
    assert.equal(options.verifyRelease, true);
  });
});

test('router-ops build rejects --skip-docs when official release verification is requested', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['build', '--verify-release', '--skip-docs']),
      /--skip-docs cannot be combined with --verify-release/,
    );
    assert.throws(
      () => parseArgs(['build', '--skip-docs', '--verify-release']),
      /--skip-docs cannot be combined with --verify-release/,
    );
  });
});

test('router-ops build rejects legacy console packaging switches', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['build', '--include-console']),
      /unknown option: --include-console/,
    );
    assert.throws(
      () => parseArgs(['build', '--skip-console']),
      /unknown option: --skip-console/,
    );
  });
});

test('router-ops install parses system mode', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    const options = parseArgs(['install', '--mode', 'system']);

    assert.equal(options.command, 'install');
    assert.equal(options.mode, 'system');
  });
});

test('router-ops resolveInstallRootOption keeps Windows absolute homes stable on posix hosts', () => {
  return loadRouterOpsModule().then(({ resolveInstallRootOption }) => {
    const resolved = resolveInstallRootOption('D:/router/runtime', {
      cwd: '/workspace/sdkwork-api-router',
      pathModule: path.posix,
    });

    assert.equal(resolved, 'D:/router/runtime');
  });
});

test('router-ops validate-config parses system mode with runtime-home overrides', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    const options = parseArgs(['validate-config', '--mode', 'system', '--home', 'D:/router/runtime']);

    assert.equal(options.command, 'validate-config');
    assert.equal(options.mode, 'system');
    assert.equal(toPortablePath(options.installRoot), 'D:/router/runtime');
  });
});

test('router-ops install parses portable mode with a custom home', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    const options = parseArgs(['install', '--mode', 'portable', '--home', 'D:/custom/router']);

    assert.equal(options.command, 'install');
    assert.equal(options.mode, 'portable');
    assert.equal(toPortablePath(options.installRoot), 'D:/custom/router');
  });
});

test('router-ops install rejects --home when the next token is another flag', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--home', '--dry-run']),
      /--home requires a value/,
    );
  });
});

test('router-ops rejects build-only flags during install', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--skip-docs']),
      /--skip-docs is only supported for the build command/,
    );
    assert.throws(
      () => parseArgs(['install', '--verify-release']),
      /--verify-release is only supported for the build command/,
    );
  });
});

test('router-ops rejects install-only flags during build', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['build', '--home', 'artifacts/install/custom']),
      /--home is only supported for install or validate-config/,
    );
  });
});

test('router-ops rejects an install mode without a following value', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--mode']),
      /--mode requires a value/,
    );
  });
});

test('start.ps1 keeps the public -Home switch via alias instead of binding to the built-in HOME variable', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');

  assert.match(script, /\[Alias\('Home'\)\]\s*\r?\n\s*\[string\]\$RuntimeHome = ''/);
  assert.doesNotMatch(script, /\[string\]\$Home = ''/);
});

test('stop.ps1 keeps the public -Home switch via alias instead of binding to the built-in HOME variable', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');

  assert.match(script, /\[Alias\('Home'\)\]\s*\r?\n\s*\[string\]\$RuntimeHome = ''/);
  assert.doesNotMatch(script, /\[string\]\$Home = ''/);
});

test('runtime-common.ps1 avoids assigning to the built-in HOST variable while resolving health URLs', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.doesNotMatch(script, /\$host\s*=/i);
  assert.match(script, /\$bindHost\s*=/);
});

test('runtime-common.ps1 avoids binding helper parameters to the built-in PID variable', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.doesNotMatch(script, /\[Parameter\(Mandatory = \$true\)\]\[int\]\$Pid\b/);
  assert.match(script, /\[Parameter\(Mandatory = \$true\)\]\[int\]\$ProcessId\b/);
  assert.match(script, /Wait-RouterProcessExit\s*{[\s\S]*?\$ProcessId/s);
  assert.match(script, /Stop-RouterProcessTree\s*{[\s\S]*?\$ProcessId/s);
});

test('start.ps1 and stop.ps1 resolve router-product-service through a shared PowerShell binary-name helper', () => {
  const startScript = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const stopScript = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');

  assert.match(startScript, /\$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'/);
  assert.match(stopScript, /\$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'/);
  assert.doesNotMatch(startScript, /\$binaryName = 'router-product-service\.exe'/);
  assert.doesNotMatch(stopScript, /\$binaryName = 'router-product-service\.exe'/);
});

test('validate-config entrypoints delegate to the managed start entrypoints in dry-run mode', () => {
  const validateConfigPs1 = readFileSync(path.join(repoRoot, 'bin', 'validate-config.ps1'), 'utf8');
  const validateConfigSh = readFileSync(path.join(repoRoot, 'bin', 'validate-config.sh'), 'utf8');

  assert.match(validateConfigPs1, /start\.ps1/);
  assert.match(validateConfigPs1, /-DryRun @args/);
  assert.match(validateConfigSh, /start\.sh/);
  assert.match(validateConfigSh, /--dry-run "\$@"/);
});

test('PowerShell stop scripts pass ProcessId into the shared process-tree helper', () => {
  const stopDevScript = readFileSync(path.join(repoRoot, 'bin', 'stop-dev.ps1'), 'utf8');
  const stopScript = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');

  assert.match(stopDevScript, /Stop-RouterProcessTree -ProcessId \(\[int\]\$pidValue\)/);
  assert.match(stopScript, /Stop-RouterProcessTree -ProcessId \(\[int\]\$pidValue\)/);
  assert.doesNotMatch(stopDevScript, /Stop-RouterProcessTree -Pid /);
  assert.doesNotMatch(stopScript, /Stop-RouterProcessTree -Pid /);
});

test('runtime-common.ps1 includes platform-aware PowerShell process and binary helpers', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(script, /function Test-RouterWindowsPlatform/);
  assert.match(script, /function Get-RouterRuntimePlatformKey/);
  assert.match(script, /function Get-RouterBinaryName/);
  assert.match(script, /function Resolve-RouterHostPath/);
  assert.match(script, /function Get-RouterReleaseDryRunPlanJson/);
  assert.match(script, /ps -o pid= -o ppid=/);
  assert.match(script, /Stop-Process -Id \$processId/);
});

test('runtime-common.ps1 carries startup summary helpers with unified links, direct links, and bootstrap identity guidance', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(script, /function Get-RouterStartupSummaryLines/);
  assert.match(script, /function Write-RouterStartupSummary/);
  assert.match(script, /function Start-RouterBackgroundProcess/);
  assert.match(script, /Unified Access/);
  assert.match(script, /Direct Service Access/);
  assert.match(script, /Identity Bootstrap/);
  assert.match(script, /\/api\/v1\/health/);
  assert.match(script, /\/admin\/health/);
  assert.match(script, /\/portal\/health/);
  assert.match(script, /\/openapi\.json/);
  assert.match(script, /\/admin\/openapi\.json/);
  assert.match(script, /\/portal\/openapi\.json/);
  assert.match(script, /SDKWORK_BOOTSTRAP_PROFILE/);
  assert.match(script, /SDKWORK_BOOTSTRAP_DATA_DIR/);
  assert.match(script, /active bootstrap profile/);
  assert.match(script, /runtime configuration/);
  assert.doesNotMatch(script, /admin@sdkwork\.local/);
  assert.doesNotMatch(script, /portal@sdkwork\.local/);
  assert.doesNotMatch(script, /ChangeMe123!/);
});

test('Get-RouterStartupSummaryLines includes direct OpenAPI schema URLs for gateway, admin, and portal services', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, () => {
  const commonScript = quoteForPowerShellSingleQuotedString(
    path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'),
  );
  const result = runPowerShellCommand(
    [
      `. '${commonScript}'`,
      "$lines = Get-RouterStartupSummaryLines -Mode 'development preview' -WebBind '127.0.0.1:9983' -GatewayBind '127.0.0.1:9980' -AdminBind '127.0.0.1:9981' -PortalBind '127.0.0.1:9982' -UnifiedAccessEnabled $true -AdminAppUrl 'http://127.0.0.1:9983/admin/' -PortalAppUrl 'http://127.0.0.1:9983/portal/' -StdoutLog 'stdout.log' -StderrLog 'stderr.log'",
      '$lines | ForEach-Object { Write-Output $_ }',
    ].join('\n'),
  );
  const output = `${result.stdout}${result.stderr}`;

  assert.equal(result.status, 0, output);
  assert.match(output, /Gateway OpenAPI 3\.x Schema: http:\/\/127\.0\.0\.1:9980\/openapi\.json/);
  assert.match(output, /Admin OpenAPI 3\.x Schema: http:\/\/127\.0\.0\.1:9981\/admin\/openapi\.json/);
  assert.match(output, /Portal OpenAPI 3\.x Schema: http:\/\/127\.0\.0\.1:9982\/portal\/openapi\.json/);
});

test('runtime-common.sh carries matching startup summary and bootstrap identity guidance for shell entrypoints', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');

  assert.match(script, /router_startup_summary/);
  assert.match(script, /router_is_windows\(\)/);
  assert.match(script, /router_runtime_key\(\)/);
  assert.match(script, /router_resolve_host_path\(\)/);
  assert.match(script, /router_render_release_dry_run_plan_json\(\)/);
  assert.match(script, /router_start_background_process\(\)/);
  assert.match(script, /powershell\.exe -NoProfile -ExecutionPolicy Bypass -Command/);
  assert.match(script, /Identity Bootstrap/);
  assert.match(script, /\/openapi\.json/);
  assert.match(script, /\/admin\/openapi\.json/);
  assert.match(script, /\/portal\/openapi\.json/);
  assert.match(script, /SDKWORK_BOOTSTRAP_PROFILE/);
  assert.match(script, /SDKWORK_BOOTSTRAP_DATA_DIR/);
  assert.match(script, /active bootstrap profile/);
  assert.match(script, /runtime configuration/);
  assert.doesNotMatch(script, /admin@sdkwork\.local/);
  assert.doesNotMatch(script, /portal@sdkwork\.local/);
  assert.doesNotMatch(script, /ChangeMe123!/);
});

test('shell runtime launchers stop waiting for health checks once the managed child exits', () => {
  const commonScript = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startDevScript = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const startScript = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');

  assert.match(commonScript, /router_wait_for_url\(\)/);
  assert.match(commonScript, /WATCH_PID="\$\{3:-\}"/);
  assert.match(commonScript, /router_is_pid_running "\$WATCH_PID"/);
  assert.match(commonScript, /router_confirm_pid_alive\(\)/);
  assert.match(startDevScript, /router_wait_for_url "\$GATEWAY_HEALTH_URL" "\$WAIT_SECONDS" "\$PID"/);
  assert.match(startDevScript, /router_start_background_process "\$NODE_BIN" "\$REPO_ROOT" "\$STDOUT_LOG" "\$STDERR_LOG" "\$@"/);
  assert.match(startDevScript, /router_confirm_pid_alive "\$PID" 2/);
  assert.match(startDevScript, /WORKSPACE_EXITED=0/);
  assert.match(startDevScript, /if ! router_is_pid_running "\$PID"; then\s+    WORKSPACE_EXITED=1/s);
  assert.match(startDevScript, /development workspace exited before backend health checks completed; see startup log above/);
  assert.match(startDevScript, /development workspace exited before web surfaces became ready; see startup log above/);
  assert.match(startDevScript, /development workspace exited immediately after reporting ready; see startup log above/);
  assert.match(startScript, /router_wait_for_url "\$GATEWAY_HEALTH_URL" "\$WAIT_SECONDS" "\$PID"/);
  assert.match(startScript, /router_start_background_process "\$SDKWORK_ROUTER_BINARY" "\$RUNTIME_HOME" "\$STDOUT_LOG" "\$STDERR_LOG"/);
  assert.match(startScript, /router_confirm_pid_alive "\$PID" 2/);
  assert.match(startScript, /RUNTIME_EXITED=0/);
  assert.match(startScript, /if ! router_is_pid_running "\$PID"; then\s+    RUNTIME_EXITED=1/s);
  assert.match(startScript, /production runtime exited before health checks completed; see startup log above/);
  assert.match(startScript, /production runtime exited immediately after reporting ready; see startup log above/);
});

test('start-dev.ps1 defaults the managed dev entrypoint to preview mode and supports explicit browser mode', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');

  assert.match(script, /\[switch\]\$Browser/);
  assert.match(script, /elseif \(-not \$Preview\) \{\s*\$Preview = \$true\s*\}/);
  assert.match(script, /if \(\$Browser\) \{\s*\$Preview = \$false\s*\$ProxyDev = \$false\s*\$Tauri = \$false\s*\}/);
  assert.match(script, /if \(\$Preview\) \{ \$startArgs \+= '--preview' \}/);
});

test('start-dev.ps1 normalizes GNU-style long options before launch mode resolution', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');

  assert.match(script, /\[CmdletBinding\(PositionalBinding = \$false\)\]/);
  assert.match(script, /\[Parameter\(ValueFromRemainingArguments = \$true\)\]\s*\[string\[\]\]\$RemainingArgs/);
  assert.match(script, /switch \(\$optionName\) \{/);
  assert.match(script, /'--proxy-dev'\s*\{[\s\S]*\$ProxyDev = \$true/);
  assert.match(script, /'--dry-run'\s*\{[\s\S]*\$DryRun = \$true/);
  assert.match(script, /'--wait-seconds'\s*\{/);
  assert.match(script, /unknown option: \$arg/);
});

test('start-dev.ps1 prefers repository bootstrap data over stale packaged bin data', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');

  assert.match(
    script,
    /if \(-not \$env:SDKWORK_BOOTSTRAP_DATA_DIR\) \{\s*if \(Test-Path \$repositoryBootstrapDataDirectory -PathType Container\) \{\s*\$env:SDKWORK_BOOTSTRAP_DATA_DIR = Convert-ToRouterPortablePath -PathValue \$repositoryBootstrapDataDirectory\s*\} elseif \(Test-Path \$packagedBootstrapDataDirectory -PathType Container\) \{\s*\$env:SDKWORK_BOOTSTRAP_DATA_DIR = Convert-ToRouterPortablePath -PathValue \$packagedBootstrapDataDirectory\s*\}/s,
  );
});

test('PowerShell runtime launchers start background processes without opening a new console window', () => {
  const startDevScript = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startScript = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const commonScript = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(commonScript, /Start-Process/);
  assert.match(commonScript, /NoNewWindow = \$true/);
  assert.match(commonScript, /RedirectStandardOutput = \$StdoutLog/);
  assert.match(commonScript, /RedirectStandardError = \$StderrLog/);
  assert.match(commonScript, /if \(\$ArgumentList\.Count -gt 0\)/);
  assert.match(commonScript, /\$startProcessArgs\.ArgumentList = \$ArgumentList/);
  assert.match(startDevScript, /\$process = Start-RouterBackgroundProcess/);
  assert.match(startScript, /\$process = Start-RouterBackgroundProcess/);
});

test('PowerShell runtime launchers stop waiting for health checks once the managed child exits', () => {
  const startDevScript = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startScript = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const commonScript = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(commonScript, /function Wait-RouterHealthUrl/);
  assert.match(commonScript, /\[int\]\$ProcessId = 0/);
  assert.match(commonScript, /Get-Process -Id \$ProcessId -ErrorAction SilentlyContinue/);
  assert.match(commonScript, /function Confirm-RouterProcessAlive/);
  assert.match(startDevScript, /Confirm-RouterProcessAlive -ProcessId \$process\.Id -WaitSeconds 2/);
  assert.match(startDevScript, /development workspace exited immediately after reporting ready/);
  assert.match(startScript, /Confirm-RouterProcessAlive -ProcessId \$process\.Id -WaitSeconds 2/);
  assert.match(startScript, /production runtime exited immediately after reporting ready/);
  assert.match(startDevScript, /Wait-RouterHealthUrl -Url \$gatewayHealthUrl -WaitSeconds \$WaitSeconds -ProcessId \$process\.Id/);
  assert.match(startDevScript, /Wait-RouterHealthUrl -Url \$adminSurfaceUrl -WaitSeconds \$WaitSeconds -ProcessId \$process\.Id/);
  assert.match(startScript, /Wait-RouterHealthUrl -Url \$gatewayHealthUrl -WaitSeconds \$WaitSeconds -ProcessId \$process\.Id/);
});

test('PowerShell source-dev wrappers use the 9980-series defaults', () => {
  const workspaceScript = readFileSync(path.join(repoRoot, 'scripts', 'dev', 'start-workspace.ps1'), 'utf8');
  const serversScript = readFileSync(path.join(repoRoot, 'scripts', 'dev', 'start-servers.ps1'), 'utf8');

  assert.match(workspaceScript, /\$AdminBind = "127\.0\.0\.1:9981"/);
  assert.match(workspaceScript, /\$GatewayBind = "127\.0\.0\.1:9980"/);
  assert.match(workspaceScript, /\$PortalBind = "127\.0\.0\.1:9982"/);
  assert.match(workspaceScript, /\$WebBind = "0\.0\.0\.0:9983"/);
  assert.match(serversScript, /\$AdminBind = "127\.0\.0\.1:9981"/);
  assert.match(serversScript, /\$GatewayBind = "127\.0\.0\.1:9980"/);
  assert.match(serversScript, /\$PortalBind = "127\.0\.0\.1:9982"/);
});

test('repository-root wrapper scripts delegate to the managed bin entrypoints', () => {
  const shellWrappers = [
    'build.sh',
    'install.sh',
    'start-dev.sh',
    'start.sh',
    'stop-dev.sh',
    'stop.sh',
    'validate-config.sh',
  ];
  const powershellWrappers = [
    'build.ps1',
    'install.ps1',
    'start-dev.ps1',
    'start.ps1',
    'stop-dev.ps1',
    'stop.ps1',
    'validate-config.ps1',
  ];

  for (const scriptName of shellWrappers) {
    const script = readFileSync(path.join(repoRoot, scriptName), 'utf8');
    assert.match(script, new RegExp(`bin/${scriptName.replace('.', '\\.')}`));
    assert.match(script, /exec "\$TARGET_SCRIPT" "\$@"/);
  }

  for (const scriptName of powershellWrappers) {
    const script = readFileSync(path.join(repoRoot, scriptName), 'utf8');
    assert.match(script, new RegExp(`bin\\\\${scriptName.replace('.', '\\.')}`));
    assert.match(script, /& \$target @args/);
    assert.match(script, /Test-Path Variable:LASTEXITCODE/);
  }
});

test('PowerShell build and install entrypoints normalize native switch names before dispatching to router-ops', () => {
  const buildScript = readFileSync(path.join(repoRoot, 'bin', 'build.ps1'), 'utf8');
  const installScript = readFileSync(path.join(repoRoot, 'bin', 'install.ps1'), 'utf8');

  assert.match(buildScript, /\$translatedArgs = @\('build'\)/);
  assert.match(buildScript, /\^\(\?i\)-DryRun\$/);
  assert.match(buildScript, /--dry-run/);
  assert.match(buildScript, /\^\(\?i\)-SkipDocs\$/);
  assert.match(buildScript, /\^\(\?i\)-Install\$/);
  assert.doesNotMatch(buildScript, /\^\(\?i\)-SkipConsole\$/);
  assert.doesNotMatch(buildScript, /\^\(\?i\)-IncludeConsole\$/);
  assert.doesNotMatch(buildScript, /--skip-console/);
  assert.doesNotMatch(buildScript, /--include-console/);
  assert.match(installScript, /\$translatedArgs = @\('install'\)/);
  assert.match(installScript, /\^\(\?i\)-DryRun\$/);
  assert.match(installScript, /\^\(\?i\)-Force\$/);
  assert.match(installScript, /\^\(\?i\)-Home\$/);
  assert.match(installScript, /--home requires a value/);
});

test('Windows shell entrypoints delegate to PowerShell counterparts before passing host paths into Windows processes', () => {
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const buildSh = readFileSync(path.join(repoRoot, 'bin', 'build.sh'), 'utf8');
  const installSh = readFileSync(path.join(repoRoot, 'bin', 'install.sh'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');

  assert.match(commonSh, /router_windows_cli_path\(\)/);
  assert.match(commonSh, /router_windows_database_url\(\)/);

  assert.match(buildSh, /if router_is_windows; then/);
  assert.match(buildSh, /build\.ps1/);
  assert.match(buildSh, /powershell\.exe -NoProfile -ExecutionPolicy Bypass -File/);

  assert.match(installSh, /if router_is_windows; then/);
  assert.match(installSh, /install\.ps1/);
  assert.match(installSh, /powershell\.exe -NoProfile -ExecutionPolicy Bypass -File/);

  assert.match(startDevSh, /if router_is_windows; then/);
  assert.match(startDevSh, /start-dev\.ps1/);
  assert.match(startDevSh, /router_windows_database_url/);
  assert.match(startDevSh, /-Browser/);
  assert.match(startDevSh, /-Preview/);
  assert.match(startDevSh, /-Tauri/);

  assert.match(startSh, /if router_is_windows; then/);
  assert.match(startSh, /start\.ps1/);
  assert.match(startSh, /router_windows_cli_path/);
  assert.match(startSh, /router_windows_database_url/);
  assert.match(startSh, /-Home/);
  assert.match(startSh, /-ConfigDir/);
  assert.match(startSh, /-ConfigFile/);
  assert.match(startSh, /-AdminSiteDir/);
  assert.match(startSh, /-PortalSiteDir/);
});

test('PowerShell runtime launchers distinguish early child exits from plain health-check timeouts', () => {
  const startDevScript = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startScript = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');

  assert.match(startDevScript, /Get-Process -Id \$process\.Id -ErrorAction SilentlyContinue/);
  assert.match(startDevScript, /development workspace exited before backend health checks completed/);
  assert.match(startDevScript, /development workspace exited before web surfaces became ready/);
  assert.match(startScript, /Get-Process -Id \$process\.Id -ErrorAction SilentlyContinue/);
  assert.match(startScript, /production runtime exited before health checks completed/);
});

test('development start and stop scripts use a cooperative stop-file handshake before kill fallbacks', () => {
  const startDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const stopDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'stop-dev.ps1'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const stopDevSh = readFileSync(path.join(repoRoot, 'bin', 'stop-dev.sh'), 'utf8');

  assert.match(startDevPs1, /\$stopFile = Join-Path \$runDirectory 'start-workspace\.stop'/);
  assert.match(startDevPs1, /if \(Test-Path \$stopFile\) \{ Remove-Item \$stopFile -Force -ErrorAction SilentlyContinue \}/);
  assert.match(startDevPs1, /'--stop-file', \$stopFile/);
  assert.match(stopDevPs1, /\$stopFile = Join-Path \$devHome 'run\\start-workspace\.stop'/);
  assert.match(stopDevPs1, /Set-Content -Path \$stopFile -Value/);
  assert.match(stopDevPs1, /Wait-RouterProcessExit -ProcessId \(\[int\]\$pidValue\) -WaitSeconds \$WaitSeconds/);
  assert.match(stopDevPs1, /Stop-RouterProcessTree -ProcessId \(\[int\]\$pidValue\)/);

  assert.match(startDevSh, /^STOP_FILE="\$RUN_DIR\/start-workspace\.stop"$/m);
  assert.match(startDevSh, /rm -f "\$STOP_FILE"/);
  assert.match(startDevSh, /--stop-file "\$STOP_FILE"/);
  assert.match(stopDevSh, /^STOP_FILE="\$DEV_HOME\/run\/start-workspace\.stop"$/m);
  assert.match(stopDevSh, /: > "\$STOP_FILE"/);
  assert.match(stopDevSh, /router_wait_for_pid_exit "\$PID" "\$WAIT_SECONDS"/);
  assert.match(stopDevSh, /router_stop_pid "\$PID" "\$WAIT_SECONDS" "\$FORCE_MODE"/);
});

test('start scripts treat healthy managed runtimes as idempotent instead of hard errors', () => {
  const commonPs1 = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startPs1 = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');

  assert.match(commonPs1, /function Get-RouterManagedProcessId/);
  assert.match(startDevPs1, /Get-RouterManagedProcessId -PidFile \$pidFile/);
  assert.match(startDevPs1, /development workspace already running \(pid=\$\(\$existingPid\)\)/);
  assert.match(startPs1, /Get-RouterManagedProcessId -PidFile \$pidFile/);
  assert.match(startPs1, /production runtime already running \(pid=\$\(\$existingPid\)\)/);

  assert.match(commonSh, /router_get_running_pid\(\)/);
  assert.match(startDevSh, /EXISTING_PID=\$\(router_get_running_pid "\$PID_FILE" "\$STATE_FILE"\)/);
  assert.match(startDevSh, /development workspace already running \(pid=\$EXISTING_PID\)/);
  assert.match(startSh, /EXISTING_PID=\$\(router_get_running_pid "\$PID_FILE" "\$STATE_FILE"\)/);
  assert.match(startSh, /production runtime already running \(pid=\$EXISTING_PID\)/);
});

test('start-dev scripts fail fast when a healthy managed workspace uses different launch settings', () => {
  const startDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');

  assert.match(startDevPs1, /\$requestedConfigurationDiffers =/);
  assert.match(startDevPs1, /Throw-RouterError "development workspace already running \(pid=\$\(\$existingPid\)\) with active managed settings that differ from the requested launch configuration; run bin\/stop-dev\.ps1 before relaunching with different settings"/);

  assert.match(startDevSh, /requestedConfigurationDiffers=/);
  assert.match(startDevSh, /router_die "development workspace already running \(pid=\$EXISTING_PID\) with active managed settings that differ from the requested launch configuration; run bin\/stop-dev\.sh before relaunching with different settings"/);
});

test('runtime launch scripts persist managed state alongside pid files for robust restarts and stops', () => {
  const commonPs1 = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startPs1 = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const stopDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'stop-dev.ps1'), 'utf8');
  const stopPs1 = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');
  const stopDevSh = readFileSync(path.join(repoRoot, 'bin', 'stop-dev.sh'), 'utf8');
  const stopSh = readFileSync(path.join(repoRoot, 'bin', 'stop.sh'), 'utf8');

  assert.match(commonPs1, /function Get-RouterProcessFingerprint/);
  assert.match(commonPs1, /function Get-RouterManagedState/);
  assert.match(commonPs1, /function Write-RouterManagedStateFile/);
  assert.match(commonPs1, /function Remove-RouterManagedStateFile/);
  assert.match(commonPs1, /Get-RouterManagedProcessId \{/);
  assert.match(commonPs1, /\[string\]\$StateFile = ''/);

  assert.match(commonSh, /router_get_process_fingerprint\(\)/);
  assert.match(commonSh, /router_read_managed_state\(\)/);
  assert.match(commonSh, /router_write_managed_state\(\)/);
  assert.match(commonSh, /router_remove_managed_state\(\)/);
  assert.match(commonSh, /router_get_running_pid\(\)/);

  assert.match(startDevPs1, /\$stateFile = Join-Path \$runDirectory 'start-workspace\.state\.env'/);
  assert.match(startDevPs1, /Get-RouterManagedProcessId -PidFile \$pidFile -StateFile \$stateFile/);
  assert.match(startDevPs1, /Get-RouterManagedState -StateFile \$stateFile/);
  assert.match(startDevPs1, /Write-RouterManagedStateFile -StateFile \$stateFile/);
  assert.match(startDevPs1, /Remove-RouterManagedStateFile -StateFile \$stateFile/);
  assert.match(startPs1, /\$stateFile = Join-Path \$runDirectory 'router-product-service\.state\.env'/);
  assert.match(startPs1, /Get-RouterManagedProcessId -PidFile \$pidFile -StateFile \$stateFile/);
  assert.match(startPs1, /Get-RouterManagedState -StateFile \$stateFile/);
  assert.match(startPs1, /Write-RouterManagedStateFile -StateFile \$stateFile/);
  assert.match(startPs1, /Remove-RouterManagedStateFile -StateFile \$stateFile/);
  assert.match(stopDevPs1, /\$stateFile = Join-Path \$devHome 'run\\start-workspace\.state\.env'/);
  assert.match(stopDevPs1, /Get-RouterManagedProcessId -PidFile \$pidFile -StateFile \$stateFile/);
  assert.match(stopDevPs1, /Remove-RouterManagedStateFile -StateFile \$stateFile/);
  assert.match(stopPs1, /\$stateFile = Join-Path \$runDirectory 'router-product-service\.state\.env'/);
  assert.match(stopPs1, /Get-RouterManagedProcessId -PidFile \$pidFile -StateFile \$stateFile/);
  assert.match(stopPs1, /Remove-RouterManagedStateFile -StateFile \$stateFile/);

  assert.match(startDevSh, /^STATE_FILE="\$RUN_DIR\/start-workspace\.state\.env"$/m);
  assert.match(startDevSh, /router_get_running_pid "\$PID_FILE" "\$STATE_FILE"/);
  assert.match(startDevSh, /router_read_managed_state "\$STATE_FILE"/);
  assert.match(startDevSh, /router_write_managed_state "\$STATE_FILE"/);
  assert.match(startDevSh, /router_remove_managed_state "\$STATE_FILE"/);
  assert.match(startSh, /^STATE_FILE="\$RUN_DIR\/router-product-service\.state\.env"$/m);
  assert.match(startSh, /router_get_running_pid "\$PID_FILE" "\$STATE_FILE"/);
  assert.match(startSh, /router_read_managed_state "\$STATE_FILE"/);
  assert.match(startSh, /router_write_managed_state "\$STATE_FILE"/);
  assert.match(startSh, /router_remove_managed_state "\$STATE_FILE"/);
  assert.match(stopDevSh, /^STATE_FILE="\$DEV_HOME\/run\/start-workspace\.state\.env"$/m);
  assert.match(stopDevSh, /router_get_running_pid "\$PID_FILE" "\$STATE_FILE"/);
  assert.match(stopDevSh, /router_remove_managed_state "\$STATE_FILE"/);
  assert.match(stopSh, /^STATE_FILE="\$RUN_DIR\/router-product-service\.state\.env"$/m);
  assert.match(stopSh, /router_get_running_pid "\$PID_FILE" "\$STATE_FILE"/);
  assert.match(stopSh, /router_remove_managed_state "\$STATE_FILE"/);
});

test('PowerShell managed state validation rejects stale pid reuse when the stored process fingerprint does not match', { skip: !canSpawnPowerShellFromNode() }, () => {
  const tempRoot = createTempDir('managed-state-');
  const pidFile = path.join(tempRoot, 'runtime.pid');
  const stateFile = path.join(tempRoot, 'runtime.state.env');
  const commonScript = quoteForPowerShellSingleQuotedString(
    path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'),
  );
  const pidFilePs = quoteForPowerShellSingleQuotedString(pidFile);
  const stateFilePs = quoteForPowerShellSingleQuotedString(stateFile);

  try {
    const result = runPowerShellCommand(
      [
        `. '${commonScript}'`,
        `$pidFile = '${pidFilePs}'`,
        `$stateFile = '${stateFilePs}'`,
        'Set-Content -Path $pidFile -Value $PID -Encoding utf8',
        '$fingerprint = Get-RouterProcessFingerprint -ProcessId $PID',
        "Write-RouterManagedStateFile -StateFile $stateFile -ProcessId $PID -ProcessFingerprint $fingerprint -Mode 'development preview' -WebBind '127.0.0.1:9983' -GatewayBind '127.0.0.1:9980' -AdminBind '127.0.0.1:9981' -PortalBind '127.0.0.1:9982' -UnifiedAccessEnabled $true -AdminAppUrl 'http://127.0.0.1:9983/admin/' -PortalAppUrl 'http://127.0.0.1:9983/portal/'",
        '$state = Get-RouterManagedState -StateFile $stateFile',
        '$resolvedPid = Get-RouterManagedProcessId -PidFile $pidFile -StateFile $stateFile',
        'Write-Output (\"resolved=\" + $resolvedPid)',
        'Write-Output (\"mode=\" + $state.Mode)',
        "Set-Content -Path $stateFile -Value @('SDKWORK_ROUTER_MANAGED_PID=' + $PID, 'SDKWORK_ROUTER_PROCESS_FINGERPRINT=stale-fingerprint') -Encoding utf8",
        '$stalePid = Get-RouterManagedProcessId -PidFile $pidFile -StateFile $stateFile',
        'Write-Output (\"stale=\" + $stalePid)',
      ].join('\n'),
    );
    const output = `${result.stdout}${result.stderr}`;

    assert.equal(result.status, 0, output);
    assert.match(output, /resolved=\d+/);
    assert.match(output, /mode=development preview/);
    assert.match(output, /stale=0/);
  } finally {
    removeTempRuntimeHome(tempRoot);
  }
});

test('PowerShell managed state writer persists empty app URLs instead of failing parameter binding', { skip: !canSpawnPowerShellFromNode() }, () => {
  const tempRoot = createTempDir('managed-state-empty-urls-');
  const stateFile = path.join(tempRoot, 'runtime.state.env');
  const commonScript = quoteForPowerShellSingleQuotedString(
    path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'),
  );
  const stateFilePs = quoteForPowerShellSingleQuotedString(stateFile);

  try {
    const result = runPowerShellCommand(
      [
        `. '${commonScript}'`,
        `$stateFile = '${stateFilePs}'`,
        "Write-RouterManagedStateFile -StateFile $stateFile -ProcessId $PID -ProcessFingerprint '' -Mode 'production release' -WebBind '127.0.0.1:9983' -GatewayBind '127.0.0.1:9980' -AdminBind '127.0.0.1:9981' -PortalBind '127.0.0.1:9982' -UnifiedAccessEnabled $true -AdminAppUrl '' -PortalAppUrl ''",
        'Get-Content $stateFile',
      ].join('\n'),
    );
    const output = `${result.stdout}${result.stderr}`;

    assert.equal(result.status, 0, output);
    assert.match(output, /SDKWORK_ROUTER_PROCESS_FINGERPRINT=""/);
    assert.match(output, /SDKWORK_ROUTER_ADMIN_APP_URL=""/);
    assert.match(output, /SDKWORK_ROUTER_PORTAL_APP_URL=""/);
  } finally {
    removeTempRuntimeHome(tempRoot);
  }
});

test('start-dev.ps1 derives distinct Windows cargo target dirs for different managed dev homes', { skip: !canSpawnPowerShellFromNode() }, () => {
  const runtimeHomeA = createTempRuntimeHome('start-dev-cargo-a-');
  const runtimeHomeB = createTempRuntimeHome('start-dev-cargo-b-');
  const managedShortTargetRoot = path.join(path.parse(repoRoot).root, 'sdkrt');
  const defaultTargetDir = path.join(repoRoot, 'bin', '.sdkwork-target-vs2022');

  try {
    const runStartDevDryRun = (runtimeHome) => spawnSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(repoRoot, 'bin', 'start-dev.ps1'),
        '-DryRun',
        '-Preview',
        '-WaitSeconds',
        '5',
      ],
      {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
          ...process.env,
          SDKWORK_ROUTER_DEV_HOME: runtimeHome,
        },
      },
    );

    const resultA = runStartDevDryRun(runtimeHomeA);
    const outputA = `${resultA.stdout}${resultA.stderr}`;
    assert.equal(resultA.status, 0, outputA);

    const resultB = runStartDevDryRun(runtimeHomeB);
    const outputB = `${resultB.stdout}${resultB.stderr}`;
    assert.equal(resultB.status, 0, outputB);

    const cargoTargetDirA = outputA.match(/CARGO_TARGET_DIR=([^\r\n]+)/)?.[1]?.trim();
    const cargoTargetDirB = outputB.match(/CARGO_TARGET_DIR=([^\r\n]+)/)?.[1]?.trim();

    assert.ok(cargoTargetDirA, outputA);
    assert.ok(cargoTargetDirB, outputB);
    assert.notEqual(cargoTargetDirA, defaultTargetDir);
    assert.notEqual(cargoTargetDirB, defaultTargetDir);
    assert.equal(path.dirname(cargoTargetDirA), managedShortTargetRoot);
    assert.equal(path.dirname(cargoTargetDirB), managedShortTargetRoot);
    assert.ok(cargoTargetDirA.length < defaultTargetDir.length, outputA);
    assert.ok(cargoTargetDirB.length < defaultTargetDir.length, outputB);
    assert.notEqual(cargoTargetDirA, cargoTargetDirB);
  } finally {
    removeTempRuntimeHome(runtimeHomeA);
    removeTempRuntimeHome(runtimeHomeB);
  }
});

test('start-dev.ps1 falls back to a repo-local managed Windows cargo target dir when the preferred root is unusable', { skip: !canSpawnPowerShellFromNode() }, () => {
  const runtimeHome = createTempRuntimeHome('start-dev-cargo-root-fallback-');
  const invalidRootDir = createTempDir('start-dev-cargo-root-unusable-');
  const invalidRootFile = path.join(invalidRootDir, 'managed-cargo-root.txt');
  const defaultTargetDir = path.join(repoRoot, 'bin', '.sdkwork-target-vs2022');

  writeFileSync(invalidRootFile, 'not-a-directory', 'utf8');

  try {
    const result = spawnSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(repoRoot, 'bin', 'start-dev.ps1'),
        '-DryRun',
        '-Preview',
        '-WaitSeconds',
        '5',
      ],
      {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
          ...process.env,
          SDKWORK_ROUTER_DEV_HOME: runtimeHome,
          SDKWORK_ROUTER_MANAGED_CARGO_ROOT: invalidRootFile,
        },
      },
    );

    const output = `${result.stdout}${result.stderr}`;
    assert.equal(result.status, 0, output);
    assert.match(output, /managed cargo target dir is unavailable or busy; using fallback/i);

    const cargoTargetDir = output.match(/CARGO_TARGET_DIR=([^\r\n]+)/)?.[1]?.trim();
    assert.ok(cargoTargetDir, output);
    assert.notEqual(cargoTargetDir, defaultTargetDir);
    assert.equal(path.dirname(cargoTargetDir), path.join(repoRoot, 'bin'));
    assert.match(path.basename(cargoTargetDir), /^\.sdkrt-[0-9a-f]{12}(?:-r\d+)?$/i);
  } finally {
    removeTempRuntimeHome(runtimeHome);
    rmSync(invalidRootDir, { recursive: true, force: true });
  }
});

test('start-dev.ps1 falls back to an alternate managed Windows cargo target dir when the preferred target is locked', { skip: !canSpawnPowerShellFromNode() }, async () => {
  const runtimeHome = createTempRuntimeHome('start-dev-cargo-lock-');
  let lockHolder = null;

  const runStartDevDryRun = () => spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(repoRoot, 'bin', 'start-dev.ps1'),
      '-DryRun',
      '-Preview',
      '-WaitSeconds',
      '5',
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
      env: {
        ...process.env,
        SDKWORK_ROUTER_DEV_HOME: runtimeHome,
      },
    },
  );

  try {
    const initialResult = runStartDevDryRun();
    const initialOutput = `${initialResult.stdout}${initialResult.stderr}`;
    assert.equal(initialResult.status, 0, initialOutput);

    const preferredTargetDir = initialOutput.match(/CARGO_TARGET_DIR=([^\r\n]+)/)?.[1]?.trim();
    assert.ok(preferredTargetDir, initialOutput);

    const lockFile = path.join(preferredTargetDir, 'debug', '.cargo-lock');
    const lockFilePs = quoteForPowerShellSingleQuotedString(lockFile);
    lockHolder = spawn(
      'powershell.exe',
      [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-Command',
        [
          `$lockFile = '${lockFilePs}'`,
          '[System.IO.Directory]::CreateDirectory((Split-Path $lockFile -Parent)) | Out-Null',
          '$lockHandle = [System.IO.File]::Open($lockFile, [System.IO.FileMode]::OpenOrCreate, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None)',
          'Write-Output "LOCKED"',
          'try { Start-Sleep -Seconds 30 } finally { $lockHandle.Dispose() }',
        ].join('\n'),
      ],
      {
        cwd: repoRoot,
        stdio: ['ignore', 'pipe', 'pipe'],
      },
    );

    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('timed out waiting for lock holder readiness')), 10000);
      lockHolder.once('error', (error) => {
        clearTimeout(timeout);
        reject(error);
      });
      lockHolder.stdout.on('data', (chunk) => {
        if (String(chunk).includes('LOCKED')) {
          clearTimeout(timeout);
          resolve();
        }
      });
      lockHolder.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`lock holder exited before readiness with code ${code ?? 'unknown'}`));
      });
    });

    const fallbackResult = runStartDevDryRun();
    const fallbackOutput = `${fallbackResult.stdout}${fallbackResult.stderr}`;
    assert.equal(fallbackResult.status, 0, fallbackOutput);
    assert.match(fallbackOutput, /managed cargo target dir is unavailable or busy; using fallback/i);

    const fallbackTargetDir = fallbackOutput.match(/CARGO_TARGET_DIR=([^\r\n]+)/)?.[1]?.trim();
    assert.ok(fallbackTargetDir, fallbackOutput);
    assert.notEqual(fallbackTargetDir, preferredTargetDir);
    assert.match(
      fallbackTargetDir,
      new RegExp(`^${escapeRegExp(preferredTargetDir)}-r\\d+$`, 'i'),
    );
  } finally {
    if (lockHolder && lockHolder.exitCode === null) {
      spawnSync('taskkill.exe', ['/PID', String(lockHolder.pid), '/T', '/F'], {
        cwd: repoRoot,
        stdio: 'ignore',
      });
    }
    removeTempRuntimeHome(runtimeHome);
  }
});

test('runtime-common.ps1 accepts pnpm virtualStoreDir entries from the current quoted modules metadata format', { skip: !canSpawnPowerShellFromNode() }, () => {
  const tempRoot = createTempDir('pnpm-virtual-store-');
  const nodeModulesPath = path.join(tempRoot, 'node_modules');
  const virtualStoreDir = path.join(nodeModulesPath, '.pnpm');
  const modulesFile = path.join(nodeModulesPath, '.modules.yaml');
  const commonScript = quoteForPowerShellSingleQuotedString(
    path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'),
  );
  const nodeModulesPathPs = quoteForPowerShellSingleQuotedString(nodeModulesPath);
  const virtualStoreDirPs = quoteForPowerShellSingleQuotedString(virtualStoreDir);

  mkdirSync(virtualStoreDir, { recursive: true });
  writeFileSync(
    modulesFile,
    `{\n  "virtualStoreDir": "${virtualStoreDir.replaceAll('\\', '\\\\')}"\n}\n`,
    'utf8',
  );

  try {
    const result = runPowerShellCommand(
      [
        `. '${commonScript}'`,
        `$nodeModulesPath = '${nodeModulesPathPs}'`,
        '$virtualStore = Get-RouterPnpmVirtualStoreDir -NodeModulesPath $nodeModulesPath',
        '$healthy = Test-RouterPnpmNodeModulesHealthy -NodeModulesPath $nodeModulesPath',
        'Write-Output ("virtual=" + $virtualStore)',
        'Write-Output ("healthy=" + $healthy)',
      ].join('\n'),
    );
    const output = `${result.stdout}${result.stderr}`;

    assert.equal(result.status, 0, output);
    assert.match(output, new RegExp(`virtual=${virtualStoreDirPs.replaceAll('\\', '\\\\')}`));
    assert.match(output, /healthy=True/);
  } finally {
    removeTempRuntimeHome(tempRoot);
  }
});

test('shell start scripts warn when background services are launched from non-interactive WSL sessions', () => {
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');

  assert.match(commonSh, /router_is_wsl\(\)/);
  assert.match(commonSh, /router_is_interactive_shell\(\)/);
  assert.match(commonSh, /router_warn_wsl_background_session\(\)/);
  assert.match(commonSh, /Background services launched from one-shot wsl\.exe commands may stop when the session exits/);
  assert.match(startDevSh, /router_warn_wsl_background_session "development workspace"/);
  assert.match(startSh, /router_warn_wsl_background_session "production runtime"/);
});

test('start-dev.sh stretches the default readiness timeout for WSL launches from Windows-mounted worktrees', () => {
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');

  assert.match(commonSh, /router_is_wsl_windows_mount_path\(\)/);
  assert.match(startDevSh, /WAIT_SECONDS_OVERRIDDEN=0/);
  assert.match(startDevSh, /WAIT_SECONDS_OVERRIDDEN=1/);
  assert.match(startDevSh, /router_is_wsl_windows_mount_path "\$REPO_ROOT"/);
  assert.match(startDevSh, /WAIT_SECONDS=1800/);
  assert.match(startDevSh, /extending readiness timeout to \$\{WAIT_SECONDS\} seconds to accommodate frontend reinstalls/);
});

test('start.ps1 dry-run falls back to host-local release paths when router.env carries unix-style values', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, () => {
  const runtimeHome = createTempRuntimeHome('start-ps1-');

  try {
    mkdirSync(path.join(runtimeHome, 'config'), { recursive: true });
    writeFileSync(
      path.join(runtimeHome, 'config', 'router.env'),
      [
        'SDKWORK_CONFIG_DIR="/tmp/router/config"',
        'SDKWORK_DATABASE_URL="sqlite:///tmp/router/data/router.db"',
        'SDKWORK_WEB_BIND="0.0.0.0:19483"',
        'SDKWORK_ADMIN_SITE_DIR="/tmp/router/admin"',
        'SDKWORK_PORTAL_SITE_DIR="/tmp/router/portal"',
        'SDKWORK_ROUTER_BINARY="/tmp/router/bin/router-product-service"',
        '',
      ].join('\n'),
      'utf8',
    );

    const result = runPowerShellStartDryRun(runtimeHome);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const runtimeHomePortable = toPortablePath(runtimeHome);
    const plan = JSON.parse(result.stdout.trim());
    assert.equal(plan.mode, 'dry-run');
    assert.equal(plan.plan_format, 'json');
    assert.equal(plan.public_web_bind, '0.0.0.0:19483');
    assert.equal(plan.config_dir, `${runtimeHomePortable}/config`);
    assert.equal(plan.database_url, `sqlite://${runtimeHomePortable}/var/data/sdkwork-api-router.db`);
    assert.equal(plan.site_dirs.admin, `${runtimeHomePortable}/sites/admin/dist`);
    assert.equal(plan.site_dirs.portal, `${runtimeHomePortable}/sites/portal/dist`);
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('start.ps1 dry-run accepts a missing relative -Home path without double-nesting runtime files', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, () => {
  const relativeRuntimeHome = path.join('artifacts', 'test-runtime', `start-ps1-relative-${Date.now()}`);
  const absoluteRuntimeHome = path.join(repoRoot, relativeRuntimeHome);

  try {
    const result = runPowerShellStartDryRun(relativeRuntimeHome);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const plan = JSON.parse(result.stdout.trim());
    const runtimeHomePortable = toPortablePath(absoluteRuntimeHome);

    assert.equal(plan.mode, 'dry-run');
    assert.equal(plan.plan_format, 'json');
    assert.equal(plan.public_web_bind, '0.0.0.0:3001');
    assert.equal(plan.config_dir, `${runtimeHomePortable}/config`);
    assert.equal(plan.database_url, `sqlite://${runtimeHomePortable}/var/data/sdkwork-api-router.db`);
  } finally {
    removeTempRuntimeHome(absoluteRuntimeHome);
  }
});

test('start.ps1 dry-run loads system install config from the release manifest instead of portable SQLite defaults', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, () => {
  const runtimeHome = createTempRuntimeHome('start-ps1-system-');
  const systemConfigDir = path.join(runtimeHome, 'system-config');
  const systemDataDir = path.join(runtimeHome, 'system-data');
  const systemLogDir = path.join(runtimeHome, 'system-log');
  const systemRunDir = path.join(runtimeHome, 'system-run');
  const routerYamlPath = path.join(systemConfigDir, 'router.yaml');
  const runtimeHomePortable = toPortablePath(runtimeHome);
  const systemConfigDirPortable = toPortablePath(systemConfigDir);
  const routerYamlPortable = toPortablePath(routerYamlPath);

  try {
    mkdirSync(systemConfigDir, { recursive: true });
    writeFileSync(
      path.join(runtimeHome, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'system',
        configRoot: systemConfigDirPortable,
        configFile: routerYamlPortable,
        mutableDataRoot: toPortablePath(systemDataDir),
        logRoot: toPortablePath(systemLogDir),
        runRoot: toPortablePath(systemRunDir),
      }, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(
      path.join(systemConfigDir, 'router.env'),
      [
        'SDKWORK_ROUTER_INSTALL_MODE="system"',
        `SDKWORK_CONFIG_DIR="${systemConfigDirPortable}"`,
        `SDKWORK_CONFIG_FILE="${routerYamlPortable}"`,
        'SDKWORK_DATABASE_URL="postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router"',
        '',
      ].join('\n'),
      'utf8',
    );

    const result = runPowerShellStartDryRun(runtimeHome);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const plan = JSON.parse(result.stdout.trim());
    assert.equal(plan.mode, 'dry-run');
    assert.equal(plan.plan_format, 'json');
    assert.equal(plan.config_dir, systemConfigDirPortable);
    assert.equal(plan.config_file, routerYamlPortable);
    assert.equal(plan.database_url, 'postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router');
    assert.notEqual(plan.config_dir, `${runtimeHomePortable}/config`);
    assert.notEqual(plan.database_url, `sqlite://${runtimeHomePortable}/var/data/sdkwork-api-router.db`);
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('start.ps1 dry-run executes the versioned release payload referenced by a current-home manifest', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, () => {
  const productRoot = createTempRuntimeHome('start-ps1-current-layout-');
  const currentRoot = path.join(productRoot, 'current');
  const releaseRoot = path.join(productRoot, 'releases', '0.1.0');
  const releaseBinDir = path.join(releaseRoot, 'bin');
  const adminSiteDir = path.join(releaseRoot, 'sites', 'admin', 'dist');
  const portalSiteDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const configDir = path.join(productRoot, 'config');
  const dataDir = path.join(productRoot, 'data');
  const logDir = path.join(productRoot, 'log');
  const runDir = path.join(productRoot, 'run');
  const routerBinaryPath = path.join(releaseBinDir, 'router-product-service.cmd');

  try {
    mkdirSync(currentRoot, { recursive: true });
    mkdirSync(releaseBinDir, { recursive: true });
    mkdirSync(adminSiteDir, { recursive: true });
    mkdirSync(portalSiteDir, { recursive: true });
    mkdirSync(configDir, { recursive: true });
    mkdirSync(dataDir, { recursive: true });
    mkdirSync(logDir, { recursive: true });
    mkdirSync(runDir, { recursive: true });

    writeFileSync(
      path.join(configDir, 'router.env'),
      [
        `SDKWORK_CONFIG_DIR="${toPortablePath(configDir)}"`,
        `SDKWORK_CONFIG_FILE="${toPortablePath(path.join(configDir, 'router.yaml'))}"`,
        '',
      ].join('\n'),
      'utf8',
    );
    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'portable',
        productRoot: toPortablePath(productRoot),
        controlRoot: toPortablePath(currentRoot),
        releaseVersion: '0.1.0',
        releaseRoot: toPortablePath(releaseRoot),
        routerBinary: toPortablePath(routerBinaryPath),
        adminSiteDistDir: toPortablePath(adminSiteDir),
        portalSiteDistDir: toPortablePath(portalSiteDir),
        configRoot: toPortablePath(configDir),
        configFile: toPortablePath(path.join(configDir, 'router.yaml')),
        mutableDataRoot: toPortablePath(dataDir),
        logRoot: toPortablePath(logDir),
        runRoot: toPortablePath(runDir),
      }, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(
      routerBinaryPath,
      [
        '@echo off',
        'if "%1"=="--dry-run" echo {"mode":"dry-run","plan_format":"json","public_web_bind":"127.0.0.1:29101","database_url":"sqlite:///from-versioned-binary.db","config_dir":"/from-versioned-binary","config_file":null,"node_id_prefix":null,"binds":{"gateway":"127.0.0.1:29102","admin":"127.0.0.1:29103","portal":"127.0.0.1:29104"},"site_dirs":{"admin":"/from-versioned-admin","portal":"/from-versioned-portal"},"upstreams":{"gateway":null,"admin":null,"portal":null}}',
        'exit /b 0',
        '',
      ].join('\r\n'),
      'utf8',
    );

    const result = runPowerShellStartDryRun(currentRoot);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const plan = JSON.parse(result.stdout.trim());
    assert.equal(plan.config_dir, '/from-versioned-binary');
    assert.equal(plan.site_dirs.admin, '/from-versioned-admin');
    assert.equal(plan.site_dirs.portal, '/from-versioned-portal');
    assert.equal(plan.database_url, 'sqlite:///from-versioned-binary.db');
  } finally {
    removeTempRuntimeHome(productRoot);
  }
});

test('start.sh dry-run falls back to host-local release paths when router.env carries windows-style values', { skip: process.platform !== 'win32' || !hasWslDistro('Ubuntu-22.04') }, () => {
  const runtimeHome = createTempRuntimeHome('start-sh-');

  try {
    mkdirSync(path.join(runtimeHome, 'config'), { recursive: true });
    writeFileSync(
      path.join(runtimeHome, 'config', 'router.env'),
      [
        'SDKWORK_CONFIG_DIR="D:/router/config"',
        'SDKWORK_DATABASE_URL="sqlite://D:/router/data/router.db"',
        'SDKWORK_WEB_BIND="0.0.0.0:19484"',
        'SDKWORK_ADMIN_SITE_DIR="D:/router/admin"',
        'SDKWORK_PORTAL_SITE_DIR="D:/router/portal"',
        'SDKWORK_ROUTER_BINARY="D:/router/bin/router-product-service.exe"',
        '',
      ].join('\n'),
      'utf8',
    );

    const result = runWslStartDryRun(runtimeHome);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const runtimeHomeWsl = toWslPath(runtimeHome);
    const plan = JSON.parse(result.stdout.trim());
    assert.equal(plan.mode, 'dry-run');
    assert.equal(plan.plan_format, 'json');
    assert.equal(plan.public_web_bind, '0.0.0.0:19484');
    assert.equal(plan.config_dir, `${runtimeHomeWsl}/config`);
    assert.equal(plan.database_url, `sqlite://${runtimeHomeWsl}/var/data/sdkwork-api-router.db`);
    assert.equal(plan.site_dirs.admin, `${runtimeHomeWsl}/sites/admin/dist`);
    assert.equal(plan.site_dirs.portal, `${runtimeHomeWsl}/sites/portal/dist`);
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('start.sh dry-run executes the versioned release payload referenced by a current-home manifest', { skip: process.platform !== 'win32' || !hasWslDistro('Ubuntu-22.04') }, () => {
  const productRoot = createTempRuntimeHome('start-sh-current-layout-');
  const currentRoot = path.join(productRoot, 'current');
  const releaseRoot = path.join(productRoot, 'releases', '0.1.0');
  const releaseBinDir = path.join(releaseRoot, 'bin');
  const adminSiteDir = path.join(releaseRoot, 'sites', 'admin', 'dist');
  const portalSiteDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const configDir = path.join(productRoot, 'config');
  const dataDir = path.join(productRoot, 'data');
  const logDir = path.join(productRoot, 'log');
  const runDir = path.join(productRoot, 'run');
  const routerBinaryPath = path.join(releaseBinDir, 'router-product-service');

  try {
    mkdirSync(currentRoot, { recursive: true });
    mkdirSync(releaseBinDir, { recursive: true });
    mkdirSync(adminSiteDir, { recursive: true });
    mkdirSync(portalSiteDir, { recursive: true });
    mkdirSync(configDir, { recursive: true });
    mkdirSync(dataDir, { recursive: true });
    mkdirSync(logDir, { recursive: true });
    mkdirSync(runDir, { recursive: true });

    writeFileSync(
      path.join(configDir, 'router.env'),
      [
        `SDKWORK_CONFIG_DIR="${toWslPath(configDir)}"`,
        `SDKWORK_CONFIG_FILE="${toWslPath(path.join(configDir, 'router.yaml'))}"`,
        '',
      ].join('\n'),
      'utf8',
    );
    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'portable',
        productRoot: toWslPath(productRoot),
        controlRoot: toWslPath(currentRoot),
        releaseVersion: '0.1.0',
        releaseRoot: toWslPath(releaseRoot),
        routerBinary: toWslPath(routerBinaryPath),
        adminSiteDistDir: toWslPath(adminSiteDir),
        portalSiteDistDir: toWslPath(portalSiteDir),
        configRoot: toWslPath(configDir),
        configFile: toWslPath(path.join(configDir, 'router.yaml')),
        mutableDataRoot: toWslPath(dataDir),
        logRoot: toWslPath(logDir),
        runRoot: toWslPath(runDir),
      }, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(
      routerBinaryPath,
      [
        '#!/usr/bin/env sh',
        'if [ "$#" -ge 2 ] && [ "$1" = "--dry-run" ] && [ "$2" = "--plan-format" ]; then',
        '  printf \'%s\\n\' \'{"mode":"dry-run","plan_format":"json","public_web_bind":"127.0.0.1:29201","database_url":"sqlite:///from-versioned-binary.db","config_dir":"/from-versioned-binary","config_file":null,"node_id_prefix":null,"binds":{"gateway":"127.0.0.1:29202","admin":"127.0.0.1:29203","portal":"127.0.0.1:29204"},"site_dirs":{"admin":"/from-versioned-admin","portal":"/from-versioned-portal"},"upstreams":{"gateway":null,"admin":null,"portal":null}}\'',
        '  exit 0',
        'fi',
        'exit 1',
        '',
      ].join('\n'),
      'utf8',
    );
    chmodSync(routerBinaryPath, 0o755);

    const result = runWslStartDryRun(currentRoot);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const plan = JSON.parse(result.stdout.trim());
    assert.equal(plan.config_dir, '/from-versioned-binary');
    assert.equal(plan.site_dirs.admin, '/from-versioned-admin');
    assert.equal(plan.site_dirs.portal, '/from-versioned-portal');
    assert.equal(plan.database_url, 'sqlite:///from-versioned-binary.db');
  } finally {
    removeTempRuntimeHome(productRoot);
  }
});

test('start.sh dry-run loads system install config from the release manifest instead of portable SQLite defaults', { skip: process.platform !== 'win32' || !hasWslDistro('Ubuntu-22.04') }, () => {
  const runtimeHome = createTempRuntimeHome('start-sh-system-');
  const systemConfigDir = path.join(runtimeHome, 'system-config');
  const systemDataDir = path.join(runtimeHome, 'system-data');
  const systemLogDir = path.join(runtimeHome, 'system-log');
  const systemRunDir = path.join(runtimeHome, 'system-run');
  const routerYamlPath = path.join(systemConfigDir, 'router.yaml');
  const runtimeHomeWsl = toWslPath(runtimeHome);
  const systemConfigDirWsl = toWslPath(systemConfigDir);
  const routerYamlWsl = toWslPath(routerYamlPath);

  try {
    mkdirSync(systemConfigDir, { recursive: true });
    writeFileSync(
      path.join(runtimeHome, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'system',
        configRoot: systemConfigDirWsl,
        configFile: routerYamlWsl,
        mutableDataRoot: toWslPath(systemDataDir),
        logRoot: toWslPath(systemLogDir),
        runRoot: toWslPath(systemRunDir),
      }, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(
      path.join(systemConfigDir, 'router.env'),
      [
        'SDKWORK_ROUTER_INSTALL_MODE="system"',
        `SDKWORK_CONFIG_DIR="${systemConfigDirWsl}"`,
        `SDKWORK_CONFIG_FILE="${routerYamlWsl}"`,
        'SDKWORK_DATABASE_URL="postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router"',
        '',
      ].join('\n'),
      'utf8',
    );

    const result = runWslStartDryRun(runtimeHome);
    assert.equal(result.status, 0, result.stderr || result.stdout);

    const plan = JSON.parse(result.stdout.trim());
    assert.equal(plan.mode, 'dry-run');
    assert.equal(plan.plan_format, 'json');
    assert.equal(plan.config_dir, systemConfigDirWsl);
    assert.equal(plan.config_file, routerYamlWsl);
    assert.equal(plan.database_url, 'postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router');
    assert.notEqual(plan.config_dir, `${runtimeHomeWsl}/config`);
    assert.notEqual(plan.database_url, `sqlite://${runtimeHomeWsl}/var/data/sdkwork-api-router.db`);
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('stop.ps1 dry-run resolves the system run directory from the release manifest', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, () => {
  const runtimeHome = createTempRuntimeHome('stop-ps1-system-');
  const systemRunDir = path.join(runtimeHome, 'system-run');

  try {
    writeFileSync(
      path.join(runtimeHome, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'system',
        runRoot: toPortablePath(systemRunDir),
      }, null, 2)}\n`,
      'utf8',
    );

    const result = runPowerShellStopDryRun(runtimeHome);
    const output = `${result.stdout}${result.stderr}`;
    assert.equal(result.status, 0, output);
    assert.match(
      output,
      new RegExp(`would stop router-product-service using pid file ${escapeRegExp(path.join(systemRunDir, 'router-product-service.pid'))}`),
    );
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('stop.sh dry-run resolves the system run directory from the release manifest', { skip: process.platform !== 'win32' || !hasWslDistro('Ubuntu-22.04') }, () => {
  const runtimeHome = createTempRuntimeHome('stop-sh-system-');
  const systemRunDir = path.join(runtimeHome, 'system-run');
  const systemRunDirWsl = toWslPath(systemRunDir);

  try {
    writeFileSync(
      path.join(runtimeHome, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'system',
        runRoot: systemRunDirWsl,
      }, null, 2)}\n`,
      'utf8',
    );

    const result = runWslStopDryRun(runtimeHome);
    const output = `${result.stdout}${result.stderr}`;
    assert.equal(result.status, 0, output);
    assert.match(
      output,
      new RegExp(`would stop router-product-service using pid file ${escapeRegExp(`${systemRunDirWsl}/router-product-service.pid`)}`),
    );
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('unix release entrypoints normalize relative runtime homes before changing directories', () => {
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');
  const stopSh = readFileSync(path.join(repoRoot, 'bin', 'stop.sh'), 'utf8');

  assert.match(commonSh, /router_resolve_absolute_path\(\)/);
  assert.match(startSh, /RUNTIME_HOME=\$\(router_resolve_absolute_path "\$PWD" "\$RUNTIME_HOME"\)/);
  assert.match(stopSh, /RUNTIME_HOME=\$\(router_resolve_absolute_path "\$PWD" "\$RUNTIME_HOME"\)/);
});

test('unix runtime entrypoints default to the installed home beside the packaged scripts when binaries are colocated', () => {
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');
  const stopSh = readFileSync(path.join(repoRoot, 'bin', 'stop.sh'), 'utf8');

  assert.match(startSh, /if \[ -f "\$SCRIPT_DIR\/\$\(router_binary_name router-product-service\)" \]; then[\s\S]*RUNTIME_HOME=\$\(CDPATH= cd -- "\$SCRIPT_DIR\/\.\." && pwd\)/);
  assert.match(stopSh, /if \[ -f "\$SCRIPT_DIR\/\$\(router_binary_name router-product-service\)" \]; then[\s\S]*RUNTIME_HOME=\$\(CDPATH= cd -- "\$SCRIPT_DIR\/\.\." && pwd\)/);
});

test('installed current-home wrappers also recognize a sibling release manifest as the default runtime home', () => {
  const startPs1 = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const stopPs1 = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');
  const stopSh = readFileSync(path.join(repoRoot, 'bin', 'stop.sh'), 'utf8');

  assert.match(startSh, /if \[ -f "\$SCRIPT_DIR\/\.\.\/release-manifest\.json" \]; then[\s\S]*RUNTIME_HOME=\$\(CDPATH= cd -- "\$SCRIPT_DIR\/\.\." && pwd\)/);
  assert.match(stopSh, /if \[ -f "\$SCRIPT_DIR\/\.\.\/release-manifest\.json" \]; then[\s\S]*RUNTIME_HOME=\$\(CDPATH= cd -- "\$SCRIPT_DIR\/\.\." && pwd\)/);
  assert.match(startPs1, /\$manifestHome = Split-Path -Parent \$scriptDir[\s\S]*Test-Path \(Join-Path \$manifestHome 'release-manifest\.json'\)[\s\S]*\$RuntimeHome = \$manifestHome/);
  assert.match(stopPs1, /\$manifestHome = Split-Path -Parent \$scriptDir[\s\S]*Test-Path \(Join-Path \$manifestHome 'release-manifest\.json'\)[\s\S]*\$RuntimeHome = \$manifestHome/);
});

test('installed unix runtime start.sh and stop.sh manage an installed home end-to-end', { skip: !canSpawnUnixShellFromNode() }, () => {
  const runtimeHome = createTempRuntimeHome('installed-unix-runtime-');
  const pidFile = path.join(runtimeHome, 'var', 'run', 'router-product-service.pid');
  const stateFile = path.join(runtimeHome, 'var', 'run', 'router-product-service.state.env');
  const runtimeHomeShell = process.platform === 'win32' ? toGitBashPath(runtimeHome) : runtimeHome;
  const { curlLogFile, fakeBinDir } = installUnixRuntimeSmokeFixture(runtimeHome);

  try {
    const env = {
      ...process.env,
      CURL_LOG_FILE: process.platform === 'win32' ? toGitBashPath(curlLogFile) : curlLogFile,
    };
    const startResult = runUnixShellCommand(
      `./bin/start.sh --home ${quoteForBash(runtimeHomeShell)} --wait-seconds 5`,
      {
        cwd: runtimeHome,
        env,
        pathPrefix: [fakeBinDir],
      },
    );
    const startOutput = `${startResult.stdout}${startResult.stderr}`;

    assert.equal(startResult.status, 0, startOutput);
    assert.equal(existsSync(pidFile), true, 'expected installed runtime start to create a pid file');
    assert.equal(existsSync(stateFile), true, 'expected installed runtime start to persist managed state');
    assert.match(startOutput, /started router-product-service \(pid=\d+\)/);
    assert.match(startOutput, /Mode: production release/);

    const curlLog = readFileSync(curlLogFile, 'utf8');
    assert.match(curlLog, /\/api\/v1\/health/);
    assert.match(curlLog, /\/api\/admin\/health/);
    assert.match(curlLog, /\/api\/portal\/health/);

    const stopResult = runUnixShellCommand(
      `./bin/stop.sh --home ${quoteForBash(runtimeHomeShell)} --wait-seconds 5`,
      {
        cwd: runtimeHome,
        env,
        pathPrefix: [fakeBinDir],
      },
    );
    const stopOutput = `${stopResult.stdout}${stopResult.stderr}`;

    assert.equal(stopResult.status, 0, stopOutput);
    assert.equal(existsSync(pidFile), false, 'expected installed runtime stop to remove the pid file');
    assert.equal(existsSync(stateFile), false, 'expected installed runtime stop to remove managed state');
    assert.match(stopOutput, /stopped router-product-service pid=\d+/);
  } finally {
    removeTempRuntimeHome(runtimeHome);
  }
});

test('PowerShell bind preflight reports conflicting listeners before launch', { skip: process.platform !== 'win32' || !canSpawnPowerShellFromNode() }, async () => {
  await withTcpListener(async ({ port }) => {
    const commonScript = quoteForPowerShellSingleQuotedString(
      path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'),
    );
    const result = runPowerShellCommand(
      [
        `. '${commonScript}'`,
        `try { Assert-RouterBindAddressesAvailable -BindAddresses @('127.0.0.1:${port}') -ServiceLabel 'development workspace' } catch { Write-Output $_.Exception.Message; exit 1 }`,
      ].join('\n'),
    );
    const combinedOutput = `${result.stdout}${result.stderr}`;

    assert.equal(result.status, 1, result.stderr || result.stdout);
    assert.match(combinedOutput, /development workspace cannot start because required listen ports are already in use/i);
    assert.match(combinedOutput, new RegExp(`127\\.0\\.0\\.1:${port}`));
  });
});

test('start scripts preflight bind conflicts before background launch and shell helpers expose matching checks', () => {
  const commonPs1 = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');
  const startDevPs1 = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');
  const startPs1 = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const commonSh = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');
  const startDevSh = readFileSync(path.join(repoRoot, 'bin', 'start-dev.sh'), 'utf8');
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');

  assert.match(commonPs1, /function Get-RouterListeningPortConflicts/);
  assert.match(commonPs1, /function Assert-RouterBindAddressesAvailable/);
  assert.match(startDevPs1, /Assert-RouterBindAddressesAvailable/);
  assert.match(startPs1, /Assert-RouterBindAddressesAvailable/);

  assert.match(commonSh, /router_collect_bind_conflicts\(\)/);
  assert.match(commonSh, /router_assert_bind_addresses_available\(\)/);
  assert.match(startDevSh, /router_assert_bind_addresses_available/);
  assert.match(startSh, /router_assert_bind_addresses_available/);
  assert.match(startDevPs1, /\$preflightBindAddresses \+= @\('127\.0\.0\.1:5173', '127\.0\.0\.1:5174'\)/);
  assert.match(startDevSh, /router_assert_bind_addresses_available[\s\S]*127\.0\.0\.1:5173[\s\S]*127\.0\.0\.1:5174/);
});

test('unix bind preflight warnings do not get misclassified as real port conflicts when probe tools are unavailable', { skip: !canSpawnUnixShellFromNode() }, () => {
  const fakeBinDir = createTempDir('bind-preflight-no-tools-');

  try {
    writeFileSync(
      path.join(fakeBinDir, 'uname'),
      [
        '#!/usr/bin/env sh',
        "printf '%s\\n' 'Linux'",
        '',
      ].join('\n'),
      'utf8',
    );
    chmodSync(path.join(fakeBinDir, 'uname'), 0o755);

    const result = runUnixShellCommand(
      [
        `PATH=${quoteForBash(process.platform === 'win32' ? toGitBashPath(fakeBinDir) : fakeBinDir)}`,
        `. ${quoteForBash(process.platform === 'win32' ? toGitBashPath(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh')) : path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'))}`,
        `router_assert_bind_addresses_available 'production runtime' '127.0.0.1:3001'`,
      ].join(' && '),
      {
        cwd: repoRoot,
      },
    );

    const combinedOutput = `${result.stdout}${result.stderr}`;
    assert.equal(result.status, 0, combinedOutput);
    assert.match(combinedOutput, /unable to preflight port conflicts because lsof, ss, and netstat are unavailable/i);
    assert.doesNotMatch(combinedOutput, /cannot start because required listen ports are already in use/i);
  } finally {
    removeTempRuntimeHome(fakeBinDir);
  }
});
