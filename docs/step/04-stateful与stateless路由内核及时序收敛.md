# Step 04 - stateful 与 stateless 路由内核及时序收敛

## 1. 目标与范围

本 step 的目标是把 `sdkwork-api-router` 的路由主链路收敛为“一套统一路由内核 + 两种明确运行形态适配”，既保证 `stateful standalone` 的完整治理能力，又保持 `stateless runtime` 的轻量嵌入能力。

### 1.1 执行输入

- step 03 冻结后的 Gateway 契约与兼容矩阵
- `docs/架构/07-路由-缓存-流式通信-异步任务设计.md`
- `docs/架构/131-多模态请求生命周期与路由时序设计-2026-04-07.md`
- `docs/架构/134-高性能与容量规划设计-2026-04-07.md`

### 1.2 本步非目标

- 不在本步内完成全部 Provider onboarding
- 不在本步内直接实现所有高级缓存能力
- 不把 stateful 数据真值能力直接复制到 stateless 运行态

### 1.3 最小输出

- 统一路由主时序
- 统一候选筛选、排序、回退与决策日志
- 明确 `stateful` 与 `stateless` 的适配边界
- 统一 usage / billing / audit 的结算挂点

## 2. 架构对齐

本 step 直接对齐：

- `docs/架构/02-架构标准与总体设计.md`
- `docs/架构/07-路由-缓存-流式通信-异步任务设计.md`
- `docs/架构/131-多模态请求生命周期与路由时序设计-2026-04-07.md`
- `docs/架构/134-高性能与容量规划设计-2026-04-07.md`

## 3. 当前现状

当前项目已经具备：

- `stateful standalone` 与 `stateless runtime` 两种运行形态
- `routing policy / profile / preferences / provider health / decision log` 等基础资产
- `gateway-service` 与产品宿主的双重运行骨架

但仍需继续收敛的问题包括：

- 两种运行形态的路由链路与审计语义需要明确同源与差异边界
- 候选筛选、降级、fallback、settlement 的统一时序需要固定
- 路由热路径与控制平面、数据平面之间需要更清晰的隔离

## 4. 设计

### 4.1 统一路由主时序

统一采用以下时序：

1. Identity Resolution
2. Admission / Auth / Quota
3. Policy / Profile / Preference Resolve
4. Candidate Filter
5. Health Snapshot / Capability Match
6. Candidate Rank / Selection
7. Provider Dispatch
8. Stream / Async / Sync Result Handling
9. Settlement / Usage / Billing / Audit
10. Decision Log / Metrics / Trace Close

### 4.2 两种运行形态适配规则

- `stateful standalone`
  - 可依赖本地控制平面、Admin Store、完整 billing / audit / runtime evidence
- `stateless runtime`
  - 只保留轻量 relay / routing / compatibility 运行时
  - 不应伪装为与 standalone 对等的本地治理平面

### 4.3 决策与结算挂点

必须统一：

- routing decision log 记录点
- usage 结算点
- billing event 生成点
- provider health 更新点
- fallback / retry 的可解释日志

### 4.4 热路径约束

路由内核必须满足：

- 热路径最短
- admission、selection、dispatch 可观测
- 控制平面写操作不阻塞热路径
- 异步长任务与流式短链路分离

## 5. 实施落地规划

### 5.1 路由内核收敛

重点收敛以下模块：

- `crates/sdkwork-api-app-routing`
- `crates/sdkwork-api-domain-routing`
- `crates/sdkwork-api-policy-routing`
- `crates/sdkwork-api-provider-core`

### 5.2 运行态装配收敛

重点落点：

- `crates/sdkwork-api-app-gateway`
- `crates/sdkwork-api-app-runtime`
- `services/gateway-service`
- `services/router-product-service`

### 5.3 stateful / stateless 双态治理

分别明确：

- 共享内核代码
- stateful 专有依赖
- stateless 专有依赖
- 验收与发布口径

### 5.4 决策日志与健康快照统一

统一以下对象的生成、存储与查询入口：

- `RoutingDecisionLog`
- `ProviderHealthSnapshot`
- route simulation / explain 结果

## 6. 测试计划

建议执行：

- 路由策略单元测试
- 候选筛选与排序集成测试
- fallback / retry / 熔断测试
- stateful / stateless 对照测试
- 流式与异步任务的结算挂点测试

## 7. 结果验证

本 step 完成时必须满足：

- 同一请求的路由时序可以被清晰解释
- 两种运行形态都有清晰边界和独立结论
- 路由、计费、审计与决策日志没有出现第二套路由系统

## 8. 检查点

- `CP04-1`：完成统一路由主时序冻结
- `CP04-2`：完成 stateful / stateless 适配边界定义
- `CP04-3`：完成决策日志、usage、billing 挂点统一
- `CP04-4`：完成路由主链路验证

### 8.1 推荐 review 产物

- `docs/review/step-04-路由主时序决议-YYYY-MM-DD.md`
- `docs/review/step-04-stateful-stateless适配决议-YYYY-MM-DD.md`
- `docs/review/step-04-决策日志与结算挂点复盘-YYYY-MM-DD.md`

### 8.2 串行依赖与推荐并行车道

- step 级依赖：必须在 `03` 后完成，是 `05-07` 的主脊柱前置
- 可并行车道：
  - `04-A`：路由策略与候选选择
  - `04-B`：stateful 运行态装配
  - `04-C`：stateless 运行态装配
  - `04-D`：决策日志与健康快照
- 收口要求：统一路由 owner 冻结最终主时序与双态边界

### 8.3 架构能力闭环判定

当所有外部协议面都汇聚到同一套路由与结算主链路时，本 step 才算闭环。

### 8.4 快速并行执行建议

- 先冻结时序，再并行实现双态适配
- 决策日志和健康快照单独作为证据车道推进

### 8.5 完成后必须回写的架构文档

- `docs/架构/07-路由-缓存-流式通信-异步任务设计.md`
- `docs/架构/131-多模态请求生命周期与路由时序设计-2026-04-07.md`
- `docs/架构/134-高性能与容量规划设计-2026-04-07.md`

### 8.6 本步完成后必须兑现的架构能力

- `docs/架构/131-*` 中 `Identity Resolution / Admission / Candidate Selection / Fallback / Stream Close` 等关键决策点已真正落在统一主时序上。
- `docs/架构/07-*` 中统一路由、统一结算、统一决策日志的要求已经兑现，不再存在第二套路由系统。
- `docs/架构/134-*` 中热路径最短、控制平面不阻塞热路径的原则已成为实现约束。

### 8.7 最快完整执行建议

1. `04-Owner` 先冻结 10 步统一主时序和双运行态分界。
2. `04-A/B/C/D` 分别推进策略选择、stateful、stateless、决策日志与健康快照。
3. 验证车道专门跑 fallback、retry、双态对照、usage/billing 挂点测试。
4. 必须等 `04` 收口后，`05-07` 才能并行扩展 Provider、控制平面和多模态能力。

## 9. 风险与回滚

### 9.1 风险

- 若双态边界不清，会造成产品定位和交付口径混乱
- 若路由与结算挂点分离，会导致 usage、billing、audit 失真

### 9.2 回滚

- 双态切换前保留旧装配入口
- 路由策略收敛优先保留解释能力和回退开关

## 10. 完成定义

满足以下条件视为完成：

- 统一路由主时序已冻结
- 双态边界与结算挂点已统一
- 路由主链路可测试、可解释、可审计

## 11. 下一步准入条件

进入 step 05 前必须确认：

- Provider 接入将复用统一路由主链路
- 不再允许 Provider 自带第二套路由与结算逻辑
