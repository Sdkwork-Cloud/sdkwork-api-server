# Install Layout

This page defines the production install layout used by `portable` and `system` modes.

## Layout Modes

### Portable

Use `portable` for:

- local validation
- CI smoke tests
- explicit single-directory installs

Default root:

- `artifacts/install/sdkwork-api-router/current/`

### System

Use `system` for:

- production servers
- long-lived private deployments
- service-managed startup

## Logical Roots

Every install mode is described by the same logical roots:

- program home
- config home
- data home
- log home
- run home
- service definition home

## Default System Layout By OS

### Linux

- program home: `/opt/sdkwork-api-router/current/`
- config home: `/etc/sdkwork-api-router/`
- config file: `/etc/sdkwork-api-router/router.yaml`
- config fragments: `/etc/sdkwork-api-router/conf.d/`
- env file: `/etc/sdkwork-api-router/router.env`
- data home: `/var/lib/sdkwork-api-router/`
- log home: `/var/log/sdkwork-api-router/`
- run home: `/run/sdkwork-api-router/`

### macOS

- program home: `/usr/local/lib/sdkwork-api-router/current/`
- config home: `/Library/Application Support/sdkwork-api-router/`
- config file: `/Library/Application Support/sdkwork-api-router/router.yaml`
- config fragments: `/Library/Application Support/sdkwork-api-router/conf.d/`
- env file: `/Library/Application Support/sdkwork-api-router/router.env`
- data home: `/Library/Application Support/sdkwork-api-router/data/`
- log home: `/Library/Logs/sdkwork-api-router/`
- run home: `/Library/Application Support/sdkwork-api-router/run/`

### Windows

- program home: `C:\Program Files\sdkwork-api-router\current\`
- config home: `C:\ProgramData\sdkwork-api-router\`
- config file: `C:\ProgramData\sdkwork-api-router\router.yaml`
- config fragments: `C:\ProgramData\sdkwork-api-router\conf.d\`
- env file: `C:\ProgramData\sdkwork-api-router\router.env`
- data home: `C:\ProgramData\sdkwork-api-router\data\`
- log home: `C:\ProgramData\sdkwork-api-router\log\`
- run home: `C:\ProgramData\sdkwork-api-router\run\`

## Config Discovery

Primary config discovery order:

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

`conf.d/*.yaml` is loaded after the primary file in lexical order.

## Config Precedence

Effective precedence from lowest to highest:

- built-in defaults
- environment fallback
- config file
- CLI

Discovery exception:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

These two variables are used first so the runtime can locate the config files to load.

## Database Defaults

- `portable`
  - SQLite is acceptable for local validation
- `system`
  - PostgreSQL is the default contract

In `system` mode, SQLite is rejected unless an explicit development override is enabled.
