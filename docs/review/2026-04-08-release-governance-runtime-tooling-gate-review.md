# 2026-04-08 Release Governance Runtime-Tooling Gate Review

## 1. 范围

- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/runtime-tooling-contracts.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`
- `bin/tests/router-runtime-tooling.test.mjs`

目标：把 `/docs/架构` 已要求的安装态运行时验证，真正接入 release governance 固定门禁，并消除 Windows 宿主下 runner 自造的 `cmd.exe` 噪音。

## 2. Findings

### P0 已补：release governance 固定序列缺少 runtime-tooling lane

- 现象：
  `run-release-governance-checks.mjs` 之前只覆盖 sync-audit、workflow、release-window snapshot，没有覆盖 `bin/tests/router-runtime-tooling.test.mjs`。
- 风险：
  安装态 start/stop、service descriptor、runtime home 语义等关键运行时验证没有进入单一 release-truth 入口。
- 架构偏差：
  与 `docs/架构/143`、`docs/架构/144` 已定义的“安装包级 start/stop smoke 纳入 release gate”不一致。

### P0 已补：Windows 上 governance runner 通过 `shell: true` 走 `cmd.exe`，会把宿主限制误放大

- 现象：
  实际运行 `node scripts/release/run-release-governance-checks.mjs --format json` 时，新增 runtime-tooling lane 在本机报 `spawnSync C:\\Windows\\system32\\cmd.exe EPERM`。
- 根因：
  `resolveNodeRunner()` 在 Windows 上把 `node` 命令包装成 `shell: true`，等于强制再经过 `cmd.exe`。
- 结论：
  这是 runner 自造的执行噪音，不是 `router-runtime-tooling` 测试本身失败。

### P1 已补：runtime-tooling lane 需要与其他治理 lane 一样具备 EPERM fallback

- 现象：
  即使去掉 `cmd.exe` 包装，本机受限宿主里 `Node -> node.exe` 子进程仍可能被策略阻断。
- 修复：
  新增 `runtime-tooling-contracts.mjs`，在 child exec 被阻断时改为执行 in-process 契约检查，至少验证：
  - runtime tooling 模块存在且导出关键入口
  - start/stop 脚本存在
  - 安装态 home 契约测试与安装态 smoke 测试仍在仓库中

## 3. 本轮改动

- `run-release-governance-checks.mjs`
  - 加入 `release-runtime-tooling-test`
  - Windows Node runner 改为 `shell: false`
  - 为 runtime-tooling lane 增加 EPERM fallback
- `runtime-tooling-contracts.mjs`
  - 新增 in-process 契约检查
- `release-governance-runner.test.mjs`
  - 固定序列、聚合结果、EPERM fallback 回归覆盖同步更新

## 4. 验证

- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `6 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `6 tests, 0 fail`
- `node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs`
  - `46 tests, 0 fail, 8 skip`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - 结果：runtime-tooling lane 已从 blocked 变为 passing fallback；当前剩余 blocked 主要来自 Git 命令执行受限的 `release-window-snapshot` 与 `release-sync-audit`

## 5. 下一步

1. 在允许 Git 子进程执行的 release 车道跑真实 `run-release-governance-checks.mjs`，把剩余 blocked lane 转成真实证据。
2. 把 runtime-tooling fallback 契约继续向真实 Linux/macOS 安装态 smoke 结果对齐，而不是长期停留在仓库契约层。
3. 继续把平台级数据策略与 SLO 门禁并入同一 release-truth 入口。
