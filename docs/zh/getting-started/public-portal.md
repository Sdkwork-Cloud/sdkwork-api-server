# Public Portal

公共 portal 是面向外部用户的自助入口，与 operator 使用的 admin 控制平面完全分离。

## Portal API

- `POST /portal/auth/register`
- `POST /portal/auth/login`
- `GET /portal/auth/me`
- `GET /portal/workspace`
- `GET /portal/api-keys`
- `POST /portal/api-keys`

## 浏览器路由

- `#/portal/register`
- `#/portal/login`
- `#/portal/dashboard`

## 默认使用流程

1. 打开 `http://127.0.0.1:5173/#/portal/register`
2. 注册一个 portal 用户
3. 进入 dashboard
4. 查看默认 workspace
5. 创建 gateway API key
6. 立即复制返回的明文 key
7. 用该 key 调用网关

示例：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

## 当前范围

当前已支持：

- 注册
- 登录
- 工作区查看
- API key 自助签发

当前未纳入本批次：

- 邀请
- 多工作区成员关系
- 密码重置邮件
- OAuth / SSO
