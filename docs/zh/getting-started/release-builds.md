# 发布构建

本页说明如何生成并运行可部署的服务二进制、浏览器静态资源和可选桌面包。

如果你要查看每个脚本在完整生命周期中的职责矩阵，请先阅读 [脚本生命周期](/zh/getting-started/script-lifecycle)。本页聚焦发布产物和部署流程。

## 推荐的托管发布流程

推荐发布生命周期如下：

1. `bin/build.*`
2. `bin/install.*`
3. 检查 `config/router.env`
4. `bin/start.*`
5. 验证启动摘要中打印的统一入口和独立入口
6. `bin/stop.*` 或交给 service manager 托管

### Build

Linux / macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

这一步会构建：

- Rust release 二进制
- admin 和 portal 静态资源
- docs 构建产物
- 可选桌面包
- `artifacts/release/` 下的原生 release 包

### Install

Linux / macOS：

```bash
./bin/install.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

默认安装目录：

- `artifacts/install/sdkwork-api-router/current/`

安装目录中的关键路径：

- `bin/`
- `config/router.env`
- `sites/admin/dist`
- `sites/portal/dist`
- `var/log/`
- `var/run/`
- `service/systemd/`
- `service/launchd/`
- `service/windows-task/`

### Configure

检查或覆盖：

- `config/router.env`

这是发布态推荐的配置入口，用来调整：

- release bind 地址
- 数据库位置
- 静态站点目录
- 代理目标

### Start

Linux / macOS：

```bash
./bin/start.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1
```

托管发布态运行时会启动 `router-product-service`，统一承载：

- `/admin/*`
- `/portal/*`
- `/api/*`

默认绑定地址：

- gateway：`127.0.0.1:8080`
- admin：`127.0.0.1:8081`
- portal：`127.0.0.1:8082`
- unified web host：`0.0.0.0:3001`

启动成功后脚本会打印：

- 统一 admin URL
- 统一 portal URL
- 统一 gateway 健康检查 URL
- 独立服务 URL
- seeded admin / portal 本地账号
- 日志文件位置

### Stop

Linux / macOS：

```bash
./bin/stop.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop.ps1
```

## 前台模式与服务管理器

当你要交给 systemd、launchd、Windows Task Scheduler 或任何其他 service manager 托管时，请使用前台模式：

- `bin/start.sh --foreground`
- `bin/start.ps1 -Foreground`

安装步骤已经把服务注册资产放到安装目录中：

- `service/systemd/`
- `service/launchd/`
- `service/windows-task/`

从安装目录中注册或注销：

- Linux / systemd：
  - `./service/systemd/install-service.sh`
  - `./service/systemd/uninstall-service.sh`
- macOS / launchd：
  - `./service/launchd/install-service.sh`
  - `./service/launchd/uninstall-service.sh`
- Windows / Task Scheduler：
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\install-service.ps1 -StartNow`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\uninstall-service.ps1`

## 发布生命周期中的脚本职责

| 脚本 | 生命周期角色 | 重要说明 |
|---|---|---|
| `bin/build.*` | 生成可发布二进制和静态资源 | 不负责安装或启动 |
| `bin/install.*` | 准备可运行的安装目录 | 不负责启动运行时 |
| `bin/start.*` | 启动安装后的发布运行时 | 假定安装目录已经存在 |
| `bin/stop.*` | 停止安装后的发布运行时 | 使用安装目录中的 PID 文件 |

## 更底层的发布命令

当你明确想绕开托管发布生命周期、直接手动控制时，可使用下面的原生命令。

### 构建 Rust 服务

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
cargo build --release -p router-web-service -p router-product-service
```

输出位于 `target/release/`。

### 输出路径

Windows 可执行文件：

- `target/release/admin-api-service.exe`
- `target/release/gateway-service.exe`
- `target/release/portal-api-service.exe`
- `target/release/router-web-service.exe`
- `target/release/router-product-service.exe`

Linux / macOS 可执行文件：

- `target/release/admin-api-service`
- `target/release/gateway-service`
- `target/release/portal-api-service`
- `target/release/router-web-service`
- `target/release/router-product-service`

### 构建 admin 浏览器资源

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
```

输出：

- `apps/sdkwork-router-admin/dist/`

### 构建 portal 浏览器资源

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

输出：

- `apps/sdkwork-router-portal/dist/`

### 构建 Tauri 桌面包

```bash
pnpm --dir apps/sdkwork-router-admin tauri:build
```

## 发布部署说明

推荐部署形态：

- 当你想统一承载 `/admin/*`、`/portal/*` 和 `/api/*` 时，优先使用 `router-product-service`
- 多用户持久化部署优先使用 PostgreSQL
- 密钥优先交由服务端 secret backend 管理
- 将 `config/router.env` 纳入环境级变更管理
- 优先复用托管安装目录，而不是再发明另一套运行目录结构

## Docker 与 Kubernetes 部署资产

Linux 的 product-server 发布包现在会一并携带 `deploy/`：

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`
- `deploy/helm/sdkwork-api-router/`

这些资产继续复用现有 `router-product-service` 运行时模型，而不是额外发明第二套部署方式：

- 公网入口绑定：`0.0.0.0:3001`
- 内部 gateway/admin/portal 绑定：`127.0.0.1:8080/8081/8082`
- bootstrap 数据目录：`/opt/sdkwork/data`
- admin 静态资源目录：`/opt/sdkwork/sites/admin/dist`
- portal 静态资源目录：`/opt/sdkwork/sites/portal/dist`
- 生产数据库：`SDKWORK_DATABASE_URL=postgresql://...`

从解压后的 Linux 发布包快速启动 Docker：

```bash
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

将同一镜像推送后，可直接用 Helm 部署到 Kubernetes：

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/your-org/sdkwork-api-router \
  --set image.tag=2026.04.15 \
  --set secrets.databaseUrl='postgresql://sdkwork:change-me@postgresql:5432/sdkwork_api_router' \
  --set secrets.adminJwtSigningSecret='change-me-admin' \
  --set secrets.portalJwtSigningSecret='change-me-portal' \
  --set secrets.credentialMasterKey='change-me-master-key' \
  --set secrets.metricsBearerToken='change-me-metrics-token'
```

## Dry-run 示例

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/start.sh --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun
```

## 发布校验

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```

## 下一步

- 查看脚本职责和生命周期：
  - [脚本生命周期](/zh/getting-started/script-lifecycle)
- 查看源码启动流程：
  - [源码运行](/zh/getting-started/source-development)
- 查看构建流水线细节：
  - [编译与打包](/zh/getting-started/build-and-packaging)
