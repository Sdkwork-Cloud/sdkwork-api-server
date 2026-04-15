import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

function absolute(repoRoot, relativePath) {
  return path.join(repoRoot, relativePath);
}

function read(repoRoot, relativePath) {
  return readFileSync(absolute(repoRoot, relativePath), 'utf8');
}

function assertFile(repoRoot, relativePath) {
  assert.equal(existsSync(absolute(repoRoot, relativePath)), true, `missing ${relativePath}`);
}

function assertPatterns(text, label, patterns) {
  for (const pattern of patterns) {
    assert.match(text, pattern, `missing ${label} contract: ${pattern}`);
  }
}

function assertTracingBootstrap(repoRoot, relativePath, serviceName) {
  assertFile(repoRoot, relativePath);
  const text = read(repoRoot, relativePath);
  assertPatterns(text, relativePath, [
    /use sdkwork_api_observability::init_tracing;/,
    new RegExp(`init_tracing\\("${serviceName}"\\);`),
  ]);
}

export async function assertObservabilityContracts({
  repoRoot,
} = {}) {
  assertFile(repoRoot, 'docs/架构/135-可观测性与SLO治理设计-2026-04-07.md');
  assertFile(repoRoot, 'crates/sdkwork-api-observability/src/lib.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-observability/tests/telemetry_smoke.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-interface-http/src/gateway_routes.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-interface-http/tests/health_route.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-interface-admin/src/routes.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-interface-portal/src/lib.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-interface-portal/tests/portal_auth.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-app-extension/tests/runtime_observability.rs');
  assertFile(repoRoot, 'crates/sdkwork-api-app-jobs/tests/marketing_recovery_jobs.rs');

  const observability = read(repoRoot, 'crates/sdkwork-api-observability/src/lib.rs');
  assertPatterns(observability, 'sdkwork-api-observability', [
    /pub const REQUEST_ID_HEADER: &str = "x-request-id";/,
    /pub fn record_provider_health\(/,
    /pub fn record_provider_health_persist_failure\(/,
    /pub fn record_provider_health_recovery_probe\(/,
    /pub fn record_commerce_reconciliation_success\(/,
    /pub fn record_commerce_reconciliation_failure\(/,
    /pub fn record_marketing_recovery_success\(/,
    /pub fn record_marketing_recovery_failure\(/,
    /pub async fn observe_http_metrics\(/,
    /pub async fn observe_http_tracing\(/,
    /pub fn init_tracing\(service: &str\)/,
    /sdkwork_provider_health_status/,
    /sdkwork_provider_health_persist_failures_total/,
    /sdkwork_provider_health_recovery_probes_total/,
    /sdkwork_commerce_reconciliation_attempts_total/,
    /sdkwork_marketing_recovery_attempts_total/,
  ]);

  const gatewayRoutes = read(repoRoot, 'crates/sdkwork-api-interface-http/src/gateway_routes.rs');
  assertPatterns(gatewayRoutes, 'gateway observability routes', [
    /\.route\("\/metrics", metrics_route\(metrics\.clone\(\), &http_exposure\)\)/,
    /\.route\("\/health", get\(\|\| async \{ "ok" \}\)\)/,
    /observe_http_metrics/,
    /observe_http_tracing/,
  ]);

  const gatewayHealthTests = read(repoRoot, 'crates/sdkwork-api-interface-http/tests/health_route.rs');
  assertPatterns(gatewayHealthTests, 'gateway health and metrics tests', [
    /\.uri\("\/health"\)/,
    /\.header\("x-request-id", "gateway-caller-id"\)/,
    /\.uri\("\/metrics"\)/,
    /sdkwork_service_info\{service=\\"gateway\\"\} 1/,
    /sdkwork_http_requests_total\{service=\\"gateway\\",method=\\"GET\\",route=\\"\/health\\",status=\\"200\\",tenant=\\"none\\",model=\\"none\\",provider=\\"none\\",billing_mode=\\"none\\",retry_outcome=\\"none\\",failover_outcome=\\"none\\",payment_outcome=\\"none\\"\} 2/,
  ]);

  const adminRoutes = read(repoRoot, 'crates/sdkwork-api-interface-admin/src/routes.rs');
  assertPatterns(adminRoutes, 'admin observability routes', [
    /\.route\("\/metrics", metrics_route\(metrics\.clone\(\), &http_exposure\)\)/,
    /\.route\("\/admin\/health", get\(\|\| async \{ "ok" \}\)\)/,
    /"\/admin\/extensions\/runtime-statuses"/,
    /"\/admin\/billing\/events"/,
    /"\/admin\/billing\/events\/summary"/,
    /"\/admin\/billing\/account-holds"/,
    /"\/admin\/billing\/request-settlements"/,
    /"\/admin\/routing\/health-snapshots"/,
    /"\/admin\/routing\/decision-logs"/,
    /observe_http_metrics/,
    /observe_http_tracing/,
  ]);

  const adminAuthGuard = read(repoRoot, 'crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs');
  assertPatterns(adminAuthGuard, 'admin metrics auth tests', [
    /\.uri\("\/metrics"\)/,
    /Bearer local-dev-metrics-token/,
    /\.get\("x-request-id"\)/,
    /sdkwork_service_info\{service=\\"admin\\"\} 1/,
  ]);

  const portalRoutes = read(repoRoot, 'crates/sdkwork-api-interface-portal/src/lib.rs');
  assertPatterns(portalRoutes, 'portal observability routes', [
    /\.route\(\s*"\/metrics",\s*http::metrics_route\(metrics\.clone\(\), &http_exposure\),\s*\)/,
    /\.route\("\/portal\/health", get\(\|\| async \{ "ok" \}\)\)/,
    /"\/portal\/billing\/events"/,
    /"\/portal\/billing\/events\/summary"/,
    /"\/portal\/billing\/account\/request-settlements"/,
    /"\/portal\/routing\/decision-logs"/,
    /observe_http_metrics/,
    /observe_http_tracing/,
  ]);

  const portalAuth = read(repoRoot, 'crates/sdkwork-api-interface-portal/tests/portal_auth.rs');
  assertPatterns(portalAuth, 'portal metrics auth tests', [
    /\.uri\("\/metrics"\)/,
    /Bearer local-dev-metrics-token/,
  ]);

  const runtimeObservability = read(repoRoot, 'crates/sdkwork-api-app-extension/tests/runtime_observability.rs');
  assertPatterns(runtimeObservability, 'runtime observability tests', [
    /list_extension_runtime_statuses/,
    /capture_provider_health_snapshots/,
    /list_provider_health_snapshots/,
    /"health_path": "\/health"/,
    /captured\[0\]\.runtime, "native_dynamic"/,
    /captured\[0\]\.runtime, "builtin"/,
  ]);

  const marketingRecovery = read(repoRoot, 'crates/sdkwork-api-app-jobs/tests/marketing_recovery_jobs.rs');
  assertPatterns(marketingRecovery, 'marketing recovery telemetry tests', [
    /recover_expired_coupon_reservations/,
    /HttpMetricsRegistry::new\("marketing-recovery-expired-reservations"\)/,
    /sdkwork_marketing_recovery_attempts_total/,
    /sdkwork_marketing_expired_reservations_total/,
    /sdkwork_marketing_released_codes_total/,
  ]);

  assertTracingBootstrap(repoRoot, 'services/gateway-service/src/main.rs', 'gateway-service');
  assertTracingBootstrap(repoRoot, 'services/admin-api-service/src/main.rs', 'admin-api-service');
  assertTracingBootstrap(repoRoot, 'services/portal-api-service/src/main.rs', 'portal-api-service');
  assertTracingBootstrap(repoRoot, 'services/router-web-service/src/main.rs', 'router-web-service');
  assertTracingBootstrap(repoRoot, 'services/router-product-service/src/main.rs', 'router-product-service');
}
