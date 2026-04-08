# Release Governance Telemetry Export Producer Plan

## Goal

把当前 `snapshot -> SLO evidence` 的双段链，继续收口为 `export bundle -> snapshot -> SLO evidence`，让 release workflow 不再直接接收 snapshot JSON，而是接收更接近原始观测出口的治理输入。

## Root Cause

- 当前仓库已具备 snapshot contract，但 upstream 仍是裸 `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON`。
- 这意味着 release truth 仍缺少 producer 边界，无法区分“原始导出”与“治理后的 snapshot”。
- 同时，SLO baseline 中部分 target 无法直接从现有 Prometheus 文本导出，需要显式 supplemental summary，而不是继续伪装成“全量原始 metrics 可直接推导”。

## Design

### Option A

- 只增加 source audit，不做 producer。
- 诚实，但 release producer 缺口仍未收口。

### Option B

- 推荐：新增 `release telemetry export` 契约。
- snapshot materializer 优先消费 export bundle，再派生 governed snapshot。
- export bundle 同时支持：
  - `prometheus`：gateway/admin/portal 原始指标文本
  - `supplemental.targets`：现阶段无法直接从原始指标稳定推导的 target 摘要

### Option C

- 缩减 SLO baseline 去适配当前指标。
- 不接受；会削弱既有架构基线。

## Scope

- `scripts/release/materialize-release-telemetry-snapshot.mjs`
- `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/release-workflow.test.mjs`
- `.github/workflows/release.yml`
- 本轮对应 `docs/review` / `docs/架构` / `docs/release` / `docs/step`

## TDD

1. 先写 export bundle red tests。
2. 再让 workflow contract red 掉旧的 `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` 入口。
3. 最后实现 export parser / derivation，并跑针对性回归。

## Hold

- 不下调 baseline。
- 不声称所有 target 都已具备直接 Prometheus 原始源。
- 不提交任何 synthetic latest artifact 作为 repo truth。
