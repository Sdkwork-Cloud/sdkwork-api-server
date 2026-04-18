import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

test('browser runtime smoke exposes a parseable CLI contract and Chromium launch plan', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createBrowserRuntimeSmokePlan, 'function');
  assert.equal(typeof module.resolveChromiumBrowserExecutable, 'function');

  const options = module.parseArgs([
    '--url',
    'http://127.0.0.1:4174/portal/',
    '--expected-text',
    'Unified AI gateway workspace',
    '--expected-text',
    'Operate routing, credentials, usage, and downloads from one product surface.',
    '--timeout-ms',
    '45000',
  ]);

  assert.deepEqual(options, {
    url: 'http://127.0.0.1:4174/portal/',
    expectedTexts: [
      'Unified AI gateway workspace',
      'Operate routing, credentials, usage, and downloads from one product surface.',
    ],
    expectedSelectors: [],
    timeoutMs: 45000,
    browserPath: '',
  });

  const plan = module.createBrowserRuntimeSmokePlan({
    ...options,
    browserPath: 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    platform: 'win32',
  });

  assert.equal(plan.url, options.url);
  assert.deepEqual(plan.expectedTexts, options.expectedTexts);
  assert.deepEqual(plan.expectedSelectors, []);
  assert.equal(plan.timeoutMs, 45000);
  assert.equal(plan.browserCommand, 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe');
  assert.ok(plan.remoteDebuggingPort > 0);
  assert.match(plan.userDataDir, /sdkwork-browser-smoke-/i);
  assert.ok(
    plan.browserArgs.includes('--headless=new'),
    'browser smoke must run in headless mode',
  );
  assert.ok(
    plan.browserArgs.includes(`--remote-debugging-port=${plan.remoteDebuggingPort}`),
    'browser smoke must expose the debugging port for CDP inspection',
  );
});

test('browser runtime smoke rejects missing required inputs', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.throws(() => module.parseArgs([]), /--url is required/i);
  assert.throws(
    () =>
      module.parseArgs([
        '--url',
        'http://127.0.0.1:4174/portal/',
      ]),
    /--expected-text or --expected-selector is required/i,
  );
});

test('browser runtime smoke can resolve a Chromium executable from a provided candidate list', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  const browserPath = module.resolveChromiumBrowserExecutable({
    platform: 'win32',
    candidatePaths: [
      'C:/missing/msedge.exe',
      'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    ],
    exists: (candidate) => candidate.includes('/Program Files (x86)/Microsoft/Edge/Application/'),
  });

  assert.equal(
    browserPath,
    'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  );
});

test('browser runtime smoke preserves unsafe integers when polling JSON endpoints', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  const payload = await module.readJsonResponse(
    new Response(
      '{"webSocketDebuggerUrl":"ws://127.0.0.1/devtools/page/1","unsafe_marker":9007199254740993}',
      {
        status: 200,
        headers: {
          'content-type': 'application/json',
        },
      },
    ),
  );

  assert.equal(payload.unsafe_marker, '9007199254740993');
});

test('browser runtime smoke plan preserves setup scripts, forbidden texts, and expected request fragments', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.createMockFetchSetupScript, 'function');

  const setupScript = module.createMockFetchSetupScript({
    localStorageEntries: {
      'sdkwork.router.portal.session-token': 'portal-token',
    },
    exactResponses: {
      '/api/portal/workspace': {
        tenant: { id: 'tenant-1', name: 'Tenant 1' },
      },
    },
    patternResponses: [{
      pattern: '^/api/admin/billing/accounts/646979632893840957/ledger$',
      body: [],
    }],
  });

  const plan = module.createBrowserRuntimeSmokePlan({
    url: 'http://127.0.0.1:4174/portal/console/account',
    expectedTexts: ['1950809575122113173'],
    expectedSelectors: ['[data-slot="portal-account-page"]'],
    forbiddenTexts: ['1950809575122113300'],
    expectedRequestIncludes: ['/api/admin/billing/accounts/646979632893840957/ledger'],
    setupScript,
    browserPath: 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    platform: 'win32',
  });

  assert.deepEqual(plan.forbiddenTexts, ['1950809575122113300']);
  assert.deepEqual(plan.expectedRequestIncludes, [
    '/api/admin/billing/accounts/646979632893840957/ledger',
  ]);
  assert.equal(plan.setupScript, setupScript);
  assert.match(setupScript, /sdkwork\.router\.portal\.session-token/);
  assert.match(setupScript, /\/api\/portal\/workspace/);
  assert.match(setupScript, /646979632893840957/);
});

test('browser runtime smoke hardens Linux CI launch plans for hosted Chromium startup', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  const plan = module.createBrowserRuntimeSmokePlan({
    url: 'http://127.0.0.1:3001/admin/',
    expectedSelectors: ['input[type="email"]'],
    browserPath: '/usr/bin/google-chrome',
    platform: 'linux',
    env: {
      GITHUB_ACTIONS: 'true',
    },
  });

  assert.equal(plan.devtoolsTimeoutMs, 30000);
  assert.ok(plan.browserArgs.includes('--no-sandbox'));
  assert.ok(plan.browserArgs.includes('--disable-dev-shm-usage'));
});
