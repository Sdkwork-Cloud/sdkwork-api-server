# 服务管理

本页定义生产安装下支持的服务管理契约。

## 支持的服务管理器

- Linux：`systemd`
- macOS：`launchd`
- Windows：Windows Service Control Manager

`service/windows-task/` 继续保留为兼容资产，但正式的 Windows 生产路径已经切换为 `service/windows-service/`。

## 启动前校验

在注册服务或重启生产服务前，先执行：

```bash
node bin/router-ops.mjs validate-config --mode system
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system
```

## 前台运行契约

服务管理器应以前台模式拉起运行时：

- `./bin/start.sh --foreground --home <install-root>`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -Foreground -Home <install-root>`

生成的服务资产已经遵循这一约定。

## Linux: systemd

生成资产：

- `service/systemd/sdkwork-api-router.service`
- `service/systemd/install-service.sh`
- `service/systemd/uninstall-service.sh`

典型流程：

```bash
./service/systemd/install-service.sh
systemctl status sdkwork-api-router
./service/systemd/uninstall-service.sh
```

## macOS: launchd

生成资产：

- `service/launchd/com.sdkwork.api-router.plist`
- `service/launchd/install-service.sh`
- `service/launchd/uninstall-service.sh`

典型流程：

```bash
./service/launchd/install-service.sh
sudo launchctl print system/com.sdkwork.api-router
./service/launchd/uninstall-service.sh
```

## Windows Service

生成资产：

- `service/windows-service/run-service.ps1`
- `service/windows-service/install-service.ps1`
- `service/windows-service/uninstall-service.ps1`

典型流程：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-service\install-service.ps1
Get-Service SDKWorkApiRouter
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-service\uninstall-service.ps1
```

## 运维注意事项

- 可变数据不要写回程序目录。
- 每次升级前都要审阅 `router.env` 与 `router.yaml`。
- 每次修改配置后都重新执行 `validate-config`。
- `start.* --dry-run` 与产品服务的 `--dry-run` 计划输出属于预检查，不代替正式服务注册。
