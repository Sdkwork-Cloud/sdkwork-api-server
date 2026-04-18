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
const ADMIN_EXPECTED_SELECTORS = [
  'input[type="email"]',
  'input[type="password"]',
  'button[type="submit"]',
];
const ADMIN_SESSION_TOKEN_KEY = 'sdkwork.router.admin.session-token';
const ADMIN_UNSAFE_ACCOUNT_IDS = [
  '646979632893840957',
  '1950809575122113173',
];
const ADMIN_FORBIDDEN_ACCOUNT_IDS = [
  '646979632893840900',
  '1950809575122113300',
];
const ADMIN_EMPTY_USAGE_SUMMARY = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};
const ADMIN_EMPTY_BILLING_SUMMARY = {
  total_entries: 0,
  project_count: 0,
  total_units: 0,
  total_amount: 0,
  active_quota_policy_count: 0,
  exhausted_project_count: 0,
  projects: [],
};
const ADMIN_EMPTY_BILLING_EVENT_SUMMARY = {
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

function createAdminCommercialAccount(accountId, index) {
  const baseTimestamp = 1_710_000_000_000 + index * 1_000;

  return {
    account: {
      account_id: accountId,
      tenant_id: `tenant-${index + 1}`,
      organization_id: `org-${index + 1}`,
      user_id: `user-${index + 1}`,
      account_type: 'primary',
      currency_code: 'USD',
      credit_unit_code: 'credit',
      status: 'active',
      allow_overdraft: false,
      overdraft_limit: 0,
      created_at_ms: baseTimestamp,
      updated_at_ms: baseTimestamp + 500,
    },
    available_balance: 2_500 - index * 200,
    held_balance: 120 + index * 10,
    consumed_balance: 320 + index * 20,
    grant_balance: 0,
    active_lot_count: 1,
  };
}

function createAdminCommercialSettlement({
  accountId,
  requestSettlementId,
  requestId,
  holdId,
  settledAtMs,
}) {
  return {
    request_settlement_id: requestSettlementId,
    tenant_id: 'tenant-1',
    organization_id: 'org-1',
    request_id: requestId,
    account_id: accountId,
    user_id: 'user-1',
    hold_id: holdId,
    status: 'captured',
    estimated_credit_hold: 120,
    released_credit_amount: 0,
    captured_credit_amount: 120,
    provider_cost_amount: 88,
    retail_charge_amount: 120,
    shortfall_amount: 0,
    refunded_amount: 0,
    settled_at_ms: settledAtMs,
    created_at_ms: settledAtMs - 5_000,
    updated_at_ms: settledAtMs,
  };
}

function createAdminCommercialLedger(accountId, index) {
  const baseTimestamp = 1_710_000_100_000 + index * 10_000;
  const requestId = `91000000000000000${index + 1}`;
  const holdId = `71000000000000000${index + 1}`;
  const ledgerEntryId = `81000000000000000${index + 1}`;

  return [{
    entry: {
      ledger_entry_id: ledgerEntryId,
      tenant_id: 'tenant-1',
      organization_id: 'org-1',
      account_id: accountId,
      user_id: 'user-1',
      request_id: requestId,
      hold_id: holdId,
      entry_type: 'settlement_capture',
      benefit_type: 'cash_credit',
      quantity: -120,
      amount: -120,
      created_at_ms: baseTimestamp,
    },
    allocations: [{
      ledger_allocation_id: `${ledgerEntryId}1`,
      tenant_id: 'tenant-1',
      organization_id: 'org-1',
      ledger_entry_id: ledgerEntryId,
      lot_id: `93000000000000000${index + 1}`,
      quantity_delta: -120,
      created_at_ms: baseTimestamp,
    }],
  }];
}

function createAdminCommercialRuntimeSetupSource() {
  const commercialAccounts = ADMIN_UNSAFE_ACCOUNT_IDS.map(createAdminCommercialAccount);
  const settlements = ADMIN_UNSAFE_ACCOUNT_IDS.map((accountId, index) =>
    createAdminCommercialSettlement({
      accountId,
      requestSettlementId: `70000000000000000${index + 1}`,
      requestId: `91000000000000000${index + 1}`,
      holdId: `71000000000000000${index + 1}`,
      settledAtMs: 1_710_000_200_000 + index * 5_000,
    }));

  return createMockFetchSetupScript({
    localStorageEntries: {
      [ADMIN_SESSION_TOKEN_KEY]: 'mock-admin-session-token',
    },
    exactResponses: {
      '/api/admin/auth/me': {
        id: 'admin-user-1',
        email: 'ops@example.com',
        display_name: 'Operations Admin',
        active: true,
        created_at_ms: 1_710_000_000_000,
      },
      '/api/admin/users/operators': [],
      '/api/admin/users/portal': [],
      '/api/admin/tenants': [],
      '/api/admin/projects': [],
      '/api/admin/api-keys': [],
      '/api/admin/api-key-groups': [],
      '/api/admin/routing/profiles': [],
      '/api/admin/routing/snapshots': [],
      '/api/admin/gateway/rate-limit-policies': [],
      '/api/admin/gateway/rate-limit-windows': [],
      '/api/admin/channels': [],
      '/api/admin/providers': [],
      '/api/admin/credentials': [],
      '/api/admin/models': [],
      '/api/admin/channel-models': [],
      '/api/admin/provider-models': [],
      '/api/admin/model-prices': [],
      '/api/admin/usage/records': [],
      '/api/admin/usage/summary': ADMIN_EMPTY_USAGE_SUMMARY,
      '/api/admin/billing/events': [],
      '/api/admin/billing/events/summary': ADMIN_EMPTY_BILLING_EVENT_SUMMARY,
      '/api/admin/billing/summary': ADMIN_EMPTY_BILLING_SUMMARY,
      '/api/admin/commerce/orders': [],
      '/api/admin/commerce/payment-methods': [],
      '/api/admin/commerce/webhook-inbox': [],
      '/api/admin/commerce/reconciliation-runs': [],
      '/api/admin/marketing/coupon-templates': [],
      '/api/admin/marketing/campaigns': [],
      '/api/admin/marketing/budgets': [],
      '/api/admin/marketing/codes': [],
      '/api/admin/marketing/reservations': [],
      '/api/admin/marketing/redemptions': [],
      '/api/admin/marketing/rollbacks': [],
      '/api/admin/billing/accounts': commercialAccounts,
      '/api/admin/billing/account-holds': [],
      '/api/admin/billing/request-settlements': settlements,
      '/api/admin/billing/pricing-plans': [],
      '/api/admin/billing/pricing-rates': [],
      '/api/admin/routing/decision-logs': [],
      '/api/admin/routing/health-snapshots': [],
      '/api/admin/extensions/runtime-statuses': [],
    },
    patternResponses: ADMIN_UNSAFE_ACCOUNT_IDS.map((accountId, index) => ({
      pattern: `^/api/admin/billing/accounts/${accountId}/ledger$`,
      body: createAdminCommercialLedger(accountId, index),
    })),
  });
}

function createAdminRouteChecks(previewUrl) {
  return [
    {
      id: 'login',
      url: previewUrl,
      expectedTexts: [],
      expectedSelectors: ADMIN_EXPECTED_SELECTORS,
      forbiddenTexts: [],
      expectedRequestIncludes: [],
      setupScript: '',
    },
    {
      id: 'commercial-unsafe-id',
      url: `${previewUrl}commercial`,
      expectedTexts: ['Commercial control plane'],
      expectedSelectors: [],
      forbiddenTexts: ADMIN_FORBIDDEN_ACCOUNT_IDS,
      expectedRequestIncludes: ADMIN_UNSAFE_ACCOUNT_IDS.map(
        (accountId) => `/api/admin/billing/accounts/${accountId}/ledger`,
      ),
      setupScript: createAdminCommercialRuntimeSetupSource(),
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

export function createAdminBrowserRuntimeSmokePlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  adminAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
  platform = process.platform,
  env = process.env,
  previewPort = 4173,
} = {}) {
  const previewUrl = `http://127.0.0.1:${previewPort}/admin/`;

  return {
    adminAppDir,
    previewUrl,
    expectedTexts: [],
    expectedSelectors: ADMIN_EXPECTED_SELECTORS,
    routeChecks: createAdminRouteChecks(previewUrl),
    buildStep: {
      label: 'admin production build',
      cwd: adminAppDir,
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
      label: 'admin preview server',
      cwd: adminAppDir,
      env,
    },
  };
}

export async function runAdminBrowserRuntimeSmoke({
  workspaceRoot = path.resolve(__dirname, '..'),
  adminAppDir = path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
  platform = process.platform,
  env = process.env,
} = {}) {
  ensureFrontendDependenciesReady({
    appRoot: adminAppDir,
    requiredPackages: ['vite', 'typescript'],
    requiredBinCommands: ['vite', 'tsc'],
    verifyInstalled: () => checkFrontendViteConfig({
      appRoot: adminAppDir,
      command: 'build',
    }),
    platform,
    env,
  });

  const previewPort = await findAvailablePort();
  const plan = createAdminBrowserRuntimeSmokePlan({
    workspaceRoot,
    adminAppDir,
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
  const result = await runAdminBrowserRuntimeSmoke();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
