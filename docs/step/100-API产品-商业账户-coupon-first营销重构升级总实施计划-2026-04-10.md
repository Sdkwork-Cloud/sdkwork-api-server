# 100-API产品-商业账户-coupon-first营销重构升级总实施计划-2026-04-10

## 1. 文档定位

- 本文档是当前 `sdkwork-api-router` 商业化重构主线的推荐入口。
- 范围只覆盖本轮专项：`API Product`、`Commercial Account`、`Coupon-First Marketing`、`Admin`、`Portal`、`Public API`、`storage/migration`、`release gate`。
- 若本文与 [docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md](../架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md) 冲突，以 `166` 为准；若与旧商业化专项 step 冲突，以本文为准。
- 旧文档 [2026-04-07-全应用商业化审查与分阶段step计划.md](./2026-04-07-全应用商业化审查与分阶段step计划.md) 与 [2026-04-07-交易闭环差距审查与整改step计划.md](./2026-04-07-交易闭环差距审查与整改step计划.md) 继续保留为历史审计输入，不再作为本轮主顺序依据。

## 2. 当前真相与本轮目标

### 2.1 当前真相

- 平台已经具备 `pricing`、`commercial account`、`marketing canonical model`、`portal commerce`、`admin commercial` 的基础能力，不是从零开始。
- 当前最大问题不是“没有商业化模块”，而是“多套语义并存且治理深度不一致”。
- 已确认的高风险点包括：
  - legacy `coupon` 与 canonical `marketing` 双模型并存。
  - `PortalCommerceQuoteKind` 混合产品语义与交易语义。
  - marketing 控制面仍弱于 pricing lifecycle。
  - Portal 对 coupon / benefit lots / invite 仍有静态或兼容残留。
  - Admin 主工作流仍暴露 legacy coupon compatibility。
  - storage/runtime 仍允许 legacy coupon 主链路残留。

### 2.2 本轮目标

- 把商业化主线收敛到 `API Product + Commercial Account + Coupon-First Marketing`。
- 让 `coupon` 明确并入 `market`，但保持一等业务语义，不退化成抽象 promotion code。
- 用可执行的 step 体系把架构文档落到代码、接口、管理系统、门户、存储和发布门禁。

## 3. 规划原则

- 先收语义，再动实现；先收主模型，再动界面；先收主链路，再删兼容层。
- 禁止继续向已退场的 legacy coupon 兼容层叠加新业务语义；coupon 语义只能落在 canonical marketing。
- 优先保留现有 crate 布局，在现有 `catalog / billing / commerce / marketing` 基础上完成语义收敛，不先做大规模物理拆包。
- 每个 step 必须同时包含：设计、实施落地规划、测试计划、结果验证、检查点、风险与回退、完成定义。
- 每个 step 必须写清：
  - 完成后应兑现的 `166` 能力
  - 执行后必须回写的 `docs/架构/*` 与 `docs/review/*`
- 每个 step 都必须显式区分：
  - 当前事实
  - 目标状态
  - 本轮延后
- 每个 step 都必须区分：
  - step 级串行依赖
  - step 内可多子 agent 并行的车道

## 4. 编号与完成规则

- 本轮 step 编号统一为 `S00-S08`。
- 检查点统一为 `CPCxx-y`，例如 `CPC03-2`。
- 任何 step 完成前都必须给出：
  - 主证据
  - 辅证据
  - 回退条件
  - 下一步准入结论
- 若任一项缺失，该 step 只允许标记为“未闭环/待收口”。

## 5. 串行主脊柱与并行窗口

### 5.1 串行主脊柱

`S00 -> S01 -> (S02 || S03) -> S04 -> (S05 || S06) -> S07 -> S08`

### 5.2 并行窗口

- `Window A`：`S01` 完成后，`S02` 与 `S03` 可并行。
- `Window B`：`S04` 完成后，`S05` 与 `S06` 可并行。
- `Window C`：`S07` 内允许 storage、migration、dependency cleanup、data bootstrap 多车道并行，但 cutover 结论必须串行收口。
- `S08` 是总验收 step，只允许验证车道与文档回写车道并行，不允许再并行发散新实现。

## 6. Step 总览

| Step | 核心目标 | 主要写入范围 | 前置 | 并行属性 | 最小输出 |
| --- | --- | --- | --- | --- | --- |
| `S00` | 冻结术语、基线、阻塞与退场规则 | `docs/架构/`、`docs/step/`、`docs/review/` | 无 | step 级串行，盘点车道可并行 | 单一术语表、阻塞清单、Owner/依赖矩阵 |
| `S01` | 收敛 API Product / Offer / Pricing / Quote 语义 | `domain-catalog`、`domain-billing/pricing`、`app-catalog`、`app-commerce`、`portal types` | `S00` | step 级串行，接口/UI 后置并行 | 产品与交易类型拆分、兼容映射 |
| `S02` | 收敛 Commercial Account / Benefit / Hold / Settlement | `domain-billing/accounts`、`app-billing`、`interface-admin/portal`、Portal Account/Billing | `S01` | 可与 `S03` 并行 | 商业账户主模型、权益来源与追踪闭环 |
| `S03` | 收敛 Coupon-First Marketing 单一事实源 | `domain-marketing`、`app-marketing`、`app-commerce`、storage、admin/portal marketing | `S01` | 可与 `S02` 并行 | canonical coupon 模型、legacy 冻结与迁移框架 |
| `S04` | 打通交易集成与 coupon 结算闭环 | `app-commerce`、`app-billing`、`interface-admin/portal` | `S02`、`S03` | step 级串行，验证车道可并行 | `checkout_discount` / `account_entitlement` 主链路闭环 |
| `S05` | 升级 Admin 控制面到专业治理版本 | `interface-admin`、Admin API/Types/UI | `S04` | 可与 `S06` 并行 | clone/revise/approve/schedule/publish/retire 工作流 |
| `S06` | 升级 Portal / Public API / SDK 自助语义 | `interface-portal`、Portal API/Types/UI | `S04` | 可与 `S05` 并行 | 产品购买 + 券权益 + 账户到账一体化 |
| `S07` | 完成 storage migration、双写灰度与 legacy 退场 | `storage-*`、`app-runtime/bootstrap`、`Cargo.toml`、`data/` | `S05`、`S06` | 车道可并行，cutover 串行 | 单一事实源、兼容层退场与回退策略 |
| `S08` | 完成集成验收、发布门禁、架构回写与后续迭代入口 | `docs/架构/`、`docs/review/`、`docs/release/`、测试矩阵 | `S07` | 验证/文档并行 | 总验收、发布证据、下一波 backlog |

## 7. 波次规划

### 7.1 波次 A：定义主模型

- `S00`
- `S01`
- `S02`
- `S03`

目标：

- 冻结主语义。
- 建立 `product / account / marketing` 三层 canonical truth。

### 7.2 波次 B：打通业务主链路

- `S04`
- `S05`
- `S06`

目标：

- 让后台、门户、公共接口基于同一套商业化主模型运行。

### 7.3 波次 C：退场与验收

- `S07`
- `S08`

目标：

- 完成 storage/runtime cutover、legacy 退场、发布门禁与架构回写。

## 8. 能力兑现矩阵

| Step | 执行后应兑现的 `166` 能力 | 收口时必须回写 |
| --- | --- | --- |
| `S00` | `Market`、三层分责、legacy freeze、术语统一成为唯一口径 | `166`、`03`、`docs/review/*` |
| `S01` | `API Product / Offer / Pricing / Quote / Transaction` 分层，`target_kind` 去混用 | `166` 第 7 章、`docs/review/*` |
| `S02` | `CommercialAccount / BenefitLot / Hold / Settlement` 成为 entitlement 真值 | `166` 第 5.2B、5.3、13、`docs/review/*` |
| `S03` | canonical coupon 模型、publication/issued coupon、lifecycle 与单一事实源 | `166` 第 5.2C、6、9、10、11、`133`、`docs/review/*` |
| `S04` | `checkout_discount` / `account_entitlement`、validate-reserve-confirm-rollback 事务闭环 | `166` 第 5.3、6.4、7.3、`docs/review/*` |
| `S05` | Admin `Market` 工作台、approval/schedule/publish/retire 与 legacy 主流程退场 | `166` 第 8.1、9、`133`、`docs/review/*` |
| `S06` | Portal / Public API 的产品购买、券权益、账户到账一体化体验 | `166` 第 4.6、8.2、8.3、13、`133`、`docs/review/*` |
| `S07` | storage/runtime 单真值、dual-write cutover、legacy runtime 退场 | `166` 第 10、11、13、`03`、`docs/review/*` |
| `S08` | 统一验收、发布门禁、架构回写、下一波 backlog 冻结 | `166` 第 12、13、14、`docs/release/*`、`docs/review/*` |

## 9. 多子 Agent 执行规则

- 每个 step 必须有唯一 `Owner` 负责收口检查点与合流。
- 多子 agent 只允许在互斥写集上并行。
- 共享文件只允许一个车道主写：
  - `crates/sdkwork-api-domain-marketing/src/lib.rs`
  - `crates/sdkwork-api-domain-billing/src/accounts.rs`
  - `crates/sdkwork-api-domain-billing/src/pricing.rs`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
  - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
  - `crates/sdkwork-api-interface-admin/src/openapi.rs`
  - `crates/sdkwork-api-interface-portal/src/openapi.rs`
- 任何车道若发现 shared contract 漂移，必须回到 step owner 串行裁决。

### 9.1 最快完整执行建议

| 波次 | 推荐拓扑 | 最快并行方式 | 串行收口点 |
| --- | --- | --- | --- |
| `Wave 0` | `Owner + Verifier` | 只做 `S00-S01`，不分散实现 | `S01` 术语、shared types、OpenAPI |
| `Wave A` | `S02 Owner`、`S03 Owner`、`Verifier/Doc` | `S02 || S03` 双主线并行，验证与回写单独车道 | `S04` 前统一 account/marketing contract |
| `Wave B` | `S04 Owner + 2 Workers + Verifier` | 交易编排、接口、测试并行，但 orchestration 主文件单写 | `S04` 交易闭环证据 |
| `Wave C` | `S05 Owner`、`S06 Owner`、`Shared Contract Verifier` | `S05 || S06` 并行，OpenAPI/TS types 由单独 owner 串行收口 | `S07` 前统一 admin/portal/public 契约 |
| `Wave D` | `Storage`、`Runtime`、`Bootstrap`、`Cleanup`、`Verifier` | `S07` 四车道并行推进 diff/backfill/cleanup | read switch 与 cutover 结论 |
| `Wave E` | `Backend Regression`、`Frontend/Contract`、`Release`、`Docs`、`Owner` | `S08` 验证并行，go/no-go 串行 | 最终 release gate |

### 9.2 并行执行硬规则

1. `S00-S01` 不并行执行实现，只允许只读盘点并行。
2. `S02/S03`、`S05/S06` 是唯一允许 step 级并行的主窗口。
3. `openapi.rs`、共享 TS types、主领域模型文件、`README` 只允许单 owner 主写。
4. 每个波次只保留一个集成窗口；未集成前不启动下一波次。
5. 每个波次必须有独立 `Verifier/Doc` 车道，避免实现与验收绑死在同一 owner。
6. `S07` 的 cutover、`S08` 的 go/no-go 只能串行裁决。

### 9.3 推荐班组与节奏

| 模式 | 角色配置 | 适用时机 |
| --- | --- | --- |
| `5-Agent 稳态版` | `Owner`、`Worker-A`、`Worker-B`、`Verifier`、`Doc` | 默认推荐；最稳妥 |
| `7-Agent 加速版` | `Owner`、`Worker-A/B/C/D`、`Verifier`、`Doc` | `S01` 完成后；共享 contract 已冻结 |

1. 每波次第一天先冻结 shared contract、写集和回退条件，再开并行。
2. 每天最多两个集成窗口：中段一次、小结一次；其余时间只允许车道内提交。
3. `S02/S03` 与 `S05/S06` 必须共用一个 contract referee，专门裁决 `openapi.rs` 与共享 TS types。
4. `S07` 进入 read switch 前，必须先由 `Verifier` 出具 diff 收敛报告。
5. `S08` 期间停止新增实现，只允许修复阻塞验收的问题。

### 9.4 每步最小收口包

1. 一份 step owner 结论。
2. 一份 `docs/review/*` 证据。
3. 一次架构回写结论：更新 `166/133/03` 或明确“不需回写”。
4. 一次下一步准入结论：`go / conditional-go / no-go`。

## 10. 统一证据与回写规则

- 每个 step 至少输出一份 `docs/review/*` 证据文档。
- 每个 step 收口时必须判断是否需要回写：
  - [docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md](../架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md)
  - [docs/架构/133-控制平面与运营后台设计-2026-04-07.md](../架构/133-控制平面与运营后台设计-2026-04-07.md)
  - [docs/架构/03-模块规划与边界.md](../架构/03-模块规划与边界.md)
- 任何实现若改变 `Current State / Target State / Deferred`，必须同步回写 `docs/架构/`。

## 11. Step 索引

- [101-Step执行卡与并行交付模板-商业化专项-2026-04-10](./101-Step执行卡与并行交付模板-商业化专项-2026-04-10.md)
- [102-S00-语义冻结-基线审计与阻塞治理-2026-04-10](./102-S00-语义冻结-基线审计与阻塞治理-2026-04-10.md)
- [103-S01-API产品与定价主模型收敛-2026-04-10](./103-S01-API产品与定价主模型收敛-2026-04-10.md)
- [104-S02-商业账户与权益账本收敛-2026-04-10](./104-S02-商业账户与权益账本收敛-2026-04-10.md)
- [105-S03-Coupon-First营销主模型收敛-2026-04-10](./105-S03-Coupon-First营销主模型收敛-2026-04-10.md)
- [106-S04-交易集成与coupon结算闭环-2026-04-10](./106-S04-交易集成与coupon结算闭环-2026-04-10.md)
- [107-S05-Admin控制面专业化升级-2026-04-10](./107-S05-Admin控制面专业化升级-2026-04-10.md)
- [108-S06-Portal与Public-API产品化收口-2026-04-10](./108-S06-Portal与Public-API产品化收口-2026-04-10.md)
- [109-S07-存储迁移-双写灰度与legacy退场-2026-04-10](./109-S07-存储迁移-双写灰度与legacy退场-2026-04-10.md)
- [110-S08-集成验收-发布门禁与持续迭代-2026-04-10](./110-S08-集成验收-发布门禁与持续迭代-2026-04-10.md)

## 12. 完成定义

只有同时满足以下条件，才允许宣称本轮 step 体系可执行：

1. `S00-S08` 每一步都有独立 step 卡。
2. 每张 step 卡都写清设计、实施、测试、验证、检查点、风险回退。
3. 每一步都明确了串行依赖与多子 agent 并行车道。
4. 所有步骤都能映射回 `166` 的能力，不存在“没人负责”的能力空洞。
5. `README` 已把本轮主线接入 `docs/step/` 入口。
