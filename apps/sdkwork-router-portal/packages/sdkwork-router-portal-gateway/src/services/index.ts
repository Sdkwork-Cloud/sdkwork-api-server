import type {
  PortalCommerceCatalog,
  PortalCommerceMembership,
  PortalDashboardSummary,
  PortalDesktopRuntimeSnapshot,
  PortalGatewayRateLimitSnapshot,
  PortalRuntimeHealthSnapshot,
  PortalRuntimeServiceHealth,
} from 'sdkwork-router-portal-types';

import type { GatewayCommandCenterSnapshot } from '../types';

function formatUnits(value: number): string {
  return new Intl.NumberFormat('en-US').format(value);
}

function joinUrl(baseUrl: string, path: string): string {
  const normalizedBase = baseUrl.replace(/\/+$/g, '');
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${normalizedBase}${normalizedPath}`;
}

function buildRuntimeCards(input: {
  desktopRuntime: PortalDesktopRuntimeSnapshot | null;
  gatewayBaseUrl: string;
}) {
  const { desktopRuntime, gatewayBaseUrl } = input;
  if (!desktopRuntime) {
    return [
      {
        id: 'runtime-context',
        label: 'Launch context',
        value: 'Browser or hosted portal session',
        detail:
          'Desktop runtime evidence becomes live when the command center runs inside the portal desktop shell with the Tauri bridge available.',
      },
      {
        id: 'runtime-gateway-base',
        label: 'Gateway base',
        value: gatewayBaseUrl,
        detail:
          'The current session can still verify the shared gateway surface even when local loopback binds are not exposed to the browser.',
      },
      {
        id: 'runtime-desktop-bridge',
        label: 'Desktop bridge',
        value: 'Unavailable in this session',
        detail:
          'Use the desktop app when you need the exact local web, gateway, admin, and portal bind evidence owned by the integrated runtime.',
      },
    ];
  }

  return [
    {
      id: 'runtime-mode',
      label: 'Launch mode',
      value: 'Desktop embedded runtime',
      detail:
        'The current portal session is backed by the local product runtime rather than a remote browser-only proxy path.',
    },
    {
      id: 'runtime-roles',
      label: 'Roles online',
      value: desktopRuntime.roles.join(', '),
      detail:
        'Desktop mode keeps web, gateway, admin, and portal online together so onboarding and control-plane work stay on one machine.',
    },
    {
      id: 'runtime-public-bind',
      label: 'Public web bind',
      value: desktopRuntime.publicBindAddr ?? 'Unavailable',
      detail:
        desktopRuntime.publicBaseUrl
          ? `The embedded web host currently resolves to ${desktopRuntime.publicBaseUrl}.`
          : 'The public web host did not expose a resolved base URL.',
    },
    {
      id: 'runtime-gateway-bind',
      label: 'Gateway bind',
      value: desktopRuntime.gatewayBindAddr ?? 'Unavailable',
      detail:
        'Local gateway traffic remains inside the same product runtime that also fronts the admin and portal APIs.',
    },
    {
      id: 'runtime-admin-bind',
      label: 'Admin bind',
      value: desktopRuntime.adminBindAddr ?? 'Unavailable',
      detail:
        'The admin control plane is started by the same desktop runtime and remains isolated to loopback in desktop mode.',
    },
    {
      id: 'runtime-portal-bind',
      label: 'Portal bind',
      value: desktopRuntime.portalBindAddr ?? 'Unavailable',
      detail:
        'The portal API stays local, so authentication, commerce reads, and command-center posture are all sourced from the same workstation runtime.',
    },
  ];
}

function buildCommerceCatalogCards(
  catalog: PortalCommerceCatalog,
  membership: PortalCommerceMembership | null,
) {
  const liveCouponCount = catalog.coupons.filter((coupon) => coupon.source === 'live').length;
  const couponCodes = catalog.coupons
    .slice(0, 3)
    .map((coupon) => coupon.code)
    .join(', ');

  return [
    {
      id: 'catalog-plans',
      label: 'Subscription plans',
      value: `${catalog.plans.length} plan(s)`,
      detail:
        'Commerce catalog membership and subscription posture is now exposed through the portal backend catalog instead of living only inside a frontend seed seam.',
    },
    {
      id: 'catalog-packs',
      label: 'Recharge packs',
      value: `${catalog.packs.length} pack(s)`,
      detail:
        'Top-up and recharge options are visible as backend-backed catalog entries, which keeps future checkout work anchored to a stable contract.',
    },
    {
      id: 'catalog-membership',
      label: 'Active membership',
      value: membership ? membership.plan_name : 'No active membership',
      detail: membership
        ? `${membership.plan_name} is active with ${formatUnits(membership.included_units)} included units on a ${membership.cadence} cadence.`
        : 'Subscription purchases will promote the current workspace into an explicit active membership state instead of leaving plans as catalog-only entries.',
    },
    {
      id: 'catalog-coupons',
      label: 'Coupon offers',
      value: `${catalog.coupons.length} coupon(s)`,
      detail:
        liveCouponCount > 0
          ? `Active live campaigns currently visible: ${couponCodes || 'available'}.`
          : 'No live campaigns were found, so the catalog falls back to the seeded launch offers.',
    },
  ];
}

function buildRateLimitCards(snapshot: PortalGatewayRateLimitSnapshot) {
  return [
    {
      id: 'rate-limit-policies',
      label: 'Policy roster',
      value: `${snapshot.policy_count} policy(s)`,
      detail:
        snapshot.policy_count > 0
          ? `${snapshot.active_policy_count} active policy(s) are currently shaping the gateway posture for project ${snapshot.project_id}.`
          : 'No project-scoped rate-limit policies are configured yet.',
    },
    {
      id: 'rate-limit-windows',
      label: 'Live windows',
      value: `${snapshot.window_count} window(s)`,
      detail:
        snapshot.window_count > 0
          ? 'Window snapshots are being tracked directly from the control plane so the portal can show live pressure instead of static policy text.'
          : 'No live window snapshots are currently available for this project.',
    },
    {
      id: 'rate-limit-exceeded',
      label: 'Over-limit windows',
      value: `${snapshot.exceeded_window_count} flagged`,
      detail:
        snapshot.exceeded_window_count > 0
          ? 'At least one active window is currently over limit, so the gateway posture should be treated as under pressure.'
          : 'No active window is currently over limit.',
    },
    {
      id: 'rate-limit-summary',
      label: 'Operator headline',
      value: snapshot.headline,
      detail: snapshot.detail,
    },
  ];
}

function buildDefaultRateLimitSnapshot(projectId: string): PortalGatewayRateLimitSnapshot {
  return {
    project_id: projectId,
    policy_count: 0,
    active_policy_count: 0,
    window_count: 0,
    exceeded_window_count: 0,
    headline: 'No project-scoped rate-limit policy is configured yet.',
    detail:
      'The command center is falling back to an empty rate-limit posture because no policy snapshot is available for the current workspace yet.',
    generated_at_ms: Date.now(),
    policies: [],
    windows: [],
  };
}

function buildServiceHealthSummary(runtimeHealth: PortalRuntimeHealthSnapshot) {
  const healthyCount = runtimeHealth.services.filter((service) => service.status === 'healthy').length;
  const degradedCount = runtimeHealth.services.filter((service) => service.status === 'degraded').length;
  const unreachableCount = runtimeHealth.services.filter((service) => service.status === 'unreachable').length;

  if (unreachableCount > 0) {
    return {
      value: `${healthyCount}/${runtimeHealth.services.length} healthy`,
      detail:
        `${unreachableCount} role(s) are unreachable from the current session, so the command center is surfacing a real runtime gap instead of static launch copy.`,
    };
  }

  if (degradedCount > 0) {
    return {
      value: `${healthyCount}/${runtimeHealth.services.length} healthy`,
      detail:
        `${degradedCount} role(s) returned degraded health responses, so the command center is flagging live service issues before launch traffic ramps up.`,
    };
  }

  return {
    value: `${healthyCount}/${runtimeHealth.services.length} healthy`,
    detail:
      'Every visible product role responded successfully to its live health route, turning the command center into an actual runtime status panel.',
  };
}

function summarizeServiceLabels(services: PortalRuntimeServiceHealth[]): string {
  return services.map((service) => service.label).join(', ');
}

function buildLaunchReadiness(input: {
  dashboard: PortalDashboardSummary;
  membership: PortalCommerceMembership | null;
  runtimeHealth: PortalRuntimeHealthSnapshot;
  rateLimitSnapshot: PortalGatewayRateLimitSnapshot;
}) {
  const { dashboard, membership, runtimeHealth, rateLimitSnapshot } = input;
  const blockers: string[] = [];
  const watchpoints: string[] = [];
  let score = 100;

  const unreachableServices = runtimeHealth.services.filter((service) => service.status === 'unreachable');
  const degradedServices = runtimeHealth.services.filter((service) => service.status === 'degraded');

  if (dashboard.api_key_count === 0) {
    blockers.push(
      'No visible project API key is available yet, so Codex, Claude Code, Gemini CLI, and OpenClaw onboarding remain blocked.',
    );
    score -= 30;
  }

  if (dashboard.billing_summary.exhausted) {
    blockers.push(
      'Runway exhausted, so launch traffic should wait until credits, a recharge pack, or a subscription order restores quota.',
    );
    score -= 30;
  } else if ((dashboard.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    watchpoints.push(
      'Remaining units are below 5,000, so the current launch runway is thin and should be reinforced before traffic ramps up.',
    );
    score -= 10;
  }

  if (unreachableServices.length > 0) {
    blockers.push(
      `${summarizeServiceLabels(unreachableServices)} ${
        unreachableServices.length === 1 ? 'is' : 'are'
      } unreachable from the current session and should be recovered before production traffic is pointed at this gateway.`,
    );
    score -= 25;
  } else if (degradedServices.length > 0) {
    watchpoints.push(
      `${summarizeServiceLabels(degradedServices)} ${
        degradedServices.length === 1 ? 'is' : 'are'
      } degraded and should be stabilized before the next launch window.`,
    );
    score -= 15;
  }

  if (rateLimitSnapshot.policy_count === 0) {
    watchpoints.push(
      'No project-scoped rate-limit policy is configured yet, so the gateway still lacks a visible request-frequency guardrail.',
    );
    score -= 10;
  } else if (rateLimitSnapshot.exceeded_window_count > 0) {
    blockers.push(
      `${rateLimitSnapshot.exceeded_window_count} live window snapshot(s) are over limit, so the gateway is currently running hot and should be throttled or rebalanced before launch traffic expands.`,
    );
    score -= 20;
  } else if (rateLimitSnapshot.window_count > 0) {
    watchpoints.push(
      `${rateLimitSnapshot.window_count} live window snapshot(s) are active and within limits, so the router has a readable request-frequency posture.`,
    );
    score -= 5;
  }

  if (!membership) {
    watchpoints.push(
      'No active membership is recorded yet, so recurring entitlement posture has not been established for this workspace.',
    );
    score -= 10;
  }

  const status = blockers.length > 0 ? 'blocked' : watchpoints.length > 0 ? 'watch' : 'ready';
  const normalizedScore = Math.max(0, Math.min(100, score));

  return {
    score: normalizedScore,
    status,
    headline:
      status === 'blocked'
        ? 'Launch readiness is currently blocked'
        : status === 'watch'
          ? 'Launch readiness is viable with watchpoints'
          : 'Launch readiness is ready',
    detail:
      status === 'blocked'
        ? 'Critical blockers are still present across access, runtime health, or billing runway, so the command center is holding the launch posture in a blocked state.'
        : status === 'watch'
          ? 'The workspace can move forward, but the command center is surfacing watchpoints that should be cleared before growth traffic expands.'
          : 'Gateway access, runtime health, and commercial runway are aligned well enough for real traffic onboarding.',
    blockersHeading: 'Critical blockers',
    blockers,
    watchpointsHeading: 'Watchpoints',
    watchpoints,
  } as const;
}

function buildRuntimeControls(desktopRuntime: PortalDesktopRuntimeSnapshot | null) {
  if (!desktopRuntime) {
    return [
      {
        id: 'restart-desktop-runtime',
        title: 'Restart desktop runtime',
        detail:
          'Desktop runtime controls are only available inside the portal desktop shell where the local app owns web, gateway, admin, and portal services directly.',
        cta: 'Restart desktop runtime',
        action: 'restart-desktop-runtime' as const,
        enabled: false,
        tone: 'ghost' as const,
      },
    ];
  }

  return [
    {
      id: 'restart-desktop-runtime',
      title: 'Restart desktop runtime',
      detail:
        `Restart the embedded ${desktopRuntime.roles.join(', ')} runtime without leaving the portal command center so service binds and local health can be recovered in place.`,
      cta: 'Restart desktop runtime',
      action: 'restart-desktop-runtime' as const,
      enabled: true,
      tone: 'secondary' as const,
    },
  ];
}

export function buildGatewayCommandCenterSnapshot(input: {
  commerceCatalog: PortalCommerceCatalog;
  membership: PortalCommerceMembership | null;
  dashboard: PortalDashboardSummary;
  desktopRuntime: PortalDesktopRuntimeSnapshot | null;
  gatewayBaseUrl: string;
  rateLimitSnapshot?: PortalGatewayRateLimitSnapshot | null;
  runtimeHealth: PortalRuntimeHealthSnapshot;
}): GatewayCommandCenterSnapshot {
  const {
    commerceCatalog,
    membership,
    dashboard,
    desktopRuntime,
    gatewayBaseUrl,
    rateLimitSnapshot: rawRateLimitSnapshot,
    runtimeHealth,
  } = input;
  const rateLimitSnapshot =
    rawRateLimitSnapshot
    ?? buildDefaultRateLimitSnapshot(dashboard.workspace.project.id);
  const remainingUnits =
    dashboard.billing_summary.remaining_units === null || dashboard.billing_summary.remaining_units === undefined
      ? 'Unlimited'
      : formatUnits(dashboard.billing_summary.remaining_units);
  const launchReadiness = buildLaunchReadiness({
    dashboard,
    membership,
    runtimeHealth,
    rateLimitSnapshot,
  });
  const serviceHealthSummary = buildServiceHealthSummary(runtimeHealth);

  return {
    gatewayBaseUrl,
    postureCards: [
      {
        id: 'entrypoint',
        label: 'Product entrypoint',
        value: 'sdkwork-router-portal',
        detail:
          'The portal now fronts the router product instead of hiding desktop mode and server mode behind separate engineering entrypoints.',
      },
      {
        id: 'compatibility',
        label: 'Protocol families',
        value: 'OpenAI + Anthropic + Gemini',
        detail:
          'One gateway surface carries OpenAI-compatible execution plus translated Anthropic Messages and Gemini Generative Language routes.',
      },
      {
        id: 'desktop',
        label: 'Desktop mode',
        value: 'Portal + Admin + Gateway + Web host',
        detail:
          'desktop mode starts the local product as one loopback-owned stack so onboarding, control-plane work, and gateway calls stay on the same machine.',
      },
      {
        id: 'gateway-traffic',
        label: 'Visible traffic',
        value: `${formatUnits(dashboard.usage_summary.total_requests)} requests`,
        detail:
          'Gateway posture is tied to the current workspace traffic instead of a blank launch story.',
      },
      {
        id: 'access-readiness',
        label: 'Access readiness',
        value: `${formatUnits(dashboard.api_key_count)} API keys`,
        detail:
          dashboard.api_key_count > 0
            ? 'The workspace already has visible project keys and can move straight into tool onboarding or route validation.'
            : 'No project key is visible yet, so the first launch step is still credential issuance.',
      },
      {
        id: 'commerce',
        label: 'Commerce posture',
        value: dashboard.billing_summary.exhausted ? 'Runway exhausted' : `${remainingUnits} units left`,
        detail:
          `API key issuance, routing posture, credits, billing, and account runway remain linked so growth and recovery decisions stay evidence-backed across ${commerceCatalog.plans.length} plan(s), ${commerceCatalog.packs.length} pack(s), and ${commerceCatalog.coupons.length} coupon offer(s).`,
      },
      {
        id: 'runtime-health',
        label: 'Live service health',
        value: serviceHealthSummary.value,
        detail: serviceHealthSummary.detail,
      },
    ],
    launchReadiness,
    runtimeCards: buildRuntimeCards({
      desktopRuntime,
      gatewayBaseUrl,
    }),
    runtimeHealth,
    serviceHealthChecks: runtimeHealth.services,
    runtimeControls: buildRuntimeControls(desktopRuntime),
    rateLimitCards: buildRateLimitCards(rateLimitSnapshot),
    rateLimitSnapshot,
    compatibilityRows: [
      {
        id: 'codex',
        tool: 'Codex',
        protocol: 'OpenAI / Responses',
        routeFamily: '/v1/responses',
        truth: 'OpenAI-compatible gateway surface',
        outcome:
          'Use one workspace API key against the routed gateway without creating a second credential boundary.',
      },
      {
        id: 'claude-code',
        tool: 'Claude Code',
        protocol: 'Anthropic Messages',
        routeFamily: '/v1/messages',
        truth: 'translated compatibility route',
        outcome:
          'Claude Code keeps Anthropic-style requests while the router preserves shared routing, quota, billing, usage recording, and upstream relay of anthropic-version plus anthropic-beta headers.',
      },
      {
        id: 'opencode',
        tool: 'OpenCode',
        protocol: 'OpenAI Chat / Responses',
        routeFamily: '/v1/chat/completions and /v1/responses',
        truth: 'OpenAI-compatible gateway surface',
        outcome:
          'OpenCode can stay on OpenAI-shaped configuration while the router handles provider abstraction behind the same base URL.',
      },
      {
        id: 'gemini',
        tool: 'Gemini CLI and Gemini-compatible clients',
        protocol: 'Gemini Generative Language',
        routeFamily: '/v1beta/models/{model}:generateContent',
        truth: 'translated compatibility route',
        outcome:
          'Gemini CLI can use the official GOOGLE_GEMINI_BASE_URL plus GEMINI_API_KEY_AUTH_MECHANISM=bearer path while the router keeps billing and routing policy in the shared gateway flow.',
      },
      {
        id: 'openclaw',
        tool: 'OpenClaw',
        protocol: 'OpenAI provider manifest',
        routeFamily: 'desktop-assisted provider install',
        truth: 'desktop-assisted setup flow',
        outcome:
          'OpenClaw instances can be pointed at the routed gateway from the portal desktop shell instead of being configured manually per instance.',
      },
    ],
    modeCards: [
      {
        id: 'desktop-mode',
        title: 'Desktop mode',
        command: 'pnpm product:start',
        summary:
          'Start the full router product locally with the portal desktop shell as the operator-facing entrypoint.',
        notes: [
          'Starts admin, gateway, portal, and the public web host together.',
          'Uses loopback-owned binds and desktop-assisted runtime base URL discovery.',
          'Best fit for local labs, private workstation gateways, and OpenClaw-assisted onboarding.',
        ],
      },
      {
        id: 'server-mode',
        title: 'Server mode',
        command: 'pnpm product:start -- server',
        summary:
          'Run the same product as a server entrypoint so the portal, admin, and gateway can be served to remote users.',
        notes: [
          'Serves /portal/*, /admin/*, and /api/v1/* from the shared router product.',
          'Keeps admin, portal, and gateway under one product plan instead of three unrelated startup paths.',
          'Best fit for hosted teams, private clusters, and shared gateway operations.',
        ],
      },
      {
        id: 'role-sliced',
        title: 'Role-sliced topology',
        command: 'pnpm server:start -- --roles web,gateway,admin,portal',
        summary:
          'Split the product across edge, control-plane, and data-plane nodes when a single process is no longer the right deployment shape.',
        notes: [
          'The canonical role set is web, gateway, admin, portal.',
          'Supports single-node all-in-one and split-role deployments from the same product runtime.',
          'Pairs cleanly with dry-run planning before rollout.',
        ],
      },
    ],
    topologyPlaybooks: [
      {
        id: 'single-node-local',
        title: 'Single-node local product',
        command: 'pnpm product:start',
        topology: 'Desktop shell plus local gateway stack',
        detail:
          'Use this when the user wants a private API router on their own machine with admin, portal, and gateway started together.',
      },
      {
        id: 'single-node-server',
        title: 'Single-node server',
        command: 'pnpm product:start -- server',
        topology: 'One process owns web, gateway, admin, and portal',
        detail:
          'Use this when the product should be hosted as one deployable router service without immediately splitting the topology.',
      },
      {
        id: 'edge-only',
        title: 'Edge-only web node',
        command:
          'pnpm server:start -- --dry-run --roles web --gateway-upstream 10.0.0.21:8080 --admin-upstream 10.0.0.22:8081 --portal-upstream 10.0.0.23:8082',
        topology: 'Web edge proxies to dedicated API nodes',
        detail:
          'Use this when traffic termination and public site serving should stay separate from control-plane and gateway execution.',
      },
      {
        id: 'split-plane',
        title: 'Split control-plane and data-plane',
        command: 'pnpm server:start -- --roles gateway or admin,portal',
        topology: 'Independent control-plane and data-plane services',
        detail:
          'Use this when operator traffic, public portal traffic, and gateway execution need different scaling or trust boundaries.',
      },
    ],
    verificationSnippets: [
      {
        id: 'openai-models',
        title: 'OpenAI-compatible route check',
        routeFamily: '/api/v1/models',
        command: [
          `curl ${joinUrl(gatewayBaseUrl, '/v1/models')} \\`,
          '  -H "Authorization: Bearer <project-api-key>"',
        ].join('\n'),
      },
      {
        id: 'anthropic-messages',
        title: 'Anthropic Messages route check',
        routeFamily: '/v1/messages',
        command: [
          `curl ${joinUrl(gatewayBaseUrl, '/v1/messages')} \\`,
          '  -H "x-api-key: <project-api-key>" \\',
          '  -H "anthropic-version: 2023-06-01" \\',
          '  -H "anthropic-beta: tools-2024-04-04" \\',
          '  -H "content-type: application/json" \\',
          '  -d \'{"model":"claude-sonnet-4","max_tokens":64,"messages":[{"role":"user","content":"Ping SDKWork Router"}]}\'',
        ].join('\n'),
      },
      {
        id: 'gemini-generate-content',
        title: 'Gemini generateContent route check',
        routeFamily: '/v1beta/models/{model}:generateContent',
        command: [
          `curl "${joinUrl(gatewayBaseUrl, '/v1beta/models/gemini-2.5-pro:generateContent')}?key=<project-api-key>" \\`,
          '  -H "content-type: application/json" \\',
          '  -d \'{"contents":[{"role":"user","parts":[{"text":"Ping SDKWork Router"}]}]}\'',
        ].join('\n'),
      },
    ],
    commerceCatalogCards: buildCommerceCatalogCards(commerceCatalog, membership),
    readinessActions: [
      {
        id: 'api-keys',
        title: 'Access and onboarding',
        detail:
          'Issue or rotate the workspace key, then use the quick-setup flow for Codex, Claude Code, OpenCode, Gemini CLI, or OpenClaw.',
        cta: 'Open API Keys',
        route: 'api-keys',
        tone: 'primary',
      },
      {
        id: 'routing',
        title: 'Routing guardrails',
        detail:
          'Confirm provider order, reliability guardrails, and preview evidence before real traffic is allowed to fan out across upstreams.',
        cta: 'Open Routing',
        route: 'routing',
        tone: 'secondary',
      },
      {
        id: 'billing',
        title: 'Runway and recovery',
        detail:
          'Connect launch posture with credits, billing, coupons, recharge packs, and account runway so traffic growth is commercially safe.',
        cta: 'Open Billing',
        route: 'billing',
        tone: 'ghost',
      },
    ],
  };
}
