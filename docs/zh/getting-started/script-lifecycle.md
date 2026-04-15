# 脚本生命周期

本文档集中说明仓库内所有启动脚本的职责、适用场景、运行时状态目录、启动顺序、停止方式，以及开发态和发布态的完整生命周期。

建议结合下面这些文档一起看：

- [快速开始](/zh/getting-started/quickstart)
- [源码开发](/zh/getting-started/source-development)
- [发布构建](/zh/getting-started/release-builds)

## 两层脚本体系

### `scripts/dev/*`

这是源码级启动器。

适用场景：

- 正在仓库里开发
- 只想拉起某一个局部服务
- 需要前台调试和快速改代码验证

特点：

- 直接从源码目录运行
- 以前台进程为主
- 适合开发和调试
- 不维护独立的 PID、日志和安装目录生命周期

### `bin/*`

这是托管式脚本层。

适用场景：

- 需要统一的开发态或发布态脚本入口
- 希望自动管理运行目录、PID、日志和环境文件
- 希望在 Windows、Linux、macOS 上保持一致用法

特点：

- 自动创建并复用运行时目录
- 维护 PID、日志、配置文件
- 支持 dry-run、foreground 和 service manager
- 启动成功后输出统一入口、独立端口、bootstrap 身份引导和日志位置

## 脚本功能总表

| 脚本 | 范围 | 功能 | 状态目录 | 停止方式 |
|---|---|---|---|---|
| `bin/build.sh` / `bin/build.ps1` | 发布态 | 编译 release 二进制、前端静态资源、文档和桌面包 | `artifacts/release/` 及 Rust target 目录 | 构建结束后退出 |
| `bin/install.sh` / `bin/install.ps1` | 发布态 | 将 release 产物安装到运行目录 | 默认 `artifacts/install/sdkwork-api-router/current/` | 安装结束后退出 |
| `bin/start.sh` / `bin/start.ps1` | 发布态 | 启动安装后的 `router-product-service` | 安装目录下的 `var/log/`、`var/run/`、`config/router.env` | `bin/stop.sh` / `bin/stop.ps1` 或系统服务停止 |
| `bin/stop.sh` / `bin/stop.ps1` | 发布态 | 按 PID 停止托管的发布态运行时 | 安装目录 `var/run/` | 进程树停止后退出 |
| `bin/start-dev.sh` / `bin/start-dev.ps1` | 托管开发态 | 启动托管开发环境 | `artifacts/runtime/dev/` | `bin/stop-dev.sh` / `bin/stop-dev.ps1` 或前台 `Ctrl+C` |
| `bin/stop-dev.sh` / `bin/stop-dev.ps1` | 托管开发态 | 停止托管开发环境 | `artifacts/runtime/dev/run/` PID 文件 | 进程树停止后退出 |
| `scripts/dev/start-workspace.mjs` / `.ps1` | 源码开发态 | 一次拉起后端和前端或桌面壳 | 仓库源码目录 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-stack.mjs` / `start-servers.ps1` | 源码开发态 | 只启动后端服务 | 仓库源码目录 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-admin.mjs` | 源码开发态 | 只启动 admin 浏览器或 Tauri | 仓库源码目录 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-portal.mjs` | 源码开发态 | 只启动 portal 浏览器 | 仓库源码目录 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-web.mjs` | 源码开发态 | 构建 admin/portal 静态资源并通过 Pingora 统一暴露 | 仓库源码目录 | 当前终端 `Ctrl+C` |

## 默认端口模型

仓库里要区分两套默认端口。

### 托管脚本默认端口

托管脚本统一使用 `998x`，避免常见本地冲突：

- gateway：`127.0.0.1:9980`
- admin：`127.0.0.1:9981`
- portal：`127.0.0.1:9982`
- 统一 web host：`127.0.0.1:9983` 或 `0.0.0.0:9983`

### 服务二进制内建默认端口

如果直接运行原始服务二进制，并且不加脚本层覆盖，仍然使用服务内建默认值：

- gateway：`127.0.0.1:8080`
- admin：`127.0.0.1:8081`
- portal：`127.0.0.1:8082`

所以要明确区分：

- 走 `bin/*` 或更新后的源码辅助脚本，看 `998x`
- 直接裸跑服务二进制，看 `808x`

## 开发态生命周期

推荐开发流程是托管开发态：

### 1. 启动托管开发环境

Linux 或 macOS：

```bash
./bin/start-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

默认行为：

- 使用 `artifacts/runtime/dev/data/` 下的 SQLite
- 默认进入 preview 模式
- 内置 web host 成为主要统一入口
- 会等待后端和前端健康检查通过
- 成功后输出格式化启动摘要

### 2. 阅读启动摘要

启动成功后会打印：

- 统一入口：
  - `http://127.0.0.1:9983/admin/`
  - `http://127.0.0.1:9983/portal/`
- 统一健康检查：
  - `http://127.0.0.1:9983/api/v1/health`
- 独立服务：
  - `http://127.0.0.1:9980/health`
  - `http://127.0.0.1:9981/admin/health`
  - `http://127.0.0.1:9982/portal/health`
- 开发身份 bootstrap 指引：
  - 身份来自当前激活的 bootstrap profile
  - 在共享本地环境前先检查 `data/identities/dev.json`

### 3. 如需切回独立前端端口

如果你明确要使用 Vite 独立开发端口，而不是统一入口：

Linux 或 macOS：

```bash
./bin/start-dev.sh --browser
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser
```

此时主要访问地址是：

- admin：`http://127.0.0.1:5173/admin/`
- portal：`http://127.0.0.1:5174/portal/`
- 后端仍然是 `9980`、`9981`、`9982`

### 4. 停止托管开发环境

Linux 或 macOS：

```bash
./bin/stop-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

停止脚本会读取 PID 文件并尽量停止其拥有的进程树。

## 发布态生命周期

推荐发布流程使用托管发布脚本。

### 1. 编译发布产物

Linux 或 macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

### 2. 安装运行目录

Linux 或 macOS：

```bash
./bin/install.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

默认安装目录：

- `artifacts/install/sdkwork-api-router/current/`

关键内容：

- `bin/`
- `config/router.env`
- `sites/admin/dist`
- `sites/portal/dist`
- `var/log/`
- `var/run/`
- `service/systemd/`
- `service/launchd/`
- `service/windows-task/`

### 3. 检查并调整发布配置

重点检查：

- `config/router.env`

建议通过这里覆盖：

- bind 地址
- 数据库路径
- 站点静态目录
- 代理目标

### 4. 启动发布态运行时

Linux 或 macOS：

```bash
./bin/start.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1
```

发布态脚本会：

- 启动 `router-product-service`
- 默认使用安装目录下的 SQLite
- 等待统一健康检查通过
- 输出与开发态一致风格的启动摘要

### 5. 停止发布态运行时

Linux 或 macOS：

```bash
./bin/stop.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop.ps1
```

### 6. 可选：注册系统服务

在安装目录中执行：

- Linux / systemd：
  - `./service/systemd/install-service.sh`
  - `./service/systemd/uninstall-service.sh`
- macOS / launchd：
  - `./service/launchd/install-service.sh`
  - `./service/launchd/uninstall-service.sh`
- Windows / Task Scheduler：
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\install-service.ps1 -StartNow`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\uninstall-service.ps1`

如果是交给 systemd、launchd 或计划任务托管，请使用前台模式：

- `bin/start.sh --foreground`
- `bin/start.ps1 -Foreground`

## Dry-Run 生命周期

所有托管脚本都支持 dry-run。

- `./bin/build.sh --dry-run`
- `./bin/install.sh --dry-run`
- `./bin/start-dev.sh --dry-run`
- `./bin/start.sh --dry-run`

Windows：

- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun`

## 注意事项

- `bin/start-dev.*` 只用于源码树内的托管开发流程，不依赖 `bin/build.*` 或 `bin/install.*`
- `bin/start.*` 只用于安装后的发布运行时，不负责构建和安装
- `bin/stop-dev.*` 与 `bin/stop.*` 只管理各自运行目录中的 PID 和进程树
- gateway 没有默认用户名密码，它面向 portal 生成的 API key
- 前端调试时如果需要 Vite 热更新和独立端口，请使用 `--browser` 或直接用 `scripts/dev/*`
- 需要统一单端口对外访问时，优先使用 preview、发布态或服务托管模式
