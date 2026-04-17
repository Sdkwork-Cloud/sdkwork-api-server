# 发布构建

本页只说明构建与打包生成，不负责解释线上部署。

如果你需要原生生产安装、PostgreSQL 初始化、服务注册或上线步骤，请改看 [生产部署](/zh/getting-started/production-deployment)。

## 托管构建流水线

Linux / macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

构建产物包括：

- Rust release 二进制
- admin 与 portal 静态资源
- docs 构建产物
- 可选桌面包
- `artifacts/release/` 下的原生发布目录

## 托管安装包生成

便携式安装：

```bash
./bin/install.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

生产标准安装：

```bash
./bin/install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

## Dry Run

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/install.sh --mode system --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -DryRun
```

## 验证

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
```

## 下一步

- [生产部署](/zh/getting-started/production-deployment)
- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)
