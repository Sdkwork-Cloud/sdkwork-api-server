# 配置说明

本页汇总最关键的环境变量与运行时配置选择。

## 绑定地址

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`

## 存储

- `SDKWORK_DATABASE_URL`

支持：

- SQLite
- PostgreSQL

示例：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
```

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## 认证密钥

- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`

## Secret 存储

- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`

支持的 secret backend：

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## 扩展运行时

- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`

## 建议

本地开发：

- 默认 SQLite
- 默认 loopback 绑定
- 本地加密文件或数据库加密 secret

共享部署：

- 使用 PostgreSQL
- 显式设置 JWT signing secret
- 明确配置 trusted signers 和 extension paths
