import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadPortalApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(appRoot, 'packages', 'sdkwork-router-portal-portal-api', 'src', 'index.ts'),
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

function textResponse(body, init) {
  return new Response(body, {
    status: 200,
    headers: {
      'content-type': 'text/plain',
    },
    ...init,
  });
}

test('browser portal api keeps the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return jsonResponse({
      token: 'portal-session',
      user: {
        id: 'portal-user',
      },
    });
  };
  globalThis.window = undefined;

  try {
    await portalApi.loginPortalUser({
      email: 'portal@example.com',
      password: 'secret',
    });
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, ['/api/portal/auth/login']);
});

test('desktop portal api prefixes requests with the tauri runtime base url', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return jsonResponse({
      token: 'portal-session',
      user: {
        id: 'portal-user',
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
    await portalApi.loginPortalUser({
      email: 'portal@example.com',
      password: 'secret',
    });
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, ['http://127.0.0.1:48123/api/portal/auth/login']);
});

test('desktop gateway base url adds the product api prefix to the tauri runtime base url', async () => {
  const portalApi = loadPortalApi();
  const originalWindow = globalThis.window;

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
    assert.equal(
      await portalApi.resolveGatewayBaseUrl(),
      'http://127.0.0.1:48123/api',
    );
  } finally {
    globalThis.window = originalWindow;
  }
});

test('desktop runtime snapshot reads the tauri runtime topology bridge', async () => {
  const portalApi = loadPortalApi();
  const originalWindow = globalThis.window;

  globalThis.window = {
    isTauri: true,
    __TAURI_INTERNALS__: {
      invoke: async (command) => {
        assert.equal(command, 'runtime_desktop_snapshot');
        return {
          mode: 'desktop',
          roles: ['web', 'gateway', 'admin', 'portal'],
          publicBaseUrl: 'http://127.0.0.1:48123',
          publicBindAddr: '127.0.0.1:48123',
          gatewayBindAddr: '127.0.0.1:8080',
          adminBindAddr: '127.0.0.1:8081',
          portalBindAddr: '127.0.0.1:8082',
        };
      },
    },
  };

  try {
    const snapshot = await portalApi.getDesktopRuntimeSnapshot();
    assert.equal(snapshot.mode, 'desktop');
    assert.deepEqual(snapshot.roles, ['web', 'gateway', 'admin', 'portal']);
    assert.equal(snapshot.publicBaseUrl, 'http://127.0.0.1:48123');
    assert.equal(snapshot.gatewayBindAddr, '127.0.0.1:8080');
  } finally {
    globalThis.window = originalWindow;
  }
});

test('browser portal api posts commerce quote previews through the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input, init) => {
    requests.push({
      url: String(input),
      method: init?.method ?? 'GET',
      body: init?.body ?? null,
    });
    return jsonResponse({
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      target_name: 'Boost 100k',
      list_price_label: '$40.00',
      payable_price_label: '$32.00',
      granted_units: 100000,
      bonus_units: 0,
      projected_remaining_units: 105000,
      applied_coupon: {
        code: 'SPRING20',
      },
    });
  };
  globalThis.window = undefined;

  try {
    await portalApi.previewPortalCommerceQuote({
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      coupon_code: 'SPRING20',
      current_remaining_units: 5000,
    }, 'portal-session');
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, [
    {
      url: '/api/portal/commerce/quote',
      method: 'POST',
      body: JSON.stringify({
        target_kind: 'recharge_pack',
        target_id: 'pack-100k',
        coupon_code: 'SPRING20',
        current_remaining_units: 5000,
      }),
    },
  ]);
});

test('browser portal api creates commerce orders through the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input, init) => {
    requests.push({
      url: String(input),
      method: init?.method ?? 'GET',
      body: init?.body ?? null,
    });
    return jsonResponse({
      order_id: 'commerce-order-1',
      project_id: 'project-demo',
      user_id: 'portal-user',
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      target_name: 'Boost 100k',
      list_price_label: '$40.00',
      payable_price_label: '$32.00',
      granted_units: 100000,
      bonus_units: 0,
      status: 'pending_payment',
      source: 'workspace_seed',
      created_at_ms: 1710000001,
    }, { status: 201 });
  };
  globalThis.window = undefined;

  try {
    await portalApi.createPortalCommerceOrder({
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      coupon_code: 'SPRING20',
    }, 'portal-session');
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, [
    {
      url: '/api/portal/commerce/orders',
      method: 'POST',
      body: JSON.stringify({
        target_kind: 'recharge_pack',
        target_id: 'pack-100k',
        coupon_code: 'SPRING20',
      }),
    },
  ]);
});

test('browser portal api settles and cancels commerce orders through the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input, init) => {
    requests.push({
      url: String(input),
      method: init?.method ?? 'GET',
      body: init?.body ?? null,
    });
    return jsonResponse({
      order_id: 'commerce-order-1',
      project_id: 'project-demo',
      user_id: 'portal-user',
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      target_name: 'Boost 100k',
      list_price_label: '$40.00',
      payable_price_label: '$32.00',
      granted_units: 100000,
      bonus_units: 0,
      status: 'fulfilled',
      source: 'workspace_seed',
      created_at_ms: 1710000001,
    });
  };
  globalThis.window = undefined;

  try {
    await portalApi.settlePortalCommerceOrder('commerce-order-1', 'portal-session');
    await portalApi.cancelPortalCommerceOrder('commerce-order-1', 'portal-session');
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, [
    {
      url: '/api/portal/commerce/orders/commerce-order-1/settle',
      method: 'POST',
      body: JSON.stringify({}),
    },
    {
      url: '/api/portal/commerce/orders/commerce-order-1/cancel',
      method: 'POST',
      body: JSON.stringify({}),
    },
  ]);
});

test('browser portal api gets commerce checkout sessions through the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return jsonResponse({
      order_id: 'commerce-order-1',
      order_status: 'pending_payment',
      session_status: 'open',
      provider: 'manual_lab',
      mode: 'operator_settlement',
      reference: 'PAY-ORDER-1',
      payable_price_label: '$32.00',
      guidance: 'Settle the order to apply quota changes.',
      methods: [
        {
          id: 'manual_settlement',
          label: 'Manual settlement',
          detail: 'Use the portal settle action in local or lab mode.',
          action: 'settle_order',
          availability: 'available',
        },
      ],
    });
  };
  globalThis.window = undefined;

  try {
    await portalApi.getPortalCommerceCheckoutSession('commerce-order-1', 'portal-session');
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, ['/api/portal/commerce/orders/commerce-order-1/checkout-session']);
});

test('browser portal api sends commerce payment events through the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input, init) => {
    requests.push({
      url: String(input),
      method: init?.method ?? 'GET',
      body: init?.body ?? null,
    });
    return jsonResponse({
      order_id: 'commerce-order-1',
      project_id: 'project-demo',
      user_id: 'portal-user',
      target_kind: 'recharge_pack',
      target_id: 'pack-100k',
      target_name: 'Boost 100k',
      list_price_label: '$40.00',
      payable_price_label: '$32.00',
      granted_units: 100000,
      bonus_units: 0,
      status: 'failed',
      source: 'workspace_seed',
      created_at_ms: 1710000001,
    });
  };
  globalThis.window = undefined;

  try {
    await portalApi.sendPortalCommercePaymentEvent(
      'commerce-order-1',
      { event_type: 'failed' },
      'portal-session',
    );
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, [
    {
      url: '/api/portal/commerce/orders/commerce-order-1/payment-events',
      method: 'POST',
      body: JSON.stringify({
        event_type: 'failed',
      }),
    },
  ]);
});

test('browser portal api gets commerce membership through the relative proxy prefix', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return jsonResponse({
      membership_id: 'membership-1',
      project_id: 'project-demo',
      user_id: 'portal-user',
      plan_id: 'growth',
      plan_name: 'Growth',
      price_cents: 7900,
      price_label: '$79.00',
      cadence: '/month',
      included_units: 100000,
      status: 'active',
      source: 'workspace_seed',
      activated_at_ms: 1710000001,
      updated_at_ms: 1710000001,
    });
  };
  globalThis.window = undefined;

  try {
    await portalApi.getPortalCommerceMembership('portal-session');
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, ['/api/portal/commerce/membership']);
});

test('browser gateway base url uses the current host api prefix outside standalone dev ports', async () => {
  const portalApi = loadPortalApi();
  const originalWindow = globalThis.window;

  globalThis.window = {
    location: {
      origin: 'https://router.example.com',
      port: '443',
    },
  };

  try {
    assert.equal(
      await portalApi.resolveGatewayBaseUrl(),
      'https://router.example.com/api',
    );
  } finally {
    globalThis.window = originalWindow;
  }
});

test('standalone portal dev ports keep the direct gateway bind for local development', async () => {
  const portalApi = loadPortalApi();
  const originalWindow = globalThis.window;

  globalThis.window = {
    location: {
      origin: 'http://127.0.0.1:5174',
      port: '5174',
    },
  };

  try {
    assert.equal(
      await portalApi.resolveGatewayBaseUrl(),
      'http://127.0.0.1:8080',
    );
  } finally {
    globalThis.window = originalWindow;
  }
});

test('desktop runtime health probes public, gateway, admin, and portal service routes', async () => {
  const requests = [];
  const portalApi = loadPortalApi();
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;

  globalThis.fetch = async (input) => {
    requests.push(String(input));
    return textResponse('ok');
  };
  globalThis.window = {
    isTauri: true,
    __TAURI_INTERNALS__: {
      invoke: async (command) => {
        assert.equal(command, 'runtime_desktop_snapshot');
        return {
          mode: 'desktop',
          roles: ['web', 'gateway', 'admin', 'portal'],
          publicBaseUrl: 'http://127.0.0.1:48123',
          publicBindAddr: '127.0.0.1:48123',
          gatewayBindAddr: '127.0.0.1:8080',
          adminBindAddr: '127.0.0.1:8081',
          portalBindAddr: '127.0.0.1:8082',
        };
      },
    },
    location: {
      origin: 'http://127.0.0.1:48123',
      port: '48123',
    },
  };

  try {
    const snapshot = await portalApi.getProductRuntimeHealthSnapshot();
    assert.equal(snapshot.mode, 'desktop');
    assert.deepEqual(
      snapshot.services.map((service) => service.id),
      ['web', 'gateway', 'admin', 'portal'],
    );
    assert.deepEqual(
      snapshot.services.map((service) => service.status),
      ['healthy', 'healthy', 'healthy', 'healthy'],
    );
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.window = originalWindow;
  }

  assert.deepEqual(requests, [
    'http://127.0.0.1:48123/',
    'http://127.0.0.1:8080/health',
    'http://127.0.0.1:8081/admin/health',
    'http://127.0.0.1:8082/portal/health',
  ]);
});

test('desktop runtime restart uses the tauri runtime management bridge', async () => {
  const commands = [];
  const portalApi = loadPortalApi();
  const originalWindow = globalThis.window;

  globalThis.window = {
    isTauri: true,
    __TAURI_INTERNALS__: {
      invoke: async (command) => {
        commands.push(command);
        assert.equal(command, 'restart_product_runtime');
        return {
          mode: 'desktop',
          roles: ['web', 'gateway', 'admin', 'portal'],
          publicBaseUrl: 'http://127.0.0.1:49123',
          publicBindAddr: '127.0.0.1:49123',
          gatewayBindAddr: '127.0.0.1:9080',
          adminBindAddr: '127.0.0.1:9081',
          portalBindAddr: '127.0.0.1:9082',
        };
      },
    },
  };

  try {
    const snapshot = await portalApi.restartDesktopRuntime();
    assert.equal(snapshot.mode, 'desktop');
    assert.equal(snapshot.publicBaseUrl, 'http://127.0.0.1:49123');
    assert.equal(snapshot.gatewayBindAddr, '127.0.0.1:9080');
  } finally {
    globalThis.window = originalWindow;
  }

  assert.deepEqual(commands, ['restart_product_runtime']);
});
