# 2026-04-08 全局架构对齐 Review

## 1. 范围

本轮对齐以下基线文档：

- `docs/架构/130-API-Router-行业对标与终局能力矩阵-2026-04-07.md`
- `docs/架构/134-高性能与容量规划设计-2026-04-07.md`
- `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
- `docs/架构/136-安全与合规评估标准-2026-04-07.md`
- `docs/架构/137-安装部署发布回滚设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`
- `docs/架构/142-多Cell多Region与灾备演进设计-2026-04-07.md`

结论口径遵循 `docs/架构/README.md`：只把可由代码、脚本、测试直接验证的事实写成“现状”，目标态单列，不混写。

## 2. 已验证事实

- `node scripts/check-rust-verification-matrix.mjs --group workspace`：本轮修复后通过，Windows Rust workspace 校验链路恢复。
- `node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs`：通过。
- `node --test --experimental-test-isolation=none scripts/dev/tests/process-supervision.test.mjs`：6/6 通过。
- `node --test --experimental-test-isolation=none scripts/dev/tests/start-workspace.test.mjs`：10/10 通过。
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`：5/5 通过。
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`：6/6 通过。
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`：通过。
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`：16/16 通过。
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`：17/17 通过。

## 3. Findings

### P0 已修复：Windows Rust 验证链路默认写入 `USERPROFILE`，在受限环境下直接阻塞工作区构建

- 现象：`scripts/check-rust-verification-matrix.mjs --group workspace` 在修复前失败于 `failed to create directory C:\\Users\\admin\\.sdkwork-target\\sdkwork-api-router\\debug`，错误为 `os error 5`。
- 根因：`scripts/workspace-target-dir.mjs` 的 Windows 默认策略与 `bin/lib/runtime-common.ps1` 不一致。前者优先写用户目录，后者优先写仓库内受管目录。
- 架构冲突：
  - 违背 `docs/架构/137` 的“工作区模式与发布模式边界清晰、可重复、可自动化”要求。
  - 违背 `docs/架构/141` 的“工作区边界 / 发布边界一致性校验”要求。
- 修复：统一为“显式 `CARGO_TARGET_DIR` 可覆盖，否则 Windows 默认落到仓库内 `bin/.sdkwork-target-vs2022`”。
- 结果：矩阵脚本与 `start-dev.ps1` 对齐，workspace cargo check 已恢复。

### P1 未闭环：平台级数据策略仍未形成可验真治理闭环

- 依据：`docs/架构/130`、`docs/架构/136` 明确写出“当前事实仍以 Provider 级数据策略为主，平台级策略是目标态”。
- 风险：
  - 企业客户无法把“是否落盘、保留期、脱敏级别、缓存策略、审计要求”收敛为统一平台策略。
  - 兼容协议层容易出现“不同 Provider / 不同协议口径不一致”的合规裂缝。
- 当前状态：本轮未发现直接的构建失败，但这是企业级 go-live 阻塞项，不是文档性建议。

### P1 未闭环：SLO / 可观测性仍停留在能力资产具备，尚未达到发布门槛级治理

- 依据：`docs/架构/135` 把健康检查、请求关联、路由证据、运行时观测写成“当前已具备资产”，同时把 burn-rate、统一 SLO 口径、Dashboard 标准写成治理要求。
- 风险：
  - 当前可以“看见日志和指标”，但未证明“可依 SLO 进行发布准入、回滚、值班处置”。
  - 在高并发和多租户热点下，问题发现与责任定位速度可能不足。
- 当前状态：脚本和测试面是绿的，但未看到可在 CI/发布门禁中强制执行的 SLO 验收证据。

### P2 未闭环：多 Cell / 多 Region / 灾备仍是演进目标，不应被表述为现状能力

- 依据：`docs/架构/142` 把该部分明确写成“演进设计”，不是当前事实。
- 风险：
  - 若对外口径误写为“已支持多 Region 高可用”，将与实际交付能力不符。
  - RTO / RPO、切换、drain、数据放置原则尚未形成端到端验证。
- 当前状态：本轮未执行跨 Region/跨 Cell 演练；因此不能把“具备多运行面”误判为“已完成多 Region 灾备”。

### P2 未闭环：跨操作系统“完美可安装运行”仍需真实安装烟测补证，不应只凭契约测试宣称完成

- 已有正向证据：
  - 发布工作流覆盖 `Windows / Linux / macOS` 和 `x64 / arm64`。
  - 开发、发布、Tauri 入口均有契约测试。
- 仍缺证据：
  - 当前工作区没有一轮“安装产物级”的 Linux/macOS 真实启动验收记录。
  - 本轮完成的是脚本契约和 Windows 工作区构建恢复，不等于三平台发行包全部实机验收完成。

## 4. 本轮已落地修复

- 修复文件：`scripts/workspace-target-dir.mjs`
- 回归测试：`scripts/check-rust-verification-matrix.test.mjs`
- 效果：
  - Windows Rust 验证链路不再依赖 `USERPROFILE\\.sdkwork-target\\...`
  - 与 `bin/lib/runtime-common.ps1` 的目标目录策略一致
  - `workspace cargo check` 通过

## 5. 实施计划

### 第一阶段：发布与启动链路收口

1. 把所有 Windows Cargo/Tauri/启动脚本的 target-dir 策略统一成“仓库内受管目录优先，显式 env 覆盖”。
2. 增加 Linux/macOS 安装产物级 smoke test，不只测脚本 plan。
3. 把“工作区模式”和“Release 模式”依赖边界写入自动门禁，阻断本地 sibling 依赖泄漏到 Release。

### 第二阶段：生产治理闭环

1. 补平台级数据策略模型：保留、缓存、落盘、脱敏、审计、租户覆盖。
2. 把路由、运行时、商业链路指标纳入统一 SLO 门禁。
3. 给控制面配置变更补 version / diff / reviewer / rollback 证据链。

### 第三阶段：高可用与灾备演进

1. 先定义单 Cell 稳态验收，再扩展到多 Cell。
2. 把 Region、数据放置、复制、故障切换、drain、RTO/RPO 形成真实演练脚本。
3. 只有演练和回滚证据稳定后，才把多 Region 能力写入对外交付口径。

## 6. 下一步

- 下一轮优先做“跨平台安装产物 smoke 验证”。
- 然后做“平台级数据策略 + SLO 门禁”的落地设计与实现。
- 多 Cell / 多 Region 继续保持为目标态，不提前宣称已生产化。
