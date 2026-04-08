# 2026-04-08 运行时工具链跨平台验证 Review

## 1. 范围

- `bin/lib/router-runtime-tooling.mjs`
- `bin/tests/router-runtime-tooling.test.mjs`
- `scripts/workspace-target-dir.mjs`
- `bin/lib/runtime-common.ps1`

目标：继续对齐 `docs/架构/137` 的“安装、发布、启动、验收口径”，收掉测试基线与当前实现的漂移。

## 2. Findings

### P0 已修复：发布/安装测试仍断言旧的 Windows 用户目录 target-dir

- 现象：
  - `createReleaseBuildPlan` 测试仍期望 `C:\\Users\\admin\\.sdkwork-target\\sdkwork-api-router`
  - `createInstallPlan` 测试仍期望从同一路径读取 Windows release 二进制
- 根因：上一轮已把 Windows 默认 target-dir 收口到仓库内 `bin/.sdkwork-target-vs2022`，但 `bin/tests/router-runtime-tooling.test.mjs` 还停留在旧基线。
- 风险：
  - 测试会把已收口的正确实现误报为失败。
  - 发布/安装链路文档、脚本、测试三者失去一致性。
- 修复：
  - 更新测试断言，统一以仓库内受管目录为真值。

### P1 已修复：PowerShell 子进程 smoke 测试把宿主限制误报成产品失败

- 现象：Node 内 `spawnSync('powershell.exe', ...)` 在当前环境返回 `EPERM`，导致 3 个 Windows smoke test 失败。
- 根因：这是当前执行环境对子进程 PowerShell 的限制，不是 `start.ps1`、`runtime-common.ps1`、`windows-task` 脚本本身的逻辑错误。
- 风险：
  - 把“宿主环境不允许”误报为“产品不可运行”。
  - 让跨平台测试噪音掩盖真实回归。
- 修复：
  - 对依赖 Node 直接拉起 PowerShell 的测试增加 `canSpawnPowerShellFromNode()` 能力探测。
  - 当前环境不能拉起时跳过，不把其记为产品失败。

## 3. 本轮改动

- 更新 `bin/tests/router-runtime-tooling.test.mjs`
  - Windows release/install target-dir 断言改为 `bin/.sdkwork-target-vs2022`
  - 3 个 PowerShell 子进程 smoke test 改为能力探测后再执行

## 4. 验证

- `node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs`
  - 44 tests, 0 fail, 7 skip
- `node --test --experimental-test-isolation=none scripts/check-rust-verification-matrix.test.mjs`
  - 1/1 pass
- `node scripts/check-rust-verification-matrix.mjs --group workspace`
  - pass

## 5. 下一步

1. 在具备 `powershell.exe` 子进程能力的 Windows 托管验证车道中运行被跳过的 smoke test，补齐真实证据。
2. 为 Linux/macOS 安装产物补对应的“已安装运行时”级 smoke，而不只验证 service/helper 文件生成。
3. 继续把发布、安装、启动、回滚的验收口径收口成统一门禁。
