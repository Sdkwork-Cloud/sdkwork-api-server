# 运行模式

SDKWork API Server 支持多种实用运行形态。关键区别不仅在于服务端还是桌面端，还在于你是要源码级直接控制，还是要托管脚本生命周期。

## 原生独立服务模式

这是最低层的运行形态。

特点：

- 服务以独立二进制运行
- gateway、admin、portal API 都直接通过 HTTP 暴露
- 如果不覆盖，二进制仍保持内建默认值
- 适合需要对单个进程做最细粒度控制的场景

典型入口：

- `cargo run -p gateway-service`
- `cargo run -p admin-api-service`
- `cargo run -p portal-api-service`

典型默认绑定：

- gateway：`127.0.0.1:8080`
- admin：`127.0.0.1:8081`
- portal：`127.0.0.1:8082`

## 源码 browser 工作区模式

这是原生源码开发工作流。

特点：

- 后端服务使用更新后的辅助脚本默认值 `9980`、`9981`、`9982`
- admin 和 portal 使用独立 Vite 开发服务
- 最适合前端联调和热更新

入口：

- `node scripts/dev/start-workspace.mjs`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1`

主要浏览器地址：

- admin：`http://127.0.0.1:5173/admin/`
- portal：`http://127.0.0.1:5174/portal/`

## 源码 preview 工作区模式

这是原生源码下的单端口工作流。

特点：

- 后端服务仍位于 `9980`、`9981`、`9982`
- Pingora 通过一个统一的浏览器可见 Host 对外暴露 admin 和 portal
- 适合需要更接近发布态形态的浏览器验证

入口：

- `node scripts/dev/start-workspace.mjs --preview`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Preview`

主要浏览器地址：

- 统一 admin：`http://127.0.0.1:9983/admin/`
- 统一 portal：`http://127.0.0.1:9983/portal/`

## 源码 Tauri 工作区模式

这是原生源码下的桌面优先工作流。

特点：

- 后端服务仍位于 `9980`、`9981`、`9982`
- admin 运行在 Tauri 桌面壳中
- Pingora 仍通过统一 Web Host 提供浏览器访问

入口：

- `node scripts/dev/start-workspace.mjs --tauri`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri`

## 托管开发模式

这是推荐的脚本化开发生命周期。

特点：

- 运行时状态由 `artifacts/runtime/dev/` 托管
- 启停由 PID 驱动
- 默认模式是 preview，因此一启动就能使用统一单端口浏览器入口
- 启动日志会打印统一 URL、独立服务 URL、账号密码和日志路径

入口：

- `./bin/start-dev.sh`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1`

适合：

- 你想要一个稳定命令用于 QA、演示或重复验证
- 你默认希望只有一个浏览器入口
- 你希望配套使用 `bin/stop-dev.*`

## 托管发布模式

这是面向生产的脚本生命周期。

特点：

- build、install、start、stop 和 service registration 分阶段执行
- 运行时安装到独立安装目录
- `router-product-service` 统一承载 `/admin/*`、`/portal/*` 和 `/api/*`
- 适合以前台模式运行，并交给 service manager 或受管脚本托管

入口：

- `./bin/build.sh`
- `./bin/install.sh`
- `./bin/start.sh`
- `./bin/stop.sh`

Windows 对应：

- `.\bin\build.ps1`
- `.\bin\install.ps1`
- `.\bin\start.ps1`
- `.\bin\stop.ps1`

## 如何选择合适的模式

在这些场景下选原生独立服务模式：

- 你需要直接控制单个二进制
- 你正在隔离调试某一个服务

在这些场景下选源码 browser 工作区模式：

- 你需要基于 Vite 的前端联调
- 你需要独立的浏览器开发服务

在这些场景下选源码 preview 模式：

- 你希望在源码工作流中使用单个浏览器可见端口
- 你希望浏览器行为更接近发布态

在这些场景下选托管开发模式：

- 你想要最容易复现的本地环境
- 你需要 PID、日志和运行目录管理
- 你需要启动摘要、URL 和默认账号输出

在这些场景下选托管发布模式：

- 你在打包或部署服务端运行时
- 你需要 systemd、launchd 或 Windows Task Scheduler 集成

## 下一步

- 查看启停职责：
  - [脚本生命周期](/zh/getting-started/script-lifecycle)
- 查看本地启动：
  - [源码运行](/zh/getting-started/source-development)
- 查看编译和打包：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 深入理解架构和监督机制：
  - [运行模式详解](/zh/architecture/runtime-modes)
