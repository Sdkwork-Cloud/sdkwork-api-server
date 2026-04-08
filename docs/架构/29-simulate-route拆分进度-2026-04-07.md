# simulate_route 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-app-routing/tests/simulate_route.rs` 原文件超过 2700 行，长期混合了基础路由选择、geo affinity、SLO aware、project/group routing context、provider health snapshot、recovery probe、runtime-backed provider 健康降级等多类职责，已经不适合继续维护。

## 本次拆分目标

- 保留顶层薄入口
- 目录模块统一承载 imports
- 按路由决策语义进行职责拆分
- 避免 support 与具体测试交叉污染

## 拆分结果

顶层入口：

- `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

目录模块：

- `crates/sdkwork-api-app-routing/tests/simulate_route/mod.rs`

子模块：

- `crates/sdkwork-api-app-routing/tests/simulate_route/basic_selection.rs`
- `crates/sdkwork-api-app-routing/tests/simulate_route/geo_slo_context.rs`
- `crates/sdkwork-api-app-routing/tests/simulate_route/provider_health.rs`
- `crates/sdkwork-api-app-routing/tests/simulate_route/runtime_backed.rs`
- `crates/sdkwork-api-app-routing/tests/simulate_route/support.rs`

## 职责边界

### basic_selection

负责基础路由决策能力：

- catalog model candidate 选择
- policy provider order
- 无 catalog 时按 policy 命中
- explicit provider order
- disabled provider demotion
- weighted random seeded 选择

### geo_slo_context

负责上下文与高级策略：

- geo affinity 命中与降级
- SLO aware 约束
- routing decision log
- requested region 落库
- project routing preferences
- api key group routing profile 覆盖

### provider_health

负责 provider health snapshot 与恢复探测：

- persisted snapshot 回退
- stale snapshot 失效
- TTL 配置覆盖
- recovery probe cohort
- recovery probe lease

### runtime_backed

负责 runtime-backed provider 的真实健康降级验证：

- connector runtime 健康检查
- native dynamic runtime 对照
- unhealthy runtime provider 降级

### support

负责共享支撑能力：

- env var scoped guard
- in-memory sqlite seed helper
- provider 插入与时间工具
- Windows 下的 connector/native fixture helper

## 当前收益

- `simulate_route.rs` 已降为 2 行薄入口
- 决策策略、上下文编译、健康快照、runtime-backed 健康验证边界清晰
- 后续新增 routing 策略测试时，不再需要把所有场景继续叠加到单文件

## 结果

本轮拆分后，`crates` 目录中超过 1000 行的真实代码/测试文件已清零。
