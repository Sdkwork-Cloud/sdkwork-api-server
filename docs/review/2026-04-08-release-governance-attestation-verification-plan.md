# Release Governance Attestation Verification Plan

## Goal

把 `docs/架构/154` 中尚未闭合的 operator verification 缺口收口为仓库内脚本与测试契约，让发布证据和发布产物的 attestation 可以被显式验证，而不是只存在于 workflow 声明里。

## Root Cause

- 当前仓库已经能在 release workflow 中生成 build-provenance attestation。
- 但仓库内还没有统一入口去验证这些 attestation。
- `docs/架构/154` 已明确指出剩余缺口是 `gh attestation verify` 的 operator walkthrough，这意味着代码、脚本、文档三者仍未完全对齐。

## Design

### Option A

- 只补文档，不补脚本。
- 不接受；这会让 operator verification 继续停留在手工知识层。

### Option B

- 推荐：新增 `scripts/release/verify-release-attestations.mjs`。
- 脚本负责：
  - 枚举 governed evidence、Unix smoke evidence、packaged release assets 的本地 subject
  - 生成并执行 `gh attestation verify` 命令计划
  - 将 `subject-path-missing`、`gh-cli-missing`、`command-exec-blocked` 与真实 verify fail 区分开
- 发布治理 runner 只先接入测试车道，不提前引入新的 live blocked lane。

### Option C

- 直接把 live attestation verify 加进 governance gate。
- 当前宿主没有真实 hosted attestation 证据，也存在 child exec 受限；现在接入只会新增长期 blocked lane。

## Scope

- `scripts/release/verify-release-attestations.mjs`
- `scripts/release/release-attestation-verification-contracts.mjs`
- `scripts/release/tests/release-attestation-verify.test.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`
- 本轮对应 `docs/review` / `docs/架构` / `docs/release` / `docs/step`

## TDD

1. 先写 attestation verification red tests。
2. 再让 governance runner red 掉缺少该测试车道的旧实现。
3. 最后补脚本、fallback contract、文档和回归验证。

## Hold

- 不声称当前本地会话已经拿到真实 GitHub attestation 记录。
- 不把 attestation verification 伪装成 live release gate。
- 不声称 `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` 的 live producer 已闭合。
