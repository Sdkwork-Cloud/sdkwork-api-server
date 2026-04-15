# 管理端 API

管理端服务在 `/admin/*` 下暴露 operator 控制平面。

## 基础地址与认证

- 默认本地基础地址：`http://127.0.0.1:8081/admin`
- 登录流程：
  - `POST /admin/auth/login`
  - `GET /admin/auth/me`
  - `POST /admin/auth/change-password`

请使用当前 bootstrap profile 或运行时存储中已存在的管理端身份。本地使用 `dev` bootstrap 数据时，登录前先检查 `data/identities/dev.json`。

登录示例：

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"<admin-email>",
    "password":"<admin-password>"
  }'
```

返回的 JWT 使用方式：

```bash
-H "Authorization: Bearer <jwt>"
```

最小校验请求：

```bash
curl http://127.0.0.1:8081/admin/auth/me \
  -H "Authorization: Bearer <jwt>"
```

## OpenAPI 清单

- OpenAPI JSON：`GET /admin/openapi.json`
- API 文档页：`GET /admin/docs`

OpenAPI 文档由当前 `axum` 路由实时生成，能够直接反映管理端服务的实际暴露面。

## Canonical 存储规范

路由控制面中的目录、凭据与访问密钥数据，现已统一收口到 `ai_*` 表，并要求字段名全部使用小写下划线命名。

关键表如下：

- `ai_channel`：channel 主目录表，默认内置 `openai`、`anthropic`、`gemini`、`openrouter`、`ollama`
- `ai_proxy_provider`：代理提供方主表
- `ai_proxy_provider_channel`：代理提供方与 channel 的绑定关系表
- `ai_model`：channel 与 model 的映射表
- `ai_model_price`：channel + model + proxy provider 的价格表
- `ai_router_credential_records`：Router Config 密钥配置表
- `ai_app_api_keys`：应用访问 API Key 表

Unified API Key 统一保存在 `ai_app_api_keys` 中，核心字段包括：

- `hashed_key`：运行时鉴权使用的查找键
- `raw_key`：原始 API Key 明文内容
- `tenant_id`
- `project_id`
- `environment`
- `label`
- `notes`
- `created_at_ms`
- `last_used_at_ms`
- `expires_at_ms`
- `active`

Router Config 中的凭据统一保存在 `ai_router_credential_records` 中，核心字段包括：

- `tenant_id`
- `proxy_provider_id`
- `key_reference`
- `secret_backend`
- `secret_local_file`
- `secret_keyring_service`
- `secret_master_key_id`
- `secret_ciphertext`
- `secret_key_version`
- `created_at_ms`
- `updated_at_ms`

其中 `secret_ciphertext` 保存加密后的 router 配置密文。管理端接口不会回传提交时的明文密钥。

全新数据库现在只会创建 `ai_*` 物理表。旧表名如 `identity_gateway_api_keys`、`credential_records`、`catalog_channels`、`identity_users` 会在启动迁移时自动搬迁到新的 canonical `ai_*` 表中，并以兼容视图的方式继续保留旧名字，确保历史数据与旧 SQL 工具链都不受影响。

## 路由分组

| 分组 | 路由 | 作用 |
|---|---|---|
| health 与 metrics | `GET /admin/health`、`GET /metrics` | 存活性与 Prometheus 风格指标 |
| auth | `POST /auth/login`、`GET /auth/me`、`POST /auth/change-password` | operator 登录、会话确认与密码轮换 |
| tenancy | `GET/POST /tenants`、`DELETE /tenants/{tenant_id}`、`GET/POST /projects`、`DELETE /projects/{project_id}` | tenant 与 project 生命周期管理 |
| gateway access | `GET/POST /api-keys`、`POST /api-keys/{hashed_key}/status`、`DELETE /api-keys/{hashed_key}` | 应用访问 API Key 的签发、状态管理与删除 |
| provider catalog | `GET/POST /channels`、`DELETE /channels/{channel_id}`、`GET/POST /providers`、`DELETE /providers/{provider_id}`、`GET/POST /credentials`、`DELETE /credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}` | channel、proxy provider、router credential 管理 |
| channel models | `GET/POST /channel-models`、`DELETE /channel-models/{channel_id}/models/{model_id}` | channel 下 model 目录管理 |
| model pricing | `GET/POST /model-prices`、`DELETE /model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}` | channel model 与 proxy provider 的价格管理 |
| 兼容模型路由 | `GET/POST /models`、`DELETE /models/{external_name}/providers/{provider_id}` | 旧的 provider 视角 model 路由，底层已适配到 canonical 表 |
| extensions | `GET/POST /extensions/installations`、`GET /extensions/packages`、`GET/POST /extensions/instances`、`GET /extensions/runtime-statuses`、`POST /extensions/runtime-reloads` | 扩展运行时管理 |
| extension rollouts | `GET/POST /extensions/runtime-rollouts`、`GET /extensions/runtime-rollouts/{rollout_id}` | 扩展 rollout 协调控制 |
| runtime config rollouts | `GET/POST /runtime-config/rollouts`、`GET /runtime-config/rollouts/{rollout_id}` | 配置重载 rollout 协调控制 |
| usage 与 billing | `GET /usage/records`、`GET /usage/summary`、`GET /billing/ledger`、`GET /billing/summary`、`GET/POST /billing/quota-policies` | 运维观测、账单与配额控制 |
| routing | `GET/POST /routing/policies`、`GET /routing/health-snapshots`、`GET /routing/decision-logs`、`POST /routing/simulations` | 路由策略与诊断能力 |

## 管理端 API 负责什么

管理端 API 是以下数据的系统事实来源：

- channel、provider、credential、model、pricing 目录
- 应用 API Key 与 router credential 状态
- routing policy
- runtime rollout 状态
- usage 与 billing 汇总
- quota 控制

如果你需要改变网关底层行为，这就是负责下发变更的控制面 API。

## 浏览器应用

operator UI 是独立浏览器应用：

- `http://127.0.0.1:5173/admin/`

Catalog 管理模块现已支持：

- channel 表格化 CRUD
- channel 新增或编辑时动态维护支持的 model 列表
- 每个 channel 的 `Manage models` 模型管理弹窗
- 每个 model 的 `Manage pricing` 定价管理弹窗
- proxy provider CRUD
- router credential CRUD

## 相关文档

- 服务边界：
  - [API 参考总览](/zh/api-reference/overview)
- 终端用户自助边界：
  - [门户 API](/zh/api-reference/portal-api)
- 架构上下文：
  - [软件架构](/zh/architecture/software-architecture)
