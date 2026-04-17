# 安装布局

本页定义 `portable` 与 `system` 两种安装模式下的生产布局标准。

## 安装模式

### Portable

适用于：

- 本地验证
- CI smoke 测试
- 单目录便携安装

默认根目录：

- `artifacts/install/sdkwork-api-router/current/`

### System

适用于：

- 生产服务器
- 长期运行的私有化部署
- 服务管理器托管启动

## 逻辑根目录

两种模式都遵循同一套逻辑根：

- program home
- config home
- data home
- log home
- run home
- service definition home

## 各操作系统的 System 默认布局

### Linux

- program home：`/opt/sdkwork-api-router/current/`
- config home：`/etc/sdkwork-api-router/`
- config file：`/etc/sdkwork-api-router/router.yaml`
- config fragments：`/etc/sdkwork-api-router/conf.d/`
- env file：`/etc/sdkwork-api-router/router.env`
- data home：`/var/lib/sdkwork-api-router/`
- log home：`/var/log/sdkwork-api-router/`
- run home：`/run/sdkwork-api-router/`

### macOS

- program home：`/usr/local/lib/sdkwork-api-router/current/`
- config home：`/Library/Application Support/sdkwork-api-router/`
- config file：`/Library/Application Support/sdkwork-api-router/router.yaml`
- config fragments：`/Library/Application Support/sdkwork-api-router/conf.d/`
- env file：`/Library/Application Support/sdkwork-api-router/router.env`
- data home：`/Library/Application Support/sdkwork-api-router/data/`
- log home：`/Library/Logs/sdkwork-api-router/`
- run home：`/Library/Application Support/sdkwork-api-router/run/`

### Windows

- program home：`C:\Program Files\sdkwork-api-router\current\`
- config home：`C:\ProgramData\sdkwork-api-router\`
- config file：`C:\ProgramData\sdkwork-api-router\router.yaml`
- config fragments：`C:\ProgramData\sdkwork-api-router\conf.d\`
- env file：`C:\ProgramData\sdkwork-api-router\router.env`
- data home：`C:\ProgramData\sdkwork-api-router\data\`
- log home：`C:\ProgramData\sdkwork-api-router\log\`
- run home：`C:\ProgramData\sdkwork-api-router\run\`

## 配置发现顺序

主配置文件发现顺序：

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

随后按字典序加载 `conf.d/*.yaml`。

## 配置优先级

从低到高的实际优先级：

- built-in defaults
- environment fallback
- config file
- CLI

发现配置文件的例外变量：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

这两个变量会最先读取，用于定位配置文件。

## 数据库默认策略

- `portable`
  - 允许使用 SQLite 进行本地验证
- `system`
  - 默认数据库契约是 PostgreSQL

在 `system` 模式下，除非显式开启开发覆盖，否则会拒绝 SQLite。
