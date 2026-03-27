import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadAdminApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(appRoot, 'packages', 'sdkwork-router-admin-admin-api', 'src', 'index.ts'),
  );
}

function jsonResponse(body, init) {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: {
      'content-type': 'application/json',
    },
    ...init,
  });
}

test('browser admin api keeps the relative proxy prefix', async () => {
  const requests = [];
  const adminApi = loadAdminApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return jsonResponse({
      token: 'admin-session',
      user: {
        id: 'admin-user',
      },
    });
  };
  globalThis.window = undefined;

  try {
    await adminApi.loginAdminUser({
      email: 'admin@example.com',
      password: 'secret',
    });
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, ['/api/admin/auth/login']);
});

test('desktop admin api prefixes requests with the tauri runtime base url', async () => {
  const requests = [];
  const adminApi = loadAdminApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return jsonResponse({
      token: 'admin-session',
      user: {
        id: 'admin-user',
      },
    });
  };
  globalThis.window = {
    isTauri: true,
    __TAURI_INTERNALS__: {
      invoke: async (command) => {
        assert.equal(command, 'runtime_base_url');
        return 'http://127.0.0.1:48123';
      },
    },
  };

  try {
    await adminApi.loginAdminUser({
      email: 'admin@example.com',
      password: 'secret',
    });
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, ['http://127.0.0.1:48123/api/admin/auth/login']);
});
