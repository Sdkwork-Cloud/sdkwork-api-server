# 公开门户

公开门户为终端用户提供自助体验，并且与仅供 operator 使用的 admin 控制平面严格隔离。

它是默认的终端用户边界，承担账户创建、dashboard 与用量查看、计费态势感知以及网关 API key 签发。

## 门户接口

- `POST /portal/auth/register`
- `POST /portal/auth/login`
- `GET /portal/auth/me`
- `POST /portal/auth/change-password`
- `GET /portal/workspace`
- `GET /portal/dashboard`
- `GET /portal/usage/records`
- `GET /portal/usage/summary`
- `GET /portal/billing/summary`
- `GET /portal/billing/ledger`
- `GET /portal/api-keys`
- `POST /portal/api-keys`

## 浏览器应用

- `http://127.0.0.1:5174/`

## 默认使用流程

1. 打开 `http://127.0.0.1:5174/`
2. 注册一个 portal 用户，或在启用开发 bootstrap profile 时使用已写入的本地身份登录
3. 进入 dashboard
4. 查看工作区身份、最近请求、token-unit 用量和计费态势
5. 在 portal 内查看优惠券兑换、充值和订阅入口
6. 创建 gateway API key
7. 立即复制返回的明文 key
8. 用该 key 调用网关

当 `SDKWORK_BOOTSTRAP_PROFILE=dev` 时，开发身份来自 `data/identities/dev.json`。

- 在共享本地环境前先检查该文件
- 需要新的本地用户时，可通过 `/portal/auth/register` 自助注册
- 默认的 `prod` bootstrap profile 不会注入开发身份

示例：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

## 安全边界

门户认证与 admin 认证是分离的：

- 不同的路由命名空间
- 不同的 JWT 边界
- 门户用户只能看到自己默认 tenant 和 project 范围内的数据

## 当前范围

当前已支持：

- 注册
- 登录
- 工作区查看
- dashboard 快照与最近请求
- 用量工作台与逐次调用 token-unit 可见性
- 计费汇总与台账读取
- 基于前端 commerce seam 的优惠券、充值和订阅入口
- API key 自助签发

当前未纳入本批次：

- 邀请
- 多工作区成员关系
- 密码重置邮件
- OAuth / SSO
- 实时支付结算

## 相关文档

- 本地启动：
  - [源码运行](/zh/getting-started/source-development)
- 服务边界：
  - [门户 API 参考](/zh/api-reference/portal-api)
- 管理端边界：
  - [管理端 API 参考](/zh/api-reference/admin-api)
