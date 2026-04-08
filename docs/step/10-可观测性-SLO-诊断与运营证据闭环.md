# Step 10 - 可观测性、SLO、诊断与运营证据闭环

## 1. 目标与范围

本 step 的目标是建立统一的 logs、metrics、traces、request-id、routing decision log、provider health、runtime status、usage / billing evidence 与诊断工具链，让平台在生产环境中具备可定位、可解释、可审计、可运营的能力。

### 1.1 执行输入

- step 04 到 step 09 的路由、数据、安全和控制平面结果
- `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
- `docs/架构/138-架构评分卡与阶段性验收标准-2026-04-07.md`
- `docs/架构/140-数据生命周期与成本治理设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

### 1.2 本步非目标

- 不把“有日志输出”误写成“具备可观测性闭环”
- 不只做技术指标而忽略运营、计费、审计和租户体验证据
- 不在缺少 runbook 的情况下宣称具备生产可运维性

### 1.3 最小输出

- 统一 telemetry 标准
- SLO 与 error budget 标准
- 统一诊断与排错入口
- 统一运营证据体系

## 2. 架构对齐

本 step 直接对齐：

- `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
- `docs/架构/138-架构评分卡与阶段性验收标准-2026-04-07.md`
- `docs/架构/140-数据生命周期与成本治理设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

## 3. 当前现状

当前仓库已具备较好基础，包括：

- `sdkwork-api-observability`
- routing decision log 与 provider health 等领域资产
- 多服务运行面与启停脚本

仍需进一步闭环：

- 跨服务 trace 与 request-id 追踪
- 技术与业务证据的一体化
- SLO 与运维 runbook 的收敛
- 针对 stateful / stateless 双态的分别监控与评分

## 4. 设计

### 4.1 统一遥测标准

必须统一：

- 结构化日志格式
- `x-request-id` / trace id 透传
- metrics 命名与标签
- spans 与关键阶段事件

### 4.2 统一证据标准

除技术遥测外，还必须统一：

- routing decision evidence
- provider health evidence
- quota / usage / billing evidence
- security / audit evidence
- rollout / rollback evidence

### 4.3 SLO 模型

至少建立：

- 可用性 SLO
- 延迟 SLO
- 错误率 SLO
- 流式稳定性 SLO
- 后台与控制平面关键工作流 SLO

### 4.4 诊断入口

统一诊断入口应覆盖：

- 健康检查
- metrics
- request trace
- provider runtime status
- 路由决策解释
- billing / usage 对账追踪

### 4.5 生命周期、成本与变更证据

运营证据必须额外覆盖：

- 数据保留期、归档、删除相关证据
- usage、billing、成本归因证据
- 配置变更、灰度、回滚与 break-glass 证据

## 5. 实施落地规划

### 5.1 遥测基础收敛

重点模块：

- `crates/sdkwork-api-observability`
- `services/gateway-service`
- `services/admin-api-service`
- `services/portal-api-service`
- `services/router-product-service`
- `services/router-web-service`

### 5.2 领域证据收敛

重点模块：

- routing 相关模块
- provider 相关模块
- usage / billing 相关模块
- 安全与审计相关模块

### 5.3 诊断工具与脚本收敛

重点模块：

- `bin/router-ops.mjs`
- `bin/`
- `scripts/dev/`

### 5.4 运维文档与 runbook

需要建立：

- 常见故障诊断路径
- 灰度、回滚、恢复 runbook
- SLO 异常处置流程

## 6. 测试计划

建议执行：

- telemetry smoke 测试
- request-id 贯通测试
- trace 完整性测试
- SLO 计算与告警演练
- 运营证据对账测试
- 生命周期、成本与变更审计证据测试

## 7. 结果验证

本 step 完成时必须满足：

- 能从任一请求追踪到协议、路由、Provider、结算、审计证据
- SLO、告警和 runbook 可直接用于生产运维
- stateful / stateless 有各自独立的观测口径

## 8. 检查点

- `CP10-1`：完成统一 telemetry 标准冻结
- `CP10-2`：完成领域证据映射闭环
- `CP10-3`：完成诊断与 runbook 收敛
- `CP10-4`：完成 SLO 与告警验证

### 8.1 推荐 review 产物

- `docs/review/step-10-telemetry标准决议-YYYY-MM-DD.md`
- `docs/review/step-10-运营证据映射复盘-YYYY-MM-DD.md`
- `docs/review/step-10-SLO与runbook复盘-YYYY-MM-DD.md`

### 8.2 串行依赖与推荐并行车道

- step 级依赖：依赖 `04-09`
- 可并行车道：
  - `10-A`：telemetry 基础
  - `10-B`：routing / provider 证据
  - `10-C`：billing / audit / security 证据
  - `10-D`：诊断工具与 runbook
- 收口要求：统一 observability owner 冻结指标、标签、SLO 和诊断入口

### 8.3 架构能力闭环判定

只有当技术证据与业务证据被统一观测、查询和诊断时，本 step 才算闭环。

### 8.4 快速并行执行建议

- 先冻结指标与证据词典
- 再并行补遥测埋点、证据映射和诊断入口

### 8.5 完成后必须回写的架构文档

- `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
- `docs/架构/138-架构评分卡与阶段性验收标准-2026-04-07.md`
- `docs/架构/140-数据生命周期与成本治理设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

### 8.6 本步完成后必须兑现的架构能力

- `docs/架构/135-*` 中 logs / metrics / traces / request-id / health / runbook 的要求已形成统一观测体系。
- `docs/架构/138-*` 中“可运维”“可追踪”“可诊断”的评分项已具备真实证据。
- routing、provider、billing、security、rollout 等关键业务链路已可从同一请求追踪到同一证据链。
- `docs/架构/140-*` 中生命周期与成本治理的关键证据已进入统一运营证据体系。
- `docs/架构/141-*` 中配置变更、灰度、回滚与紧急授权事件已可被统一追踪与审计。

### 8.7 最快完整执行建议

1. 先冻结 telemetry 词典、标签规则、SLO 模型和证据对象清单。
2. `10-A/B/C/D` 分别推进基础遥测、路由与 Provider 证据、billing/audit 证据、诊断工具与 runbook。
3. 验证车道独立跑 request-id 贯通、trace 完整性、SLO 计算和 runbook 演练。
4. 没有被接入统一证据链的能力，不得在评分卡中视为“已具备生产运维能力”。

## 9. 风险与回滚

### 9.1 风险

- 若缺少统一证据链，生产问题将无法稳定定位
- 若没有 runbook，观测数据无法转化为处置能力

### 9.2 回滚

- 对高噪音或高成本埋点先灰度启用
- 保留旧诊断入口直到新链路稳定

## 10. 完成定义

满足以下条件视为完成：

- 可观测性、SLO、诊断、运营证据已形成闭环
- 生产运维与客户排障都具备可执行入口

## 11. 下一步准入条件

进入 step 11 前必须确认：

- 安装、部署、发布、回滚将复用统一观测与诊断能力
- 交付文档不再缺少真实监控与运维依据
