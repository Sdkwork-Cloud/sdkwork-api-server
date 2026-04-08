# portal_commerce 拆分进度

日期：2026-04-07

## 本次目标

将 `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs` 从单体 Portal 交易测试文件拆分为职责明确的目录模块，按目录/报价、订单流程、支付事件、账户对账等边界分层，降低交易闭环测试的维护成本。

## 拆分结果

已完成以下结构调整：

- 顶层薄入口：`crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`
- 目录模块入口：`crates/sdkwork-api-interface-portal/tests/portal_commerce/mod.rs`
- 共享支持：`crates/sdkwork-api-interface-portal/tests/portal_commerce/support.rs`
- 订单列表与聚合视图：`crates/sdkwork-api-interface-portal/tests/portal_commerce/order_views.rs`
- 账户历史与对账修复：`crates/sdkwork-api-interface-portal/tests/portal_commerce/account_reconciliation.rs`
- 目录与报价：`crates/sdkwork-api-interface-portal/tests/portal_commerce/catalog_quote.rs`
- 充值与订单结账流程：`crates/sdkwork-api-interface-portal/tests/portal_commerce/order_checkout.rs`
- 订阅结账与会员激活：`crates/sdkwork-api-interface-portal/tests/portal_commerce/subscription_checkout.rs`
- 支付事件安全约束：`crates/sdkwork-api-interface-portal/tests/portal_commerce/payment_events.rs`

## 模块职责

- `support.rs`
  - Portal 登录与 workspace 上下文
  - 充值容量初始化
  - 营销券目录种子数据
  - 下单与支付事件辅助函数
- `order_views.rs`
  - 订单列表、checkout-session、order-center 聚合视图
  - workspace account reconciliation backlog 展示
- `account_reconciliation.rs`
  - recharge/refund 后 canonical account 历史同步
  - ledger 写失败后的支付事件重放修复
- `catalog_quote.rs`
  - commerce catalog 展示
  - 营销券过期回收
  - recharge/coupon/custom recharge 报价
- `order_checkout.rs`
  - 充值订单创建、结账、取消
  - coupon redemption 与 paid checkout 队列流转
- `subscription_checkout.rs`
  - subscription 结账前后会员激活
  - failed checkout 与无效恢复保护
  - settlement 事件激活 membership 与 quota
- `payment_events.rs`
  - refund 幂等
  - refund 安全校验
  - provider 名称校验
  - provider mismatch 与 provider_event_id 约束

## 关键收益

- Portal 交易闭环测试从超大文件拆成按业务能力定位的模块
- 支付事件安全、对账修复、目录报价、订单聚合视图互相解耦
- 后续继续扩展真实支付渠道、退款规则、对账收口时不再需要在单文件内堆叠测试

## 当前状态

- 已完成代码拆分
- 暂未执行测试，保留到最后统一回归
