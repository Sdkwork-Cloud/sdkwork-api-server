# SLO Live Evidence Release Gate

> 日期：2026-04-08
> 目标：把量化 SLO 从“仅测试基线”收口为“发布工作流可执行门禁”，同时保持证据真实、可审计、可跨平台消费。

## 1. 设计

- 输入
  - 文件：`--evidence` 或 `SDKWORK_SLO_GOVERNANCE_EVIDENCE_PATH`
  - JSON：`--evidence-json` 或 `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON`
- 校验
  - 只校验证据形状与基线完整性
  - 不在物化阶段判断是否达标
- 输出
  - 默认产物：`docs/release/slo-governance-latest.json`
  - 写入 `baselineId`、`baselineDate`、`generatedAt`、`targets`

## 2. 发布链路

1. `Materialize external release dependencies`
2. `Materialize SLO governance evidence`
3. `Run release governance gate`
4. 安装依赖与构建

该顺序保证：

- 先收齐发布依赖
- 再生成受治理的 SLO 证据
- 再让 `release-slo-governance` 和其他 live lane 一起判定

## 3. 门禁语义

- `release-slo-governance-test`
  - 证明基线、校验器、合同完整
- `release-slo-governance`
  - 证明 live artifact 是否存在且可被评估
- `evidence-missing`
  - 视为 `blocked`
  - 不视为 `failing`
  - 语义是“发布证据未就绪”，不是“阈值不达标”

## 4. 跨平台要求

- 证据文件允许 UTF-8 BOM
- Windows PowerShell 生成的 BOM JSON 必须可被消费
- child-process 受限宿主下，runner 仍需通过 in-process fallback 执行 live SLO 评估

## 5. 边界

- 当前已闭环：基线、物化、workflow 接线、live lane、fallback、Windows BOM 兼容
- 当前未闭环：真实遥测/外部观测系统到 `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON` 的自动供给
- 因此当前状态是“门已装好且会拦截”，不是“已具备真实放行证据”
