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
import { fileURLToPath } from 'node:url';

import { resolveDesktopReleaseTarget } from './desktop-targets.mjs';
import { buildNativeProductServerArchiveBaseName } from './package-release-assets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const HELM_IMAGE = 'alpine/helm:3.17.3';
const KUBECONFORM_IMAGE = 'ghcr.io/yannh/kubeconform:latest';
const DEFAULT_COMMAND_TIMEOUT_MS = 5 * 60 * 1000;

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
    `helm-render-smoke-${platform}-${arch}.json`,
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
  const result = spawnSync(command, args, {
    cwd,
    env,
    encoding: 'utf8',
    shell: false,
    timeout: timeoutMs,
  });

  if (!allowFailure && (result.error || result.status !== 0)) {
    throw buildCommandFailure(label, result);
  }

  return result;
}

function extractArchive(bundlePath, extractRoot) {
  runCommand('tar', ['-xzf', bundlePath, '-C', extractRoot], {
    label: 'extract Linux product bundle for Helm render',
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

function buildHelmValues() {
  return {
    databaseUrl: 'postgresql://sdkwork:sdkwork-release-smoke@postgres:5432/sdkwork_api_router',
    adminJwtSigningSecret: 'sdkwork-admin-release-smoke-secret',
    portalJwtSigningSecret: 'sdkwork-portal-release-smoke-secret',
    credentialMasterKey: 'sdkwork-credential-master-key-0001',
    metricsBearerToken: 'sdkwork-release-metrics-token',
    ingressEnabled: true,
  };
}

function buildHelmTemplateArgs(plan, bundleRoot) {
  return [
    'run',
    '--rm',
    '-v',
    `${bundleRoot}:/workspace`,
    '-w',
    '/workspace',
    HELM_IMAGE,
    'template',
    'sdkwork-api-router',
    plan.chartRelativePath,
    '--set-string',
    'image.repository=sdkwork-api-router-smoke',
    '--set-string',
    'image.tag=smoke',
    '--set-string',
    `secrets.databaseUrl=${plan.helmValues.databaseUrl}`,
    '--set-string',
    `secrets.adminJwtSigningSecret=${plan.helmValues.adminJwtSigningSecret}`,
    '--set-string',
    `secrets.portalJwtSigningSecret=${plan.helmValues.portalJwtSigningSecret}`,
    '--set-string',
    `secrets.credentialMasterKey=${plan.helmValues.credentialMasterKey}`,
    '--set-string',
    `secrets.metricsBearerToken=${plan.helmValues.metricsBearerToken}`,
    '--set',
    `ingress.enabled=${plan.helmValues.ingressEnabled ? 'true' : 'false'}`,
  ];
}

export function createLinuxHelmRenderSmokeOptions({
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
    throw new Error('run-linux-helm-render-smoke only supports linux release lanes');
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

  return createLinuxHelmRenderSmokeOptions({
    repoRoot: rootDir,
    ...options,
  });
}

export function createLinuxHelmRenderSmokePlan({
  repoRoot = rootDir,
  platform,
  arch,
  bundlePath,
  evidencePath,
} = {}) {
  const options = createLinuxHelmRenderSmokeOptions({
    repoRoot,
    platform,
    arch,
    bundlePath,
    evidencePath,
  });
  const renderedManifestPath = path.resolve(
    repoRoot,
    'artifacts',
    'release-smoke',
    `helm-render-${options.platform}-${options.arch}.yaml`,
  );

  return {
    ...options,
    chartRelativePath: 'deploy/helm/sdkwork-api-router',
    renderedManifestPath,
    renderedManifestRelativePath: toPortableRelativePath(repoRoot, renderedManifestPath),
    requiredTemplateKinds: ['Secret', 'Service', 'Deployment', 'Ingress'],
    helmValues: buildHelmValues(),
    helmImage: HELM_IMAGE,
    kubeconformImage: KUBECONFORM_IMAGE,
  };
}

export function createLinuxHelmRenderSmokeEvidence({
  repoRoot = rootDir,
  plan,
  ok,
  renderedKinds = [],
  kubeconformSummary = '',
  failure = null,
} = {}) {
  const evidence = {
    generatedAt: new Date().toISOString(),
    ok,
    platform: plan.platform,
    arch: plan.arch,
    bundlePath: toPortableRelativePath(repoRoot, plan.bundlePath),
    evidencePath: toPortableRelativePath(repoRoot, plan.evidencePath),
    renderedManifestPath: plan.renderedManifestRelativePath,
    renderedKinds,
  };

  if (kubeconformSummary) {
    evidence.kubeconformSummary = kubeconformSummary;
  }

  if (!ok) {
    evidence.failure = {
      message: failure instanceof Error ? failure.message : String(failure ?? 'unknown error'),
    };
  }

  return evidence;
}

function writeLinuxHelmRenderSmokeEvidence({
  evidencePath,
  evidence,
} = {}) {
  mkdirSync(path.dirname(evidencePath), { recursive: true });
  writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
}

function assertRequiredKinds(plan, manifestText) {
  const renderedKinds = [];

  for (const kind of plan.requiredTemplateKinds) {
    if (!new RegExp(`^kind:\\s*${kind}$`, 'm').test(manifestText)) {
      throw new Error(`rendered Helm manifest is missing required kind: ${kind}`);
    }

    renderedKinds.push(kind);
  }

  return renderedKinds;
}

export async function runLinuxHelmRenderSmoke({
  repoRoot = rootDir,
  platform,
  arch,
  bundlePath,
  evidencePath,
  env = process.env,
} = {}) {
  const plan = createLinuxHelmRenderSmokePlan({
    repoRoot,
    platform,
    arch,
    bundlePath,
    evidencePath,
  });

  if (!existsSync(plan.bundlePath)) {
    throw new Error(`missing packaged product bundle: ${plan.bundlePath}`);
  }

  const extractRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-helm-smoke-'));
  let bundleRoot = path.join(
    extractRoot,
    path.basename(plan.bundlePath).replace(/\.tar\.gz$/u, ''),
  );
  let failure = null;
  let renderedKinds = [];
  let kubeconformSummary = '';

  try {
    extractArchive(plan.bundlePath, extractRoot);
    bundleRoot = resolveExtractedBundleRoot({
      extractRoot,
      bundlePath: plan.bundlePath,
    });

    const templateResult = runCommand('docker', buildHelmTemplateArgs(plan, bundleRoot), {
      env,
      label: 'dockerized helm template',
    });

    mkdirSync(path.dirname(plan.renderedManifestPath), { recursive: true });
    writeFileSync(plan.renderedManifestPath, templateResult.stdout, 'utf8');

    const manifestText = readFileSync(plan.renderedManifestPath, 'utf8');
    renderedKinds = assertRequiredKinds(plan, manifestText);

    const kubeconformResult = runCommand(
      'docker',
      [
        'run',
        '--rm',
        '-v',
        `${path.dirname(plan.renderedManifestPath)}:/workspace`,
        KUBECONFORM_IMAGE,
        '-summary',
        `/workspace/${path.basename(plan.renderedManifestPath)}`,
      ],
      {
        env,
        label: 'dockerized kubeconform validation',
      },
    );
    kubeconformSummary = truncateText(
      `${kubeconformResult.stdout ?? ''}${kubeconformResult.stderr ?? ''}`,
      4000,
    );
  } catch (error) {
    failure = error instanceof Error ? error : new Error(String(error));
  }

  rmSync(extractRoot, { recursive: true, force: true });

  if (failure) {
    const evidence = createLinuxHelmRenderSmokeEvidence({
      repoRoot,
      plan,
      ok: false,
      renderedKinds,
      kubeconformSummary,
      failure,
    });
    writeLinuxHelmRenderSmokeEvidence({
      evidencePath: plan.evidencePath,
      evidence,
    });
    throw failure;
  }

  const evidence = createLinuxHelmRenderSmokeEvidence({
    repoRoot,
    plan,
    ok: true,
    renderedKinds,
    kubeconformSummary,
  });
  writeLinuxHelmRenderSmokeEvidence({
    evidencePath: plan.evidencePath,
    evidence,
  });
  return evidence;
}

async function main() {
  const options = parseArgs();
  const evidence = await runLinuxHelmRenderSmoke(options);
  console.log(JSON.stringify(evidence, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
