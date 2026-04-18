#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import net from 'node:net';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import {
  createMockFetchSetupScript,
  runBrowserRuntimeSmoke,
} from './browser-runtime-smoke.mjs';
import {
  checkFrontendViteConfig,
  ensureFrontendDependenciesReady,
  pnpmSpawnOptions,
} from './dev/pnpm-launch-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEFAULT_TIMEOUT_MS = 45_000;
const PORTAL_EXPECTED_TEXTS = [
  'Unified AI gateway workspace',
  'Operate routing, credentials, usage, and downloads from one product surface.',
];
const PORTAL_EXPECTED_SELECTORS = [
  '[data-slot="portal-home-page"]',
  '[data-slot="portal-home-metrics"]',
];
const PORTAL_SESSION_TOKEN_KEY = 'sdkwork.router.portal.session-token';
const PORTAL_UNSAFE_ACCOUNT_ID = '1950809575122113173';
const PORTAL_FORBIDDEN_ACCOUNT_ID = '1950809575122113300';
const PORTAL_USER = {
  id: 'portal-user-1',
  email: 'workspace@example.com',
  display_name: 'Workspace Operator',
  workspace_tenant_id: 'tenant-portal-1',
  workspace_project_id: 'project-portal-1',
  active: true,
  created_at_ms: 1_710_000_000_000,
};
const PORTAL_WORKSPACE = {
  user: PORTAL_USER,
  tenant: {
    id: 'tenant-portal-1',
    name: 'Acme Tenant',
  },
  project: {
    tenant_id: 'tenant-portal-1',
    id: 'project-portal-1',
    name: 'Acme Project',
  },
};
const PORTAL_USAGE_SUMMARY = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};
const PORTAL_BILLING_SUMMARY = {
  project_id: 'project-portal-1',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  quota_policy_id: null,
  quota_limit_units: null,
  remaining_units: null,
  exhausted: false,
};
const PORTAL_BILLING_EVENT_SUMMARY = {
  total_events: 0,
  project_count: 0,
  group_count: 0,
  capability_count: 0,
  total_request_count: 0,
  total_units: 0,
  total_input_tokens: 0,
  total_output_tokens: 0,
  total_tokens: 0,
  total_image_count: 0,
  total_audio_seconds: 0,
  total_video_seconds: 0,
  total_music_seconds: 0,
  total_upstream_cost: 0,
  total_customer_charge: 0,
  projects: [],
  groups: [],
  capabilities: [],
  accounting_modes: [],
};

function createPortalCommercialAccount() {
  return {
    account: {
      account_id: PORTAL_UNSAFE_ACCOUNT_ID,
      tenant_id: 'tenant-portal-1',
      organization_id: 'org-portal-1',
      user_id: 'portal-user-1',
      account_type: 'primary',
      currency_code: 'USD',
      credit_unit_code: 'credit',
      status: 'active',
      allow_overdraft: false,
      overdraft_limit: 0,
      created_at_ms: 1_710_000_100_000,
      updated_at_ms: 1_710_000_100_500,
    },
    available_balance: 3_200,
    held_balance: 120,
    consumed_balance: 480,
    grant_balance: 0,
    active_lot_count: 1,
  };
}

function createPortalCommercialBalance() {
  return {
    account_id: PORTAL_UNSAFE_ACCOUNT_ID,
    available_balance: 3_200,
    held_balance: 120,
    consumed_balance: 480,
    grant_balance: 0,
    active_lot_count: 1,
    lots: [],
  };
}

function createPortalCommercialSettlements() {
  return [{
    request_settlement_id: '720000000000000001',
    tenant_id: 'tenant-portal-1',
    organization_id: 'org-portal-1',
    request_id: '820000000000000001',
    account_id: PORTAL_UNSAFE_ACCOUNT_ID,
    user_id: 'portal-user-1',
    hold_id: '620000000000000001',
    status: 'captured',
    estimated_credit_hold: 120,
    released_credit_amount: 0,
    captured_credit_amount: 120,
    provider_cost_amount: 82,
    retail_charge_amount: 120,
    shortfall_amount: 0,
    refunded_amount: 0,
    settled_at_ms: 1_710_000_200_000,
    created_at_ms: 1_710_000_195_000,
    updated_at_ms: 1_710_000_200_000,
  }];
}

function createPortalCommercialHistorySnapshot() {
  const commercialAccount = createPortalCommercialAccount();

  return {
    account: commercialAccount.account,
    balance: createPortalCommercialBalance(),
    benefit_lots: [],
    holds: [],
    request_settlements: createPortalCommercialSettlements(),
    ledger: [],
  };
}

function createPortalCommercialRuntimeSetupSource() {
  return createMockFetchSetupScript({
    localStorageEntries: {
      [PORTAL_SESSION_TOKEN_KEY]: 'mock-portal-session-token',
    },
    exactResponses: {
      '/api/portal/auth/me': PORTAL_USER,
      '/api/portal/workspace': PORTAL_WORKSPACE,
      '/api/portal/dashboard': {
        workspace: PORTAL_WORKSPACE,
        usage_summary: PORTAL_USAGE_SUMMARY,
        billing_summary: PORTAL_BILLING_SUMMARY,
        recent_requests: [],
        api_key_count: 0,
      },
      '/api/portal/usage/records': [],
      '/api/portal/usage/summary': PORTAL_USAGE_SUMMARY,
      '/api/portal/billing/summary': PORTAL_BILLING_SUMMARY,
      '/api/portal/billing/account': createPortalCommercialAccount(),
      '/api/portal/billing/account-history': createPortalCommercialHistorySnapshot(),
      '/api/portal/billing/account/balance': createPortalCommercialBalance(),
      '/api/portal/billing/account/benefit-lots': [],
      '/api/portal/billing/account/holds': [],
      '/api/portal/billing/account/request-settlements': createPortalCommercialSettlements(),
      '/api/portal/billing/pricing-plans': [],
      '/api/portal/billing/pricing-rates': [],
      '/api/portal/billing/events': [],
      '/api/portal/billing/events/summary': PORTAL_BILLING_EVENT_SUMMARY,
      '/api/portal/billing/ledger': [],
      '/api/portal/commerce/membership': null,
      '/api/portal/commerce/catalog': {
        products: [],
        offers: [],
        plans: [],
        packs: [],
        recharge_options: [],
        custom_recharge_policy: null,
        coupons: [],
      },
      '/api/portal/commerce/order-center': {
        project_id: 'project-portal-1',
        payment_simulation_enabled: false,
        membership: null,
        reconciliation: null,
        orders: [],
      },
    },
  });
}

function createPortalRouteChecks(previewUrl) {
  const commercialSetupScript = createPortalCommercialRuntimeSetupSource();

  return [
    {
      id: 'home',
      url: previewUrl,
      expectedTexts: [],
      expectedSelectors: PORTAL_EXPECTED_SELECTORS,
      forbiddenTexts: [],
      expectedRequestIncludes: [],
      setupScript: '',
    },
    {
      id: 'account-unsafe-id',
      url: `${previewUrl}console/account`,
      expectedTexts: [PORTAL_UNSAFE_ACCOUNT_ID],
      expectedSelectors: ['[data-slot="portal-account-page"]'],
      forbiddenTexts: [PORTAL_FORBIDDEN_ACCOUNT_ID],
      expectedRequestIncludes: [],
      setupScript: commercialSetupScript,
    },
    {
      id: 'billing-unsafe-id',
      url: `${previewUrl}console/billing`,
      expectedTexts: [PORTAL_UNSAFE_ACCOUNT_ID],
      expectedSelectors: ['[data-slot="portal-billing-toolbar"]'],
      forbiddenTexts: [PORTAL_FORBIDDEN_ACCOUNT_ID],
      expectedRequestIncludes: [],
      setupScript: commercialSetupScript,
    },
    {
      id: 'settlements-unsafe-id',
      url: `${previewUrl}console/settlements`,
      expectedTexts: [PORTAL_UNSAFE_ACCOUNT_ID],
      expectedSelectors: ['[data-slot="portal-settlements-page"]'],
      forbiddenTexts: [PORTAL_FORBIDDEN_ACCOUNT_ID],
      expectedRequestIncludes: [],
      setupScript: commercialSetupScript,
    },
  ];
}

function truncateText(value, maxLength = 400) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function runForegroundStep(step, {
  env = process.env,
  platform = process.platform,
} = {}) {
  const result = spawnSync(step.command, step.args, {
    ...pnpmSpawnOptions({
      platform,
      env,
      cwd: step.cwd,
      stdio: 'pipe',
    }),
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });

  if ((result.status ?? 1) !== 0) {
    throw new Error(
      `${step.label} failed with exit code ${result.status ?? 'unknown'}: ${truncateText(`${result.stdout ?? ''}\n${result.stderr ?? ''}`, 1200)}`,
    );
  }
}

function killProcessTree(child, platform = process.platform) {
  if (!child?.pid) {
    return;
  }

  if (platform === 'win32') {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    });
    return;
  }

  child.kill('SIGTERM');
}

async function waitForHttpOk(url, timeoutMs = DEFAULT_TIMEOUT_MS) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, {
        signal: AbortSignal.timeout(3000),
      });
      if (!response.ok) {
        throw new Error(`${url} returned HTTP ${response.status}`);
      }

      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await delay(250);
    }
  }

  throw new Error(
    `${url} did not become reachable within ${timeoutMs}ms: ${lastError?.message ?? 'unknown error'}`,
  );
}

async function findAvailablePort() {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve(port);
      });
    });
  });
}

export function createPortalBrowserRuntimeSmokePlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  portalAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  platform = process.platform,
  env = process.env,
  previewPort = 4174,
} = {}) {
  const previewUrl = `http://127.0.0.1:${previewPort}/portal/`;

  return {
    portalAppDir,
    previewUrl,
    expectedTexts: PORTAL_EXPECTED_TEXTS,
    expectedSelectors: PORTAL_EXPECTED_SELECTORS,
    routeChecks: createPortalRouteChecks(previewUrl),
    buildStep: {
      label: 'portal production build',
      cwd: portalAppDir,
      command: process.execPath,
      args: [
        path.join(workspaceRoot, 'scripts', 'dev', 'run-vite-cli.mjs'),
        'build',
      ],
      env,
    },
    previewStep: {
      command: process.execPath,
      args: [
        path.join(workspaceRoot, 'scripts', 'dev', 'run-vite-cli.mjs'),
        'preview',
        '--host',
        '127.0.0.1',
        '--port',
        String(previewPort),
        '--strictPort',
      ],
      label: 'portal preview server',
      cwd: portalAppDir,
      env,
    },
  };
}

export async function runPortalBrowserRuntimeSmoke({
  workspaceRoot = path.resolve(__dirname, '..'),
  portalAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  platform = process.platform,
  env = process.env,
} = {}) {
  ensureFrontendDependenciesReady({
    appRoot: portalAppDir,
    requiredPackages: ['vite', 'typescript'],
    requiredBinCommands: ['vite', 'tsc'],
    verifyInstalled: () => checkFrontendViteConfig({
      appRoot: portalAppDir,
      command: 'build',
    }),
    platform,
    env,
  });

  const previewPort = await findAvailablePort();
  const plan = createPortalBrowserRuntimeSmokePlan({
    workspaceRoot,
    portalAppDir,
    platform,
    env,
    previewPort,
  });

  runForegroundStep(plan.buildStep, { env, platform });

  const previewProcess = spawn(plan.previewStep.command, plan.previewStep.args, {
    ...pnpmSpawnOptions({
      platform,
      env,
      cwd: plan.previewStep.cwd,
      stdio: 'pipe',
    }),
  });
  let previewStdout = '';
  let previewStderr = '';

  previewProcess.stdout?.on('data', (chunk) => {
    previewStdout += String(chunk);
  });
  previewProcess.stderr?.on('data', (chunk) => {
    previewStderr += String(chunk);
  });

  try {
    await waitForHttpOk(plan.previewUrl, DEFAULT_TIMEOUT_MS);
    const checks = [];

    for (const routeCheck of plan.routeChecks) {
      // eslint-disable-next-line no-await-in-loop
      const result = await runBrowserRuntimeSmoke({
        url: routeCheck.url,
        expectedTexts: routeCheck.expectedTexts,
        expectedSelectors: routeCheck.expectedSelectors,
        forbiddenTexts: routeCheck.forbiddenTexts,
        expectedRequestIncludes: routeCheck.expectedRequestIncludes,
        setupScript: routeCheck.setupScript,
        timeoutMs: DEFAULT_TIMEOUT_MS,
        platform,
        env,
      });

      checks.push({
        id: routeCheck.id,
        ...result,
      });
    }

    return {
      previewUrl: plan.previewUrl,
      checks,
    };
  } finally {
    killProcessTree(previewProcess, platform);
    await delay(250).catch(() => {});
  }
}

async function main() {
  const result = await runPortalBrowserRuntimeSmoke();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
