# marketing_checkout_closure 拆分进度

日期：2026-04-07

## 本次目标

将 `crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure.rs` 从单体交易闭环测试文件拆分为高内聚模块，按报价、支付方式校验、优惠券生命周期、结算补偿等边界分层，提升交易核心测试的可维护性。

## 拆分结果

已完成以下结构调整：

- 顶层薄入口：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure.rs`
- 目录模块入口：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure/mod.rs`
- 共享支持：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure/support.rs`
- 报价预览：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure/quote_preview.rs`
- 优惠券生命周期：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure/coupon_lifecycle.rs`
- 支付方式与 provider 校验：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure/checkout_methods.rs`
- 结算与退款补偿：`crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure/settlement_compensation.rs`

## 模块职责

- `support.rs`
  - SQLite store 构造
  - 营销优惠券模板、活动、预算、券码种子数据
- `quote_preview.rs`
  - 报价预览与优惠券适用性校验
- `coupon_lifecycle.rs`
  - 预留、核销、失败释放、过期回收等优惠券生命周期闭环
- `checkout_methods.rs`
  - 结算会话支付方式展示
  - provider alias 归一化
  - webhook provider_event_id 约束
  - settlement/refund provider 一致性校验
- `settlement_compensation.rs`
  - 订单结算写失败补偿
  - 退款写失败补偿
  - 优惠券回滚失败补偿与重放恢复

## 关键收益

- 交易闭环测试从单个超大文件拆成可按业务能力定位的小模块
- 支付方式与 provider 校验逻辑从营销券生命周期中剥离，职责更清晰
- 退款补偿与订单主流程拆开后，更适合继续扩展支付渠道、退款规则和财务收口场景

## 当前状态

- 已完成代码拆分
- 暂未执行测试，保留到最后统一回归
