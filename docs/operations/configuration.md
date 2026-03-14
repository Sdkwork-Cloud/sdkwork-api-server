# Configuration

This page summarizes the most important environment variables and runtime configuration choices.

## Bind Addresses

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`

## Persistence

- `SDKWORK_DATABASE_URL`

Supported databases:

- SQLite
- PostgreSQL

Examples:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
```

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## Authentication Secrets

- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`

## Secret Storage

- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`

Supported secret backends:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## Extension Runtime

- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`

## Runtime Snapshotting

- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`

## Configuration Strategy

For local development:

- default to SQLite
- keep loopback bind addresses
- use local encrypted secret storage or database-encrypted storage

For shared deployments:

- use PostgreSQL
- manage signing secrets explicitly
- use a deliberate secret backend strategy
- control extension search paths and trusted signers explicitly
