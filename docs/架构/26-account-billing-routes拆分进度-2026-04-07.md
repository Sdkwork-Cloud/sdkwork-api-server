# account_billing_routes 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs` 原文件超过 1500 行，混合了账务视图、交易审计、定价 CRUD、定价生命周期等多类职责，维护成本高，定位回归点困难，不符合高内聚低耦合的测试组织标准。

## 本次拆分目标

- 保留顶层薄入口，避免直接删除原测试入口文件
- 将共享 fixture、登录、JSON 解析逻辑下沉到 `support.rs`
- 按业务边界拆分为独立子模块，而不是按行数机械切块
- 控制单文件规模，降低阅读和后续扩展成本

## 拆分结果

顶层入口：

- `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`

目录模块：

- `crates/sdkwork-api-interface-admin/tests/account_billing_routes/mod.rs`

共享支持：

- `crates/sdkwork-api-interface-admin/tests/account_billing_routes/support.rs`

业务子模块：

- `crates/sdkwork-api-interface-admin/tests/account_billing_routes/billing_views.rs`
- `crates/sdkwork-api-interface-admin/tests/account_billing_routes/commerce_audit.rs`
- `crates/sdkwork-api-interface-admin/tests/account_billing_routes/pricing_crud.rs`
- `crates/sdkwork-api-interface-admin/tests/account_billing_routes/pricing_lifecycle.rs`

## 职责划分

### billing_views

负责账务查询与调查视图：

- 账户余额汇总
- benefit lot 明细
- hold / settlement / pricing 查询
- ledger 历史与 allocation 明细
- 未知账户错误返回

### commerce_audit

负责交易审计相关只读能力：

- 最近订单列表
- 支付事件链路
- 订单审计聚合视图
- 未知订单错误返回

### pricing_crud

负责定价基础管理能力：

- 创建 pricing plan
- 创建 pricing rate
- 更新 pricing plan
- 更新 pricing rate
- 克隆 plan version 及其 rates

### pricing_lifecycle

负责定价状态流转与自动推进：

- draft plan 发布
- future effective 版本发布拦截
- future 版本计划生效
- 读取时自动激活 due planned version
- 显式 lifecycle synchronize
- retire plan 与 rate

## 当前收益

- `account_billing_routes.rs` 已降为薄入口
- 共享 fixture 与具体测试职责解耦
- 定价 CRUD 与定价生命周期边界清晰，后续可继续独立演进
- admin 账务与 commerce 审计测试意图更加明确

## 后续建议

- 继续处理 `crates` 下仍超过 1000 行的测试或源码文件
- 对同类 admin / portal / http route 测试保持统一拆分风格
- 在统一回归测试阶段，再集中执行相关 integration test 与格式/静态检查
