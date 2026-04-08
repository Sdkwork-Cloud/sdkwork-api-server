# Step 06 - 控制平面、Admin、Portal 与商业化工作区收口

## 1. 目标与范围

本 step 的目标是围绕 `tenant / project / api key / provider / model / routing policy / quota / usage / billing / commerce / extension runtime` 等核心平台对象，完成控制平面、Admin、Portal 与商业化工作区的主流程收口。

### 1.1 执行输入

- step 05 的 Provider 与扩展运行时标准
- `docs/架构/05-数据模型与存储设计.md`
- `docs/架构/08-安全-多租户-SaaS-私有化-部署设计.md`
- `docs/架构/133-控制平面与运营后台设计-2026-04-07.md`
- `docs/架构/139-权限与能力模型设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

### 1.2 本步非目标

- 不在本步内完成最终性能压测与容量证明
- 不把前端视觉优化当作本 step 的主要成果
- 不跳过后端流程就直接堆前端页面

### 1.3 最小输出

- 控制平面对象与真值归属清晰
- Admin 工作区收口
- Portal 自助工作区收口
- 商业化关键流程收口

## 2. 架构对齐

本 step 直接对齐：

- `docs/架构/03-模块规划与边界.md`
- `docs/架构/05-数据模型与存储设计.md`
- `docs/架构/08-安全-多租户-SaaS-私有化-部署设计.md`
- `docs/架构/133-控制平面与运营后台设计-2026-04-07.md`
- `docs/架构/139-权限与能力模型设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

## 3. 当前现状

当前仓库已经具备：

- `services/admin-api-service`
- `services/portal-api-service`
- `apps/sdkwork-router-admin`
- `apps/sdkwork-router-portal`
- 大量控制平面与商业化相关 `app-* / domain-*` crate

仍需收敛的问题包括：

- 对象真值归属是否清晰
- 后端工作流与前端工作区是否完全对齐
- Admin 与 Portal 的权限、视图与流程边界是否清晰
- 商业化、配额、使用量、结算与路由治理是否形成闭环

## 4. 设计

### 4.1 控制平面对象分层

建议按以下对象分层治理：

- 平台级：tenant、workspace、project、member、role、credential
- 能力级：capability、permission policy、feature flag、approval boundary
- 接入级：provider、model、channel、extension runtime
- 路由级：policy、profile、preference、simulation、health evidence
- 商业级：quota、usage、billing、pricing、coupon、campaign
- 运维级：audit、runtime status、deployment / rollout evidence

### 4.2 Admin 与 Portal 分工

- Admin：
  - 平台治理、运营、风控、资源配置、审核、发布
- Portal：
  - 租户自助、项目管理、API Key、自助 Provider 使用、消费与账单查看

### 4.3 工作流要求

必须收敛以下核心流程：

- 租户与项目创建
- 凭据与 API Key 生命周期
- Provider / model 可用性管理
- 路由策略与 quota / billing 联动
- usage / settlement / billing evidence 查询
- 控制面配置变更、审批、灰度、回退与审计

### 4.4 组件化要求

对前端工作区必须明确：

- 工作区级页面边界
- 表单、列表、详情、审核、实验、运营面板等通用组件边界
- 不允许同一业务对象在 Admin 与 Portal 中形成两套字段定义

## 5. 实施落地规划

### 5.1 后端对象与流程收敛

重点模块：

- `crates/sdkwork-api-app-tenant`
- `crates/sdkwork-api-app-identity`
- `crates/sdkwork-api-app-credential`
- `crates/sdkwork-api-app-billing`
- `crates/sdkwork-api-app-usage`
- `crates/sdkwork-api-app-catalog`
- `crates/sdkwork-api-app-commerce`
- `crates/sdkwork-api-app-coupon`
- `crates/sdkwork-api-app-marketing`

### 5.2 接口层收敛

重点模块：

- `crates/sdkwork-api-interface-admin`
- `crates/sdkwork-api-interface-portal`
- `services/admin-api-service`
- `services/portal-api-service`

### 5.3 前端工作区收敛

重点模块：

- `apps/sdkwork-router-admin`
- `apps/sdkwork-router-portal`

重点动作：

- 按工作区和业务对象进行组件化
- 建立统一状态、查询、提交流程
- 以真实后端契约驱动页面而非页面先行发明接口

### 5.4 商业化闭环

必须至少收敛：

- quota 配置与命中
- usage 采集与对账
- billing events 与账单展示
- pricing、优惠、活动与结算工作流

## 6. 测试计划

建议执行：

- 控制平面 API 集成测试
- 角色与权限测试
- 能力矩阵、权限矩阵与配置变更审批测试
- Admin / Portal 核心页面 smoke
- 商业化主流程 E2E 测试
- usage / quota / billing 对账测试

## 7. 结果验证

本 step 完成时必须满足：

- 控制平面对象真值归属清晰
- Admin 与 Portal 的工作区边界清晰
- 核心商业化闭环能贯通到路由与结算证据
- 能力矩阵、变更权限与对象真值在 Admin / Portal / API 三个面保持一致

## 8. 检查点

- `CP06-1`：完成控制平面对象真值归属冻结
- `CP06-2`：完成 Admin / Portal 后端接口收敛
- `CP06-3`：完成前端工作区主流程收口
- `CP06-4`：完成 quota / usage / billing 主链路验证

### 8.1 推荐 review 产物

- `docs/review/step-06-控制平面对象矩阵-YYYY-MM-DD.md`
- `docs/review/step-06-admin-portal工作区复盘-YYYY-MM-DD.md`
- `docs/review/step-06-商业化闭环复盘-YYYY-MM-DD.md`

### 8.2 串行依赖与推荐并行车道

- step 级依赖：在 `05` 后完成，`07-10` 会依赖本步结果
- 可并行车道：
  - `06-A`：控制平面后端
  - `06-B`：商业化后端
  - `06-C`：Admin 前端工作区
  - `06-D`：Portal 前端工作区
  - `06-E`：对账与运营证据
- 收口要求：对象真值、能力模型与配置变更权限由单一 control-plane owner 裁决

### 8.3 架构能力闭环判定

当控制平面对象、工作流与前后端工作区已经围绕同一套真值运行时，本 step 才算闭环。

### 8.4 快速并行执行建议

- 后端工作流和前端工作区并行推进
- 商业化证据与对账作为独立验证车道

### 8.5 完成后必须回写的架构文档

- `docs/架构/05-数据模型与存储设计.md`
- `docs/架构/08-安全-多租户-SaaS-私有化-部署设计.md`
- `docs/架构/133-控制平面与运营后台设计-2026-04-07.md`
- `docs/架构/139-权限与能力模型设计-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

### 8.6 本步完成后必须兑现的架构能力

- `docs/架构/133-*` 中控制平面统一治理 `tenant / project / key / provider / channel / model / routing policy / quota / usage / billing / extension runtime` 的要求已成为真实工作流。
- `docs/架构/05-*` 中对象真值归属已经在后端流程、接口层和前端工作区中保持一致。
- `docs/架构/08-*` 中多租户、自助门户和平台治理边界已在 Admin / Portal 中清晰分离。
- `docs/架构/139-*` 中角色、能力、授权边界已在控制平面对象与工作区能力开关中可验证。
- `docs/架构/141-*` 中控制面配置审批、灰度、回退和审计要求已进入主工作流。

### 8.7 最快完整执行建议

1. 先由 control-plane owner 冻结对象真值矩阵、权限矩阵和主工作流。
2. `06-A/B/C/D/E` 分别推进控制平面后端、商业化后端、Admin、Portal、对账证据。
3. 验证车道只跑角色权限、主流程 E2E、usage / quota / billing 对账，不参与业务字段发明。
4. 只有当前后端对象、工作区和证据链同时一致时，才允许进入 `07-10`。

## 9. 风险与回滚

### 9.1 风险

- 若对象真值不清，会在前后端和商业化链路中制造大量重复实现
- 若前端页面先行，会使后端契约不断返工

### 9.2 回滚

- 新工作区先与旧流程并行可见
- 后端对象迁移优先保留旧查询接口做兼容桥接

## 10. 完成定义

满足以下条件视为完成：

- 控制平面对象、Admin、Portal、商业化主流程已收口
- 核心工作区能支撑真实运营与租户自助场景

## 11. 下一步准入条件

进入 step 07 前必须确认：

- 多模态与异步任务所需的 quota、usage、billing、对象治理入口已具备
- 前后端工作区可承载资源类对象与任务类对象
