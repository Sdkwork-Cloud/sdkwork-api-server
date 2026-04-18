import { existsSync, readdirSync, rmSync, statSync } from 'node:fs';
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

function withWindowsNodeExecutableOnPath(env, platform, execPath = process.execPath) {
  if (platform !== 'win32') {
    return env;
  }

  const nodeExecutableDir = path.dirname(path.normalize(execPath));
  const pathKeys = Object.keys(env).filter((key) => key.toLowerCase() === 'path');
  const selectedPathKey = pathKeys[0] ?? 'PATH';
  const rawPathValue = pathKeys
    .map((key) => env[key])
    .find((value) => typeof value === 'string' && value.length > 0) ?? '';
  const pathEntries = String(rawPathValue)
    .split(path.delimiter)
    .map((entry) => entry.trim())
    .filter(Boolean);
  const normalizedNodeExecutableDir = nodeExecutableDir.toLowerCase();
  const dedupedPathEntries = pathEntries.filter(
    (entry, index, entries) => entries.findIndex(
      (candidate) => candidate.toLowerCase() === entry.toLowerCase(),
    ) === index,
  );

  if (!dedupedPathEntries.some((entry) => entry.toLowerCase() === normalizedNodeExecutableDir)) {
    dedupedPathEntries.unshift(nodeExecutableDir);
  }

  const nextEnv = { ...env };
  for (const key of pathKeys) {
    if (key !== selectedPathKey) {
      delete nextEnv[key];
    }
  }
  nextEnv[selectedPathKey] = dedupedPathEntries.join(path.delimiter);
  return nextEnv;
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
  execPath = process.execPath,
  cwd,
  stdio = 'inherit',
} = {}) {
  const effectiveEnv = withWindowsNodeExecutableOnPath(
    withWindowsNodeOptions(env, platform),
    platform,
    execPath,
  );
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

function collectMissingFrontendPackages(appRoot, requiredPackages) {
  const nodeModulesRoot = frontendNodeModulesRoot(appRoot);
  return requiredPackages.filter((packageName) => !existsSync(
    path.join(nodeModulesRoot, ...packagePathSegments(packageName), 'package.json'),
  ));
}

function collectMissingFrontendBinCommands(
  appRoot,
  requiredBinCommands,
  platform = process.platform,
) {
  if (!Array.isArray(requiredBinCommands) || requiredBinCommands.length === 0) {
    return [];
  }

  const binRoot = frontendBinRoot(appRoot);
  if (!existsSync(binRoot)) {
    return requiredBinCommands
      .map((commandName) => String(commandName ?? '').trim())
      .filter(Boolean);
  }

  const availableCommands = new Set(
    readdirSync(binRoot, { withFileTypes: true })
      .filter((entry) => entry.isFile() || entry.isSymbolicLink())
      .map((entry) => entry.name.toLowerCase()),
  );

  return requiredBinCommands.filter((commandName) => {
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

function truncateDiagnosticText(value, maxLength = 1200) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 15))}...[truncated]`;
}

function normalizeFrontendVerificationResult(result) {
  if (result == null || typeof result === 'boolean') {
    return {
      ok: Boolean(result),
      reason: '',
      stdout: '',
      stderr: '',
    };
  }

  if (typeof result !== 'object') {
    return {
      ok: Boolean(result),
      reason: '',
      stdout: '',
      stderr: '',
    };
  }

  const ok = result.ok === true || result.ready === true || result.status === 'ready';
  const reason = truncateDiagnosticText(
    result.reason ?? result.message ?? result.summary ?? '',
  );
  const stdout = truncateDiagnosticText(result.stdout ?? '');
  const stderr = truncateDiagnosticText(result.stderr ?? '');

  return {
    ok,
    reason,
    stdout,
    stderr,
  };
}

function formatFrontendInstallReport(report = {}) {
  const details = [];

  if (Array.isArray(report.missingPackages) && report.missingPackages.length > 0) {
    details.push(`missing packages: ${report.missingPackages.join(', ')}`);
  }

  if (Array.isArray(report.missingBinCommands) && report.missingBinCommands.length > 0) {
    details.push(`missing bin commands: ${report.missingBinCommands.join(', ')}`);
  }

  if (report.verify?.reason) {
    details.push(`verification: ${report.verify.reason}`);
  }

  if (report.verify?.stderr) {
    details.push(`verification stderr:\n${report.verify.stderr}`);
  }

  if (report.verify?.stdout) {
    details.push(`verification stdout:\n${report.verify.stdout}`);
  }

  return details.join('\n');
}

export function frontendInstallReport({
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
    return {
      status: 'missing',
      missingPackages: [...requiredPackages],
      missingBinCommands: [...requiredBinCommands],
      verify: null,
    };
  }

  if (!existsSync(path.join(nodeModulesRoot, '.modules.yaml'))) {
    return {
      status: 'missing',
      missingPackages: [...requiredPackages],
      missingBinCommands: [...requiredBinCommands],
      verify: null,
    };
  }

  const missingPackages = collectMissingFrontendPackages(appRoot, requiredPackages);
  if (missingPackages.length > 0) {
    return {
      status: 'missing',
      missingPackages,
      missingBinCommands: [],
      verify: null,
    };
  }

  const missingBinCommands = collectMissingFrontendBinCommands(
    appRoot,
    requiredBinCommands,
    platform,
  );
  if (missingBinCommands.length > 0) {
    return {
      status: 'unhealthy',
      missingPackages: [],
      missingBinCommands,
      verify: null,
    };
  }

  if (typeof verifyInstalled === 'function') {
    let verify;
    try {
      verify = normalizeFrontendVerificationResult(verifyInstalled({ appRoot }));
    } catch (error) {
      verify = {
        ok: false,
        reason: truncateDiagnosticText(error instanceof Error ? error.message : String(error)),
        stdout: '',
        stderr: '',
      };
    }

    if (!verify.ok) {
      return {
        status: 'unhealthy',
        missingPackages: [],
        missingBinCommands: [],
        verify,
      };
    }
  }

  return {
    status: 'ready',
    missingPackages: [],
    missingBinCommands: [],
    verify: null,
  };
}

export function frontendInstallStatus({
  appRoot,
  requiredPackages = [],
  requiredBinCommands = [],
  verifyInstalled = null,
  platform = process.platform,
} = {}) {
  return frontendInstallReport({
    appRoot,
    requiredPackages,
    requiredBinCommands,
    verifyInstalled,
    platform,
  }).status;
}

export function frontendInstallRequired(options = {}) {
  return frontendInstallStatus(options) !== 'ready';
}

export function strictFrontendInstallsEnabled(env = process.env) {
  const value = String(env.SDKWORK_STRICT_FRONTEND_INSTALLS ?? '').trim().toLowerCase();
  return value === '1' || value === 'true' || value === 'yes';
}

export function ensureFrontendDependenciesReady({
  appRoot,
  requiredPackages = [],
  requiredBinCommands = [],
  verifyInstalled = null,
  platform = process.platform,
  env = process.env,
  execPath = process.execPath,
  installStepArgs = null,
  spawnInstall = spawnSync,
} = {}) {
  if (!appRoot) {
    throw new Error('appRoot is required');
  }

  const installReport = frontendInstallReport({
    appRoot,
    requiredPackages,
    requiredBinCommands,
    verifyInstalled,
    platform,
  });
  const installStatus = installReport.status;

  if (installStatus === 'ready') {
    return installStatus;
  }

  if (strictFrontendInstallsEnabled(env)) {
    const reportDetails = formatFrontendInstallReport(installReport);
    throw new Error(
      `strict frontend install mode requires a prior frozen install step for ${appRoot}; current frontend install status is ${installStatus}${reportDetails ? `\n${reportDetails}` : ''}`,
    );
  }

  const stepArgs = Array.isArray(installStepArgs) && installStepArgs.length > 0
    ? installStepArgs
    : ['--dir', appRoot, 'install'];
  const installProcess = pnpmProcessSpec(stepArgs, {
    platform,
    execPath,
  });
  const result = spawnInstall(
    installProcess.command,
    installProcess.args,
    {
      ...pnpmSpawnOptions({
        platform,
        env,
        execPath,
      }),
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );

  if (result.error) {
    throw result.error;
  }

  if ((result.status ?? 1) !== 0) {
    throw new Error(`pnpm install exited with code ${result.status ?? 1} for ${appRoot}`);
  }

  return installStatus;
}

export function frontendDistReady(distDir = '') {
  if (!distDir || !existsSync(distDir)) {
    return false;
  }

  return existsSync(path.join(distDir, 'index.html'));
}

function latestTrackedMtime(targetPath) {
  if (!targetPath || !existsSync(targetPath)) {
    return 0;
  }

  const targetStat = statSync(targetPath);
  if (targetStat.isFile()) {
    return targetStat.mtimeMs;
  }

  if (!targetStat.isDirectory()) {
    return 0;
  }

  let latestMtime = 0;
  for (const entry of readdirSync(targetPath, { withFileTypes: true })) {
    const entryPath = path.join(targetPath, entry.name);
    if (entry.isDirectory()) {
      latestMtime = Math.max(latestMtime, latestTrackedMtime(entryPath));
      continue;
    }

    if (entry.isFile()) {
      latestMtime = Math.max(latestMtime, statSync(entryPath).mtimeMs);
    }
  }

  return latestMtime;
}

export function frontendDistUpToDate({
  appRoot,
  distDir = '',
  buildInputs = [],
} = {}) {
  if (!appRoot) {
    throw new Error('appRoot is required');
  }

  const resolvedDistDir = distDir || path.join(appRoot, 'dist');
  if (!frontendDistReady(resolvedDistDir)) {
    return false;
  }

  const distMtime = latestTrackedMtime(resolvedDistDir);
  if (distMtime <= 0) {
    return false;
  }

  const trackedInputs = Array.isArray(buildInputs) && buildInputs.length > 0
    ? buildInputs
    : ['index.html', 'package.json', 'tsconfig.json', 'vite.config.ts', 'src'];

  let latestInputMtime = 0;
  for (const inputPath of trackedInputs) {
    latestInputMtime = Math.max(
      latestInputMtime,
      latestTrackedMtime(path.join(appRoot, inputPath)),
    );
  }

  if (latestInputMtime <= 0) {
    return false;
  }

  return distMtime >= latestInputMtime;
}

export function frontendViteConfigHealthy({
  appRoot,
  command = 'serve',
  mode = 'development',
  env = process.env,
  platform = process.platform,
} = {}) {
  return checkFrontendViteConfig({
    appRoot,
    command,
    mode,
    env,
    platform,
  }).ok;
}

export function checkFrontendViteConfig({
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

  return {
    ok: result.status === 0,
    status: result.status ?? 1,
    stdout: String(result.stdout ?? ''),
    stderr: String(result.stderr ?? ''),
  };
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
