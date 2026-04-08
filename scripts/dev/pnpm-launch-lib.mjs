import { existsSync, readdirSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
const viteWindowsRealpathPreloadOption = `--import=${new URL(
  './vite-windows-realpath-preload.mjs',
  import.meta.url,
).href}`;

function withWindowsNodeOptions(env, platform) {
  if (platform !== 'win32') {
    return env;
  }

  const currentNodeOptions = String(env.NODE_OPTIONS ?? '').trim();
  if (currentNodeOptions.includes(viteWindowsRealpathPreloadOption)) {
    return env;
  }

  return {
    ...env,
    NODE_OPTIONS: currentNodeOptions
      ? `${currentNodeOptions} ${viteWindowsRealpathPreloadOption}`
      : viteWindowsRealpathPreloadOption,
  };
}

export function pnpmExecutable(platform = process.platform, execPath = process.execPath) {
  return platform === 'win32' ? execPath : 'pnpm';
}

export function pnpmArgumentPrefix({
  platform = process.platform,
  execPath = process.execPath,
} = {}) {
  if (platform !== 'win32') {
    return [];
  }

  const normalizedExecPath = path.normalize(execPath);
  return [path.join(path.dirname(normalizedExecPath), 'node_modules', 'pnpm', 'bin', 'pnpm.cjs')];
}

export function pnpmCommandArgs(stepArgs = [], options = {}) {
  return [...pnpmArgumentPrefix(options), ...stepArgs];
}

function quotePowerShellLiteral(value) {
  return `'${String(value).replaceAll("'", "''")}'`;
}

function windowsPnpmLauncherArgs(stepArgs = [], options = {}) {
  const command = pnpmExecutable('win32', options.execPath);
  const commandArgs = pnpmCommandArgs(stepArgs, {
    platform: 'win32',
    execPath: options.execPath,
  });
  const commandLine = ['&', quotePowerShellLiteral(command), ...commandArgs.map(quotePowerShellLiteral)].join(' ');

  return [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
    commandLine,
  ];
}

export function pnpmProcessSpec(stepArgs = [], {
  platform = process.platform,
  execPath = process.execPath,
} = {}) {
  if (platform === 'win32') {
    return {
      command: 'powershell.exe',
      args: windowsPnpmLauncherArgs(stepArgs, { execPath }),
    };
  }

  return {
    command: pnpmExecutable(platform, execPath),
    args: pnpmCommandArgs(stepArgs, { platform, execPath }),
  };
}

export function pnpmDisplayCommand(stepArgs = [], {
  platform = process.platform,
  execPath = process.execPath,
} = {}) {
  const command = pnpmExecutable(platform, execPath);
  const args = pnpmCommandArgs(stepArgs, { platform, execPath });
  return [command, ...args].join(' ');
}

export function pnpmSpawnOptions({
  platform = process.platform,
  env = process.env,
  cwd,
  stdio = 'inherit',
} = {}) {
  const effectiveEnv = withWindowsNodeOptions(env, platform);
  const options = {
    env: effectiveEnv,
    shell: false,
    stdio,
    windowsHide: platform === 'win32',
  };

  if (cwd) {
    options.cwd = cwd;
  }

  return options;
}

function packagePathSegments(packageName) {
  return packageName.split('/');
}

function frontendNodeModulesRoot(appRoot) {
  return path.join(appRoot, 'node_modules');
}

function frontendBinRoot(appRoot) {
  return path.join(appRoot, 'node_modules', '.bin');
}

function missingFrontendPackages(appRoot, requiredPackages) {
  const nodeModulesRoot = frontendNodeModulesRoot(appRoot);
  return requiredPackages.some((packageName) => !existsSync(
    path.join(nodeModulesRoot, ...packagePathSegments(packageName), 'package.json'),
  ));
}

function missingFrontendBinCommands(
  appRoot,
  requiredBinCommands,
  platform = process.platform,
) {
  if (!Array.isArray(requiredBinCommands) || requiredBinCommands.length === 0) {
    return false;
  }

  const binRoot = frontendBinRoot(appRoot);
  if (!existsSync(binRoot)) {
    return true;
  }

  const availableCommands = new Set(
    readdirSync(binRoot, { withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => entry.name.toLowerCase()),
  );

  return requiredBinCommands.some((commandName) => {
    const normalizedCommandName = String(commandName ?? '').trim().toLowerCase();
    if (!normalizedCommandName) {
      return false;
    }

    const expectedCommandName = platform === 'win32'
      ? `${normalizedCommandName}.cmd`
      : normalizedCommandName;

    return !availableCommands.has(expectedCommandName);
  });
}

export function frontendInstallStatus({
  appRoot,
  requiredPackages = [],
  requiredBinCommands = [],
  verifyInstalled = null,
  platform = process.platform,
} = {}) {
  if (!appRoot) {
    throw new Error('appRoot is required');
  }

  const nodeModulesRoot = frontendNodeModulesRoot(appRoot);
  if (!existsSync(nodeModulesRoot)) {
    return 'missing';
  }

  if (!existsSync(path.join(nodeModulesRoot, '.modules.yaml'))) {
    return 'missing';
  }

  if (missingFrontendPackages(appRoot, requiredPackages)) {
    return 'missing';
  }

  if (missingFrontendBinCommands(appRoot, requiredBinCommands, platform)) {
    return 'unhealthy';
  }

  if (typeof verifyInstalled === 'function' && !verifyInstalled({ appRoot })) {
    return 'unhealthy';
  }

  return 'ready';
}

export function frontendInstallRequired(options = {}) {
  return frontendInstallStatus(options) !== 'ready';
}

export function frontendDistReady(distDir = '') {
  if (!distDir || !existsSync(distDir)) {
    return false;
  }

  return existsSync(path.join(distDir, 'index.html'));
}

export function frontendViteConfigHealthy({
  appRoot,
  command = 'serve',
  mode = 'development',
  env = process.env,
  platform = process.platform,
} = {}) {
  if (!appRoot) {
    throw new Error('appRoot is required');
  }

  const checkScript = [
    'import path from "node:path";',
    'import { pathToFileURL } from "node:url";',
    'const viteModuleUrl = pathToFileURL(path.resolve("node_modules/vite/dist/node/index.js")).href;',
    'const { loadConfigFromFile } = await import(viteModuleUrl);',
    `const command = ${JSON.stringify(command)};`,
    `const mode = ${JSON.stringify(mode)};`,
    'const result = await loadConfigFromFile({ command, mode }, path.resolve("vite.config.ts"));',
    'if (!result) { process.exit(1); }',
  ].join(' ');

  const result = spawnSync(
    process.execPath,
    ['--input-type=module', '--eval', checkScript],
    {
      env: withWindowsNodeOptions(env, platform),
      cwd: appRoot,
      stdio: 'pipe',
      windowsHide: platform === 'win32',
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );

  return result.status === 0;
}

export function removeFrontendNodeModules(appRoot) {
  if (!appRoot) {
    throw new Error('appRoot is required');
  }

  const resolvedAppRoot = path.resolve(appRoot);
  const resolvedNodeModulesRoot = path.resolve(frontendNodeModulesRoot(appRoot));

  if (path.basename(resolvedNodeModulesRoot) !== 'node_modules' || path.dirname(resolvedNodeModulesRoot) !== resolvedAppRoot) {
    throw new Error(`refusing to remove unexpected node_modules path: ${resolvedNodeModulesRoot}`);
  }

  if (!existsSync(resolvedNodeModulesRoot)) {
    return;
  }

  rmSync(resolvedNodeModulesRoot, { recursive: true, force: true });
}

function normalizeCommandOutput(value) {
  if (value == null) {
    return '';
  }

  if (typeof value === 'string') {
    return value;
  }

  if (value instanceof Uint8Array) {
    return Buffer.from(value).toString('utf8');
  }

  return String(value);
}

export function shouldReuseExistingFrontendDist({
  platform = process.platform,
  stepArgs = [],
  status = 0,
  stdout = '',
  stderr = '',
  errorMessage = '',
  distReady = false,
  allowInstallReuse = false,
} = {}) {
  if (platform !== 'win32' || status === 0 || !distReady) {
    return false;
  }

  if (!Array.isArray(stepArgs)) {
    return false;
  }

  const isBuildStep = stepArgs.includes('build');
  const isReusableInstallStep = allowInstallReuse && stepArgs.includes('install');
  if (!isBuildStep && !isReusableInstallStep) {
    return false;
  }

  const combinedOutput = [
    normalizeCommandOutput(stdout),
    normalizeCommandOutput(stderr),
    normalizeCommandOutput(errorMessage),
  ].join('\n');

  return /spawn(?:sync)?[\s\S]*EPERM/i.test(combinedOutput);
}
