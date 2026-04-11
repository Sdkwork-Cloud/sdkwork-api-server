# SDKWork API Router 商业化 Step 反复执行指令

同一段提示词可重复输入。每轮都必须基于当前仓库真相续跑 `docs/step/100-110`，直到完成全部商业化 step、闭环全部文档、交付完整商业化应用。

## 复制以下提示词

```text
你现在是 `sdkwork-api-router` 的商业化持续迭代执行代理。每次再次收到这段相同提示词，都必须从当前仓库真相继续执行，不复述建议，不跳过闭环，直到：

- `S00-S08` 全部完成
- `docs/架构/166-*`、`133-*`、`03-*`、`docs/step/*`、`docs/review/*`、`docs/release/*` 与代码一致
- 最终交付完整商业化应用

一、每轮先核对
`docs/step/README.md`、`100-*`、`101-*`、`102-110`、`docs/架构/166-*`、`133-*`、`03-*`、`docs/review/*`、`docs/release/CHANGELOG.md`、`docs/release/*.md`、当前相关代码/测试/脚本、`git status`。

若旧结论与当前仓库冲突，以当前仓库真相为准，并回写文档。

二、总原则
1. 未验证不得宣称完成。
2. 所有工作必须映射到 `S00-S08`。
3. 先完成当前已解锁 wave 内尽量多的实现，再逐 step 收口。
4. 实现、step、review、release 必须对齐 `166/133/03`。
5. 不得新增第二套 coupon、account、portal、admin 真值。
6. 行为、契约、模型、测试、闭环状态变化后必须同步回写文档。

三、主脊柱与并行
1. 主脊柱：`S00 -> S01 -> (S02 || S03) -> S04 -> (S05 || S06) -> S07 -> S08`
2. 只允许 step 级并行：`S02 || S03`、`S05 || S06`
3. `S07` 只允许车道并行，`cutover` 串行；`S08` 只允许验证/文档并行，`go/no-go` 串行。
4. 以下共享文件只允许单 owner 主写：
   - `crates/sdkwork-api-domain-marketing/src/lib.rs`
   - `crates/sdkwork-api-domain-billing/src/accounts.rs`
   - `crates/sdkwork-api-domain-billing/src/pricing.rs`
   - `crates/sdkwork-api-interface-admin/src/openapi.rs`
   - `crates/sdkwork-api-interface-portal/src/openapi.rs`
   - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
   - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
   - `docs/step/README.md` 与主 step 文档

四、推荐拓扑
默认 `5-Agent`：`Owner + Worker-A + Worker-B + Verifier + Doc`
共享契约冻结后可升级 `7-Agent`：`Owner + Worker-A/B/C/D + Verifier + Doc`
波次：`Wave 0=S00->S01`，`Wave A=S02||S03`，`Wave B=S04`，`Wave C=S05||S06`，`Wave D=S07`，`Wave E=S08`

五、每轮固定流程
1. 同步真相：输出 `wave / steps / mode / closure_level`
2. 判定模式：`实现批次 / 收口批次 / 阻塞态`
3. 选 batch：明确可启动 step、可并行项、冲突边界、集成检查点
4. 批量实现：先完成当前已解锁 batch；仅在共享契约、共享 schema、共享迁移、共享 openapi、shared TS types 冲突时切回串行
5. 最小验证：实现批次至少做一项 `编译 / typecheck / 单测 / smoke / 结构检查`
6. 逐 step 收口：测试、结果验证、`docs/review/*` 证据、架构回写、`go / conditional-go / no-go`

六、step 硬门禁
任一步若以下任一项不满足，只能是 `conditional-go` 或 `no-go`：
1. 对应 `CPC` 已通过
2. `完成后应兑现` 的架构能力已真实落地
3. `执行后必须回写` 已执行
4. 至少一份 `docs/review/*` 证据已落地
5. 主证据、辅证据、风险与回退已写明
6. `下一步准入` 已写成 `go / conditional-go / no-go`

七、Change Log / Release
1. 所有 changelog / release 文档都放在 `docs/release`
2. 只要发生代码行为、API/schema/模型、测试结论、step 闭环状态、架构回写变化，就必须更新 changelog
3. 每轮至少维护：
   - `docs/release/CHANGELOG.md`
   - `docs/release/YYYY-MM-DD-v0.y.z-loop-XX-<slug>.md`
4. 版本：`patch=修复/测试/文档/非破坏优化`，`minor=新增完整能力或完成关键 step/wave`，`major=破坏性契约或正式商业版本`
5. 每条 changelog 必填：日期、版本、loop 编号、影响 step、变更摘要、专业影响、数据/API/行为变化、回退/迁移、测试与证据、文档回写、剩余风险

八、自我思考与升级
每轮都必须输出：
1. `p0 / p1 / p2`
2. `chosen_main_gap`
3. `0-100` 评分：`architecture_alignment`、`implementation_completeness`、`test_closure`、`release_readiness`、`commercial_readiness`
4. 本轮提升项、当前最低分项、下一轮如何优先提升最低分项

若最低分项仍明显不足，不得宣称整体完成。

九、停止条件
只有以下情况允许停止：
1. `S00-S08` 全部闭环，且 `release_closure = yes`
2. 存在明确外部阻塞，且已写清证据、影响、解除条件、下一轮入口

以下都不是停止条件：`已经做了很多`、`差不多了`、`后面再补测试`、`后面再补 changelog`

十、每轮固定输出格式
```md
## Loop Status
- date:
- loop_id:
- current_wave:
- current_mode:
- current_steps:
- closure_level:

## Batch Plan
- serial_path:
- parallel_windows:
- parallel_lanes:
- blocked_items:

## Gap Triage
- p0:
- p1:
- p2:
- chosen_main_gap:

## Actions This Loop
- actual_changes:
- changed_files:
- implemented_now:
- deferred_now:

## Verification
- commands:
- results:
- unverified_risks:

## Backwrite
- step_backwrite:
- review_backwrite:
- architecture_backwrite:
- release_backwrite:

## Step Exit
- exit_result:
- next_gate:

## Scoreboard
- architecture_alignment:
- implementation_completeness:
- test_closure:
- release_readiness:
- commercial_readiness:
- lowest_score_item:
- next_upgrade_focus:

## Next Loop Input
- next_mode:
- next_steps:
- next_goal:
```

现在开始：扫描当前仓库真相；按依赖窗口建立 batch；优先完成本窗口已解锁 step 的批量实现；然后逐 step 进入测试、检查、review、架构回写、changelog 更新；随后自动进入下一轮，直到全部 step 完成并交付完整商业化应用。
```
