# Step 05 - Provider 接入与扩展运行时治理落地

## 1. 目标与范围

本 step 的目标是把 `sdkwork-api-router` 的 Provider onboarding、模型能力声明、健康检查、密钥注入、扩展运行时与灰度发布治理收敛成统一标准，让平台在接入 `OpenAI / OpenRouter / Ollama / connector / native-dynamic` 等生态时保持高扩展性与高安全性。

### 1.1 执行输入

- step 04 的统一路由内核
- `docs/架构/04-技术选型与可插拔策略.md`
- `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`
- `docs/架构/136-安全与合规评估标准-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

### 1.2 本步非目标

- 不直接完成全部商业化工作区的产品体验打磨
- 不把任何单一 Provider 的临时兼容逻辑写成平台标准
- 不在没有治理证据的情况下允许动态扩展进入生产默认路径

### 1.3 最小输出

- Provider 接入生命周期标准
- 模型与能力声明标准
- 扩展运行时治理标准
- 密钥注入、健康检查、reload / rollout / rollback 标准

## 2. 架构对齐

本 step 直接对齐：

- `docs/架构/04-技术选型与可插拔策略.md`
- `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`
- `docs/架构/136-安全与合规评估标准-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

## 3. 当前现状

当前仓库已经具备较强基础：

- 内置 Provider：`openai`、`openrouter`、`ollama`
- 扩展运行时：`builtin`、`connector`、`native-dynamic`
- 相关基础 crate：`provider-*`、`extension-*`、`secret-*`

仍需继续收敛的问题包括：

- onboarding 生命周期是否统一
- provider 模型能力声明是否足够权威
- connector 与 native-dynamic 的安全边界与灰度规则是否足够硬
- 密钥、健康检查、回滚和审计的治理链路是否完整

## 4. 设计

### 4.1 Provider 生命周期

统一生命周期建议为：

1. 注册
2. 配置与凭据注入
3. 能力声明与兼容校验
4. 健康检查与预热
5. 灰度放量
6. 正式启用
7. 变更、暂停、回滚
8. 下线与审计归档

### 4.2 能力声明标准

必须统一声明：

- 支持的协议家族
- 支持的模型与模态
- 是否支持 sync / stream / async / batch
- 限流与配额约束
- 使用成本与计费元数据

### 4.3 扩展运行时标准

三类运行时必须清晰区分：

- `builtin`
- `connector`
- `native-dynamic`

每一类都必须定义：

- 安装方式
- 配置方式
- 密钥来源
- 健康探测
- 失败隔离
- reload / rollout / rollback

### 4.4 安全约束

所有 Provider 与扩展运行时接入必须满足：

- 凭据最小暴露
- 审计可追踪
- 故障可隔离
- 行为可回滚

### 4.5 配置变更与灰度安全

所有 Provider、channel、model catalog、runtime reload 相关变更必须满足：

- 配置审批与变更审计
- 灰度、生效、回退边界明确
- 不允许未审计的动态变更直接进入生产默认路径

## 5. 实施落地规划

### 5.1 Provider 核心收敛

重点收敛：

- `crates/sdkwork-api-provider-core`
- `crates/sdkwork-api-provider-openai`
- `crates/sdkwork-api-provider-openrouter`
- `crates/sdkwork-api-provider-ollama`

### 5.2 扩展运行时收敛

重点收敛：

- `crates/sdkwork-api-extension-abi`
- `crates/sdkwork-api-extension-core`
- `crates/sdkwork-api-extension-host`
- `crates/sdkwork-api-ext-provider-native-mock`
- `crates/sdkwork-api-runtime-host`

### 5.3 密钥与凭据收敛

重点收敛：

- `crates/sdkwork-api-secret-core`
- `crates/sdkwork-api-secret-local`
- `crates/sdkwork-api-secret-keyring`
- `crates/sdkwork-api-app-credential`

### 5.4 控制平面接入治理

把 Provider onboarding、model catalog、extension runtime status 接入：

- `crates/sdkwork-api-interface-admin`
- `crates/sdkwork-api-interface-portal`
- `services/admin-api-service`
- `services/portal-api-service`

## 6. 测试计划

建议执行：

- 内置 Provider 契约测试
- connector / native-dynamic 生命周期测试
- 密钥注入与轮转测试
- 健康检查、恢复、熔断、回滚测试
- 配置变更、reload、rollout、rollback 安全测试
- onboarding 到下线的全生命周期测试

## 7. 结果验证

本 step 完成时必须满足：

- Provider 接入不再依赖手工特判
- 扩展运行时具备治理、审计与回滚能力
- 任一 Provider 失败不会破坏平台整体稳定性

## 8. 检查点

- `CP05-1`：完成 Provider 生命周期标准冻结
- `CP05-2`：完成能力声明标准与健康检查标准冻结
- `CP05-3`：完成扩展运行时治理落地
- `CP05-4`：完成密钥与凭据链路验证

### 8.1 推荐 review 产物

- `docs/review/step-05-provider生命周期决议-YYYY-MM-DD.md`
- `docs/review/step-05-extension-runtime治理复盘-YYYY-MM-DD.md`
- `docs/review/step-05-密钥注入与健康检查复盘-YYYY-MM-DD.md`

### 8.2 串行依赖与推荐并行车道

- step 级依赖：必须在 `04` 后完成，之后才能稳定推进 `06-08`
- 可并行车道：
  - `05-A`：builtin Provider 治理
  - `05-B`：connector / native-dynamic 治理
  - `05-C`：secret 与 credential 治理
  - `05-D`：Admin / Portal Provider 工作区接入
- 收口要求：单一 provider owner 冻结生命周期与安全边界

### 8.3 架构能力闭环判定

当新 Provider 能按统一标准被接入、灰度、回滚并审计时，本 step 才算闭环。

### 8.4 快速并行执行建议

- 优先并行处理内置 Provider 与扩展运行时
- 控制平面接入作为单独车道，在核心生命周期稳定后再收口

### 8.5 完成后必须回写的架构文档

- `docs/架构/04-技术选型与可插拔策略.md`
- `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`
- `docs/架构/136-安全与合规评估标准-2026-04-07.md`
- `docs/架构/141-控制面配置治理与变更安全设计-2026-04-07.md`

### 8.6 本步完成后必须兑现的架构能力

- `docs/架构/132-*` 中 `builtin / connector / native-dynamic` 三类运行时治理标准已经转成统一生命周期。
- `docs/架构/04-*` 中可插拔、可灰度、可回滚的技术选型原则已在 Provider 接入链上兑现。
- `docs/架构/136-*` 中凭据最小暴露、扩展运行时隔离和可审计要求已落实到 onboarding 流程。
- `docs/架构/141-*` 中配置审批、灰度发布、回退与变更审计要求已进入 Provider 与 runtime 变更链路。

### 8.7 最快完整执行建议

1. 先冻结 Provider 生命周期和能力声明 schema，再允许多车道并行实现。
2. `05-A/B/C/D` 分别负责 builtin、connector/native-dynamic、secret/credential、控制平面接入。
3. 验证车道独立执行健康检查、reload、rollback、credential rotation 测试。
4. 未通过灰度和回滚验证的 Provider，不允许进入 `06` 的正式工作区。

## 9. 风险与回滚

### 9.1 风险

- 若 onboarding 无统一标准，Provider 数量增长后维护成本会失控
- 若动态扩展无安全边界，平台会引入高风险供应链与稳定性问题

### 9.2 回滚

- Provider 上线必须支持灰度与快速摘流
- 扩展运行时必须支持禁用、卸载与恢复到 builtin 路径

## 10. 完成定义

满足以下条件视为完成：

- Provider 接入、扩展运行时、密钥与健康治理已形成统一标准
- 关键 Provider 具备治理、审计与回滚能力

## 11. 下一步准入条件

进入 step 06 前必须确认：

- 控制平面已能围绕统一 Provider 标准开展工作区治理
- 不再依赖隐式配置或手工脚本驱动 Provider 生命周期
