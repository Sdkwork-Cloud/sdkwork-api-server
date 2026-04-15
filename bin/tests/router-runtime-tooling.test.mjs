import assert from 'node:assert/strict';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
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

test('createReleaseBuildPlan builds release binaries, web apps, and native package output', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'linux',
    arch: 'x64',
    installDependencies: false,
    includeDocs: true,
    includeConsole: true,
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
  assert.equal(plan.steps.some((step) => step.label === 'console build'), true);
  assert.equal(plan.steps.some((step) => step.label === 'docs build'), true);
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

test('createReleaseBuildPlan normalizes broken Windows CMake generator defaults for release cargo builds', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
      CMAKE_GENERATOR: 'Visual Studio 18 2026',
    },
    includeDocs: false,
    includeConsole: false,
  });

  assert.equal(plan.steps[0].env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(plan.steps[0].env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(
    plan.steps[0].env.CARGO_TARGET_DIR,
    path.join(repoRoot, 'bin', '.sdkwork-target-vs2022'),
  );
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
    includeConsole: false,
  });

  const jobIndex = plan.steps[0].args.indexOf('-j');
  assert.notEqual(jobIndex, -1, 'expected cargo build to pin an explicit job count');
  assert.equal(plan.steps[0].args[jobIndex + 1], '1');
  assert.equal(plan.steps[0].env.CARGO_BUILD_JOBS, '1');
  assert.equal(
    plan.steps.find((step) => step.label === 'admin desktop release build')?.env.CARGO_BUILD_JOBS,
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
    includeConsole: false,
  });

  const jobIndex = plan.steps[0].args.indexOf('-j');
  assert.notEqual(jobIndex, -1, 'expected cargo build to keep an explicit job count');
  assert.equal(plan.steps[0].args[jobIndex + 1], '4');
  assert.equal(plan.steps[0].env.CARGO_BUILD_JOBS, '4');
});

test('createInstallPlan copies product assets, runtime scripts, and service descriptors into install home', async () => {
  const module = await loadModule();
  const installRoot = path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router', 'current');

  const plan = module.createInstallPlan({
    repoRoot,
    installRoot,
    platform: 'darwin',
  });

  assert.equal(plan.directories.includes(path.join(installRoot, 'bin')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'data')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'sites', 'admin')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'sites', 'portal')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'var', 'log')), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('bin', 'start.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('bin', 'stop.ps1'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'launchd', 'com.sdkwork.api-router.plist'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'systemd', 'sdkwork-api-router.service'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'windows-task', 'sdkwork-api-router.xml'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'systemd', 'install-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'systemd', 'uninstall-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'launchd', 'install-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'launchd', 'uninstall-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'windows-task', 'install-service.ps1'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'windows-task', 'uninstall-service.ps1'))), true);
  assert.equal(
    plan.files.some((file) =>
      file.type === 'directory'
      && file.sourcePath === path.join(repoRoot, 'data')
      && file.targetPath === path.join(installRoot, 'data')),
    true,
  );
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('sites', 'admin', 'dist'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('sites', 'portal', 'dist'))), true);
});

test('createInstallPlan reads release binaries from the managed short Windows target directory when needed', async () => {
  const module = await loadModule();
  const installRoot = path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router', 'current');

  const plan = module.createInstallPlan({
    repoRoot,
    installRoot,
    platform: 'win32',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
  });

  const binaryCopy = plan.files.find((file) => file.targetPath.endsWith(path.join('bin', 'router-product-service.exe')));
  assert.ok(binaryCopy, 'expected router-product-service.exe copy entry');
  assert.equal(
    binaryCopy.sourcePath,
    path.join(repoRoot, 'bin', '.sdkwork-target-vs2022', 'x86_64-pc-windows-msvc', 'release', 'router-product-service.exe'),
  );
});

test('createInstallPlan treats normalized windows platform ids as Windows executable layouts', async () => {
  const module = await loadModule();
  const installRoot = path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router', 'current');

  const plan = module.createInstallPlan({
    repoRoot,
    installRoot,
    platform: 'windows',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
  });

  const binaryCopy = plan.files.find((file) => file.targetPath.endsWith(path.join('bin', 'router-product-service.exe')));
  assert.ok(binaryCopy, 'expected router-product-service.exe copy entry for normalized windows platform ids');
  assert.equal(
    binaryCopy.sourcePath,
    path.join(repoRoot, 'bin', '.sdkwork-target-vs2022', 'x86_64-pc-windows-msvc', 'release', 'router-product-service.exe'),
  );
});

test('renderRuntimeEnvTemplate defaults release runtime to writable local data and 9980-series ports', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork-api-router/current';

  const envFile = module.renderRuntimeEnvTemplate({
    installRoot,
    platform: 'linux',
  });

  assert.match(envFile, /SDKWORK_CONFIG_DIR="\/opt\/sdkwork-api-router\/current\/config"/);
  assert.match(envFile, /SDKWORK_DATABASE_URL="sqlite:\/\/\/opt\/sdkwork-api-router\/current\/var\/data\/sdkwork-api-router\.db"/);
  assert.match(envFile, /SDKWORK_WEB_BIND="0\.0\.0\.0:9983"/);
  assert.match(envFile, /SDKWORK_GATEWAY_BIND="127\.0\.0\.1:9980"/);
  assert.match(envFile, /SDKWORK_ADMIN_BIND="127\.0\.0\.1:9981"/);
  assert.match(envFile, /SDKWORK_PORTAL_BIND="127\.0\.0\.1:9982"/);
  assert.match(envFile, /SDKWORK_ADMIN_SITE_DIR="\/opt\/sdkwork-api-router\/current\/sites\/admin\/dist"/);
  assert.match(envFile, /SDKWORK_PORTAL_SITE_DIR="\/opt\/sdkwork-api-router\/current\/sites\/portal\/dist"/);
});

test('service descriptors start the production runtime in foreground mode from the installed home', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork-api-router/current';

  const systemdUnit = module.renderSystemdUnit({
    installRoot,
    serviceName: 'sdkwork-api-router',
  });
  const launchdPlist = module.renderLaunchdPlist({
    installRoot,
    serviceName: 'com.sdkwork.api-router',
  });
  const windowsTaskXml = module.renderWindowsTaskXml({
    installRoot: 'C:/sdkwork/api-router/current',
    taskName: 'sdkwork-api-router',
  });

  assert.match(systemdUnit, /ExecStart="\/opt\/sdkwork-api-router\/current\/bin\/start\.sh" --foreground --home "\/opt\/sdkwork-api-router\/current"/);
  assert.match(systemdUnit, /EnvironmentFile=-\/opt\/sdkwork-api-router\/current\/config\/router\.env/);
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
  const installRoot = '/opt/sdkwork router/current build';

  const envFile = module.renderRuntimeEnvTemplate({
    installRoot,
    platform: 'linux',
  });
  const systemdUnit = module.renderSystemdUnit({
    installRoot,
    serviceName: 'sdkwork-api-router',
  });

  assert.match(envFile, /^SDKWORK_CONFIG_DIR="\/opt\/sdkwork router\/current build\/config"$/m);
  assert.match(envFile, /^SDKWORK_DATABASE_URL="sqlite:\/\/\/opt\/sdkwork router\/current build\/var\/data\/sdkwork-api-router\.db"$/m);
  assert.match(envFile, /^SDKWORK_ADMIN_SITE_DIR="\/opt\/sdkwork router\/current build\/sites\/admin\/dist"$/m);
  assert.match(envFile, /^SDKWORK_ROUTER_BINARY="\/opt\/sdkwork router\/current build\/bin\/router-product-service"$/m);

  assert.match(systemdUnit, /WorkingDirectory=\/opt\/sdkwork\\ router\/current\\ build/);
  assert.match(systemdUnit, /EnvironmentFile=-\/opt\/sdkwork\\ router\/current\\ build\/config\/router\.env/);
  assert.match(systemdUnit, /ExecStart="\/opt\/sdkwork router\/current build\/bin\/start\.sh" --foreground --home "\/opt\/sdkwork router\/current build"/);
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
  });
});

test('router-ops rejects install-only flags during build', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['build', '--home', 'artifacts/install/custom']),
      /--home is only supported for the install command/,
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
  assert.match(script, /SDKWORK_BOOTSTRAP_PROFILE/);
  assert.match(script, /SDKWORK_BOOTSTRAP_DATA_DIR/);
  assert.match(script, /active bootstrap profile/);
  assert.match(script, /runtime configuration/);
  assert.doesNotMatch(script, /admin@sdkwork\.local/);
  assert.doesNotMatch(script, /portal@sdkwork\.local/);
  assert.doesNotMatch(script, /ChangeMe123!/);
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
  ];
  const powershellWrappers = [
    'build.ps1',
    'install.ps1',
    'start-dev.ps1',
    'start.ps1',
    'stop-dev.ps1',
    'stop.ps1',
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
  assert.match(buildScript, /\^\(\?i\)-SkipConsole\$/);
  assert.match(buildScript, /\^\(\?i\)-Install\$/);
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

test('unix runtime entrypoints default to the installed home beside the packaged scripts when binaries are colocated', () => {
  const startSh = readFileSync(path.join(repoRoot, 'bin', 'start.sh'), 'utf8');
  const stopSh = readFileSync(path.join(repoRoot, 'bin', 'stop.sh'), 'utf8');

  assert.match(startSh, /if \[ -f "\$SCRIPT_DIR\/\$\(router_binary_name router-product-service\)" \]; then[\s\S]*RUNTIME_HOME=\$\(CDPATH= cd -- "\$SCRIPT_DIR\/\.\." && pwd\)/);
  assert.match(stopSh, /if \[ -f "\$SCRIPT_DIR\/\$\(router_binary_name router-product-service\)" \]; then[\s\S]*RUNTIME_HOME=\$\(CDPATH= cd -- "\$SCRIPT_DIR\/\.\." && pwd\)/);
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
