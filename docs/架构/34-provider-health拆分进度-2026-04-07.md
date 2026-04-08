# provider_health 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-app-routing/tests/simulate_route/provider_health.rs` 在第一次目录化之后仍然超过 1000 行，内部长期混合了：

- persisted snapshot / stale snapshot
- provider health TTL
- recovery probe cohort
- recovery probe lease

这些场景都属于 provider health，但测试关注点不同，继续放在一个文件里会让维护成本继续上升。

## 本次拆分目标

- 保留 `simulate_route/mod.rs` 对 `provider_health` 的现有挂载方式
- 在 `provider_health.rs` 内部继续按子主题拆分
- 让文件名直接表达测试目的

## 拆分结果

入口文件：

- `crates/sdkwork-api-app-routing/tests/simulate_route/provider_health.rs`

子模块：

- `crates/sdkwork-api-app-routing/tests/simulate_route/provider_health/snapshot_ttl.rs`
- `crates/sdkwork-api-app-routing/tests/simulate_route/provider_health/recovery_probe.rs`
- `crates/sdkwork-api-app-routing/tests/simulate_route/provider_health/recovery_probe_lease.rs`

## 职责边界

### snapshot_ttl

负责 snapshot freshness 与 TTL：

- persisted snapshot fallback
- stale snapshot ignored
- configured TTL keep fresh
- configured TTL expire

### recovery_probe

负责 recovery probe cohort 逻辑：

- default probe selection
- probe cohort enabled
- outside cohort keeps backup

### recovery_probe_lease

负责 recovery probe lock / lease 语义：

- lease available 时选择 stale primary
- lease held 时保留 backup

## 当前收益

- `provider_health.rs` 已降为薄聚合入口
- provider health 测试结构与实际产品语义对齐
- 子文件规模控制在 279 到 426 行

## 当前剩余超限文件

- `crates/sdkwork-api-provider-openai/tests/http_execution/evals_batches_vector_stores.rs`
- `crates/sdkwork-api-interface-portal/tests/marketing_coupon_routes.rs`
