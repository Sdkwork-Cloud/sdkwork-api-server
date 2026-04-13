# API 参考总览

这组文档围绕三个公开 HTTP 接口面组织：

- OpenAI 兼容网关
- 原生管理控制平面
- 原生公开门户

## 基础路径与职责边界

| 接口面 | 默认本地基地址 | 主要鉴权模型 | 适用场景 |
|---|---|---|---|
| gateway | `http://127.0.0.1:8080/v1` | gateway API key | 应用流量与模型执行 |
| admin | `http://127.0.0.1:8081/admin` | admin JWT | operator、控制平面、计费、路由、运行时控制 |
| portal | `http://127.0.0.1:8082/portal` | portal JWT | 终端用户、自助认证、工作区与 API key |

## 鉴权边界

| 接口面 | Header | 令牌签发方 |
|---|---|---|
| gateway | `Authorization: Bearer skw_live_...` | gateway API key 存储 |
| admin | `Authorization: Bearer <jwt>` | admin 登录流程 |
| portal | `Authorization: Bearer <jwt>` | portal 登录流程 |

admin 与 portal 令牌刻意隔离。portal session 不是 admin session。

## API 约定

- gateway 路由在已文档化的范围内以 OpenAI 兼容为目标
- admin 与 portal 路由是 SDKWork 自有的原生 API
- 独立服务会在主 API 命名空间之外暴露 `/metrics` 与健康检查端点
- 请求追踪会透传 `x-request-id`
- gateway 路由可选接收 `x-sdkwork-region`，用于 geo-affinity 路由提示

## 兼容真值标签

SDKWork 使用五种标签描述真实执行方式：

| 标签 | 含义 |
|---|---|
| `native` | 由 SDKWork 直接实现 |
| `relay` | 转发给兼容上游 |
| `translated` | 本地接收后映射为不同的上游原语 |
| `emulated` | 本地返回兼容形状 |
| `unsupported` | 当前运行时不可用 |

详情见 [API 兼容矩阵](/zh/reference/api-compatibility) 和 [完整兼容矩阵](/api/compatibility-matrix)。

## 首个已鉴权请求

admin 登录：

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

portal 默认登录：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

portal 注册：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"PortalPass123!",
    "display_name":"Portal User"
  }'
```

gateway 首个请求：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

如果你要跑完整引导路径，请看 [快速开始](/zh/getting-started/quickstart)。

## 浏览器应用

浏览器端明确拆分为两套独立应用：

- admin 应用：`http://127.0.0.1:5173/admin/`
- portal 应用：`http://127.0.0.1:5174/`

## 各服务参考页

- [网关 API](/zh/api-reference/gateway-api)
- [管理端 API](/zh/api-reference/admin-api)
- [门户 API](/zh/api-reference/portal-api)

## 与 OpenAI 参考的一致性

在数据平面上，SDKWork 有意贴近 OpenAI 官方文档的组织方式。对于上游 schema 与资源语义，OpenAI 官方文档依然是主要参考：

- OpenAI API reference: <https://platform.openai.com/docs/api-reference>
- OpenAI docs overview: <https://platform.openai.com/docs/overview>
