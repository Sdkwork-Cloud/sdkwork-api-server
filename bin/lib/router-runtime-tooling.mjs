#!/usr/bin/env node

import { spawn } from 'node:child_process';
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import { pnpmCommand, resolveRustRunner } from '../../scripts/check-router-product.mjs';
import { resolveDesktopReleaseTarget } from '../../scripts/release/desktop-targets.mjs';
import { withSupportedWindowsCmakeGenerator } from '../../scripts/run-tauri-cli.mjs';
import { resolveWorkspaceTargetDir, withManagedWorkspaceTargetDir } from '../../scripts/workspace-target-dir.mjs';

export const RELEASE_BINARY_NAMES = [
  'admin-api-service',
  'gateway-service',
  'portal-api-service',
  'router-web-service',
  'router-product-service',
];

export const PROD_DEFAULTS = {
  webBind: '0.0.0.0:3001',
  gatewayBind: '127.0.0.1:8080',
  adminBind: '127.0.0.1:8081',
  portalBind: '127.0.0.1:8082',
};

function normalizeRuntimePlatform(platform = process.platform) {
  if (platform === 'windows') {
    return 'win32';
  }
  if (platform === 'macos') {
    return 'darwin';
  }

  return platform;
}

export function toPortablePath(value) {
  return String(value).replaceAll('\\', '/');
}

export function withExecutable(binaryName, platform = process.platform) {
  return normalizeRuntimePlatform(platform) === 'win32' ? `${binaryName}.exe` : binaryName;
}

export function defaultReleaseOutputDir(repoRoot) {
  return path.join(repoRoot, 'artifacts', 'release');
}

function normalizeInstallMode(mode = 'portable') {
  const normalized = String(mode ?? 'portable').trim().toLowerCase() || 'portable';
  if (!['portable', 'system'].includes(normalized)) {
    throw new Error(`unsupported install mode: ${mode}`);
  }

  return normalized;
}

function runtimePathApi(platform = process.platform) {
  return normalizeRuntimePlatform(platform) === 'win32' ? path.win32 : path.posix;
}

function sqliteUrlForFilePath(filePath) {
  const normalized = toPortablePath(filePath);
  return normalized.startsWith('/') ? `sqlite://${normalized}` : `sqlite:///${normalized}`;
}

function resolveSystemLayoutRoots({
  platform = process.platform,
  env = process.env,
} = {}) {
  const runtimePlatform = normalizeRuntimePlatform(platform);
  const pathApi = runtimePathApi(runtimePlatform);

  if (runtimePlatform === 'win32') {
    const programFilesRoot = env.ProgramFiles ?? 'C:\\Program Files';
    const programDataRoot = env.ProgramData ?? 'C:\\ProgramData';

    return {
      installRoot: pathApi.join(programFilesRoot, 'sdkwork-api-router', 'current'),
      configRoot: pathApi.join(programDataRoot, 'sdkwork-api-router'),
      dataRoot: pathApi.join(programDataRoot, 'sdkwork-api-router', 'data'),
      logRoot: pathApi.join(programDataRoot, 'sdkwork-api-router', 'log'),
      runRoot: pathApi.join(programDataRoot, 'sdkwork-api-router', 'run'),
    };
  }

  if (runtimePlatform === 'darwin') {
    return {
      installRoot: pathApi.join('/usr/local/lib', 'sdkwork-api-router', 'current'),
      configRoot: pathApi.join('/Library/Application Support', 'sdkwork-api-router'),
      dataRoot: pathApi.join('/Library/Application Support', 'sdkwork-api-router', 'data'),
      logRoot: pathApi.join('/Library/Logs', 'sdkwork-api-router'),
      runRoot: pathApi.join('/Library/Application Support', 'sdkwork-api-router', 'run'),
    };
  }

  return {
    installRoot: pathApi.join('/opt', 'sdkwork-api-router', 'current'),
    configRoot: pathApi.join('/etc', 'sdkwork-api-router'),
    dataRoot: pathApi.join('/var', 'lib', 'sdkwork-api-router'),
    logRoot: pathApi.join('/var', 'log', 'sdkwork-api-router'),
    runRoot: pathApi.join('/run', 'sdkwork-api-router'),
  };
}

function resolveRuntimeLayout({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
} = {}) {
  const normalizedMode = normalizeInstallMode(mode);
  const runtimePlatform = normalizeRuntimePlatform(platform);

  if (normalizedMode === 'system') {
    const roots = resolveSystemLayoutRoots({
      platform: runtimePlatform,
      env,
    });
    const pathApi = runtimePathApi(runtimePlatform);
    const programRoot = installRoot ?? roots.installRoot;

    return {
      mode: normalizedMode,
      runtimePlatform,
      pathApi,
      installRoot: programRoot,
      binDir: pathApi.join(programRoot, 'bin'),
      binLibDir: pathApi.join(programRoot, 'bin', 'lib'),
      staticDataDir: pathApi.join(programRoot, 'data'),
      serviceSystemdDir: pathApi.join(programRoot, 'service', 'systemd'),
      serviceLaunchdDir: pathApi.join(programRoot, 'service', 'launchd'),
      serviceWindowsTaskDir: pathApi.join(programRoot, 'service', 'windows-task'),
      sitesAdminDir: pathApi.join(programRoot, 'sites', 'admin'),
      sitesPortalDir: pathApi.join(programRoot, 'sites', 'portal'),
      adminSiteDistDir: pathApi.join(programRoot, 'sites', 'admin', 'dist'),
      portalSiteDistDir: pathApi.join(programRoot, 'sites', 'portal', 'dist'),
      configRoot: roots.configRoot,
      configFile: pathApi.join(roots.configRoot, 'router.yaml'),
      configFragmentDir: pathApi.join(roots.configRoot, 'conf.d'),
      envFile: pathApi.join(roots.configRoot, 'router.env'),
      envExampleFile: pathApi.join(roots.configRoot, 'router.env.example'),
      dataRoot: roots.dataRoot,
      logRoot: roots.logRoot,
      runRoot: roots.runRoot,
      routerBinary: pathApi.join(
        programRoot,
        'bin',
        withExecutable('router-product-service', runtimePlatform),
      ),
    };
  }

  const programRoot = installRoot;
  return {
    mode: normalizedMode,
    runtimePlatform,
    pathApi: path,
    installRoot: programRoot,
    binDir: path.join(programRoot, 'bin'),
    binLibDir: path.join(programRoot, 'bin', 'lib'),
    staticDataDir: path.join(programRoot, 'data'),
    serviceSystemdDir: path.join(programRoot, 'service', 'systemd'),
    serviceLaunchdDir: path.join(programRoot, 'service', 'launchd'),
    serviceWindowsTaskDir: path.join(programRoot, 'service', 'windows-task'),
    sitesAdminDir: path.join(programRoot, 'sites', 'admin'),
    sitesPortalDir: path.join(programRoot, 'sites', 'portal'),
    adminSiteDistDir: path.join(programRoot, 'sites', 'admin', 'dist'),
    portalSiteDistDir: path.join(programRoot, 'sites', 'portal', 'dist'),
    configRoot: path.join(programRoot, 'config'),
    configFile: path.join(programRoot, 'config', 'router.yaml'),
    configFragmentDir: path.join(programRoot, 'config', 'conf.d'),
    envFile: path.join(programRoot, 'config', 'router.env'),
    envExampleFile: path.join(programRoot, 'config', 'router.env.example'),
    dataRoot: path.join(programRoot, 'var', 'data'),
    logRoot: path.join(programRoot, 'var', 'log'),
    runRoot: path.join(programRoot, 'var', 'run'),
    routerBinary: path.join(
      programRoot,
      'bin',
      withExecutable('router-product-service', runtimePlatform),
    ),
  };
}

function defaultDatabaseUrlForLayout(layout) {
  if (layout.mode === 'system') {
    return 'postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router';
  }

  return sqliteUrlForFilePath(layout.pathApi.join(layout.dataRoot, 'sdkwork-api-router.db'));
}

export function defaultInstallRoot(repoRoot, {
  mode = 'portable',
  platform = process.platform,
  env = process.env,
} = {}) {
  const normalizedMode = normalizeInstallMode(mode);
  if (normalizedMode === 'portable') {
    return path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router', 'current');
  }

  return resolveSystemLayoutRoots({
    platform,
    env,
  }).installRoot;
}

function quoteEnvValue(value) {
  return `"${String(value)
    .replaceAll('\\', '\\\\')
    .replaceAll('"', '\\"')}"`;
}

function systemdEscapeValue(value) {
  return String(value)
    .replaceAll('\\', '\\\\')
    .replaceAll(' ', '\\ ');
}

function systemdQuoteArg(value) {
  return `"${String(value)
    .replaceAll('\\', '\\\\')
    .replaceAll('"', '\\"')}"`;
}

function resolveReleaseCargoJobs({
  platform = process.platform,
  env = process.env,
} = {}) {
  const runtimePlatform = normalizeRuntimePlatform(platform);
  const requestedJobs = String(env.CARGO_BUILD_JOBS ?? '').trim();
  if (requestedJobs.length > 0) {
    return requestedJobs;
  }

  return runtimePlatform === 'win32' ? '1' : '';
}

export function createReleaseBuildPlan({
  repoRoot,
  platform = process.platform,
  arch = process.arch,
  env = process.env,
  installDependencies = false,
  includeDocs = true,
  includeConsole = true,
  releaseOutputDir = defaultReleaseOutputDir(repoRoot),
  exists = existsSync,
} = {}) {
  const runtimePlatform = normalizeRuntimePlatform(platform);
  const buildEnv = withSupportedWindowsCmakeGenerator(
    withManagedWorkspaceTargetDir({
      workspaceRoot: repoRoot,
      env,
      platform: runtimePlatform,
    }),
    runtimePlatform,
  );
  const releaseCargoJobs = resolveReleaseCargoJobs({
    platform: runtimePlatform,
    env: buildEnv,
  });
  const target = resolveDesktopReleaseTarget({
    platform: runtimePlatform,
    arch,
    env: buildEnv,
  });
  if (releaseCargoJobs) {
    buildEnv.CARGO_BUILD_JOBS = releaseCargoJobs;
  }
  const rustRunner = resolveRustRunner(platform, buildEnv);
  const cargoArgs = [
    ...rustRunner.args,
    'build',
    '--release',
    '--target',
    target.targetTriple,
  ];
  for (const binaryName of RELEASE_BINARY_NAMES) {
    cargoArgs.push('-p', binaryName);
  }
  if (releaseCargoJobs) {
    cargoArgs.push('-j', releaseCargoJobs);
  }

  const steps = [
    {
      label: 'cargo release build',
      command: rustRunner.command,
      args: cargoArgs,
      cwd: repoRoot,
      env: buildEnv,
      shell: rustRunner.shell,
      windowsHide: runtimePlatform === 'win32',
    },
  ];

  const pnpm = pnpmCommand(platform);
  const appDirs = [
    {
      key: 'admin',
      label: 'admin app',
      dir: path.join(repoRoot, 'apps', 'sdkwork-router-admin'),
    },
    {
      key: 'portal',
      label: 'portal app',
      dir: path.join(repoRoot, 'apps', 'sdkwork-router-portal'),
    },
  ];

  if (includeConsole) {
    appDirs.push({
      key: 'console',
      label: 'console',
      dir: path.join(repoRoot, 'console'),
    });
  }

  if (includeDocs) {
    appDirs.push({
      key: 'docs',
      label: 'docs',
      dir: path.join(repoRoot, 'docs'),
    });
  }

  for (const app of appDirs) {
    const nodeModulesDir = path.join(app.dir, 'node_modules');
    if (installDependencies || !exists(nodeModulesDir)) {
      steps.push({
        label: `${app.label} install`,
        command: pnpm,
        args: ['--dir', toPortablePath(path.relative(repoRoot, app.dir)), 'install'],
        cwd: repoRoot,
        env: buildEnv,
        shell: runtimePlatform === 'win32',
        windowsHide: runtimePlatform === 'win32',
      });
    }

    steps.push({
      label: `${app.label} build`,
      command: pnpm,
      args: ['--dir', toPortablePath(path.relative(repoRoot, app.dir)), 'build'],
      cwd: repoRoot,
      env: buildEnv,
      shell: runtimePlatform === 'win32',
      windowsHide: runtimePlatform === 'win32',
    });
  }

  const nodeCommand = process.execPath;
  steps.push(
    {
      label: 'admin desktop release build',
      command: nodeCommand,
      args: [
        path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-build.mjs'),
        '--app',
        'admin',
        '--target',
        target.targetTriple,
      ],
      cwd: repoRoot,
      env: buildEnv,
      shell: false,
      windowsHide: runtimePlatform === 'win32',
    },
    {
      label: 'portal desktop release build',
      command: nodeCommand,
      args: [
        path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-build.mjs'),
        '--app',
        'portal',
        '--target',
        target.targetTriple,
      ],
      cwd: repoRoot,
      env: buildEnv,
      shell: false,
      windowsHide: runtimePlatform === 'win32',
    },
    {
      label: 'native release package',
      command: nodeCommand,
      args: [
        path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
        'native',
        '--platform',
        target.platform,
        '--arch',
        target.arch,
        '--target',
        target.targetTriple,
        '--output-dir',
        releaseOutputDir,
      ],
      cwd: repoRoot,
      env: buildEnv,
      shell: false,
      windowsHide: runtimePlatform === 'win32',
    },
  );

  return {
    target,
    steps,
  };
}

export function renderRuntimeEnvTemplate({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  defaults = PROD_DEFAULTS,
} = {}) {
  const layout = resolveRuntimeLayout({
    installRoot,
    mode,
    platform,
    env,
  });
  const configDir = toPortablePath(layout.configRoot);
  const configFile = toPortablePath(layout.configFile);
  const binaryPath = toPortablePath(layout.routerBinary);
  const adminSiteDir = toPortablePath(layout.adminSiteDistDir);
  const portalSiteDir = toPortablePath(layout.portalSiteDistDir);

  return [
    '# SDKWork Router production runtime defaults',
    '# Canonical runtime config should live in router.yaml.',
    `SDKWORK_ROUTER_INSTALL_MODE=${quoteEnvValue(layout.mode)}`,
    `SDKWORK_CONFIG_DIR=${quoteEnvValue(configDir)}`,
    `SDKWORK_CONFIG_FILE=${quoteEnvValue(configFile)}`,
    `SDKWORK_DATABASE_URL=${quoteEnvValue(defaultDatabaseUrlForLayout(layout))}`,
    `SDKWORK_WEB_BIND=${quoteEnvValue(defaults.webBind)}`,
    `SDKWORK_GATEWAY_BIND=${quoteEnvValue(defaults.gatewayBind)}`,
    `SDKWORK_ADMIN_BIND=${quoteEnvValue(defaults.adminBind)}`,
    `SDKWORK_PORTAL_BIND=${quoteEnvValue(defaults.portalBind)}`,
    `SDKWORK_ADMIN_SITE_DIR=${quoteEnvValue(adminSiteDir)}`,
    `SDKWORK_PORTAL_SITE_DIR=${quoteEnvValue(portalSiteDir)}`,
    `SDKWORK_ROUTER_BINARY=${quoteEnvValue(binaryPath)}`,
    '',
  ].join('\n');
}

function renderRouterConfigTemplate({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  defaults = PROD_DEFAULTS,
} = {}) {
  const layout = resolveRuntimeLayout({
    installRoot,
    mode,
    platform,
    env,
  });

  return [
    '# SDKWork Router canonical runtime config',
    `gateway_bind: "${defaults.gatewayBind}"`,
    `admin_bind: "${defaults.adminBind}"`,
    `portal_bind: "${defaults.portalBind}"`,
    `database_url: "${defaultDatabaseUrlForLayout(layout)}"`,
    'bootstrap_profile: "prod"',
    'allow_insecure_dev_defaults: false',
    '',
  ].join('\n');
}

export function renderSystemdUnit({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  serviceName = 'sdkwork-api-router',
} = {}) {
  const layout = resolveRuntimeLayout({
    installRoot,
    mode,
    platform,
    env,
  });
  const portableRoot = toPortablePath(layout.installRoot);
  const startScript = toPortablePath(layout.pathApi.join(layout.binDir, 'start.sh'));
  const envFile = toPortablePath(layout.envFile);

  return [
    '[Unit]',
    `Description=${serviceName}`,
    'After=network.target',
    '',
    '[Service]',
    'Type=simple',
    `WorkingDirectory=${systemdEscapeValue(portableRoot)}`,
    `EnvironmentFile=-${systemdEscapeValue(envFile)}`,
    `ExecStart=${systemdQuoteArg(startScript)} --foreground --home ${systemdQuoteArg(portableRoot)}`,
    'Restart=on-failure',
    'RestartSec=5',
    'TimeoutStopSec=30',
    '',
    '[Install]',
    'WantedBy=multi-user.target',
    '',
  ].join('\n');
}

function xmlEscape(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&apos;');
}

export function renderLaunchdPlist({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  serviceName = 'com.sdkwork.api-router',
} = {}) {
  const layout = resolveRuntimeLayout({
    installRoot,
    mode,
    platform,
    env,
  });
  const portableRoot = toPortablePath(layout.installRoot);
  const startScript = toPortablePath(layout.pathApi.join(layout.binDir, 'start.sh'));
  const stdoutPath = toPortablePath(
    layout.pathApi.join(layout.logRoot, 'router-product-service.launchd.stdout.log'),
  );
  const stderrPath = toPortablePath(
    layout.pathApi.join(layout.logRoot, 'router-product-service.launchd.stderr.log'),
  );

  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">',
    '<plist version="1.0">',
    '<dict>',
    '  <key>Label</key>',
    `  <string>${xmlEscape(serviceName)}</string>`,
    '  <key>ProgramArguments</key>',
    '  <array>',
    `    <string>${xmlEscape(startScript)}</string>`,
    '    <string>--foreground</string>',
    '    <string>--home</string>',
    `    <string>${xmlEscape(portableRoot)}</string>`,
    '  </array>',
    '  <key>WorkingDirectory</key>',
    `  <string>${xmlEscape(portableRoot)}</string>`,
    '  <key>RunAtLoad</key>',
    '  <true/>',
    '  <key>KeepAlive</key>',
    '  <true/>',
    '  <key>StandardOutPath</key>',
    `  <string>${xmlEscape(stdoutPath)}</string>`,
    '  <key>StandardErrorPath</key>',
    `  <string>${xmlEscape(stderrPath)}</string>`,
    '</dict>',
    '</plist>',
    '',
  ].join('\n');
}

export function renderWindowsTaskXml({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  taskName = 'sdkwork-api-router',
} = {}) {
  const layout = resolveRuntimeLayout({
    installRoot,
    mode,
    platform,
    env,
  });
  const portableRoot = toPortablePath(layout.installRoot);
  const startScript = toPortablePath(layout.pathApi.join(layout.binDir, 'start.ps1'));
  const taskAuthor = xmlEscape(taskName);
  const command = 'powershell.exe';
  const argumentsText = [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-File',
    `"${startScript}"`,
    '-Foreground',
    '-Home',
    `"${portableRoot}"`,
  ].join(' ');

  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">',
    '  <RegistrationInfo>',
    `    <Author>${taskAuthor}</Author>`,
    `    <Description>${taskAuthor} boot task</Description>`,
    '  </RegistrationInfo>',
    '  <Triggers>',
    '    <BootTrigger>',
    '      <Enabled>true</Enabled>',
    '    </BootTrigger>',
    '  </Triggers>',
    '  <Principals>',
    '    <Principal id="Author">',
    '      <UserId>SYSTEM</UserId>',
    '      <RunLevel>HighestAvailable</RunLevel>',
    '    </Principal>',
    '  </Principals>',
    '  <Settings>',
    '    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>',
    '    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>',
    '    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>',
    '    <AllowHardTerminate>true</AllowHardTerminate>',
    '    <StartWhenAvailable>true</StartWhenAvailable>',
    '    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>',
    '    <IdleSettings>',
    '      <StopOnIdleEnd>false</StopOnIdleEnd>',
    '      <RestartOnIdle>false</RestartOnIdle>',
    '    </IdleSettings>',
    '    <AllowStartOnDemand>true</AllowStartOnDemand>',
    '    <Enabled>true</Enabled>',
    '    <Hidden>false</Hidden>',
    '    <RunOnlyIfIdle>false</RunOnlyIfIdle>',
    '    <WakeToRun>false</WakeToRun>',
    '    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>',
    '    <Priority>7</Priority>',
    '  </Settings>',
    '  <Actions Context="Author">',
    '    <Exec>',
    `      <Command>${xmlEscape(command)}</Command>`,
    `      <Arguments>${xmlEscape(argumentsText)}</Arguments>`,
    `      <WorkingDirectory>${xmlEscape(portableRoot)}</WorkingDirectory>`,
    '    </Exec>',
    '  </Actions>',
    '</Task>',
    '',
  ].join('\n');
}

export function renderSystemdInstallScript({
  serviceName = 'sdkwork-api-router',
} = {}) {
  const unitName = `${serviceName}.service`;

  return [
    '#!/usr/bin/env sh',
    '',
    'set -eu',
    '',
    `SERVICE_NAME='${serviceName}'`,
    `UNIT_NAME='${unitName}'`,
    "SYSTEMD_DIR=${SYSTEMD_DIR:-/etc/systemd/system}",
    "SYSTEMCTL_BIN=${SYSTEMCTL_BIN:-systemctl}",
    'SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)',
    'UNIT_SOURCE="$SCRIPT_DIR/$UNIT_NAME"',
    'UNIT_TARGET="$SYSTEMD_DIR/$UNIT_NAME"',
    '',
    'require_privileged_runner() {',
    '  if [ "$(id -u)" -eq 0 ]; then',
    '    printf %s ""',
    '    return 0',
    '  fi',
    '  if command -v sudo >/dev/null 2>&1; then',
    '    printf %s "sudo"',
    '    return 0',
    '  fi',
    '  printf %s "Root privileges or sudo are required to install the systemd service." >&2',
    '  exit 1',
    '}',
    '',
    'run_privileged() {',
    '  RUNNER="$(require_privileged_runner)"',
    '  if [ -n "$RUNNER" ]; then',
    '    "$RUNNER" "$@"',
    '    return',
    '  fi',
    '  "$@"',
    '}',
    '',
    'if ! command -v "$SYSTEMCTL_BIN" >/dev/null 2>&1; then',
    '  printf %s "systemctl was not found in PATH." >&2',
    '  exit 1',
    'fi',
    '',
    'run_privileged mkdir -p "$SYSTEMD_DIR"',
    'run_privileged cp "$UNIT_SOURCE" "$UNIT_TARGET"',
    'run_privileged chmod 0644 "$UNIT_TARGET"',
    'run_privileged "$SYSTEMCTL_BIN" daemon-reload',
    'run_privileged "$SYSTEMCTL_BIN" enable --now "$UNIT_NAME"',
    'printf \'Installed and started %s using %s\\n\' "$SERVICE_NAME" "$UNIT_TARGET"',
    '',
  ].join('\n');
}

export function renderSystemdUninstallScript({
  serviceName = 'sdkwork-api-router',
} = {}) {
  const unitName = `${serviceName}.service`;

  return [
    '#!/usr/bin/env sh',
    '',
    'set -eu',
    '',
    `SERVICE_NAME='${serviceName}'`,
    `UNIT_NAME='${unitName}'`,
    "SYSTEMD_DIR=${SYSTEMD_DIR:-/etc/systemd/system}",
    "SYSTEMCTL_BIN=${SYSTEMCTL_BIN:-systemctl}",
    'UNIT_TARGET="$SYSTEMD_DIR/$UNIT_NAME"',
    '',
    'require_privileged_runner() {',
    '  if [ "$(id -u)" -eq 0 ]; then',
    '    printf %s ""',
    '    return 0',
    '  fi',
    '  if command -v sudo >/dev/null 2>&1; then',
    '    printf %s "sudo"',
    '    return 0',
    '  fi',
    '  printf %s "Root privileges or sudo are required to uninstall the systemd service." >&2',
    '  exit 1',
    '}',
    '',
    'run_privileged() {',
    '  RUNNER="$(require_privileged_runner)"',
    '  if [ -n "$RUNNER" ]; then',
    '    "$RUNNER" "$@"',
    '    return',
    '  fi',
    '  "$@"',
    '}',
    '',
    'if ! command -v "$SYSTEMCTL_BIN" >/dev/null 2>&1; then',
    '  printf %s "systemctl was not found in PATH." >&2',
    '  exit 1',
    'fi',
    '',
    'run_privileged "$SYSTEMCTL_BIN" disable --now "$UNIT_NAME" >/dev/null 2>&1 || true',
    'run_privileged rm -f "$UNIT_TARGET"',
    'run_privileged "$SYSTEMCTL_BIN" daemon-reload',
    'run_privileged "$SYSTEMCTL_BIN" reset-failed "$UNIT_NAME" >/dev/null 2>&1 || true',
    'printf \'Uninstalled %s from %s\\n\' "$SERVICE_NAME" "$UNIT_TARGET"',
    '',
  ].join('\n');
}

export function renderLaunchdInstallScript({
  serviceName = 'com.sdkwork.api-router',
} = {}) {
  const plistName = `${serviceName}.plist`;

  return [
    '#!/usr/bin/env sh',
    '',
    'set -eu',
    '',
    `SERVICE_LABEL='${serviceName}'`,
    `PLIST_NAME='${plistName}'`,
    "LAUNCHD_TARGET_DIR=${LAUNCHD_TARGET_DIR:-/Library/LaunchDaemons}",
    "LAUNCHCTL_BIN=${LAUNCHCTL_BIN:-launchctl}",
    'SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)',
    'PLIST_SOURCE="$SCRIPT_DIR/$PLIST_NAME"',
    'PLIST_TARGET="$LAUNCHD_TARGET_DIR/$PLIST_NAME"',
    '',
    'require_privileged_runner() {',
    '  if [ "$(id -u)" -eq 0 ]; then',
    '    printf %s ""',
    '    return 0',
    '  fi',
    '  if command -v sudo >/dev/null 2>&1; then',
    '    printf %s "sudo"',
    '    return 0',
    '  fi',
    '  printf %s "Root privileges or sudo are required to install the launchd daemon." >&2',
    '  exit 1',
    '}',
    '',
    'run_privileged() {',
    '  RUNNER="$(require_privileged_runner)"',
    '  if [ -n "$RUNNER" ]; then',
    '    "$RUNNER" "$@"',
    '    return',
    '  fi',
    '  "$@"',
    '}',
    '',
    'if ! command -v "$LAUNCHCTL_BIN" >/dev/null 2>&1; then',
    '  printf %s "launchctl was not found in PATH." >&2',
    '  exit 1',
    'fi',
    '',
    'run_privileged mkdir -p "$LAUNCHD_TARGET_DIR"',
    'run_privileged "$LAUNCHCTL_BIN" bootout "system/$SERVICE_LABEL" >/dev/null 2>&1 || true',
    'run_privileged cp "$PLIST_SOURCE" "$PLIST_TARGET"',
    'run_privileged chmod 0644 "$PLIST_TARGET"',
    'run_privileged chown root:wheel "$PLIST_TARGET" >/dev/null 2>&1 || true',
    'run_privileged "$LAUNCHCTL_BIN" bootstrap system "$PLIST_TARGET"',
    'run_privileged "$LAUNCHCTL_BIN" enable "system/$SERVICE_LABEL" >/dev/null 2>&1 || true',
    'printf \'Installed and bootstrapped %s using %s\\n\' "$SERVICE_LABEL" "$PLIST_TARGET"',
    '',
  ].join('\n');
}

export function renderLaunchdUninstallScript({
  serviceName = 'com.sdkwork.api-router',
} = {}) {
  const plistName = `${serviceName}.plist`;

  return [
    '#!/usr/bin/env sh',
    '',
    'set -eu',
    '',
    `SERVICE_LABEL='${serviceName}'`,
    `PLIST_NAME='${plistName}'`,
    "LAUNCHD_TARGET_DIR=${LAUNCHD_TARGET_DIR:-/Library/LaunchDaemons}",
    "LAUNCHCTL_BIN=${LAUNCHCTL_BIN:-launchctl}",
    'PLIST_TARGET="$LAUNCHD_TARGET_DIR/$PLIST_NAME"',
    '',
    'require_privileged_runner() {',
    '  if [ "$(id -u)" -eq 0 ]; then',
    '    printf %s ""',
    '    return 0',
    '  fi',
    '  if command -v sudo >/dev/null 2>&1; then',
    '    printf %s "sudo"',
    '    return 0',
    '  fi',
    '  printf %s "Root privileges or sudo are required to uninstall the launchd daemon." >&2',
    '  exit 1',
    '}',
    '',
    'run_privileged() {',
    '  RUNNER="$(require_privileged_runner)"',
    '  if [ -n "$RUNNER" ]; then',
    '    "$RUNNER" "$@"',
    '    return',
    '  fi',
    '  "$@"',
    '}',
    '',
    'if ! command -v "$LAUNCHCTL_BIN" >/dev/null 2>&1; then',
    '  printf %s "launchctl was not found in PATH." >&2',
    '  exit 1',
    'fi',
    '',
    'run_privileged "$LAUNCHCTL_BIN" bootout "system/$SERVICE_LABEL" >/dev/null 2>&1 || true',
    'run_privileged "$LAUNCHCTL_BIN" disable "system/$SERVICE_LABEL" >/dev/null 2>&1 || true',
    'run_privileged rm -f "$PLIST_TARGET"',
    'printf \'Uninstalled %s from %s\\n\' "$SERVICE_LABEL" "$PLIST_TARGET"',
    '',
  ].join('\n');
}

export function renderWindowsTaskInstallScript({
  taskName = 'sdkwork-api-router',
} = {}) {
  return [
    'param(',
    `    [string]$TaskName = '${taskName}',`,
    '    [switch]$StartNow,',
    '    [string]$SchTasksBin = $env:SCHTASKS_BIN,',
    '    [switch]$SkipAdminCheck',
    ')',
    '',
    "Set-StrictMode -Version Latest",
    "$ErrorActionPreference = 'Stop'",
    '',
    'function Test-IsAdministrator {',
    '    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())',
    '    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)',
    '}',
    '',
    'if (-not $SchTasksBin) {',
    "    $SchTasksBin = 'schtasks.exe'",
    '}',
    '',
    'if (-not $SkipAdminCheck -and -not (Test-IsAdministrator)) {',
    '    throw "Administrator privileges are required to register the scheduled task."',
    '}',
    '',
    "$taskXml = Join-Path $PSScriptRoot 'sdkwork-api-router.xml'",
    'if (-not (Test-Path $taskXml -PathType Leaf)) {',
    '    throw "Task XML not found: $taskXml"',
    '}',
    '',
    '& $SchTasksBin /Create /TN $TaskName /XML $taskXml /F | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    throw "$SchTasksBin failed to register task $TaskName"',
    '}',
    '',
    'if ($StartNow) {',
    '    & $SchTasksBin /Run /TN $TaskName | Out-Null',
    '    if ($LASTEXITCODE -ne 0) {',
    '        throw "$SchTasksBin failed to start task $TaskName"',
    '    }',
    '}',
    '',
    'Write-Host "Installed scheduled task $TaskName from $taskXml"',
    '',
  ].join('\n');
}

export function renderWindowsTaskUninstallScript({
  taskName = 'sdkwork-api-router',
} = {}) {
  return [
    'param(',
    `    [string]$TaskName = '${taskName}',`,
    '    [string]$SchTasksBin = $env:SCHTASKS_BIN,',
    '    [switch]$SkipAdminCheck',
    ')',
    '',
    "Set-StrictMode -Version Latest",
    "$ErrorActionPreference = 'Stop'",
    '',
    'function Test-IsAdministrator {',
    '    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())',
    '    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)',
    '}',
    '',
    'if (-not $SchTasksBin) {',
    "    $SchTasksBin = 'schtasks.exe'",
    '}',
    '',
    'if (-not $SkipAdminCheck -and -not (Test-IsAdministrator)) {',
    '    throw "Administrator privileges are required to unregister the scheduled task."',
    '}',
    '',
    '& $SchTasksBin /Query /TN $TaskName | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    Write-Host "Scheduled task $TaskName is not registered."',
    '    exit 0',
    '}',
    '',
    '& $SchTasksBin /End /TN $TaskName | Out-Null',
    '& $SchTasksBin /Delete /TN $TaskName /F | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    throw "$SchTasksBin failed to delete task $TaskName"',
    '}',
    '',
    'Write-Host "Removed scheduled task $TaskName"',
    '',
  ].join('\n');
}

export function renderWindowsServiceRunScript({
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
} = {}) {
  const layout = resolveRuntimeLayout({
    installRoot,
    mode,
    platform,
    env,
  });
  const portableRoot = toPortablePath(layout.installRoot);
  const envFile = toPortablePath(layout.envFile);

  return [
    "$ErrorActionPreference = 'Stop'",
    "$runtimeHome = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)",
    "$runtimeCommon = Join-Path $runtimeHome 'bin\\lib\\runtime-common.ps1'",
    "if (-not (Test-Path $runtimeCommon -PathType Leaf)) {",
    '    throw "Runtime common helpers not found: $runtimeCommon"',
    '}',
    '. $runtimeCommon',
    `Import-RouterEnvFile -EnvFile '${envFile.replaceAll("'", "''")}'`,
    `& (Join-Path $runtimeHome 'bin\\start.ps1') -Foreground -Home '${portableRoot.replaceAll("'", "''")}'`,
    'exit $LASTEXITCODE',
    '',
  ].join('\n');
}

export function renderWindowsServiceInstallScript({
  serviceName = 'sdkwork-api-router',
  displayName = 'SDKWork API Router',
} = {}) {
  return [
    'param(',
    `    [string]$ServiceName = '${serviceName}',`,
    `    [string]$DisplayName = '${displayName}',`,
    '    [string]$ScExe = $env:SC_EXE,',
    '    [switch]$SkipAdminCheck',
    ')',
    '',
    "Set-StrictMode -Version Latest",
    "$ErrorActionPreference = 'Stop'",
    '',
    'function Test-IsAdministrator {',
    '    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())',
    '    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)',
    '}',
    '',
    'if (-not $ScExe) {',
    "    $ScExe = 'sc.exe'",
    '}',
    '',
    'if (-not $SkipAdminCheck -and -not (Test-IsAdministrator)) {',
    '    throw "Administrator privileges are required to register the Windows service."',
    '}',
    '',
    "$runScript = Join-Path $PSScriptRoot 'run-service.ps1'",
    'if (-not (Test-Path $runScript -PathType Leaf)) {',
    '    throw "Windows service runner not found: $runScript"',
    '}',
    '',
    '$binPath = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$runScript`""',
    '& $ScExe query $ServiceName | Out-Null',
    'if ($LASTEXITCODE -eq 0) {',
    '    & $ScExe stop $ServiceName | Out-Null',
    '    & $ScExe delete $ServiceName | Out-Null',
    '    Start-Sleep -Seconds 1',
    '}',
    '& $ScExe create $ServiceName "binPath= $binPath" "start= auto" "DisplayName= $DisplayName" | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    throw "$ScExe failed to create service $ServiceName"',
    '}',
    '& $ScExe description $ServiceName $DisplayName | Out-Null',
    '& $ScExe start $ServiceName | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    throw "$ScExe failed to start service $ServiceName"',
    '}',
    'Write-Host "Installed Windows service $ServiceName using $runScript"',
    '',
  ].join('\n');
}

export function renderWindowsServiceUninstallScript({
  serviceName = 'sdkwork-api-router',
} = {}) {
  return [
    'param(',
    `    [string]$ServiceName = '${serviceName}',`,
    '    [string]$ScExe = $env:SC_EXE,',
    '    [switch]$SkipAdminCheck',
    ')',
    '',
    "Set-StrictMode -Version Latest",
    "$ErrorActionPreference = 'Stop'",
    '',
    'function Test-IsAdministrator {',
    '    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())',
    '    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)',
    '}',
    '',
    'if (-not $ScExe) {',
    "    $ScExe = 'sc.exe'",
    '}',
    '',
    'if (-not $SkipAdminCheck -and -not (Test-IsAdministrator)) {',
    '    throw "Administrator privileges are required to unregister the Windows service."',
    '}',
    '',
    '& $ScExe query $ServiceName | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    Write-Host "Windows service $ServiceName is not registered."',
    '    exit 0',
    '}',
    '',
    '& $ScExe stop $ServiceName | Out-Null',
    '& $ScExe delete $ServiceName | Out-Null',
    'if ($LASTEXITCODE -ne 0) {',
    '    throw "$ScExe failed to delete service $ServiceName"',
    '}',
    '',
    'Write-Host "Removed Windows service $ServiceName"',
    '',
  ].join('\n');
}

export function createInstallPlan({
  repoRoot,
  installRoot,
  mode = 'portable',
  platform = process.platform,
  arch = process.arch,
  env = process.env,
} = {}) {
  const runtimePlatform = normalizeRuntimePlatform(platform);
  const normalizedMode = normalizeInstallMode(mode);
  const resolvedInstallRoot = installRoot ?? defaultInstallRoot(repoRoot, {
    mode: normalizedMode,
    platform: runtimePlatform,
    env,
  });
  const layout = resolveRuntimeLayout({
    installRoot: resolvedInstallRoot,
    mode: normalizedMode,
    platform: runtimePlatform,
    env,
  });
  const target = resolveDesktopReleaseTarget({
    platform: runtimePlatform,
    arch,
    env,
  });
  const releaseRoot = path.join(
    resolveWorkspaceTargetDir({
      workspaceRoot: repoRoot,
      env,
      platform: runtimePlatform,
    }),
    target.targetTriple,
    'release',
  );
  const directories = Array.from(new Set([
    layout.installRoot,
    layout.binDir,
    layout.binLibDir,
    layout.configRoot,
    layout.configFragmentDir,
    layout.staticDataDir,
    layout.serviceSystemdDir,
    layout.serviceLaunchdDir,
    layout.pathApi.join(layout.installRoot, 'service', 'windows-service'),
    layout.serviceWindowsTaskDir,
    layout.sitesAdminDir,
    layout.sitesPortalDir,
    layout.dataRoot,
    layout.logRoot,
    layout.runRoot,
  ]));

  const files = [];

  for (const binaryName of RELEASE_BINARY_NAMES) {
    files.push({
      type: 'file',
      sourcePath: path.join(releaseRoot, withExecutable(binaryName, runtimePlatform)),
      targetPath: layout.pathApi.join(
        layout.binDir,
        withExecutable(binaryName, runtimePlatform),
      ),
    });
  }

  const runtimeScripts = [
    'start.sh',
    'stop.sh',
    'start.ps1',
    'stop.ps1',
  ];
  for (const scriptName of runtimeScripts) {
    files.push({
      type: 'file',
      sourcePath: path.join(repoRoot, 'bin', scriptName),
      targetPath: layout.pathApi.join(layout.binDir, scriptName),
    });
  }

  const runtimeLibs = ['runtime-common.sh', 'runtime-common.ps1'];
  for (const libName of runtimeLibs) {
    files.push({
      type: 'file',
      sourcePath: path.join(repoRoot, 'bin', 'lib', libName),
      targetPath: layout.pathApi.join(layout.binLibDir, libName),
    });
  }

  files.push(
    {
      type: 'directory',
      sourcePath: path.join(repoRoot, 'data'),
      targetPath: layout.staticDataDir,
    },
    {
      type: 'directory',
      sourcePath: path.join(repoRoot, 'apps', 'sdkwork-router-admin', 'dist'),
      targetPath: layout.adminSiteDistDir,
    },
    {
      type: 'directory',
      sourcePath: path.join(repoRoot, 'apps', 'sdkwork-router-portal', 'dist'),
      targetPath: layout.portalSiteDistDir,
    },
    {
      type: 'text',
      targetPath: layout.configFile,
      contents: renderRouterConfigTemplate({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
      skipIfExists: true,
    },
    {
      type: 'text',
      targetPath: layout.envFile,
      contents: renderRuntimeEnvTemplate({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
      skipIfExists: true,
    },
    {
      type: 'text',
      targetPath: layout.envExampleFile,
      contents: renderRuntimeEnvTemplate({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.serviceSystemdDir,
        'sdkwork-api-router.service',
      ),
      contents: renderSystemdUnit({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(layout.serviceSystemdDir, 'install-service.sh'),
      contents: renderSystemdInstallScript(),
      mode: 0o755,
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(layout.serviceSystemdDir, 'uninstall-service.sh'),
      contents: renderSystemdUninstallScript(),
      mode: 0o755,
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.serviceLaunchdDir,
        'com.sdkwork.api-router.plist',
      ),
      contents: renderLaunchdPlist({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(layout.serviceLaunchdDir, 'install-service.sh'),
      contents: renderLaunchdInstallScript(),
      mode: 0o755,
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(layout.serviceLaunchdDir, 'uninstall-service.sh'),
      contents: renderLaunchdUninstallScript(),
      mode: 0o755,
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.installRoot,
        'service',
        'windows-service',
        'run-service.ps1',
      ),
      contents: renderWindowsServiceRunScript({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.installRoot,
        'service',
        'windows-service',
        'install-service.ps1',
      ),
      contents: renderWindowsServiceInstallScript(),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.installRoot,
        'service',
        'windows-service',
        'uninstall-service.ps1',
      ),
      contents: renderWindowsServiceUninstallScript(),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.serviceWindowsTaskDir,
        'sdkwork-api-router.xml',
      ),
      contents: renderWindowsTaskXml({
        installRoot: layout.installRoot,
        mode: normalizedMode,
        platform: runtimePlatform,
        env,
      }),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.serviceWindowsTaskDir,
        'install-service.ps1',
      ),
      contents: renderWindowsTaskInstallScript(),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(
        layout.serviceWindowsTaskDir,
        'uninstall-service.ps1',
      ),
      contents: renderWindowsTaskUninstallScript(),
    },
    {
      type: 'text',
      targetPath: layout.pathApi.join(layout.installRoot, 'release-manifest.json'),
      contents: `${JSON.stringify({
        runtime: 'sdkwork-api-router',
        installMode: normalizedMode,
        target,
        installedBinaries: RELEASE_BINARY_NAMES,
        bootstrapDataRoot: 'data',
        configRoot: layout.configRoot,
        configFile: layout.configFile,
        mutableDataRoot: layout.dataRoot,
        logRoot: layout.logRoot,
        runRoot: layout.runRoot,
        installedAt: new Date().toISOString(),
      }, null, 2)}\n`,
    },
  );

  return {
    mode: normalizedMode,
    target,
    directories,
    files,
  };
}

function ensureDirectory(directoryPath) {
  mkdirSync(directoryPath, { recursive: true });
}

export function applyInstallPlan(plan, { force = false } = {}) {
  if (force && plan?.directories?.[0]) {
    rmSync(plan.directories[0], { recursive: true, force: true });
  }

  for (const directoryPath of plan.directories) {
    ensureDirectory(directoryPath);
  }

  for (const file of plan.files) {
    ensureDirectory(path.dirname(file.targetPath));

    if (file.type === 'directory') {
      rmSync(file.targetPath, { recursive: true, force: true });
      cpSync(file.sourcePath, file.targetPath, { recursive: true });
      continue;
    }

    if (file.type === 'file') {
      cpSync(file.sourcePath, file.targetPath);
      if (file.mode != null) {
        chmodSync(file.targetPath, file.mode);
      }
      continue;
    }

    if (file.type === 'text') {
      if (file.skipIfExists && existsSync(file.targetPath)) {
        continue;
      }
      writeFileSync(file.targetPath, file.contents, 'utf8');
      if (file.mode != null) {
        chmodSync(file.targetPath, file.mode);
      }
    }
  }
}

export function assertInstallInputsExist(plan) {
  for (const file of plan.files) {
    if (file.type === 'file' || file.type === 'directory') {
      if (!existsSync(file.sourcePath)) {
        throw new Error(`Missing install input: ${file.sourcePath}`);
      }
    }
  }
}

function parseEnvValue(value) {
  const trimmed = String(value ?? '').trim();
  if (trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return trimmed.slice(1, -1)
      .replaceAll('\\"', '"')
      .replaceAll('\\\\', '\\');
  }
  if (trimmed.startsWith("'") && trimmed.endsWith("'")) {
    return trimmed.slice(1, -1);
  }

  return trimmed;
}

function readEnvFileValues(envFilePath, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  if (!exists(envFilePath)) {
    return {};
  }

  const values = {};
  const content = String(readFile(envFilePath, 'utf8'));
  for (const rawLine of content.split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) {
      continue;
    }

    const separatorIndex = line.indexOf('=');
    if (separatorIndex <= 0) {
      continue;
    }

    const key = line.slice(0, separatorIndex).trim();
    const value = line.slice(separatorIndex + 1);
    values[key] = parseEnvValue(value);
  }

  return values;
}

export function createValidateConfigPlan({
  repoRoot,
  installRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const runtimePlatform = normalizeRuntimePlatform(platform);
  const normalizedMode = normalizeInstallMode(mode);
  const resolvedInstallRoot = installRoot ?? defaultInstallRoot(repoRoot, {
    mode: normalizedMode,
    platform: runtimePlatform,
    env,
  });
  const layout = resolveRuntimeLayout({
    installRoot: resolvedInstallRoot,
    mode: normalizedMode,
    platform: runtimePlatform,
    env,
  });
  const envFileValues = readEnvFileValues(layout.envFile, {
    exists,
    readFile,
  });
  const validationEnv = {
    ...env,
    ...envFileValues,
  };

  validationEnv.SDKWORK_ROUTER_INSTALL_MODE ??= normalizedMode;
  validationEnv.SDKWORK_CONFIG_DIR ??= toPortablePath(layout.configRoot);
  validationEnv.SDKWORK_CONFIG_FILE ??= toPortablePath(layout.configFile);
  validationEnv.SDKWORK_DATABASE_URL ??= defaultDatabaseUrlForLayout(layout);
  validationEnv.SDKWORK_ADMIN_SITE_DIR ??= toPortablePath(layout.adminSiteDistDir);
  validationEnv.SDKWORK_PORTAL_SITE_DIR ??= toPortablePath(layout.portalSiteDistDir);
  validationEnv.SDKWORK_ROUTER_BINARY ??= toPortablePath(layout.routerBinary);

  const binaryPath = validationEnv.SDKWORK_ROUTER_BINARY;
  if (binaryPath && exists(binaryPath)) {
    return {
      label: 'validate-config',
      command: binaryPath,
      args: ['--dry-run', '--plan-format', 'json'],
      cwd: layout.installRoot,
      env: validationEnv,
      shell: false,
      windowsHide: runtimePlatform === 'win32',
      installRoot: layout.installRoot,
      mode: normalizedMode,
      layout,
      source: 'binary',
    };
  }

  const rustRunner = resolveRustRunner(runtimePlatform, validationEnv);
  return {
    label: 'validate-config',
    command: rustRunner.command,
    args: [
      ...rustRunner.args,
      'run',
      '-p',
      'router-product-service',
      '--',
      '--dry-run',
      '--plan-format',
      'json',
    ],
    cwd: repoRoot,
    env: validationEnv,
    shell: rustRunner.shell,
    windowsHide: runtimePlatform === 'win32',
    installRoot: layout.installRoot,
    mode: normalizedMode,
    layout,
    source: 'cargo',
  };
}

export async function runCommandStep(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell ?? false,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`${step.label} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`${step.label} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

export async function executeReleaseBuildPlan(plan) {
  for (const step of plan.steps) {
    // eslint-disable-next-line no-await-in-loop
    await runCommandStep(step);
  }
}

export async function executeValidateConfigPlan(plan) {
  await runCommandStep(plan);
}

export function readTextFile(filePath) {
  return readFileSync(filePath, 'utf8');
}
