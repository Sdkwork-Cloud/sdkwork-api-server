# 生产部署

本页是 SDKWork API Router 的唯一生产部署入口。

当你要发布线上版本、执行原生服务器安装、使用 Docker Compose 或 Helm 部署时，请从这里开始。

## 生产契约

- 原生生产安装标准是 `system` 模式。
- `system` 模式默认数据库契约是 PostgreSQL。
- 配置文件是业务配置的主来源。
- 环境变量主要用于发现配置文件和补全缺省值。
- 服务托管应交给 `systemd`、`launchd` 或 Windows Service Control Manager。

## 选择部署路径

### Docker Compose

适合单机快速上线，并默认带 PostgreSQL。

主要资产：

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`

### Helm

适合 Kubernetes 集群部署，默认假设 PostgreSQL 由外部托管。

主要资产：

- `deploy/helm/sdkwork-api-router/`

### 原生 System 安装

适合需要操作系统标准目录和服务化启动的服务器环境。

## 构建发布产物

Linux / macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

跨平台发布约束：

- 仅在 Windows 入口脚本和 CI 中设置 Windows 专属的 `CMAKE_GENERATOR`、`HOST_CMAKE_GENERATOR`
- 不要把 Visual Studio 的 CMake 生成器默认值写进全局 Cargo 配置或 Unix shell profile
- 如果在 Docker 中执行 Unix installed-runtime smoke，`cargo build` 与 `run-unix-installed-runtime-smoke.mjs` 两步必须使用同一个 `CARGO_TARGET_DIR`
- 即使 `ss`、`netstat` 和 `lsof` 都不可用，运行时仍能正确启动；如果你希望在启动前获得更丰富的端口冲突诊断，建议额外安装其中任意一个工具

## 本地 Release Governance 准备

如果你是在开发机上执行 release governance，而本地 sibling 仓库并不是干净的独立 release checkout，请先把发布工具指向一个受管 external dependency root。这样 `materialize-external-deps`、`verify-release-sync` 和 `run-release-governance-checks` 就会基于受管克隆执行，而不是误读当前机器上的脏 worktree。

Linux / macOS：

```bash
export SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT="$PWD/artifacts/external-release-deps"
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

Windows：

```powershell
$env:SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT = (Join-Path (Get-Location) 'artifacts\external-release-deps')
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

当 sibling 审计出现 `not-standalone-root`、`dirty-working-tree`、`branch-not-synced` 或 `head-mismatch` 这类原因时，优先使用这条路径。

## 生成原生生产安装

Linux / macOS：

```bash
./bin/install.sh --mode system
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

生成内容包括：

- 标准 `router.yaml`
- `conf.d/` 覆盖目录
- `router.env`
- `router.env.example`
- `systemd`、`launchd` 与 Windows Service 描述文件

## 初始化生产配置

首次启动前请审阅并修改：

- `router.yaml`
  - 运行时主配置
- `conf.d/*.yaml`
  - 可选的分域覆盖配置
- `router.env`
  - 配置发现信息与少量兜底变量

建议优先修改：

- PostgreSQL 连接串
- JWT、凭据主密钥、Metrics Token 等密钥
- 对外与对内绑定地址
- admin / portal 静态目录

## 服务注册前先校验

在构建或发布工具环境中执行：

```bash
./bin/validate-config.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\validate-config.ps1
```

校验内容包括：

- 配置发现与合并顺序
- 生产安全姿态
- `system` 模式下默认拒绝 SQLite，除非显式开启开发覆盖

## 注册并启动服务

通过前台模式交给系统服务管理器托管：

- Linux：`./service/systemd/install-service.sh`
- macOS：`./service/launchd/install-service.sh`
- Windows：`powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-service\install-service.ps1`

配套说明：

- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)

## Docker Compose 快速部署

```bash
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

## Helm 快速部署

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/your-org/sdkwork-api-router \
  --set image.tag=2026.04.15 \
  --set secrets.databaseUrl='postgresql://sdkwork:change-me@postgresql:5432/sdkwork_api_router'
```

## 初始化检查清单

- 目标平台的发布包已生成
- PostgreSQL 已创建并可连通
- `router.yaml` 已审阅
- `router.env` 中的密钥已替换
- `validate-config` 已成功通过
- 已通过操作系统原生服务管理器注册
- 首次启动后已验证健康检查端点
