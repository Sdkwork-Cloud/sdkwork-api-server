# Service Management

This page defines the supported service-manager contract for production installs.

## Supported Managers

- Linux: `systemd`
- macOS: `launchd`
- Windows: Windows Service Control Manager

`service/windows-task/` remains a compatibility asset. The formal Windows production path is `service/windows-service/`.

## Pre-Start Validation

Before registering or restarting a production service, run:

```bash
node bin/router-ops.mjs validate-config --mode system
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system
```

## Foreground Runtime Contract

Service managers should execute the runtime in foreground mode:

- `./bin/start.sh --foreground --home <install-root>`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -Foreground -Home <install-root>`

The generated service assets already follow this contract.

## Linux: systemd

Generated assets:

- `service/systemd/sdkwork-api-router.service`
- `service/systemd/install-service.sh`
- `service/systemd/uninstall-service.sh`

Typical lifecycle:

```bash
./service/systemd/install-service.sh
systemctl status sdkwork-api-router
./service/systemd/uninstall-service.sh
```

## macOS: launchd

Generated assets:

- `service/launchd/com.sdkwork.api-router.plist`
- `service/launchd/install-service.sh`
- `service/launchd/uninstall-service.sh`

Typical lifecycle:

```bash
./service/launchd/install-service.sh
sudo launchctl print system/com.sdkwork.api-router
./service/launchd/uninstall-service.sh
```

## Windows Service

Generated assets:

- `service/windows-service/run-service.ps1`
- `service/windows-service/install-service.ps1`
- `service/windows-service/uninstall-service.ps1`

Typical lifecycle:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-service\install-service.ps1
Get-Service SDKWorkApiRouter
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-service\uninstall-service.ps1
```

## Operational Notes

- Keep mutable files out of the program directory.
- Review `router.env` and `router.yaml` before each upgrade.
- Re-run `validate-config` after every config change.
- Treat `start.* --dry-run` and the product-service `--dry-run` plan output as preflight checks, not as a replacement for service registration.
