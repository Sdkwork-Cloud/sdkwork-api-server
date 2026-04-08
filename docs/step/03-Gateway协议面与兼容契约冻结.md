# Step 03 - Gateway 协议面与兼容契约冻结

## 1. 目标与范围

本 step 的目标是冻结 `sdkwork-api-router` 对外 Gateway 协议面的统一契约、兼容矩阵、错误语义、流式行为与版本治理规则，避免后续路由内核、Provider 接入和控制平面实现建立在漂移的接口定义之上。

### 1.1 执行输入

- step 02 的模块边界收敛结果
- `docs/架构/06-Gateway-API-与协议设计.md`
- `docs/架构/130-API-Router-行业对标与终局能力矩阵-2026-04-07.md`
- `docs/架构/131-多模态请求生命周期与路由时序设计-2026-04-07.md`
- `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`

### 1.2 本步非目标

- 不直接优化路由性能与容量
- 不直接完成 Provider 治理闭环
- 不把未来拟支持协议全部写成当前已落地

### 1.3 最小输出

- 统一 Gateway 契约模型
- 统一兼容矩阵
- 统一错误码、限流、认证、usage、stream 语义
- 统一版本治理与弃用规则

## 2. 架构对齐

本 step 直接对齐：

- `docs/架构/06-Gateway-API-与协议设计.md`
- `docs/架构/07-路由-缓存-流式通信-异步任务设计.md`
- `docs/架构/131-多模态请求生命周期与路由时序设计-2026-04-07.md`
- `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`

## 3. 当前现状

当前仓库已经明确支持 `OpenAI Compatible`、`Anthropic Messages Compatible`、`Gemini Compatible` 三类高价值协议面，但仍需进一步冻结以下高风险内容：

- 对外兼容面与内部规范面之间的关系
- `stateful standalone` 与 `stateless runtime` 在协议行为上的差异边界
- 多模态资源与长任务的统一错误、状态与结果语义
- usage、billing、trace、request-id 与 decision log 的协议映射

## 4. 设计

### 4.1 协议分层模型

建议采用：

- 统一网关规范层：平台统一 admission、quota、route、usage、audit 语义
- 协议兼容适配层：OpenAI / Anthropic / Gemini 的请求响应映射
- Provider 适配层：把兼容协议进一步映射到 Provider 能力模型

### 4.2 统一对象模型

至少统一以下对象：

- 请求上下文
- 身份与项目解析结果
- 路由候选上下文
- 统一错误模型
- 统一 usage / settlement 模型
- 统一流式事件与完成信号
- 统一异步任务状态对象

### 4.3 兼容矩阵治理

兼容矩阵必须明确：

- 已支持
- 部分支持
- 兼容转换支持
- 明确不支持
- 计划支持

并区分：

- `stateful standalone`
- `stateless runtime`

### 4.4 版本与弃用策略

必须冻结：

- 对外稳定字段
- 兼容转换字段
- 弃用字段
- provider 私有扩展字段的承载方式

## 5. 实施落地规划

### 5.1 契约模型冻结

重点收敛或新增以下落点：

- `crates/sdkwork-api-contract-gateway`
- `crates/sdkwork-api-contract-openai`
- 必要时新增或补强 `anthropic / gemini` 契约子模块或 crate
- `crates/sdkwork-api-openapi`

### 5.2 接口层收敛

重点改造：

- `crates/sdkwork-api-interface-http`
- `services/gateway-service`

确保：

- 路由、认证、限流、usage、stream 映射一致
- 协议兼容逻辑不再散落在多个入口中

### 5.3 文档与矩阵收敛

同步更新：

- `docs/api/compatibility-matrix.md`
- `docs/zh/reference/api-compatibility.md`
- API 参考文档与错误码清单

### 5.4 契约测试资产

建立或强化：

- 请求 / 响应 golden fixtures
- stream 事件 golden fixtures
- error mapping fixtures
- stateful / stateless 差异用例

## 6. 测试计划

建议执行：

- 契约单元测试
- 协议兼容集成测试
- 流式输出与中断恢复测试
- 限流、鉴权、配额、错误码负向测试
- stateful / stateless 差异测试

## 7. 结果验证

本 step 完成时，必须能够明确回答：

- 哪些协议当前真的支持到什么程度
- 同一请求在不同运行形态下有哪些明确差异
- Gateway 层的错误、usage、stream 与 audit 语义是否统一

## 8. 检查点

- `CP03-1`：完成统一对象模型与兼容矩阵冻结
- `CP03-2`：完成 Gateway 接口层收敛
- `CP03-3`：完成契约测试基线
- `CP03-4`：完成 API 文档与兼容文档回写

### 8.1 推荐 review 产物

- `docs/review/step-03-兼容矩阵决议-YYYY-MM-DD.md`
- `docs/review/step-03-错误语义与stream决议-YYYY-MM-DD.md`
- `docs/review/step-03-stateful-stateless协议差异清单-YYYY-MM-DD.md`

### 8.2 串行依赖与推荐并行车道

- step 级依赖：必须在 `02` 后、`04` 前完成
- 可并行车道：
  - `03-A`：OpenAI 兼容契约
  - `03-B`：Anthropic / Gemini 兼容契约
  - `03-C`：错误、usage、stream 统一语义
  - `03-D`：文档与兼容矩阵
- 收口要求：统一由协议 owner 冻结字段、错误码与兼容等级

### 8.3 架构能力闭环判定

当协议层已成为后续路由、Provider、控制平面都能复用的唯一权威入口时，本 step 才算闭环。

### 8.4 快速并行执行建议

- 先并行整理各协议家族
- 再统一收口到一个网关规范层
- 最后一次性回写兼容矩阵与 API 文档

### 8.5 完成后必须回写的架构文档

- `docs/架构/06-Gateway-API-与协议设计.md`
- `docs/架构/131-多模态请求生命周期与路由时序设计-2026-04-07.md`
- `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`

### 8.6 本步完成后必须兑现的架构能力

- `docs/架构/06-*` 中 Gateway 作为统一对外协议面的要求，已经转成单一权威契约层。
- `docs/架构/131-*` 中多模态、流式、长任务的请求与结果语义，已经有统一兼容规则和错误模型。
- `docs/架构/132-*` 中 Provider 适配将以统一协议规范层为前提，不再反向定义 Gateway 行为。

### 8.7 最快完整执行建议

1. 先由协议 owner 冻结统一对象模型、错误语义与兼容等级。
2. `03-A/B/C/D` 分别推进 OpenAI、Anthropic/Gemini、stream/usage/error、文档矩阵。
3. 验证车道使用 golden fixtures 和差异测试独立验证 `stateful` / `stateless` 协议行为。
4. 未写入兼容矩阵的行为，一律不得在后续实现中默认宣称支持。

## 9. 风险与回滚

### 9.1 风险

- 若协议层没有冻结，后续路由与 Provider 改造会持续返工
- 若兼容矩阵不准确，会直接损伤对外产品可信度

### 9.2 回滚

- 在切换新契约前保留旧适配 facade
- 对外文档先标注兼容级别，避免一次性切断历史行为

## 10. 完成定义

满足以下条件视为完成：

- Gateway 契约与兼容矩阵已冻结
- 错误、usage、stream、异步状态语义统一
- 对外 API 文档与实际行为一致

## 11. 下一步准入条件

进入 step 04 前必须确认：

- 路由内核已经有稳定契约可依赖
- Provider 与控制平面不会再反向定义 Gateway 字段
