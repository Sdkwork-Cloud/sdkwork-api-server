# 门户 API

门户服务在 `/portal/*` 下暴露终端用户自助边界。

## 基础地址与鉴权

- 默认本地基地址：`http://127.0.0.1:8082/portal`
- 健康检查：`GET /portal/health`
- 鉴权边界：portal JWT，与 admin JWT 完全独立

最小注册示例：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"PortalPass123!",
    "display_name":"Portal User"
  }'
```

默认本地演示账号登录：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

密码修改：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/change-password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <portal-jwt>" \
  -d '{
    "current_password":"ChangeMe123!",
    "new_password":"PortalPassword456!"
  }'
```

## 路由家族

| 家族 | 路由 | 用途 |
|---|---|---|
| health | `GET /portal/health` | 存活性 |
| auth | `POST /portal/auth/register`、`POST /portal/auth/login`、`GET /portal/auth/me`、`POST /portal/auth/change-password` | 终端用户注册、会话生命周期与密码轮换 |
| workspace | `GET /portal/workspace` | 查看调用者拥有的默认工作区 |
| dashboard | `GET /portal/dashboard` | 返回当前项目的工作区身份、用量与计费组合快照 |
| usage | `GET /portal/usage/records`、`GET /portal/usage/summary` | 返回最近请求、token-unit 历史与聚合请求统计 |
| billing | `GET /portal/billing/summary`、`GET /portal/billing/ledger` | 返回 quota 态势、已用或剩余额度以及账本视图 |
| API keys | `GET /portal/api-keys`、`POST /portal/api-keys` | 自助查询和创建 gateway API key |

## 典型用户路径

1. 注册 portal 账户
2. 登录并获取 portal JWT
3. 查看默认 tenant 和 project 工作区
4. 打开 dashboard 快照查看最近请求、token units 与 quota 态势
5. 查看 usage 与 billing 明细视图
6. 创建 gateway API key
7. 使用该 key 调用 gateway 的 `/v1/*` 接口

## 浏览器应用

门户浏览器体验是独立应用：

- `http://127.0.0.1:5174/`

## 相关文档

- 产品使用流程：
  - [公开门户](/zh/getting-started/public-portal)
- operator 控制平面：
  - [管理端 API](/zh/api-reference/admin-api)
