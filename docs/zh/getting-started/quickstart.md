# 快速开始

本页仅用于本地开发。

如果你需要线上部署、`system` 安装、PostgreSQL 默认配置或服务化启动，请改看 [生产部署](/zh/getting-started/production-deployment)。

## 前置依赖

- Rust + Cargo
- Node.js 20+
- pnpm 10+

## 启动托管开发环境

Linux / macOS：

```bash
./bin/start-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

默认开发入口：

- admin：`http://127.0.0.1:9983/admin/`
- portal：`http://127.0.0.1:9983/portal/`
- gateway 健康检查：`http://127.0.0.1:9983/api/v1/health`

## 健康检查

```bash
curl http://127.0.0.1:9980/health
curl http://127.0.0.1:9981/admin/health
curl http://127.0.0.1:9982/portal/health
```

每个端点都应返回 `ok`。

## 停止托管开发环境

Linux / macOS：

```bash
./bin/stop-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

## 下一步

- [源码运行](/zh/getting-started/source-development)
- [脚本生命周期](/zh/getting-started/script-lifecycle)
- [生产部署](/zh/getting-started/production-deployment)
