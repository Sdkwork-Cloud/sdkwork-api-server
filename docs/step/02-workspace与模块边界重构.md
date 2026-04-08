# Step 02 - workspace 与模块边界重构

## 1. 目标与范围

本 step 的目标是把当前已经较为丰富的 Rust workspace、服务入口、前端应用与宿主脚本，收敛成真正可持续维护的高内聚、低耦合结构，为后续协议、路由、控制平面与交付治理提供稳定落点。

### 1.1 执行输入

- step 01 的基线审计、差距矩阵和高风险清单
- `docs/架构/03-模块规划与边界.md`
- `docs/架构/04-技术选型与可插拔策略.md`
- 当前 `Cargo.toml`、`crates/`、`services/`、`apps/`

### 1.2 本步非目标

- 不直接冻结对外协议字段
- 不直接实现路由策略或 Provider 接入新能力
- 不为了“拆分而拆分”制造过度抽象

### 1.3 最小输出

- workspace 依赖方向收敛
- crate 主责边界矩阵
- 服务组合边界清晰
- 前后端契约边界清晰
- 高风险大文件拆分计划与首批落地

## 2. 架构对齐

本 step 直接对齐：

- `docs/架构/02-架构标准与总体设计.md`
- `docs/架构/03-模块规划与边界.md`
- `docs/架构/04-技术选型与可插拔策略.md`
- `docs/架构/133-控制平面与运营后台设计-2026-04-07.md`

## 3. 当前现状

当前仓库已经具备较好的层次化倾向，例如：

- `sdkwork-api-app-*`
- `sdkwork-api-domain-*`
- `sdkwork-api-storage-*`
- `sdkwork-api-provider-*`
- `sdkwork-api-extension-*`
- `sdkwork-api-interface-*`

但仍需继续收敛的典型问题包括：

- 部分能力在服务入口、接口层和应用层之间边界不够严格
- 大文件拆分仍处于专项推进状态，尚未完全系统化
- 前端工作区、后端接口与共享契约的边界仍需继续硬化
- 某些运行时能力仍有跨层直接依赖风险

## 4. 设计

### 4.1 分层标准

统一采用以下分层：

- `kernel / config / observability / cache / secret`：基础设施与横切能力
- `contract / openapi / interface-*`：协议与接口层
- `domain-*`：核心业务规则与模型
- `app-*`：用例编排与流程收口
- `storage-* / provider-* / extension-*`：外部适配与执行层
- `services/*`：进程级组合与启动

### 4.2 依赖规则

强制依赖方向：

- `service -> interface -> app -> domain`
- `app -> storage/provider/extension` 通过抽象边界访问
- `domain` 不直接依赖 `service`
- `interface-admin`、`interface-portal`、`interface-http` 不应各自发明第二套业务规则

### 4.3 服务组合规则

服务只负责：

- 组装依赖
- 暴露进程入口
- 注入配置、日志、观测与运行时

服务不应成为：

- 业务规则主实现层
- 数据模型真值层
- Provider 细节堆叠层

### 4.4 前端边界规则

对 `apps/sdkwork-router-admin` 与 `apps/sdkwork-router-portal` 必须明确：

- 业务工作区边界
- 与后端接口的契约边界
- 共享组件与页面逻辑的边界
- 不允许前端绕过控制平面契约直接发明业务字段

## 5. 实施落地规划

### 5.1 workspace 依赖治理

重点动作：

- 收敛根 `Cargo.toml` 与 workspace member 组织方式
- 建立 crate 责任矩阵
- 标记禁止新增的跨层依赖

### 5.2 大文件与高风险模块拆分

基于 step 01 的高风险清单，优先处理：

- 路由相关超大文件
- 资源路由相关超大文件
- extension / provider / health 等已识别高风险文件

拆分原则：

- 按职责拆，不按行数机械拆
- 优先拆路由、响应、选择、适配、辅助工具
- 每次拆分都保留测试与兼容入口

### 5.3 服务入口收敛

明确以下服务的装配职责：

- `services/gateway-service`
- `services/admin-api-service`
- `services/portal-api-service`
- `services/router-web-service`
- `services/router-product-service`

### 5.4 前后端与接口边界收敛

重点收敛：

- `crates/sdkwork-api-interface-admin`
- `crates/sdkwork-api-interface-portal`
- `crates/sdkwork-api-interface-http`
- `apps/sdkwork-router-admin`
- `apps/sdkwork-router-portal`

### 5.5 结构治理自动化

补充或强化以下自动化检查：

- 文件长度与模块大小检查
- workspace 依赖层级检查
- 架构边界 lint 或 review 规则

## 6. 测试计划

建议至少执行：

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- 关键服务启动 smoke
- Admin / Portal 基础构建与页面 smoke

## 7. 结果验证

本 step 完成时必须满足：

- 后续协议、路由、控制平面改造已有稳定落点
- 依赖方向清晰，不再依赖“约定俗成”
- 高风险大文件已进入可持续拆分轨道

## 8. 检查点

- `CP02-1`：完成 workspace / crate 责任矩阵
- `CP02-2`：完成服务装配边界定义
- `CP02-3`：完成首批高风险大文件拆分或明确拆分清单
- `CP02-4`：完成前后端接口边界收敛

### 8.1 推荐 review 产物

- `docs/review/step-02-crate边界矩阵-YYYY-MM-DD.md`
- `docs/review/step-02-服务装配决议-YYYY-MM-DD.md`
- `docs/review/step-02-高风险文件拆分复盘-YYYY-MM-DD.md`

### 8.2 串行依赖与推荐并行车道

- step 级依赖：必须在 `01` 后、`03` 前完成
- 可并行车道：
  - `02-A`：workspace 与 crate 边界
  - `02-B`：服务入口与装配边界
  - `02-C`：高风险文件拆分
  - `02-D`：Admin / Portal 组件与契约边界
- 收口要求：依赖方向与目录归属必须由单一 owner 最终裁决

### 8.3 架构能力闭环判定

当模块边界已足以支撑协议层、路由层、控制平面与交付层继续演进时，本 step 才算闭环。

### 8.4 快速并行执行建议

- 把“边界矩阵”“大文件拆分”“服务装配”“前端边界”分成四条车道
- 统一在日终由 owner 对依赖方向做一次总审计

### 8.5 完成后必须回写的架构文档

- `docs/架构/03-模块规划与边界.md`
- `docs/架构/04-技术选型与可插拔策略.md`
- 与大文件拆分对应的专项进度文档

### 8.6 本步完成后必须兑现的架构能力

- `docs/架构/03-*` 中关于 `interface / app / domain / storage-provider-runtime` 分层的要求已落成真实依赖方向。
- `docs/架构/03-*` 中服务规划、前端组件化边界和高风险模块整改方向已有明确工程落点。
- 后续协议、路由、控制平面与交付能力都建立在稳定的 workspace 与模块边界之上。

### 8.7 最快完整执行建议

1. 先由 `02-Owner` 冻结 crate 责任矩阵和禁止跨层依赖清单。
2. `02-A/B/C/D` 按 workspace、服务装配、大文件拆分、前端边界四条线并行推进。
3. 验证车道重点检查依赖方向、文件风险和前后端契约越界。
4. 只有当高风险落点明确后，`03` 才允许冻结协议契约。

## 9. 风险与回滚

### 9.1 风险

- 若只拆文件、不收敛依赖，会形成更碎但不更清晰的代码
- 若前端与后端契约边界不清，后续控制平面改造会反复返工

### 9.2 回滚

- 新边界先引入 facade，再切换调用
- 高风险拆分优先保留旧入口兼容层

## 10. 完成定义

满足以下条件视为完成：

- workspace 分层与服务装配边界已冻结
- 高风险模块拆分进入受控状态
- 前后端契约边界清晰可执行

## 11. 下一步准入条件

进入 step 03 前必须确认：

- 协议契约将落到明确的 contract / interface 边界上
- 不再需要通过服务大文件直接定义协议语义
