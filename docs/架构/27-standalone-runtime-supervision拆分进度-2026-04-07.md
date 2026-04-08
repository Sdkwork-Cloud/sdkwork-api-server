# standalone_runtime_supervision 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs` 原文件超过 1300 行，混合了 standalone listener 重绑、配置热重载、pricing lifecycle 后台推进、secret manager 重载、cluster rollout 等多类测试职责，不利于维护与定位问题。

## 本次拆分目标

- 保留顶层测试入口文件，改为薄入口
- 将 imports 与共享 support 保持在目录模块内部
- 按业务语义而不是按行数机械切块
- 控制单文件大小，降低维护成本

## 拆分结果

顶层入口：

- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

目录模块：

- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision/mod.rs`

子模块：

- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision/listener_reload.rs`
- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision/pricing_and_secrets.rs`
- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision/cluster_rollouts.rs`
- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision/support.rs`

## 职责边界

### listener_reload

负责 standalone runtime 的单机重载能力：

- listener host rebind
- ephemeral bind 真实地址校验
- extension runtime reload
- listener 重绑与失败重试
- store / jwt reload
- heartbeat 在 store reload 后持续写入

### pricing_and_secrets

负责后台管理侧周期任务与密钥管理：

- pricing lifecycle 后台自动激活 due planned version
- secret manager 热重载与旧密钥兼容校验

### cluster_rollouts

负责集群级 rollout 与共享协调：

- extension runtime rollout
- standalone config rollout
- cache restart required 场景
- 安全姿态校验失败
- rollout timeout 场景

### support

负责共享支撑能力：

- config 写入 helper
- sqlite 路径处理
- lifecycle / health / rollout 等待器
- native dynamic fixture manifest 与路径解析
- 临时目录和环境守卫

## 当前收益

- `standalone_runtime_supervision.rs` 已降为 2 行薄入口
- 原先混杂的单机重载、后台任务、集群 rollout 逻辑已明确解耦
- `support.rs` 统一承载 helper，避免子模块重复实现
- 后续扩展新的 supervision/rollout 测试时，不需要继续向单一巨型文件追加

## 本轮拆分后剩余 >1000 行真实文件

- `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

说明：

- `docs/plans/...` 与 `docs/superpowers/specs/...` 仍有超过 1000 行的文档文件，但当前优先级仍放在真实业务代码与测试模块的结构治理上。
