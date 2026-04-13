# Configuration

This page defines the runtime configuration contract for the standalone SDKWork API Server services.

## Resolution Order

The runtime config merge order is:

1. built-in local defaults
2. local config file
3. `SDKWORK_*` environment variables

This means environment variables always win over values from `config.yaml`, `config.yml`, or `config.json`.

Runtime config file reload keeps using the original process-start environment override snapshot. Editing `config.yaml` while the service is running is supported for the reloadable fields listed below, but changing parent-shell environment variables after the process has already started is not observed.

All three standalone services now use a durable node identity for shared runtime coordination. `gateway-service` and `admin-api-service` participate in extension-runtime rollout, while `gateway-service`, `admin-api-service`, and `portal-api-service` all participate in standalone config rollout. Set `SDKWORK_SERVICE_INSTANCE_ID` when you want that identity to be stable across restarts and easy to correlate in rollout status.

## Default Local Config Root

The default local config root is:

- Linux and macOS: `~/.sdkwork/router/`
- Windows: `%USERPROFILE%\\.sdkwork\\router\\`

The services resolve the following built-in default paths under that root:

- primary YAML config: `config.yaml`
- fallback YAML config: `config.yml`
- fallback JSON config: `config.json`
- default SQLite database: `sdkwork-api-server.db`
- local encrypted secrets file: `secrets.json`
- extension directory: `extensions/`

## Config File Discovery

When `SDKWORK_CONFIG_FILE` is not set, the runtime searches in this order:

1. `config.yaml`
2. `config.yml`
3. `config.json`

The first existing file wins.

To override the root directory:

- `SDKWORK_CONFIG_DIR`

To override the file directly:

- `SDKWORK_CONFIG_FILE`

If `SDKWORK_CONFIG_FILE` is relative, it is resolved relative to `SDKWORK_CONFIG_DIR` or the default config root.

## Built-In Defaults

If no config file exists, the services still start with these values:

- `gateway_bind`: `127.0.0.1:8080`
- `admin_bind`: `127.0.0.1:8081`
- `portal_bind`: `127.0.0.1:8082`
- `database_url`: `sqlite://<config-root>/sdkwork-api-server.db`
- `cache_backend`: `memory`
- `cache_url`: unset
- `extension_paths`: `["<config-root>/extensions"]`
- `secret_local_file`: `<config-root>/secrets.json`
- `enable_connector_extensions`: `true`
- `enable_native_dynamic_extensions`: `false`
- `extension_hot_reload_interval_secs`: `0`
- `require_signed_connector_extensions`: `false`
- `require_signed_native_dynamic_extensions`: `true`
- `runtime_snapshot_interval_secs`: `0`
- `secret_backend`: `database_encrypted`
- `secret_keyring_service`: `sdkwork-api-server`

## File Schema

The local config file uses a flat top-level schema matching `StandaloneConfig`.

Supported fields:

- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `database_url`
- `cache_backend`
- `cache_url`
- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `runtime_snapshot_interval_secs`
- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

## Cache Backend Status

The config contract accepts `cache_backend` and optional `cache_url`, and the standalone runtime now links both built-in cache drivers through the cache driver registry:

- `memory`: supported for embedded, test, and single-process deployments
- `redis`: supported for shared, multi-process deployments through the in-repo Redis cache driver

The current cache consumers do not all activate in the same way:

- route-decision cache is enabled in the standalone gateway because it is process-local handoff state
- capability catalog cache is opt-in and currently enabled only when a runtime explicitly injects a shared cache store, such as the combined product runtime
- standalone `gateway-service` and `admin-api-service` enable capability catalog cache only when the configured backend supports shared cross-process coherence, which is currently `redis`
- standalone `gateway-service` and `admin-api-service` keep capability catalog cache disabled with the default `memory` backend to avoid cross-process stale reads

## Runtime Reload Behavior

All three standalone services poll their resolved config file set once per second.

Reloadable without restart:

- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `runtime_snapshot_interval_secs`
- `database_url`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `secret_backend`
- `credential_master_key`
- `credential_legacy_master_keys`
- `secret_local_file`
- `secret_keyring_service`

Still restart-required:
- `cache_backend`
- `cache_url`
- changes made only in the parent shell after the process already started
- binary upgrades and other out-of-process deployment changes

When a restart-required field changes on disk, the running process keeps the last applied value for that field, records the pending restart requirement, and logs that the change was detected but still needs process restart.

If a config file mixes reloadable and restart-required changes, the runtime applies only the reloadable subset immediately and keeps the restart-required subset pending.

Standalone config rollout reports these restart-required-only or mixed restart-required changes as participant `failed` with a `restart required for ...` message instead of marking the rollout as succeeded.

## YAML Example

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-server.db"
cache_backend: "memory"
extension_paths:
  - "extensions"
  - "extensions/partner"
enable_connector_extensions: true
enable_native_dynamic_extensions: false
extension_hot_reload_interval_secs: 5
extension_trusted_signers:
  sdkwork: "ZXhwaWNpdC1wdWJsaWMta2V5"
  partner: "c2Vjb25kLXB1YmxpYy1rZXk="
require_signed_connector_extensions: false
require_signed_native_dynamic_extensions: true
admin_jwt_signing_secret: "change-me-admin"
portal_jwt_signing_secret: "change-me-portal"
runtime_snapshot_interval_secs: 30
secret_backend: "local_encrypted_file"
credential_master_key: "change-me-master-key"
secret_local_file: "secrets.json"
secret_keyring_service: "sdkwork-api-server"
```

## JSON Example

```json
{
  "gateway_bind": "127.0.0.1:8080",
  "admin_bind": "127.0.0.1:8081",
  "portal_bind": "127.0.0.1:8082",
  "database_url": "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server",
  "cache_backend": "memory",
  "extension_paths": [
    "extensions"
  ],
  "enable_connector_extensions": true,
  "enable_native_dynamic_extensions": false,
  "secret_backend": "database_encrypted",
  "secret_local_file": "secrets.json"
}
```

## Path Normalization Rules

When values come from a config file:

- relative `secret_local_file` paths are resolved relative to the config file directory
- relative `extension_paths` entries are resolved relative to the config file directory
- relative SQLite file URLs are resolved relative to the config file directory and normalized into absolute SQLite URLs

Example:

- config file: `~/.sdkwork/router/config.yaml`
- `database_url: "sqlite://router.db"`
- resolved runtime value: `sqlite://~/.sdkwork/router/router.db`

Environment variables are applied after file loading and are used as-is.

## Environment Variables

The most important runtime environment variables are:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_CACHE_BACKEND`
- `SDKWORK_CACHE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_SERVICE_INSTANCE_ID`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`

## Startup Security Validation

The standalone runtime rejects insecure local-development secrets by default.

Rules:

- `SDKWORK_ADMIN_JWT_SIGNING_SECRET=local-dev-admin-jwt-secret` is rejected unless `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET=local-dev-portal-jwt-secret` is rejected unless `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`
- `SDKWORK_CREDENTIAL_MASTER_KEY=local-dev-master-key` is rejected unless `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`
- `SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS` must not contain `local-dev-master-key` unless `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`
- the built-in `admin@sdkwork.local` and `portal@sdkwork.local` demo accounts are seeded only when `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`

Recommended production posture:

- set strong, unique admin and portal JWT signing secrets
- set a non-demo credential master key
- leave `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP` unset or explicitly `false`

## Cluster Runtime Coordination

Standalone services now heartbeat a shared coordination identity into the admin store.

Rules:

- `SDKWORK_SERVICE_INSTANCE_ID` is used as the durable node ID when present
- otherwise the process synthesizes a node ID from service kind, process id, and startup time
- active gateway and admin nodes are targeted for `POST /admin/extensions/runtime-rollouts`
- active gateway, admin, and portal nodes are targeted for `POST /admin/runtime-config/rollouts`
- rollout participants are snapshotted at creation time, so later heartbeats do not mutate an already-created rollout
- standalone config rollout keeps its coordination ledger on the startup store snapshot by default, so `database_url` hot swaps do not move an in-flight rollout to a different database unexpectedly

Example:

```bash
export SDKWORK_SERVICE_INSTANCE_ID="gateway-us-east-01"
./target/release/gateway-service
```

## Startup Examples

Linux or macOS:

```bash
mkdir -p "$HOME/.sdkwork/router"
cat > "$HOME/.sdkwork/router/config.yaml" <<'EOF'
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
EOF

./target/release/gateway-service
```

Windows PowerShell:

```powershell
New-Item -ItemType Directory -Force "$HOME\\.sdkwork\\router" | Out-Null
@"
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
"@ | Set-Content -Encoding UTF8 "$HOME\\.sdkwork\\router\\config.yaml"

.\target\release\gateway-service.exe
```

Explicit file override:

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.json"
./target/release/gateway-service
```
