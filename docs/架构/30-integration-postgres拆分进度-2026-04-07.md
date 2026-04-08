# integration_postgres 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs` 原文件超过 1200 行，混合了 catalog、routing、pricing、account kernel、usage、provider health、quota、billing event、query plan、transaction round trip 等多类持久化验证，不利于后续扩展。

## 本次拆分目标

- 保留顶层薄入口
- 目录模块统一管理 imports
- 只将真正共享的 schema helper 下沉到 `support.rs`
- 按存储边界和读写主题拆分测试

## 拆分结果

顶层入口：

- `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

目录模块：

- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/mod.rs`

子模块：

- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/catalog_routing.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/schema_and_accounts.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/routing_usage.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/query_and_transactions.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/support.rs`

## 职责边界

### catalog_routing

负责 catalog、credential、routing policy 的基础落库能力：

- channel/provider/model/credential
- routing policy 基础持久化

### schema_and_accounts

负责 schema 与 billing/account kernel：

- canonical account kernel tables
- canonical identity kernel tables
- pricing plan / rate
- commercial account read models
- remaining account kernel records

### routing_usage

负责 routing / usage / finance 读写：

- SLO policy 字段
- routing decision log
- requested region
- provider health snapshot
- quota policy
- billing event

### query_and_transactions

负责查询性能相关验证与事务 round trip：

- latest project routing log / usage lookup
- any model / providers / bindings 查询
- account kernel transaction executor

### support

负责共享 helper：

- `assert_pg_column`

## 当前收益

- `integration_postgres.rs` 已降为 2 行薄入口
- Postgres integration test 具备更清晰的存储边界
- schema helper 与业务测试解耦
- 后续新增 Postgres read/write 场景时可以直接落到对应子模块

## 结果

本轮拆分后，`crates` 目录中超过 1000 行的真实代码/测试文件已清零，仅剩文档类大文件。
