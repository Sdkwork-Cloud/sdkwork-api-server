#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEFAULT_TIMEOUT_MS = 30_000;
const DEFAULT_DEVTOOLS_TIMEOUT_MS = 10_000;
const DEFAULT_POLL_INTERVAL_MS = 200;
const MAX_SAFE_INTEGER_TEXT = String(Number.MAX_SAFE_INTEGER);

function isHostedLinuxCiRuntime(platform = process.platform, env = process.env) {
  return platform === 'linux' && String(env.GITHUB_ACTIONS ?? '').toLowerCase() === 'true';
}

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new Error(`${token} requires a value`);
  }

  return next;
}

function truncateText(value, maxLength = 400) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function uniqueStrings(values = []) {
  return values.filter((value, index, collection) => value && collection.indexOf(value) === index);
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    url: '',
    expectedTexts: [],
    expectedSelectors: [],
    timeoutMs: DEFAULT_TIMEOUT_MS,
    browserPath: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--url') {
      options.url = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--expected-text') {
      options.expectedTexts.push(readOptionValue(token, next));
      index += 1;
      continue;
    }
    if (token === '--expected-selector') {
      options.expectedSelectors.push(readOptionValue(token, next));
      index += 1;
      continue;
    }
    if (token === '--timeout-ms') {
      options.timeoutMs = Number.parseInt(readOptionValue(token, next), 10);
      index += 1;
      continue;
    }
    if (token === '--browser-path') {
      options.browserPath = readOptionValue(token, next);
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!options.url) {
    throw new Error('--url is required');
  }
  if (!Number.isInteger(options.timeoutMs) || options.timeoutMs <= 0) {
    throw new Error('--timeout-ms must be a positive integer');
  }
  options.expectedTexts = uniqueStrings(options.expectedTexts.map((value) => String(value).trim()));
  options.expectedSelectors = uniqueStrings(
    options.expectedSelectors.map((value) => String(value).trim()),
  );
  if (options.expectedTexts.length === 0 && options.expectedSelectors.length === 0) {
    throw new Error('--expected-text or --expected-selector is required at least once');
  }

  return options;
}

function defaultChromiumCandidatePaths(platform = process.platform, env = process.env) {
  const envCandidates = [
    env.SDKWORK_BROWSER_PATH,
    env.MSEDGE_BIN,
    env.CHROME_BIN,
    env.CHROMIUM_BIN,
  ].filter(Boolean);

  if (platform === 'win32') {
    return [
      ...envCandidates,
      'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
      'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
      'C:/Program Files/Google/Chrome/Application/chrome.exe',
      'C:/Program Files (x86)/Google/Chrome/Application/chrome.exe',
    ];
  }

  if (platform === 'darwin') {
    return [
      ...envCandidates,
      '/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge',
      '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
      '/Applications/Chromium.app/Contents/MacOS/Chromium',
    ];
  }

  return [
    ...envCandidates,
    '/usr/bin/microsoft-edge',
    '/usr/bin/microsoft-edge-stable',
    '/usr/bin/google-chrome',
    '/usr/bin/google-chrome-stable',
    '/usr/bin/chromium-browser',
    '/usr/bin/chromium',
  ];
}

export function resolveChromiumBrowserExecutable({
  platform = process.platform,
  env = process.env,
  browserPath = '',
  candidatePaths = [],
  exists = existsSync,
} = {}) {
  if (browserPath) {
    return browserPath;
  }

  const candidates = uniqueStrings([
    ...candidatePaths,
    ...defaultChromiumCandidatePaths(platform, env),
  ]);

  const resolved = candidates.find((candidate) => exists(candidate));
  if (resolved) {
    return resolved;
  }

  throw new Error(
    `unable to resolve a Chromium-based browser executable for browser runtime smoke on ${platform}`,
  );
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

function createEvalExpression(expectedTexts, expectedSelectors) {
  const serializedExpectedTexts = JSON.stringify(expectedTexts);
  const serializedExpectedSelectors = JSON.stringify(expectedSelectors);
  return `(() => {
    const expectedTexts = ${serializedExpectedTexts};
    const expectedSelectors = ${serializedExpectedSelectors};
    const title = document.title ?? '';
    const bodyText = document.body?.innerText ?? '';
    const matchedTexts = expectedTexts.filter((entry) => title.includes(entry) || bodyText.includes(entry));
    const matchedSelectors = expectedSelectors.filter((selector) => document.querySelector(selector));
    return {
      title,
      bodyText,
      matchedTexts,
      matchedSelectors,
      expectedTexts,
      expectedSelectors,
      readyState: document.readyState,
    };
  })()`;
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

export function createBrowserRuntimeSmokePlan({
  url,
  expectedTexts = [],
  expectedSelectors = [],
  forbiddenTexts = [],
  expectedRequestIncludes = [],
  timeoutMs = DEFAULT_TIMEOUT_MS,
  browserPath = '',
  setupScript = '',
  platform = process.platform,
  env = process.env,
  remoteDebuggingPort = 9222,
  userDataDir = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-browser-smoke-')),
} = {}) {
  if (!url) {
    throw new Error('url is required');
  }
  if (
    (!Array.isArray(expectedTexts) || expectedTexts.length === 0)
    && (!Array.isArray(expectedSelectors) || expectedSelectors.length === 0)
  ) {
    throw new Error('expectedTexts or expectedSelectors is required');
  }

  const browserCommand = resolveChromiumBrowserExecutable({
    platform,
    env,
    browserPath,
  });
  const browserArgs = [
    '--headless=new',
    '--disable-gpu',
  ];
  if (isHostedLinuxCiRuntime(platform, env)) {
    browserArgs.push('--no-sandbox', '--disable-dev-shm-usage');
  }
  browserArgs.push(
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-background-networking',
    '--disable-background-timer-throttling',
    '--disable-renderer-backgrounding',
    '--disable-sync',
    '--mute-audio',
    `--remote-debugging-port=${remoteDebuggingPort}`,
    `--user-data-dir=${userDataDir}`,
    'about:blank',
  );

  return {
    url,
    expectedTexts: uniqueStrings(expectedTexts),
    expectedSelectors: uniqueStrings(expectedSelectors),
    forbiddenTexts: uniqueStrings(forbiddenTexts),
    expectedRequestIncludes: uniqueStrings(expectedRequestIncludes),
    timeoutMs,
    devtoolsTimeoutMs: isHostedLinuxCiRuntime(platform, env)
      ? 30_000
      : DEFAULT_DEVTOOLS_TIMEOUT_MS,
    browserCommand,
    setupScript: String(setupScript || ''),
    remoteDebuggingPort,
    userDataDir,
    browserArgs,
  };
}

async function waitForJson(url, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, {
        signal: AbortSignal.timeout(2000),
      });
      if (!response.ok) {
        throw new Error(`${url} returned HTTP ${response.status}`);
      }

      return await readJsonResponse(response);
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await delay(DEFAULT_POLL_INTERVAL_MS);
    }
  }

  throw new Error(
    `${url} did not become reachable within ${timeoutMs}ms: ${lastError?.message ?? 'unknown error'}`,
  );
}

export async function readJsonResponse(response) {
  if (typeof response.text === 'function') {
    const body = await response.text();
    return body ? parseJsonBody(body) : null;
  }

  if (typeof response.json === 'function') {
    return response.json();
  }

  return null;
}

function parseJsonBody(body) {
  return JSON.parse(quoteUnsafeIntegerTokens(body));
}

function quoteUnsafeIntegerTokens(body) {
  let result = '';
  let index = 0;
  let inString = false;
  let escaped = false;

  while (index < body.length) {
    const character = body[index];

    if (inString) {
      result += character;
      if (escaped) {
        escaped = false;
      } else if (character === '\\') {
        escaped = true;
      } else if (character === '"') {
        inString = false;
      }
      index += 1;
      continue;
    }

    if (character === '"') {
      inString = true;
      result += character;
      index += 1;
      continue;
    }

    if (character === '-' || isDigit(character)) {
      const tokenEnd = findNumericTokenEnd(body, index);
      const token = body.slice(index, tokenEnd);

      if (shouldQuoteIntegerToken(token)) {
        result += `"${token}"`;
      } else {
        result += token;
      }

      index = tokenEnd;
      continue;
    }

    result += character;
    index += 1;
  }

  return result;
}

function findNumericTokenEnd(body, start) {
  let index = start;

  if (body[index] === '-') {
    index += 1;
  }

  while (index < body.length && isDigit(body[index])) {
    index += 1;
  }

  if (body[index] === '.') {
    index += 1;
    while (index < body.length && isDigit(body[index])) {
      index += 1;
    }
  }

  if (body[index] === 'e' || body[index] === 'E') {
    index += 1;
    if (body[index] === '+' || body[index] === '-') {
      index += 1;
    }
    while (index < body.length && isDigit(body[index])) {
      index += 1;
    }
  }

  return index;
}

function shouldQuoteIntegerToken(token) {
  if (!/^-?\d+$/.test(token)) {
    return false;
  }

  const normalized = token.startsWith('-') ? token.slice(1) : token;
  if (normalized.length < MAX_SAFE_INTEGER_TEXT.length) {
    return false;
  }
  if (normalized.length > MAX_SAFE_INTEGER_TEXT.length) {
    return true;
  }
  return normalized > MAX_SAFE_INTEGER_TEXT;
}

function isDigit(character) {
  return character != null && character >= '0' && character <= '9';
}

async function connectWebSocket(url) {
  const socket = new WebSocket(url);
  await new Promise((resolve, reject) => {
    const onOpen = () => {
      socket.removeEventListener('error', onError);
      resolve();
    };
    const onError = (event) => {
      socket.removeEventListener('open', onOpen);
      reject(event.error ?? new Error(`failed to connect to ${url}`));
    };

    socket.addEventListener('open', onOpen, { once: true });
    socket.addEventListener('error', onError, { once: true });
  });

  return socket;
}

function createCdpClient(socket) {
  let nextId = 1;
  const pending = new Map();
  const eventHandlers = new Map();

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(String(event.data));
    if (message.id) {
      const pendingRequest = pending.get(message.id);
      if (!pendingRequest) {
        return;
      }

      pending.delete(message.id);
      if (message.error) {
        pendingRequest.reject(new Error(message.error.message));
        return;
      }

      pendingRequest.resolve(message.result ?? {});
      return;
    }

    const handlers = eventHandlers.get(message.method) ?? [];
    for (const handler of handlers) {
      handler(message.params ?? {});
    }
  });

  return {
    send(method, params = {}) {
      const id = nextId;
      nextId += 1;

      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
    on(method, handler) {
      const handlers = eventHandlers.get(method) ?? [];
      handlers.push(handler);
      eventHandlers.set(method, handlers);
    },
    async close() {
      for (const [, pendingRequest] of pending) {
        pendingRequest.reject(new Error('browser runtime smoke connection closed'));
      }
      pending.clear();
      socket.close();
      await new Promise((resolve) => {
        socket.addEventListener('close', resolve, { once: true });
      });
    },
  };
}

async function waitForPageWebSocketDebuggerUrl(port, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      const targets = await waitForJson(`http://127.0.0.1:${port}/json/list`, 2000);
      const pageTarget = Array.isArray(targets)
        ? targets.find((target) => target.type === 'page' && target.webSocketDebuggerUrl)
        : null;

      if (!pageTarget?.webSocketDebuggerUrl) {
        throw new Error('page debugger target is not ready yet');
      }

      return pageTarget.webSocketDebuggerUrl;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await delay(DEFAULT_POLL_INTERVAL_MS);
    }
  }

  throw new Error(
    `page debugger target did not become ready within ${timeoutMs}ms: ${lastError?.message ?? 'unknown error'}`,
  );
}

function matchedRequestIncludes(requestLog, expectedRequestIncludes) {
  return expectedRequestIncludes.filter((entry) =>
    requestLog.some((requestUrl) => String(requestUrl).includes(entry)));
}

async function readBrowserFetchRequestLog(client) {
  const evaluation = await client.send('Runtime.evaluate', {
    expression: 'globalThis.__SDKWORK_BROWSER_RUNTIME_FETCH_REQUESTS__ ?? []',
    returnByValue: true,
  });

  return Array.isArray(evaluation.result?.value) ? evaluation.result.value : [];
}

async function waitForExpectedRequestIncludes(
  client,
  requestLog,
  expectedRequestIncludes,
  timeoutMs,
) {
  const deadline = Date.now() + timeoutMs;
  let lastMatched = [];
  let lastObserved = requestLog;

  while (Date.now() < deadline) {
    const browserFetchLog = await readBrowserFetchRequestLog(client).catch(() => []);
    lastObserved = uniqueStrings([...requestLog, ...browserFetchLog]);
    lastMatched = matchedRequestIncludes(lastObserved, expectedRequestIncludes);
    if (lastMatched.length === expectedRequestIncludes.length) {
      return {
        matchedRequestUrls: lastMatched,
        observedRequests: lastObserved,
      };
    }

    await delay(DEFAULT_POLL_INTERVAL_MS);
  }

  throw new Error(
    `browser runtime smoke did not observe the expected request urls before timeout; expected: ${truncateText(JSON.stringify(expectedRequestIncludes), 400)}; observed: ${truncateText(JSON.stringify(lastObserved), 600)}`,
  );
}

async function waitForExpectedTargets(client, expectedTexts, expectedSelectors, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let lastSnapshot = null;

  while (Date.now() < deadline) {
    const evaluation = await client.send('Runtime.evaluate', {
      expression: createEvalExpression(expectedTexts, expectedSelectors),
      returnByValue: true,
    });
    const snapshot = evaluation.result?.value ?? {};
    lastSnapshot = snapshot;

    const textReady = expectedTexts.length === 0
      || (Array.isArray(snapshot.matchedTexts) && snapshot.matchedTexts.length === expectedTexts.length);
    const selectorReady = expectedSelectors.length === 0
      || (
        Array.isArray(snapshot.matchedSelectors)
        && snapshot.matchedSelectors.length === expectedSelectors.length
      );

    if (textReady && selectorReady) {
      return snapshot;
    }

    await delay(DEFAULT_POLL_INTERVAL_MS);
  }

  throw new Error(
    `browser runtime smoke did not observe the expected runtime markers before timeout; last snapshot: ${truncateText(JSON.stringify(lastSnapshot), 600)}`,
  );
}

function matchedForbiddenTexts(snapshot, forbiddenTexts) {
  const title = String(snapshot?.title ?? '');
  const bodyText = String(snapshot?.bodyText ?? '');

  return forbiddenTexts.filter((entry) =>
    title.includes(entry) || bodyText.includes(entry));
}

export function createMockFetchSetupScript({
  localStorageEntries = {},
  exactResponses = {},
  patternResponses = [],
} = {}) {
  return `(() => {
    const localStorageEntries = ${JSON.stringify(localStorageEntries)};
    const exactResponses = ${JSON.stringify(exactResponses)};
    const patternResponses = ${JSON.stringify(patternResponses)};
    const originalFetch = globalThis.fetch ? globalThis.fetch.bind(globalThis) : null;
    const fetchRequests = [];

    globalThis.__SDKWORK_BROWSER_RUNTIME_FETCH_REQUESTS__ = fetchRequests;

    for (const [key, value] of Object.entries(localStorageEntries)) {
      try {
        globalThis.localStorage?.setItem(key, String(value));
      } catch {}
    }

    function jsonResponse(status, body) {
      return new Response(JSON.stringify(body), {
        status,
        headers: {
          'content-type': 'application/json',
        },
      });
    }

    function resolveMockResponse(pathname) {
      if (Object.prototype.hasOwnProperty.call(exactResponses, pathname)) {
        return {
          status: 200,
          body: exactResponses[pathname],
        };
      }

      for (const entry of patternResponses) {
        const pattern = new RegExp(entry.pattern);
        if (pattern.test(pathname)) {
          return {
            status: entry.status ?? 200,
            body: entry.body,
          };
        }
      }

      return null;
    }

    globalThis.fetch = async (input, init) => {
      const rawUrl = typeof input === 'string'
        ? input
        : (input && typeof input === 'object' && 'url' in input ? input.url : String(input));
      const requestUrl = new URL(rawUrl, globalThis.location?.origin ?? 'http://127.0.0.1');
      fetchRequests.push(requestUrl.pathname + requestUrl.search);
      const mock = resolveMockResponse(requestUrl.pathname);

      if (mock) {
        return jsonResponse(mock.status, mock.body);
      }

      if (!originalFetch) {
        return jsonResponse(404, {
          error: {
            message: 'Unhandled mocked fetch request: ' + requestUrl.pathname,
          },
        });
      }

      return originalFetch(input, init);
    };
  })();`;
}

export async function runBrowserRuntimeSmoke({
  url,
  expectedTexts = [],
  expectedSelectors = [],
  forbiddenTexts = [],
  expectedRequestIncludes = [],
  timeoutMs = DEFAULT_TIMEOUT_MS,
  browserPath = '',
  setupScript = '',
  platform = process.platform,
  env = process.env,
} = {}) {
  const remoteDebuggingPort = await findAvailablePort();
  const userDataDir = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-browser-smoke-'));
  const plan = createBrowserRuntimeSmokePlan({
    url,
    expectedTexts,
    expectedSelectors,
    forbiddenTexts,
    expectedRequestIncludes,
    timeoutMs,
    browserPath,
    setupScript,
    platform,
    env,
    remoteDebuggingPort,
    userDataDir,
  });

  const browserProcess = spawn(plan.browserCommand, plan.browserArgs, {
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
    shell: false,
    windowsHide: platform === 'win32',
  });
  let browserStdout = '';
  let browserStderr = '';

  browserProcess.stdout?.on('data', (chunk) => {
    browserStdout += String(chunk);
  });
  browserProcess.stderr?.on('data', (chunk) => {
    browserStderr += String(chunk);
  });

  let client = null;
  const requestLog = [];

  try {
    await waitForJson(
      `http://127.0.0.1:${plan.remoteDebuggingPort}/json/version`,
      plan.devtoolsTimeoutMs,
    );
    const pageDebuggerUrl = await waitForPageWebSocketDebuggerUrl(
      plan.remoteDebuggingPort,
      plan.devtoolsTimeoutMs,
    );
    const socket = await connectWebSocket(pageDebuggerUrl);
    client = createCdpClient(socket);

    const exceptions = [];
    client.on('Runtime.exceptionThrown', (params) => {
      exceptions.push(params.exceptionDetails?.text ?? 'unhandled browser exception');
    });
    client.on('Network.requestWillBeSent', (params) => {
      const requestUrl = params.request?.url?.trim();
      if (requestUrl) {
        requestLog.push(requestUrl);
      }
    });

    await client.send('Page.enable');
    await client.send('Runtime.enable');
    await client.send('Log.enable');
    await client.send('Network.enable');

    if (plan.setupScript) {
      await client.send('Page.addScriptToEvaluateOnNewDocument', {
        source: plan.setupScript,
      });
    }

    await client.send('Page.navigate', { url: plan.url });
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error(`page load did not complete within ${plan.timeoutMs}ms`));
      }, plan.timeoutMs);

      client.on('Page.loadEventFired', () => {
        clearTimeout(timeout);
        resolve();
      });
    });

    const snapshot = await waitForExpectedTargets(
      client,
      plan.expectedTexts,
      plan.expectedSelectors,
      plan.timeoutMs,
    );

    const requestCheck = plan.expectedRequestIncludes.length > 0
      ? await waitForExpectedRequestIncludes(
        client,
        requestLog,
        plan.expectedRequestIncludes,
        plan.timeoutMs,
      )
      : {
        matchedRequestUrls: [],
        observedRequests: requestLog,
      };
    await delay(500);

    const forbiddenMatches = matchedForbiddenTexts(snapshot, plan.forbiddenTexts);
    if (forbiddenMatches.length > 0) {
      throw new Error(
        `browser runtime smoke observed forbidden runtime text on ${plan.url}: ${truncateText(JSON.stringify(forbiddenMatches), 400)}`,
      );
    }

    if (exceptions.length > 0) {
      throw new Error(
        `browser runtime smoke observed JavaScript exceptions on ${plan.url}: ${truncateText(exceptions.join('\n'), 1200)}`,
      );
    }

    return {
      url: plan.url,
      expectedTexts: plan.expectedTexts,
      expectedSelectors: plan.expectedSelectors,
      title: snapshot.title ?? '',
      matchedTexts: snapshot.matchedTexts ?? [],
      matchedSelectors: snapshot.matchedSelectors ?? [],
      forbiddenTexts: plan.forbiddenTexts,
      expectedRequestIncludes: plan.expectedRequestIncludes,
      matchedRequestUrls: requestCheck.matchedRequestUrls,
      observedRequests: requestCheck.observedRequests,
    };
  } finally {
    if (client) {
      await client.close().catch(() => {});
    }

    killProcessTree(browserProcess, platform);
    await delay(250).catch(() => {});
    try {
      rmSync(plan.userDataDir, { recursive: true, force: true });
    } catch (error) {
      const message = String(error instanceof Error ? error.message : error);
      if (!/EBUSY|EPERM/i.test(message)) {
        throw error;
      }
    }
  }
}

async function main() {
  const options = parseArgs();
  const result = await runBrowserRuntimeSmoke(options);
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
