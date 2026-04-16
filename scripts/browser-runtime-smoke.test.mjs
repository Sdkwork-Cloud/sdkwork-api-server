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
