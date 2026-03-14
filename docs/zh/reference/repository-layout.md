# 仓库结构

## 顶层结构

```text
.
|-- crates/
|-- services/
|-- console/
|-- docs/
|-- scripts/
|-- Cargo.toml
|-- README.md
`-- README.zh-CN.md
```

## 后端分层

- `crates/sdkwork-api-interface-*`
  - HTTP 与接口边界
- `crates/sdkwork-api-app-*`
  - 应用服务层
- `crates/sdkwork-api-domain-*`
  - 领域模型与策略
- `crates/sdkwork-api-storage-*`
  - 仓储与持久化实现

## 独立服务

- `services/gateway-service`
- `services/admin-api-service`
- `services/portal-api-service`

## 前端分层

- `console/src/`
  - 壳层组合
- `console/packages/`
  - 可复用业务模块
- `console/src-tauri/`
  - 桌面原生宿主边界

## 文档与运维资产

- `docs/`
  - VitePress 文档站与深度技术文档
- `docs/plans/`
  - 历史设计与实施记录
- `scripts/dev/`
  - 跨平台启动脚本
