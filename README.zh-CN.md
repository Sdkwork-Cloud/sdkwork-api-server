# sdkwork-api-server

[English Guide](./README.md)

SDKWork API Server 是一个基于 Axum 的 OpenAI 兼容网关、管理控制平面、公共门户和扩展运行时，技术栈包含 Rust、React、pnpm 和 Tauri。

## 仓库提供的运行面

运行时组成：

- `gateway-service`
  - OpenAI 兼容 `/v1/*` 网关
- `admin-api-service`
  - 面向运维人员的 `/admin/*` 控制平面
- `portal-api-service`
  - 面向最终用户的 `/portal/*` 自助 API
- `router-web-service`
  - 基于 Pingora 的公共 Web Host，对外承载 `/admin/*`、`/portal/*` 和 API 代理入口
- `apps/sdkwork-router-admin/`
  - 独立的超管浏览器应用，以及 admin 自带的 Tauri 桌面壳
- `apps/sdkwork-router-portal/`
  - 独立的开发者自助门户浏览器应用
- `docs/`
  - 使用 VitePress 构建的中英文运维与使用文档

当前基础能力：

- 基于 Axum 的 Rust 服务
- SQLite 与 PostgreSQL 存储
- 基于 Pingora 的 admin / portal 对外交付
- 独立浏览器 admin 和 portal 应用
- admin 自带 Tauri 桌面形态
- builtin、connector、native-dynamic 三类扩展运行时
- public portal 的注册、登录、控制台、用量、计费和 API key 签发
- 基于 `~/.sdkwork/router/` 的本地 JSON / YAML 运行配置

## 支持平台

- Windows
- Linux
- macOS

## 快速开始

独立服务支持本地配置目录和内建默认值。

默认配置根目录：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\`

配置文件查找顺序：

1. `config.yaml`
2. `config.yml`
3. `config.json`

配置优先级：

1. 内建本地默认值
2. 本地配置文件
3. `SDKWORK_*` 环境变量

如果没有配置文件，服务仍然会按本地默认值启动：

- gateway bind：`127.0.0.1:8080`
- admin bind：`127.0.0.1:8081`
- portal bind：`127.0.0.1:8082`
- SQLite 数据库：`~/.sdkwork/router/sdkwork-api-server.db`
- 扩展目录：`~/.sdkwork/router/extensions`
- 本地密钥文件：`~/.sdkwork/router/secrets.json`

示例 `config.yaml`：

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
secret_local_file: "secrets.json"
extension_paths:
  - "extensions"
enable_connector_extensions: true
enable_native_dynamic_extensions: false
```

配置文件中的相对路径会相对于配置文件所在目录解析。

如果要覆盖默认位置，可使用：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

## 文档索引

本地安装和预览文档站：

```bash
pnpm --dir docs install
pnpm --dir docs dev
```

构建文档站：

```bash
pnpm --dir docs build
```

英文文档主结构：

- Getting Started：
  - [Quickstart](./docs/getting-started/quickstart.md)
  - [Installation](./docs/getting-started/installation.md)
  - [Source Development](./docs/getting-started/source-development.md)
  - [Script Lifecycle](./docs/getting-started/script-lifecycle.md)
  - [Build and Packaging](./docs/getting-started/build-and-packaging.md)
  - [Release Builds](./docs/getting-started/release-builds.md)
- Architecture：
  - [Software Architecture](./docs/architecture/software-architecture.md)
  - [Functional Modules](./docs/architecture/functional-modules.md)
  - [Runtime Modes Deep Dive](./docs/architecture/runtime-modes.md)
- API Reference：
  - [Overview](./docs/api-reference/overview.md)
  - [Gateway API](./docs/api-reference/gateway-api.md)
  - [Admin API](./docs/api-reference/admin-api.md)
  - [Portal API](./docs/api-reference/portal-api.md)
- Operations：
  - [Configuration](./docs/operations/configuration.md)
  - [Health and Metrics](./docs/operations/health-and-metrics.md)
- Reference：
  - [API Compatibility](./docs/reference/api-compatibility.md)
  - [Repository Layout](./docs/reference/repository-layout.md)
  - [Build and Tooling](./docs/reference/build-and-tooling.md)

中文文档主结构：

- 开始使用：
  - [快速开始](./docs/zh/getting-started/quickstart.md)
  - [安装准备](./docs/zh/getting-started/installation.md)
  - [源码运行](./docs/zh/getting-started/source-development.md)
  - [脚本生命周期](./docs/zh/getting-started/script-lifecycle.md)
  - [编译与打包](./docs/zh/getting-started/build-and-packaging.md)
  - [发布构建](./docs/zh/getting-started/release-builds.md)
- 架构：
  - [软件架构](./docs/zh/architecture/software-architecture.md)
  - [功能模块](./docs/zh/architecture/functional-modules.md)
  - [运行模式详解](./docs/zh/architecture/runtime-modes.md)
- API 参考：
  - [总览](./docs/zh/api-reference/overview.md)
  - [网关 API](./docs/zh/api-reference/gateway-api.md)
  - [管理端 API](./docs/zh/api-reference/admin-api.md)
  - [门户 API](./docs/zh/api-reference/portal-api.md)
- 运维：
  - [配置说明](./docs/zh/operations/configuration.md)
  - [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)
- 参考：
  - [API 兼容矩阵](./docs/zh/reference/api-compatibility.md)
  - [仓库结构](./docs/zh/reference/repository-layout.md)
  - [构建与工具链](./docs/zh/reference/build-and-tooling.md)

## 前置依赖

必需：

- Rust stable + Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+
- Tauri CLI

```bash
cargo install tauri-cli
```

## 源码启动

推荐的全栈源码启动方式：

| 工作流 | Windows | Linux / macOS |
|---|---|---|
| browser mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1` | `node scripts/dev/start-workspace.mjs` |
| preview mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Preview` | `node scripts/dev/start-workspace.mjs --preview` |
| desktop mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |

常用访问地址：

- browser 模式 admin：`http://127.0.0.1:5173/admin/`
- browser 模式 portal：`http://127.0.0.1:5174/portal/`
- preview / desktop 模式统一 portal：`http://127.0.0.1:9983/portal/`
- preview / desktop 模式统一 admin：`http://127.0.0.1:9983/admin/`

说明：

- 源码辅助脚本默认使用 `127.0.0.1:9980`、`127.0.0.1:9981`、`127.0.0.1:9982` 和 `0.0.0.0:9983`
- `start-workspace --tauri` 会同时拉起 admin 桌面壳和 Pingora 统一 Web Host
- `start-workspace --preview` 会构建 admin / portal 静态资源，并通过 Pingora 统一暴露

如果要指定配置根目录：

Windows：

```powershell
$env:SDKWORK_CONFIG_DIR="$HOME\.sdkwork\router"
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Linux / macOS：

```bash
export SDKWORK_CONFIG_DIR="$HOME/.sdkwork/router"
node scripts/dev/start-workspace.mjs
```

更底层的源码辅助脚本：

- 仅后端：
  - `scripts/dev/start-servers.ps1`
  - `node scripts/dev/start-stack.mjs`
- 仅 admin：
  - `node scripts/dev/start-admin.mjs`
- 仅 portal：
  - `node scripts/dev/start-portal.mjs`
- 仅统一 Web Host：
  - `node scripts/dev/start-web.mjs`

详细源码说明：

- [源码运行](./docs/zh/getting-started/source-development.md)

## 托管 bin 脚本体系

仓库包含一套面向开发态和发布态的 `bin/` 脚本，它们位于更底层的 `scripts/dev/*` 之上。

如果你只想看一页就理解完整的 `build -> install -> start -> verify -> stop -> service registration` 生命周期，优先阅读：

- [脚本生命周期](./docs/zh/getting-started/script-lifecycle.md)

推荐用途：

- `scripts/dev/*`
  - 适合直接源码开发和原生前台调试
- `bin/start-dev.sh` / `bin/start-dev.ps1`
  - 托管开发态启动入口，使用 `artifacts/runtime/dev/` 下的本地可写运行目录
  - 默认使用 `998x` 端口避免常见 `808x` 冲突：
    - gateway：`127.0.0.1:9980`
    - admin：`127.0.0.1:9981`
    - portal：`127.0.0.1:9982`
    - shared web host：`127.0.0.1:9983`
  - 默认进入 preview 模式，内置 Pingora Web Host 成为统一浏览器入口
  - 如果明确需要 Vite 独立开发服务，可使用 `--browser` 或 `-Browser`
  - 启动成功后会输出格式化摘要，包含统一入口、独立端口、日志文件和默认账号密码
  - 内部对子进程做了监督和回收处理，`Ctrl+C`、`bin/stop-dev.*` 与异常退出时更不容易残留孤儿 `pnpm` / `cargo` 进程
  - Windows 下 `start-dev.sh` 会委托给 `start-dev.ps1`，这样 Git Bash / MSYS 路径不会直接传入 Windows 版 Node；命令名保持不变，但底层生命周期实现与 PowerShell 统一
- `bin/build.sh` / `bin/build.ps1`
  - 面向发布态的构建流水线，会构建：
    - Rust release 二进制
    - admin / portal 浏览器静态资源
    - docs 构建产物
    - admin Tauri release 包
  - 原生 release 输出到 `artifacts/release/`
  - Windows 下如果未显式设置 `CARGO_TARGET_DIR`，构建与安装流程会自动使用较短的目标目录以降低 MSVC / CMake 路径过长失败
  - Windows 下如果未显式设置 `CARGO_BUILD_JOBS`，release 构建默认使用 `1`
  - Windows 下 `build.sh` 会委托给 `build.ps1`，确保 Git Bash 与 PowerShell 走同一条已验证的构建路径
- `bin/install.sh` / `bin/install.ps1`
  - 默认将发布态运行时安装到 `artifacts/install/sdkwork-api-router/current`
  - 复制 release 二进制与 admin / portal 静态站点
  - 安装目录中只保留生产运行所需内容，`build.*`、`install.*`、`start-dev.*` 仍属于源码树工具
  - 生成：
    - `config/router.env`
    - `service/systemd/sdkwork-api-router.service`
    - `service/systemd/install-service.sh`
    - `service/systemd/uninstall-service.sh`
    - `service/launchd/com.sdkwork.api-router.plist`
    - `service/launchd/install-service.sh`
    - `service/launchd/uninstall-service.sh`
    - `service/windows-task/sdkwork-api-router.xml`
    - `service/windows-task/install-service.ps1`
    - `service/windows-task/uninstall-service.ps1`
  - Windows 下 `install.sh` 同样会委托给 `install.ps1`，路径处理与安装行为统一复用已验证的 PowerShell 版本
- `bin/start.sh` / `bin/start.ps1`
  - 发布态运行入口
  - 启动 `router-product-service`，统一承载 `/admin/*`、`/portal/*` 和 `/api/*`
  - 默认使用安装目录 `var/data/` 下的本地 SQLite
  - 启动成功后输出格式化摘要，包含统一入口、独立端口、日志文件和默认账号密码
  - 可直接作为 daemon 使用，也可以交给系统服务管理器以前台方式托管
  - Windows 下 `start.sh` 会委托给 `start.ps1`，既保留 Git Bash 用法，又避免重新引入 MSYS 路径转换导致的发布态启动问题
- `bin/stop.sh` / `bin/stop.ps1`
  - 停止托管发布态运行时
  - PowerShell 版会在运行时解析平台对应的二进制名称和停止逻辑，因此同一套 `pwsh` 入口可复用于 Windows、Linux、macOS 安装目录

典型发布流程：

Linux / macOS：

```bash
./bin/build.sh
./bin/install.sh
./bin/start.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1
```

Dry-run 示例：

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/start-dev.sh --dry-run
./bin/start.sh --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun
```

重要运行时说明：

- `bin/start.sh --foreground` 与 `bin/start.ps1 -Foreground` 是服务管理器友好的前台模式
- `bin/start-dev.*` 和 `bin/start.*` 成功启动后都会打印可点击的统一入口和独立服务 URL
- `bin/start-dev.*` 与 `bin/start.*` 默认共用 `9980` 段端口。如果这些端口已经被其他运行时占用，脚本会在真正拉起子进程前直接失败，并明确打印冲突的绑定地址，而不是等到健康检查阶段才表现成“启动后一会退出”。
- 托管启动脚本现在会在 pid 文件旁边维护一个 `.state.env` 状态文件，记录当前实例的绑定地址、前端模式和进程指纹，用来区分“健康运行中的托管实例”和“残留 pid / pid 被系统复用”的情况。
- 如果当前已经有一套健康的托管运行时使用了另一组端口，再次执行 `bin/start-dev.*` 或 `bin/start.*` 时会直接打印这套已运行实例的真实地址，而不会再落成模糊的 health-check 失败。
- Windows 下 `.sh` wrapper 属于兼容入口，实际会转交给对应的 `.ps1` 脚本执行。这样既保留 Git Bash 的命令习惯，也避免 `/d/...` 这类 MSYS 路径直接进入 Windows Node / cargo 进程。
- 托管启动脚本现在会打印 bootstrap profile 身份指引，而不是固定演示账号密码
  - 启动摘要现在会指向当前的 bootstrap profile 身份，而不会再显示固定演示账户
  - 如果你在本地使用 `dev` bootstrap profile，请先检查 `data/identities/dev.json`，并使用实际注入的身份登录
- gateway 没有默认用户名密码，它面向 portal 生成的 API key
- `bin/install.*` 会写出原生 service / daemon 描述文件，但不会在通用安装步骤里自动注册
- 安装目录故意只保留生产运行相关脚本和服务管理资产
- `config/router.env` 是发布态覆盖 bind、数据库路径、站点目录和代理目标的主要入口
- `config/router.env` 中的值会自动带引号，确保带空格路径在 shell helper 与 `systemd` 环境加载中都安全
- Windows 下 `bin/install.*` 默认会复用 `bin/build.*` 选择的短路径 `CARGO_TARGET_DIR`
- 开发态脚本刻意避免使用可能在受限环境中只读的用户主目录 SQLite 路径

从安装目录执行服务注册：

Linux / systemd：

```bash
./service/systemd/install-service.sh
./service/systemd/uninstall-service.sh
```

macOS / launchd：

```bash
./service/launchd/install-service.sh
./service/launchd/uninstall-service.sh
```

Windows / Task Scheduler：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\install-service.ps1 -StartNow
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\uninstall-service.ps1
```

## 发布构建与启动

推荐优先使用上面的 `bin/build.*`、`bin/install.*`、`bin/start.*` 和 `bin/stop.*`。下面这些命令适合你想手动拆解底层步骤时使用。

构建 release 服务二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service
```

构建 admin 浏览器资源：

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
```

构建 portal 浏览器资源：

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

构建 Tauri 桌面包：

```bash
pnpm --dir apps/sdkwork-router-admin tauri:build
```

使用默认本地配置根目录直接运行 release 二进制：

Windows：

```powershell
New-Item -ItemType Directory -Force "$HOME\.sdkwork\router" | Out-Null
.\target\release\admin-api-service.exe
.\target\release\gateway-service.exe
.\target\release\portal-api-service.exe
```

Linux / macOS：

```bash
mkdir -p "$HOME/.sdkwork/router"
./target/release/admin-api-service
./target/release/gateway-service
./target/release/portal-api-service
```

显式指定配置文件运行：

Windows：

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\gateway-service.exe
```

Linux / macOS：

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/gateway-service
```

像 `SDKWORK_DATABASE_URL` 这样的环境变量仍然会覆盖配置文件中的值。

详细发布说明：

- [发布构建](./docs/zh/getting-started/release-builds.md)

## 运行与运维

托管脚本的典型访问入口：

- 统一 admin：`http://127.0.0.1:9983/admin/`
- 统一 portal：`http://127.0.0.1:9983/portal/`
- 统一 gateway 健康检查：`http://127.0.0.1:9983/api/v1/health`
- 独立 gateway 健康检查：`http://127.0.0.1:9980/health`
- 独立 admin 健康检查：`http://127.0.0.1:9981/admin/health`
- 独立 portal 健康检查：`http://127.0.0.1:9982/portal/health`

如果直接裸跑服务二进制，在未覆盖绑定地址时，仍然会采用本 README 前面列出的 `8080`、`8081`、`8082` 内建默认值。

常用环境变量：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_EXTENSION_PATHS`

进一步的运维文档：

- [配置说明](./docs/zh/operations/configuration.md)
- [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)
- [运行模式](./docs/zh/getting-started/runtime-modes.md)
- [软件架构](./docs/zh/architecture/software-architecture.md)
- [API 参考总览](./docs/zh/api-reference/overview.md)

## 网关协议兼容性

网关支持对常见代理客户端进行协议兼容，而不引入第二套路由系统：

- Claude Code 和其他 Anthropic Messages 客户端可调用 `POST /v1/messages` 与 `POST /v1/messages/count_tokens`
- Gemini CLI gateway mode 以及其他 Google Generative Language 客户端可调用：
  - `POST /v1beta/models/{model}:generateContent`
  - `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`
  - `POST /v1beta/models/{model}:countTokens`
- 有状态网关部署在接受标准 `Authorization: Bearer ...` 的同时，也兼容：
  - Anthropic 风格的 `x-api-key`
  - Gemini 风格的 `x-goog-api-key` 或 `?key=`
- 这些兼容路由最终都会进入现有 OpenAI 风格聊天和 token 统计执行路径，因此路由策略、配额、计费、用量统计和上游转发仍然共享同一套主逻辑

## 验证

当前推荐的校验基线：

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```
