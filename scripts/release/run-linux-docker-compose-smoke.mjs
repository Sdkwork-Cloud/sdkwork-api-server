#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import { runBrowserRuntimeSmoke } from '../browser-runtime-smoke.mjs';
import { resolveDesktopReleaseTarget } from './desktop-targets.mjs';
import { buildNativeProductServerArchiveBaseName } from './package-release-assets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_HTTP_ATTEMPTS = 45;
const DEFAULT_HTTP_DELAY_MS = 2000;
const DEFAULT_POSTGRES_ATTEMPTS = 30;
const DEFAULT_POSTGRES_DELAY_MS = 2000;
const DEFAULT_BUILD_TIMEOUT_MS = 20 * 60 * 1000;
const DEFAULT_COMMAND_TIMEOUT_MS = 2 * 60 * 1000;
const DEFAULT_WEB_PORT = 3001;

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new Error(`${token} requires a value`);
  }

  return next;
}

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function toPortableRelativePath(repoRoot, targetPath) {
  return (path.relative(repoRoot, targetPath) || '.').replaceAll('\\', '/');
}

function sanitizeDockerImageTagComponent(value) {
  const sanitized = String(value ?? '')
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/gu, '-')
    .replace(/-{2,}/gu, '-')
    .replace(/^-+|-+$/gu, '');

  return sanitized || 'sdkwork-router-smoke';
}

export function resolveLinuxDockerComposeSmokeExecutionMode({
  hostPlatform = process.platform,
} = {}) {
  return hostPlatform === 'win32' ? 'docker-run' : 'docker-compose';
}

export function createLinuxDockerRunFallbackResources({
  composeProjectName,
} = {}) {
  if (!composeProjectName) {
    throw new Error('composeProjectName is required to build Docker fallback resources');
  }

  return {
    networkName: `${composeProjectName}-default`,
    postgresVolumeName: `${composeProjectName}-sdkwork-postgres`,
    postgresContainerName: `${composeProjectName}-postgres-1`,
    routerContainerName: `${composeProjectName}-router-1`,
    routerImageTag: `${sanitizeDockerImageTagComponent(`${composeProjectName}-router-smoke`)}:latest`,
  };
}

export function normalizeSpawnEnvironmentForPlatform({
  platform = process.platform,
  env = process.env,
} = {}) {
  if (platform !== 'win32') {
    return env;
  }

  const normalized = { ...env };
  const windowsPath = env.PATH || env.Path || '';

  delete normalized.PATH;
  delete normalized.Path;
  normalized.Path = windowsPath;

  return normalized;
}

export function normalizeDockerFallbackLogCapture({
  label,
  capturedOutput = '',
} = {}) {
  const content = String(capturedOutput ?? '').trim();
  if (!content) {
    return {
      content: '',
      diagnostic: '',
    };
  }

  if (/No such (container|object):/iu.test(content)) {
    return {
      content: '',
      diagnostic: `${label} logs were unavailable because the Docker host reported an object lookup miss while detached containers were otherwise observable via docker ps`,
    };
  }

  return {
    content,
    diagnostic: '',
  };
}

export function shouldContinueWithoutBrowserSmoke({
  hostPlatform = process.platform,
  env = process.env,
  error,
} = {}) {
  return hostPlatform === 'linux'
    && String(env.GITHUB_ACTIONS ?? '').toLowerCase() === 'true'
    && /unable to resolve a Chromium-based browser executable/i.test(
      String(error instanceof Error ? error.message : error ?? ''),
    );
}

export function createDockerRunLogEvidence({
  routerLogOutput = '',
  postgresLogOutput = '',
} = {}) {
  const routerLogCapture = normalizeDockerFallbackLogCapture({
    label: 'router',
    capturedOutput: routerLogOutput,
  });
  const postgresLogCapture = normalizeDockerFallbackLogCapture({
    label: 'postgres',
    capturedOutput: postgresLogOutput,
  });

  return {
    logs: {
      router: routerLogCapture.content,
      postgres: postgresLogCapture.content,
    },
    diagnostics: [
      routerLogCapture.diagnostic,
      postgresLogCapture.diagnostic,
    ].filter(Boolean),
  };
}

export function isCliEntrypoint({
  argv1 = process.argv[1] ?? '',
  moduleFile = __filename,
  platform = process.platform,
} = {}) {
  if (!argv1) {
    return false;
  }

  const resolvedArgv1 = path.resolve(argv1);
  const resolvedModuleFile = path.resolve(moduleFile);

  if (platform === 'win32') {
    return resolvedArgv1.toLowerCase() === resolvedModuleFile.toLowerCase();
  }

  return resolvedArgv1 === resolvedModuleFile;
}

function resolveBundlePath(repoRoot, bundlePath, { platform, arch }) {
  if (bundlePath) {
    return path.isAbsolute(bundlePath)
      ? bundlePath
      : path.resolve(repoRoot, bundlePath);
  }

  const archiveBaseName = buildNativeProductServerArchiveBaseName({
    platformId: platform,
    archId: arch,
  });

  return path.resolve(
    repoRoot,
    'artifacts',
    'release',
    'native',
    platform,
    arch,
    'bundles',
    `${archiveBaseName}.tar.gz`,
  );
}

function resolveEvidencePath(repoRoot, evidencePath, { platform, arch }) {
  if (evidencePath) {
    return path.isAbsolute(evidencePath)
      ? evidencePath
      : path.resolve(repoRoot, evidencePath);
  }

  return path.resolve(
    repoRoot,
    'artifacts',
    'release-governance',
    `docker-compose-smoke-${platform}-${arch}.json`,
  );
}

function buildCommandFailure(label, result) {
  const fragments = [];

  if (result?.error) {
    fragments.push(`error: ${result.error.message}`);
  }
  if (String(result?.stdout ?? '').trim()) {
    fragments.push(`stdout: ${truncateText(result.stdout)}`);
  }
  if (String(result?.stderr ?? '').trim()) {
    fragments.push(`stderr: ${truncateText(result.stderr)}`);
  }

  return new Error(
    `${label} failed with exit code ${result?.status ?? 'unknown'}${fragments.length > 0 ? `\n${fragments.join('\n')}` : ''}`,
  );
}

function runCommand(command, args, {
  cwd = rootDir,
  env = process.env,
  label = `${command} ${args.join(' ')}`,
  timeoutMs = DEFAULT_COMMAND_TIMEOUT_MS,
  allowFailure = false,
} = {}) {
  const spawnEnv = normalizeSpawnEnvironmentForPlatform({
    env,
  });
  const result = spawnSync(command, args, {
    cwd,
    env: spawnEnv,
    encoding: 'utf8',
    shell: false,
    timeout: timeoutMs,
  });

  if (!allowFailure && (result.error || result.status !== 0)) {
    throw buildCommandFailure(label, result);
  }

  return result;
}

async function assertHealthyResponse(url) {
  const response = await fetch(url, {
    signal: AbortSignal.timeout(5000),
  });
  const body = String(await response.text()).trim();

  if (!response.ok) {
    throw new Error(`${url} returned HTTP ${response.status}: ${truncateText(body, 400)}`);
  }

  if (body.length > 0 && body.toLowerCase() !== 'ok') {
    throw new Error(`${url} returned unexpected body: ${truncateText(body, 400)}`);
  }
}

async function assertSiteResponse(url) {
  const response = await fetch(url, {
    signal: AbortSignal.timeout(5000),
  });

  if (!response.ok) {
    throw new Error(`${url} returned HTTP ${response.status}`);
  }
}

async function waitForResponses(urls, assertion, label) {
  let lastError = null;

  for (let attempt = 0; attempt < DEFAULT_HTTP_ATTEMPTS; attempt += 1) {
    try {
      for (const url of urls) {
        // eslint-disable-next-line no-await-in-loop
        await assertion(url);
      }

      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (attempt + 1 >= DEFAULT_HTTP_ATTEMPTS) {
        break;
      }
      // eslint-disable-next-line no-await-in-loop
      await delay(DEFAULT_HTTP_DELAY_MS);
    }
  }

  throw new Error(
    `${label} did not stabilize after ${DEFAULT_HTTP_ATTEMPTS} attempts: ${lastError?.message ?? 'unknown error'}`,
  );
}

function extractArchive(bundlePath, extractRoot) {
  runCommand('tar', ['-xzf', bundlePath, '-C', extractRoot], {
    cwd: rootDir,
    label: 'extract Linux product bundle',
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });
}

export function resolveExtractedBundleRoot({
  extractRoot,
  bundlePath,
} = {}) {
  if (!extractRoot) {
    throw new Error('extractRoot is required');
  }
  if (!bundlePath) {
    throw new Error('bundlePath is required');
  }

  const expectedBundleRoot = path.join(
    extractRoot,
    path.basename(bundlePath).replace(/\.tar\.gz$/u, ''),
  );
  if (existsSync(expectedBundleRoot) && statSync(expectedBundleRoot).isDirectory()) {
    return expectedBundleRoot;
  }

  const extractedDirectories = readdirSync(extractRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(extractRoot, entry.name));

  if (extractedDirectories.length === 1) {
    return extractedDirectories[0];
  }

  throw new Error(
    `unable to resolve extracted product bundle root under ${extractRoot}; expected ${expectedBundleRoot}`,
  );
}

function captureComposeOutput(args, options) {
  const result = runCommand('docker', args, {
    ...options,
    allowFailure: true,
  });

  return truncateText(`${result.stdout ?? ''}${result.stderr ?? ''}`, 8000);
}

function captureDockerOutput(args, options) {
  const result = runCommand('docker', args, {
    ...options,
    allowFailure: true,
  });

  return truncateText(`${result.stdout ?? ''}${result.stderr ?? ''}`, 8000);
}

function pullRequiredImages(plan, {
  cwd = rootDir,
  env = process.env,
} = {}) {
  for (const imageRef of plan.requiredImageRefs ?? []) {
    runCommand('docker', ['pull', imageRef], {
      cwd,
      env,
      label: `docker pull ${imageRef}`,
      timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
    });
  }
}

function buildDockerRunProcessListingArgs(plan) {
  return [
    'ps',
    '-a',
    '--filter',
    `name=${plan.composeProjectName}-`,
    '--format',
    'table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}',
  ];
}

function buildDockerRunSidecarQueryArgs(plan, sql) {
  return [
    'run',
    '--rm',
    '--network',
    plan.fallbackResources.networkName,
    '-e',
    `PGPASSWORD=${plan.postgres.password}`,
    plan.postgres.imageRef,
    'psql',
    '-h',
    'postgres',
    '-U',
    plan.postgres.user,
    '-d',
    plan.postgres.database,
    '-At',
    '-c',
    sql,
  ];
}

function runDockerSidecarQuery(plan, sql, {
  cwd = rootDir,
  env = process.env,
  label = 'query postgres via sidecar',
  allowFailure = false,
  timeoutMs = DEFAULT_COMMAND_TIMEOUT_MS,
} = {}) {
  return runCommand('docker', buildDockerRunSidecarQueryArgs(plan, sql), {
    cwd,
    env,
    label,
    allowFailure,
    timeoutMs,
  });
}

async function waitForDockerRunPostgresReady(plan, {
  cwd = rootDir,
  env = process.env,
} = {}) {
  let lastError = null;

  for (let attempt = 0; attempt < DEFAULT_POSTGRES_ATTEMPTS; attempt += 1) {
    try {
      const result = runDockerSidecarQuery(plan, 'select 1;', {
        cwd,
        env,
        label: 'probe postgres via sidecar',
      });
      const output = String(result.stdout ?? '').trim();
      if (output !== '1') {
        throw new Error(`unexpected postgres probe output: ${truncateText(output, 120)}`);
      }

      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (attempt + 1 >= DEFAULT_POSTGRES_ATTEMPTS) {
        break;
      }

      // eslint-disable-next-line no-await-in-loop
      await delay(DEFAULT_POSTGRES_DELAY_MS);
    }
  }

  throw new Error(
    `postgres sidecar probe did not stabilize after ${DEFAULT_POSTGRES_ATTEMPTS} attempts: ${lastError?.message ?? 'unknown error'}`,
  );
}

function buildDockerRunRouterArgs(plan) {
  const routerEnvironmentEntries = Object.entries({
    SDKWORK_DATABASE_URL: `postgresql://${plan.postgres.user}:${plan.postgres.password}@postgres:5432/${plan.postgres.database}`,
    ...plan.routerEnvironment,
  }).flatMap(([key, value]) => ['-e', `${key}=${value}`]);

  return [
    'run',
    '-d',
    '--name',
    plan.fallbackResources.routerContainerName,
    '--network',
    plan.fallbackResources.networkName,
    '--network-alias',
    'router',
    '--restart',
    'unless-stopped',
    '-p',
    `${DEFAULT_WEB_PORT}:3001`,
    ...routerEnvironmentEntries,
    plan.fallbackResources.routerImageTag,
  ];
}

function buildDockerRunPostgresArgs(plan) {
  return [
    'run',
    '-d',
    '--name',
    plan.fallbackResources.postgresContainerName,
    '--network',
    plan.fallbackResources.networkName,
    '--network-alias',
    'postgres',
    '--restart',
    'unless-stopped',
    '-e',
    `POSTGRES_DB=${plan.postgres.database}`,
    '-e',
    `POSTGRES_USER=${plan.postgres.user}`,
    '-e',
    `POSTGRES_PASSWORD=${plan.postgres.password}`,
    '-v',
    `${plan.fallbackResources.postgresVolumeName}:/var/lib/postgresql/data`,
    '--health-cmd',
    'pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"',
    '--health-interval',
    '10s',
    '--health-timeout',
    '5s',
    '--health-retries',
    '10',
    plan.postgres.imageRef,
  ];
}

async function cleanupDockerRunFallback(plan, {
  cwd = rootDir,
  env = process.env,
} = {}) {
  runCommand('docker', [
    'rm',
    '-f',
    plan.fallbackResources.routerContainerName,
    plan.fallbackResources.postgresContainerName,
  ], {
    cwd,
    env,
    label: 'cleanup Docker fallback containers',
    allowFailure: true,
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });

  // Docker Desktop may take a moment to detach ephemeral network endpoints.
  await delay(1500);

  runCommand('docker', [
    'network',
    'rm',
    plan.fallbackResources.networkName,
  ], {
    cwd,
    env,
    label: 'cleanup Docker fallback network',
    allowFailure: true,
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });

  runCommand('docker', [
    'volume',
    'rm',
    '-f',
    plan.fallbackResources.postgresVolumeName,
  ], {
    cwd,
    env,
    label: 'cleanup Docker fallback postgres volume',
    allowFailure: true,
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });

  runCommand('docker', [
    'image',
    'rm',
    plan.fallbackResources.routerImageTag,
  ], {
    cwd,
    env,
    label: 'cleanup Docker fallback router image',
    allowFailure: true,
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });
}

function createBundleSmokeSecrets() {
  return {
    postgresDb: 'sdkwork_api_router',
    postgresUser: 'sdkwork',
    postgresPassword: 'sdkwork-release-smoke',
    bootstrapProfile: 'prod',
    adminJwtSigningSecret: 'sdkwork-admin-release-smoke-secret',
    portalJwtSigningSecret: 'sdkwork-portal-release-smoke-secret',
    credentialMasterKey: 'sdkwork-credential-master-key-0001',
    metricsBearerToken: 'sdkwork-release-metrics-token',
    browserAllowedOrigins: `http://localhost:${DEFAULT_WEB_PORT}`,
  };
}

export function createLinuxDockerComposeSmokeOptions({
  repoRoot = rootDir,
  platform = process.platform,
  arch = process.arch,
  bundlePath = '',
  evidencePath = '',
} = {}) {
  const resolvedTarget = resolveDesktopReleaseTarget({
    platform,
    arch,
  });

  if (resolvedTarget.platform !== 'linux') {
    throw new Error('run-linux-docker-compose-smoke only supports linux release lanes');
  }

  return {
    platform: resolvedTarget.platform,
    arch: resolvedTarget.arch,
    bundlePath: resolveBundlePath(repoRoot, bundlePath, resolvedTarget),
    evidencePath: resolveEvidencePath(repoRoot, evidencePath, resolvedTarget),
  };
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    platform: '',
    arch: '',
    bundlePath: '',
    evidencePath: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--platform') {
      options.platform = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--arch') {
      options.arch = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--bundle-path') {
      options.bundlePath = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--evidence-path') {
      options.evidencePath = readOptionValue(token, next);
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!options.platform) {
    throw new Error('--platform is required');
  }
  if (!options.arch) {
    throw new Error('--arch is required');
  }

  return createLinuxDockerComposeSmokeOptions({
    repoRoot: rootDir,
    ...options,
  });
}

export function createLinuxDockerComposeSmokePlan({
  repoRoot = rootDir,
  hostPlatform = process.platform,
  platform,
  arch,
  bundlePath,
  evidencePath,
} = {}) {
  const options = createLinuxDockerComposeSmokeOptions({
    repoRoot,
    platform,
    arch,
    bundlePath,
    evidencePath,
  });
  const secrets = createBundleSmokeSecrets();
  const composeProjectName = `sdkwork-release-smoke-${options.platform}-${options.arch}`;
  const executionMode = resolveLinuxDockerComposeSmokeExecutionMode({
    hostPlatform,
  });

  return {
    ...options,
    executionMode,
    composeProjectName,
    composeRelativePath: 'deploy/docker/docker-compose.yml',
    envRelativePath: 'deploy/docker/.env',
    overrideRelativePath: 'deploy/docker/docker-compose.smoke.override.yml',
    fallbackResources: createLinuxDockerRunFallbackResources({
      composeProjectName,
    }),
    requiredImageRefs: ['docker.io/library/postgres:16-alpine'],
    healthUrls: [
      `http://127.0.0.1:${DEFAULT_WEB_PORT}/api/v1/health`,
      `http://127.0.0.1:${DEFAULT_WEB_PORT}/api/admin/health`,
      `http://127.0.0.1:${DEFAULT_WEB_PORT}/api/portal/health`,
    ],
    siteUrls: [
      `http://127.0.0.1:${DEFAULT_WEB_PORT}/admin/`,
      `http://127.0.0.1:${DEFAULT_WEB_PORT}/portal/`,
    ],
    browserSmokeTargets: [
      {
        label: 'admin',
        url: `http://127.0.0.1:${DEFAULT_WEB_PORT}/admin/`,
        expectedTexts: [],
        expectedSelectors: [
          'input[type="email"]',
          'input[type="password"]',
          'button[type="submit"]',
        ],
      },
      {
        label: 'portal',
        url: `http://127.0.0.1:${DEFAULT_WEB_PORT}/portal/`,
        expectedTexts: [
          'Unified AI gateway workspace',
          'Operate routing, credentials, usage, and downloads from one product surface.',
        ],
        expectedSelectors: [
          '[data-slot="portal-home-page"]',
          '[data-slot="portal-home-metrics"]',
        ],
      },
    ],
    envContents: [
      `SDKWORK_POSTGRES_DB=${secrets.postgresDb}`,
      `SDKWORK_POSTGRES_USER=${secrets.postgresUser}`,
      `SDKWORK_POSTGRES_PASSWORD=${secrets.postgresPassword}`,
      `SDKWORK_BOOTSTRAP_PROFILE=${secrets.bootstrapProfile}`,
      `SDKWORK_ADMIN_JWT_SIGNING_SECRET=${secrets.adminJwtSigningSecret}`,
      `SDKWORK_PORTAL_JWT_SIGNING_SECRET=${secrets.portalJwtSigningSecret}`,
      `SDKWORK_CREDENTIAL_MASTER_KEY=${secrets.credentialMasterKey}`,
      `SDKWORK_METRICS_BEARER_TOKEN=${secrets.metricsBearerToken}`,
      `SDKWORK_BROWSER_ALLOWED_ORIGINS=${secrets.browserAllowedOrigins}`,
      '',
    ].join('\n'),
    overrideContents: [
      'services:',
      '  router:',
      '    ports:',
      `      - "${DEFAULT_WEB_PORT}:3001"`,
      '',
    ].join('\n'),
    routerEnvironment: {
      SDKWORK_BOOTSTRAP_PROFILE: secrets.bootstrapProfile,
      SDKWORK_BOOTSTRAP_DATA_DIR: '/opt/sdkwork/data',
      SDKWORK_ADMIN_SITE_DIR: '/opt/sdkwork/sites/admin/dist',
      SDKWORK_PORTAL_SITE_DIR: '/opt/sdkwork/sites/portal/dist',
      SDKWORK_WEB_BIND: '0.0.0.0:3001',
      SDKWORK_GATEWAY_BIND: '127.0.0.1:8080',
      SDKWORK_ADMIN_BIND: '127.0.0.1:8081',
      SDKWORK_PORTAL_BIND: '127.0.0.1:8082',
      SDKWORK_ADMIN_JWT_SIGNING_SECRET: secrets.adminJwtSigningSecret,
      SDKWORK_PORTAL_JWT_SIGNING_SECRET: secrets.portalJwtSigningSecret,
      SDKWORK_CREDENTIAL_MASTER_KEY: secrets.credentialMasterKey,
      SDKWORK_METRICS_BEARER_TOKEN: secrets.metricsBearerToken,
      SDKWORK_BROWSER_ALLOWED_ORIGINS: secrets.browserAllowedOrigins,
    },
    databaseAssertions: [
      { table: 'ai_channel', minimumCount: 1 },
      { table: 'ai_proxy_provider', minimumCount: 1 },
      { table: 'ai_marketing_coupon_template', minimumCount: 1 },
      { table: 'ai_marketing_campaign', minimumCount: 1 },
      { table: 'ai_marketing_coupon_code', minimumCount: 1 },
    ],
    postgres: {
      imageRef: 'docker.io/library/postgres:16-alpine',
      database: secrets.postgresDb,
      user: secrets.postgresUser,
      password: secrets.postgresPassword,
    },
  };
}

export function createLinuxDockerComposeSmokeEvidence({
  repoRoot = rootDir,
  plan,
  ok,
  databaseAssertions = [],
  browserSmokeResults = [],
  composePs = '',
  logs = {},
  diagnostics = [],
  failure = null,
} = {}) {
  const evidence = {
    generatedAt: new Date().toISOString(),
    ok,
    platform: plan.platform,
    arch: plan.arch,
    executionMode: plan.executionMode,
    bundlePath: toPortableRelativePath(repoRoot, plan.bundlePath),
    evidencePath: toPortableRelativePath(repoRoot, plan.evidencePath),
    healthUrls: plan.healthUrls,
    siteUrls: plan.siteUrls,
    browserSmokeTargets: plan.browserSmokeTargets,
    databaseAssertions,
  };

  if (Array.isArray(browserSmokeResults) && browserSmokeResults.length > 0) {
    evidence.browserSmokeResults = browserSmokeResults;
  }

  if (composePs) {
    evidence.composePs = composePs;
  }

  const sanitizedLogs = Object.fromEntries(
    Object.entries(logs).filter(([, value]) => String(value ?? '').trim().length > 0),
  );
  if (Object.keys(sanitizedLogs).length > 0) {
    evidence.logs = sanitizedLogs;
  }

  const sanitizedDiagnostics = diagnostics
    .map((entry) => String(entry ?? '').trim())
    .filter((entry, index, collection) => entry.length > 0 && collection.indexOf(entry) === index);
  if (sanitizedDiagnostics.length > 0) {
    evidence.diagnostics = sanitizedDiagnostics;
  }

  if (!ok) {
    evidence.failure = {
      message: failure instanceof Error ? failure.message : String(failure ?? 'unknown error'),
    };
  }

  return evidence;
}

function writeLinuxDockerComposeSmokeEvidence({
  evidencePath,
  evidence,
} = {}) {
  mkdirSync(path.dirname(evidencePath), { recursive: true });
  writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
}

function buildDockerComposeArgs(plan, ...composeArgs) {
  return [
    'compose',
    '--project-name',
    plan.composeProjectName,
    '-f',
    'docker-compose.yml',
    '-f',
    'docker-compose.smoke.override.yml',
    ...composeArgs,
  ];
}

function queryTableCount(plan, composeCwd, env, table) {
  const sql = `select count(*) from ${table};`;
  const result = runCommand(
    'docker',
    buildDockerComposeArgs(
      plan,
      'exec',
      '-T',
      'postgres',
      'psql',
      '-U',
      plan.postgres.user,
      '-d',
      plan.postgres.database,
      '-At',
      '-c',
      sql,
    ),
    {
      cwd: composeCwd,
      env,
      label: `query bootstrap table ${table}`,
    },
  );

  const count = Number.parseInt(String(result.stdout ?? '').trim(), 10);
  if (!Number.isInteger(count)) {
    throw new Error(`failed to parse bootstrap count for ${table}: ${truncateText(result.stdout, 200)}`);
  }

  return count;
}

function queryTableCountViaDockerRun(plan, bundleRoot, env, table) {
  const sql = `select count(*) from ${table};`;
  const result = runDockerSidecarQuery(plan, sql, {
    cwd: bundleRoot,
    env,
    label: `query bootstrap table ${table} via sidecar`,
  });

  const count = Number.parseInt(String(result.stdout ?? '').trim(), 10);
  if (!Number.isInteger(count)) {
    throw new Error(`failed to parse bootstrap count for ${table}: ${truncateText(result.stdout, 200)}`);
  }

  return count;
}

async function runBrowserSmokeTargets(plan, {
  env = process.env,
  hostPlatform = process.platform,
} = {}) {
  const browserSmokeResults = [];
  const diagnostics = [];
  let skipBrowserSmokeReason = '';

  for (const target of plan.browserSmokeTargets) {
    if (skipBrowserSmokeReason) {
      browserSmokeResults.push({
        label: target.label,
        url: target.url,
        expectedTexts: target.expectedTexts ?? [],
        expectedSelectors: target.expectedSelectors ?? [],
        skipped: true,
        reason: skipBrowserSmokeReason,
      });
      continue;
    }

    try {
      // eslint-disable-next-line no-await-in-loop
      const result = await runBrowserRuntimeSmoke({
        url: target.url,
        expectedTexts: target.expectedTexts,
        expectedSelectors: target.expectedSelectors,
        env,
        platform: hostPlatform,
      });
      browserSmokeResults.push({
        label: target.label,
        ...result,
      });
    } catch (error) {
      if (!shouldContinueWithoutBrowserSmoke({
        hostPlatform,
        env,
        error,
      })) {
        throw error;
      }

      skipBrowserSmokeReason = String(
        error instanceof Error ? error.message : error ?? 'missing Chromium browser runtime',
      );
      diagnostics.push(
        'browser runtime smoke skipped because hosted Linux CI did not provide a Chromium-based browser executable',
      );
      browserSmokeResults.push({
        label: target.label,
        url: target.url,
        expectedTexts: target.expectedTexts ?? [],
        expectedSelectors: target.expectedSelectors ?? [],
        skipped: true,
        reason: skipBrowserSmokeReason,
      });
    }
  }

  return {
    browserSmokeResults,
    diagnostics,
  };
}

function collectDatabaseAssertions(plan, queryCount) {
  return plan.databaseAssertions.map((assertion) => {
    const count = queryCount(assertion.table);
    if (count < assertion.minimumCount) {
      throw new Error(`bootstrap table ${assertion.table} only has ${count} rows`);
    }

    return {
      ...assertion,
      count,
    };
  });
}

async function runLinuxDockerComposeSmokeViaCompose({
  plan,
  bundleRoot,
  env,
} = {}) {
  const composeCwd = path.join(bundleRoot, 'deploy', 'docker');
  const envFilePath = path.join(composeCwd, '.env');
  const overrideFilePath = path.join(composeCwd, 'docker-compose.smoke.override.yml');

  writeFileSync(envFilePath, plan.envContents, 'utf8');
  writeFileSync(overrideFilePath, plan.overrideContents, 'utf8');

  runCommand('docker', buildDockerComposeArgs(plan, 'version'), {
    cwd: composeCwd,
    env,
    label: 'docker compose version',
  });
  runCommand('docker', buildDockerComposeArgs(plan, 'down', '-v', '--remove-orphans'), {
    cwd: composeCwd,
    env,
    label: 'pre-clean docker compose workspace',
    allowFailure: true,
  });
  pullRequiredImages(plan, {
    cwd: composeCwd,
    env,
  });
  runCommand('docker', buildDockerComposeArgs(plan, 'up', '-d', '--build'), {
    cwd: composeCwd,
    env,
    label: 'docker compose smoke up',
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });

  await waitForResponses(plan.healthUrls, assertHealthyResponse, 'health checks');
  await waitForResponses(plan.siteUrls, assertSiteResponse, 'site checks');
  const browserSmoke = await runBrowserSmokeTargets(plan, {
    env,
  });

  return {
    composeCwd,
    browserSmokeResults: browserSmoke.browserSmokeResults,
    databaseAssertions: collectDatabaseAssertions(
      plan,
      (table) => queryTableCount(plan, composeCwd, env, table),
    ),
    diagnostics: browserSmoke.diagnostics,
    composePs: captureComposeOutput(
      buildDockerComposeArgs(plan, 'ps'),
      { cwd: composeCwd, env },
    ),
    logs: {
      router: captureComposeOutput(
        buildDockerComposeArgs(plan, 'logs', 'router', '--tail', '200'),
        { cwd: composeCwd, env },
      ),
      postgres: captureComposeOutput(
        buildDockerComposeArgs(plan, 'logs', 'postgres', '--tail', '100'),
        { cwd: composeCwd, env },
      ),
    },
  };
}

async function runLinuxDockerComposeSmokeViaDockerRun({
  plan,
  bundleRoot,
  env,
} = {}) {
  const composeCwd = path.join(bundleRoot, 'deploy', 'docker');

  runCommand('docker', ['version'], {
    cwd: composeCwd,
    env,
    label: 'docker version',
  });
  await cleanupDockerRunFallback(plan, { cwd: composeCwd, env });
  pullRequiredImages(plan, {
    cwd: composeCwd,
    env,
  });
  runCommand('docker', [
    'build',
    '-t',
    plan.fallbackResources.routerImageTag,
    '-f',
    'deploy/docker/Dockerfile',
    '.',
  ], {
    cwd: bundleRoot,
    env,
    label: 'docker build router fallback image',
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });
  runCommand('docker', [
    'network',
    'create',
    plan.fallbackResources.networkName,
  ], {
    cwd: composeCwd,
    env,
    label: 'create Docker fallback network',
  });
  runCommand('docker', [
    'volume',
    'create',
    plan.fallbackResources.postgresVolumeName,
  ], {
    cwd: composeCwd,
    env,
    label: 'create Docker fallback postgres volume',
  });
  runCommand('docker', buildDockerRunPostgresArgs(plan), {
    cwd: composeCwd,
    env,
    label: 'start Docker fallback postgres',
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });
  await waitForDockerRunPostgresReady(plan, {
    cwd: composeCwd,
    env,
  });
  runCommand('docker', buildDockerRunRouterArgs(plan), {
    cwd: composeCwd,
    env,
    label: 'start Docker fallback router',
    timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
  });

  await waitForResponses(plan.healthUrls, assertHealthyResponse, 'health checks');
  await waitForResponses(plan.siteUrls, assertSiteResponse, 'site checks');
  const browserSmoke = await runBrowserSmokeTargets(plan, {
    env,
  });
  const logEvidence = createDockerRunLogEvidence({
    routerLogOutput: captureDockerOutput(
      ['logs', '--tail', '200', plan.fallbackResources.routerContainerName],
      { cwd: composeCwd, env },
    ),
    postgresLogOutput: captureDockerOutput(
      ['logs', '--tail', '100', plan.fallbackResources.postgresContainerName],
      { cwd: composeCwd, env },
    ),
  });

  return {
    composeCwd,
    browserSmokeResults: browserSmoke.browserSmokeResults,
    databaseAssertions: collectDatabaseAssertions(
      plan,
      (table) => queryTableCountViaDockerRun(plan, composeCwd, env, table),
    ),
    composePs: captureDockerOutput(
      buildDockerRunProcessListingArgs(plan),
      { cwd: composeCwd, env },
    ),
    logs: logEvidence.logs,
    diagnostics: [
      ...logEvidence.diagnostics,
      ...browserSmoke.diagnostics,
    ],
  };
}

export async function runLinuxDockerComposeSmoke({
  repoRoot = rootDir,
  platform,
  arch,
  bundlePath,
  evidencePath,
  env = process.env,
} = {}) {
  const plan = createLinuxDockerComposeSmokePlan({
    repoRoot,
    platform,
    arch,
    bundlePath,
    evidencePath,
  });

  if (!existsSync(plan.bundlePath)) {
    throw new Error(`missing packaged product bundle: ${plan.bundlePath}`);
  }

  const extractRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-docker-smoke-'));
  let bundleRoot = path.join(
    extractRoot,
    path.basename(plan.bundlePath).replace(/\.tar\.gz$/u, ''),
  );
  let failure = null;
  let composePs = '';
  let logs = {};
  let diagnostics = [];
  let databaseAssertions = [];
  let browserSmokeResults = [];
  let composeCwd = path.join(bundleRoot, 'deploy', 'docker');

  try {
    extractArchive(plan.bundlePath, extractRoot);
    bundleRoot = resolveExtractedBundleRoot({
      extractRoot,
      bundlePath: plan.bundlePath,
    });
    composeCwd = path.join(bundleRoot, 'deploy', 'docker');
    const result = plan.executionMode === 'docker-run'
      ? await runLinuxDockerComposeSmokeViaDockerRun({
        plan,
        bundleRoot,
        env,
      })
      : await runLinuxDockerComposeSmokeViaCompose({
        plan,
        bundleRoot,
        env,
      });
    composePs = result.composePs;
    logs = result.logs;
    diagnostics = result.diagnostics ?? [];
    databaseAssertions = result.databaseAssertions;
    browserSmokeResults = result.browserSmokeResults;
  } catch (error) {
    failure = error instanceof Error ? error : new Error(String(error));
  }

  if (existsSync(composeCwd)) {
    if (!composePs && plan.executionMode === 'docker-compose') {
      composePs = captureComposeOutput(
        buildDockerComposeArgs(plan, 'ps'),
        { cwd: composeCwd, env },
      );
    }
    if (!composePs && plan.executionMode === 'docker-run') {
      composePs = captureDockerOutput(
        buildDockerRunProcessListingArgs(plan),
        { cwd: composeCwd, env },
      );
    }
    if (!logs.router && plan.executionMode === 'docker-compose') {
      logs.router = captureComposeOutput(
        buildDockerComposeArgs(plan, 'logs', 'router', '--tail', '200'),
        { cwd: composeCwd, env },
      );
    }
    if (!logs.router && plan.executionMode === 'docker-run') {
      const logEvidence = createDockerRunLogEvidence({
        routerLogOutput: captureDockerOutput(
          ['logs', '--tail', '200', plan.fallbackResources.routerContainerName],
          { cwd: composeCwd, env },
        ),
        postgresLogOutput: !logs.postgres
          ? captureDockerOutput(
          ['logs', '--tail', '100', plan.fallbackResources.postgresContainerName],
          { cwd: composeCwd, env },
        )
          : '',
      });
      logs.router = logEvidence.logs.router;
      if (!logs.postgres) {
        logs.postgres = logEvidence.logs.postgres;
      }
      diagnostics.push(...logEvidence.diagnostics);
    }
    if (!logs.postgres && plan.executionMode === 'docker-compose') {
      logs.postgres = captureComposeOutput(
        buildDockerComposeArgs(plan, 'logs', 'postgres', '--tail', '100'),
        { cwd: composeCwd, env },
      );
    }
    if (!logs.postgres && plan.executionMode === 'docker-run') {
      const logEvidence = createDockerRunLogEvidence({
        postgresLogOutput: captureDockerOutput(
          ['logs', '--tail', '100', plan.fallbackResources.postgresContainerName],
          { cwd: composeCwd, env },
        ),
      });
      logs.postgres = logEvidence.logs.postgres;
      diagnostics.push(...logEvidence.diagnostics);
    }

    if (plan.executionMode === 'docker-run') {
      await cleanupDockerRunFallback(plan, {
        cwd: composeCwd,
        env,
      });
    } else {
      runCommand('docker', buildDockerComposeArgs(plan, 'down', '-v', '--remove-orphans'), {
        cwd: composeCwd,
        env,
        label: 'docker compose smoke down',
        allowFailure: true,
        timeoutMs: DEFAULT_BUILD_TIMEOUT_MS,
      });
    }
  }

  rmSync(extractRoot, { recursive: true, force: true });

  if (failure) {
    const evidence = createLinuxDockerComposeSmokeEvidence({
      repoRoot,
      plan,
      ok: false,
      databaseAssertions,
      composePs,
      logs,
      diagnostics,
      browserSmokeResults,
      failure,
    });
    writeLinuxDockerComposeSmokeEvidence({
      evidencePath: plan.evidencePath,
      evidence,
    });
    throw failure;
  }

  const evidence = createLinuxDockerComposeSmokeEvidence({
    repoRoot,
    plan,
    ok: true,
    databaseAssertions,
    composePs,
    logs,
    diagnostics,
    browserSmokeResults,
  });
  writeLinuxDockerComposeSmokeEvidence({
    evidencePath: plan.evidencePath,
    evidence,
  });
  return evidence;
}

async function main() {
  const options = parseArgs();
  const evidence = await runLinuxDockerComposeSmoke(options);
  console.log(JSON.stringify(evidence, null, 2));
}

if (isCliEntrypoint()) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
