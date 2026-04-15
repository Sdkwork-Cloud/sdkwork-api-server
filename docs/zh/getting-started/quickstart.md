# 快速开始

本页是从克隆仓库到验证本地 SDKWork 栈成功运行的最短路径。

它采用一条直接的引导路径：

1. 启动运行时
2. 验证控制平面
3. 登录一个管理端身份
4. 登录一个门户身份
5. 创建 gateway API key
6. 发起首个已鉴权 gateway 请求

## 开始前

请先完成：

- [安装准备](/zh/getting-started/installation)

你需要：

- Rust + Cargo
- Node.js 20+
- pnpm 10+

## 第 1 步：启动完整栈

推荐的 quickstart 路径是托管开发脚本，因为它会：

- 默认避开常见 `808x` 端口冲突
- 自动拉起内置统一 Web Host
- 在启动完成后打印可点击的访问链接和 bootstrap 身份引导

Linux / macOS：

```bash
./bin/start-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

托管开发态默认访问地址：

- 统一 admin：`http://127.0.0.1:9983/admin/`
- 统一 portal：`http://127.0.0.1:9983/portal/`
- 统一 gateway 健康检查：`http://127.0.0.1:9983/api/v1/health`
- 独立 gateway 健康检查：`http://127.0.0.1:9980/health`
- 独立 admin 健康检查：`http://127.0.0.1:9981/admin/health`
- 独立 portal 健康检查：`http://127.0.0.1:9982/portal/health`

开发身份 bootstrap 指引：

- 开发身份来自当前激活的 bootstrap profile
- 在共享本地环境前先检查 `data/identities/dev.json`
- 默认的 `prod` bootstrap profile 不会注入开发身份

如果你明确想要使用独立 Vite 前端而不是统一 Web Host，可使用：

- `./bin/start-dev.sh --browser`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser`

## 第 2 步：验证运行时健康状态

```bash
curl http://127.0.0.1:9980/health
curl http://127.0.0.1:9981/admin/health
curl http://127.0.0.1:9982/portal/health
```

预期结果：每个端点都返回 `ok`。

## 第 3 步：登录管理控制平面

请使用当前 bootstrap profile 或运行时配置中已经写入的管理端身份。

如果你在本地使用 `dev` bootstrap profile：

- 先检查 `data/identities/dev.json`
- 将下面示例中的 `<admin-email>` 与 `<admin-password>` 替换成实际注入的身份

```bash
curl -X POST http://127.0.0.1:9981/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"<admin-email>",
    "password":"<admin-password>"
  }'
```

然后查看当前 admin 身份：

```bash
export ADMIN_JWT="<粘贴 token>"
curl http://127.0.0.1:9981/admin/auth/me \
  -H "Authorization: Bearer $ADMIN_JWT"
```

浏览器中的 admin UI 位于：

- `http://127.0.0.1:9983/admin/`

## 第 4 步：登录门户

请使用当前 bootstrap profile 已写入的门户身份，或者先通过 `/portal/auth/register` 注册一个账户。

如果你在本地使用 `dev` bootstrap profile：

- 先检查 `data/identities/dev.json`
- 将下面示例中的 `<portal-email>` 与 `<portal-password>` 替换成实际注入的身份

```bash
curl -X POST http://127.0.0.1:9982/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"<portal-email>",
    "password":"<portal-password>"
  }'
```

保存返回的 token：

```bash
export PORTAL_JWT="<粘贴 token>"
```

浏览器中的 portal UI 位于：

- `http://127.0.0.1:9983/portal/`

## 第 5 步：查看门户工作区

```bash
curl http://127.0.0.1:9982/portal/workspace \
  -H "Authorization: Bearer $PORTAL_JWT"
```

这一步用于确认默认工作区引导已经完成。

## 第 6 步：创建 gateway API key

```bash
curl -X POST http://127.0.0.1:9982/portal/api-keys \
  -H "Authorization: Bearer $PORTAL_JWT" \
  -H "Content-Type: application/json" \
  -d '{"environment":"live"}'
```

响应会一次性返回 `plaintext` key，请立即保存：

```bash
export GATEWAY_KEY="<粘贴明文 key>"
```

## 第 7 步：发起首个 gateway 请求

使用该 key 调用 OpenAI 兼容 gateway：

```bash
curl http://127.0.0.1:9980/v1/models \
  -H "Authorization: Bearer $GATEWAY_KEY"
```

预期结果：

- `200 OK`
- 返回标准 OpenAI 风格列表响应
- 在你通过 admin API 配置 model catalog 和 provider 之前，`data` 可能为空

## 第 8 步：停止托管开发运行时

Linux / macOS：

```bash
./bin/stop-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

## 下一步

- 查看完整脚本职责和生命周期：
  - [脚本生命周期](/zh/getting-started/script-lifecycle)
- 查看源码级启动方式：
  - [源码运行](/zh/getting-started/source-development)
- 理解三套 HTTP 接口面：
  - [API 参考总览](/zh/api-reference/overview)
- 配置 models、providers、credentials 和 routing：
  - [管理端 API](/zh/api-reference/admin-api)
- 理解运行时结构：
  - [软件架构](/zh/architecture/software-architecture)
- 编译二进制和前端产物：
  - [编译与打包](/zh/getting-started/build-and-packaging)
