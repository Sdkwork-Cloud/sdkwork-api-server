# sdkwork-api-router

[English Guide](./README.md)

SDKWork API Router 是一个基于 Rust 的 OpenAI 兼容网关、管理控制平面、公共门户与产品运行时仓库。仓库同时提供源码开发工作流，以及面向线上发布的 build/install/service 管理工具链。

## 正式产品

仓库对外发布的正式用户产品只有两个：

- `sdkwork-api-router-product-server`
  - 标准 server 产品，用于原生安装、Docker、Docker Compose 和 Helm 部署
- `sdkwork-router-portal-desktop`
  - portal-first 的 desktop 外壳，内置本地 product runtime

其余内容都属于源码开发界面、中间构建产物或 release governance 证据，不是正式发布产品。

## 生产部署入口

准备线上发布时，优先阅读：

- [Production Deployment](./docs/getting-started/production-deployment.md)
- [Install Layout](./docs/operations/install-layout.md)
- [Service Management](./docs/operations/service-management.md)
- [Docker And Helm Assets](./deploy/README.md)

仅用于本地开发时，优先阅读：

- [快速开始](./docs/zh/getting-started/quickstart.md)
- [源码运行](./docs/zh/getting-started/source-development.md)

## 运行面

- `gateway-service`
  - OpenAI 兼容 `/v1/*` 网关
- `admin-api-service`
  - 面向运维的 `/admin/*` 控制平面
- `portal-api-service`
  - 面向开发者的 `/portal/*` 自助 API
- `router-web-service`
  - 基于 Pingora 的公共 Web Host
- `router-product-service`
  - 一体化产品运行时，统一承载 `/admin/*`、`/portal/*`、`/api/*`

## 配置契约

主配置发现顺序：

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

普通字段的生效优先级从低到高如下：

- 内建默认值 -> 环境变量兜底 -> 配置文件 -> CLI

运行说明：

- `SDKWORK_CONFIG_DIR` 与 `SDKWORK_CONFIG_FILE` 只用于发现配置文件。
- `conf.d/*.yaml` 会在主配置之后按字典序叠加。
- 系统安装默认使用 PostgreSQL。
- SQLite 继续支持本地开发与 `portable` 验证。

示例 `router.yaml`：

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-router.db"
secret_backend: "local_encrypted_file"
secret_local_file: "secrets.json"
extension_paths:
  - "extensions"
enable_connector_extensions: true
enable_native_dynamic_extensions: false
```

更多细节参见：

- [配置说明](./docs/zh/operations/configuration.md)
- [安装布局](./docs/zh/operations/install-layout.md)

## 发布模式

发布安装工具链支持两种模式：

- `portable`
  - 单目录本地验证和 CI 友好安装
- `system`
  - 面向生产的标准操作系统目录布局，分离 program/config/data/log/run 目录

`system` 模式是默认推荐的生产标准。

## 推荐生产流程

构建发布产物：

```bash
./bin/build.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

生成生产级安装目录：

```bash
./bin/install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

在注册系统服务前校验生成后的配置：

```bash
./current/bin/validate-config.sh --home ./current
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home .\current
```

继续阅读：

- [Production Deployment](./docs/getting-started/production-deployment.md)
- [Service Management](./docs/operations/service-management.md)

## 本地开发

推荐使用托管开发入口：

```bash
./bin/start-dev.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

本地开发文档：

- [快速开始](./docs/zh/getting-started/quickstart.md)
- [源码运行](./docs/zh/getting-started/source-development.md)
- [脚本生命周期](./docs/zh/getting-started/script-lifecycle.md)

## 发布校验

推荐校验基线：

```bash
node --test scripts/check-router-docs-safety.test.mjs
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
cargo test -p sdkwork-api-config --test config_loading
cargo test -p router-product-service
pnpm --dir docs build
```
