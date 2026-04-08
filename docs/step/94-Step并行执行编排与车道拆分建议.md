# 94 - Step 并行执行编排与车道拆分建议

## 1. 文档定位

本文件用于回答一个核心执行问题：

`哪些 step 必须串行推进，哪些能力可以在同一波次内按多子 agent / 多人车道并行推进，同时不破坏架构边界和集成稳定性。`

## 2. 总体结论

最快且风险可控的执行方式不是“全部并行”，而是：

`串行主脊柱 + 波次内并行车道 + 独立验证车道 + 统一集成 owner 收口`

## 3. 必须串行的 Step

以下 step 属于主脊柱，必须保持 step 级串行：

- `00`：先冻结门禁
- `01`：先冻结事实和差距
- `02`：先收敛 workspace 与模块边界
- `03`：先冻结 Gateway 契约
- `04`：先统一路由主时序
- `13`：最终放行必须最后收口

这些 step 内部可以拆车道，但 step 级准入不能跳过。

## 4. 适合并行的波次与车道

### 4.1 波次 A：`00-03`

可并行车道：

- `A1`：文档与审计车道
- `A2`：workspace / crate / service 边界车道
- `A3`：协议兼容与契约车道
- `A4`：验证与 review 车道

### 4.2 波次 B：`04-06`

可并行车道：

- `B1`：路由策略与决策日志
- `B2`：stateful 运行态
- `B3`：stateless 运行态
- `B4`：Provider / extension / secret 治理
- `B5`：控制平面后端
- `B6`：Admin / Portal 前端

### 4.3 波次 C：`07-10`

可并行车道：

- `C1`：资源与任务治理
- `C2`：数据与缓存治理
- `C3`：安全与多租户治理
- `C4`：可观测性与运维证据

### 4.4 波次 D：`11-13`

可并行车道：

- `D1`：安装部署、发布资产与 Release 依赖真值
- `D2`：升级回滚与私有化交付
- `D3`：压测、HA、DR 演练
- `D4`：CLI / 样例 / 客户端验证
- `D5`：评分卡、release notes 与最终放行

## 5. 并行拆分原则

- 按主写入范围拆，不按“概念兴趣”拆
- 高风险边界文件只能有一个最终 owner
- 验证车道尽量不主写业务逻辑
- 总 owner 只做裁决与集成，不和所有车道抢写同一文件
- 文档回写车道不能省略；并行完成不等于架构兑现

## 6. 推荐角色

- 总集成 owner
- 协议 owner
- 路由 owner
- Provider / extension owner
- 控制平面 owner
- 数据 / 安全 owner
- 交付 / 性能 owner
- 验证 owner

## 6.1 波次级车道主写入范围

为了保证多子 agent 并行时不互相踩文件，推荐按以下主写入范围拆分：

### 波次 A

- `A1-审计车道`：`docs/step/`、`docs/review/`、`docs/架构/`
- `A2-结构车道`：`Cargo.toml`、`crates/`、`services/`、`apps/`
- `A3-协议车道`：`crates/sdkwork-api-contract-*`、`crates/sdkwork-api-interface-http`、`services/gateway-service`

### 波次 B

- `B1-路由车道`：`sdkwork-api-app-routing`、`sdkwork-api-domain-routing`、`sdkwork-api-policy-routing`
- `B2-stateful车道`：`sdkwork-api-app-runtime`、`services/router-product-service`
- `B3-stateless车道`：`sdkwork-api-app-gateway`、`services/gateway-service`
- `B4-provider车道`：`provider-*`、`extension-*`、`secret-*`
- `B5-control-plane车道`：`interface-admin`、`interface-portal`、`admin-api-service`、`portal-api-service`
- `B6-frontend车道`：`apps/sdkwork-router-admin`、`apps/sdkwork-router-portal`

### 波次 C

- `C1-resource-job车道`：`app-jobs`、`domain-jobs`、`interface-http`
- `C2-data车道`：`storage-*`、`cache-*`、`config`
- `C3-security车道`：`identity/tenant/credential`、`secret-*`、`interface-*`
- `C4-observability车道`：`observability`、`router-ops.mjs`、运维证据文档

### 波次 D

- `D1-delivery车道`：`bin/`、`scripts/dev/`、发布资产、sibling SDK 一致性校验、release notes 基线
- `D2-performance车道`：benchmark、压测、HA、DR 演练资产
- `D3-client车道`：样例、兼容客户端、CLI、接入文档
- `D4-release车道`：评分卡、发布说明、最终放行材料

## 7. 多子 agent 可并行执行的内容

下列工作天然适合多子 agent 并行：

- 基线盘点与差距整理
- 大文件拆分的不同模块
- 各协议家族的兼容测试资产
- stateful 与 stateless 装配分线
- Admin / Portal 前后端不同工作区
- 资源治理与长任务治理不同子域
- 存储、缓存、配置 rollout 分线
- 安全、审计、观测、交付文档与演练资产

## 8. 必须由单一 owner 串行裁决的内容

- 事实与目标边界
- crate 依赖方向
- Gateway 权威字段和错误语义
- 统一路由主时序
- 权限模型与安全硬标准
- 发布放行与最终评分卡

## 9. 极致执行建议

如果目标是“最快速且完整地执行完全部 step”，推荐采用以下节奏：

### 9.1 阶段打法

1. `A 波次` 采用“强串行 + 轻并行”，先连续完成 `00-03`，不允许跳步。
2. `B 波次` 采用“主脊柱 + 双态 + 控制平面”并行，把 `04` 作为冻结点，把 `05-06` 作为并行落地区。
3. `C 波次` 采用“对象治理 + 安全观测”并行，让 `07-10` 分别围绕资源、数据、安全、证据四条线推进。
4. `D 波次` 采用“交付、演练、最终放行”并行，把 `11-13` 压缩到最少集成轮次内完成。

### 9.2 每个 Step 的固定节奏

建议每个 step 都按以下四段执行：

1. `冻结段`：owner 用半天以内冻结输入、非目标、主写入范围、阻塞等级。
2. `并行段`：各车道按写入范围推进实现，不跨界扩散。
3. `验证段`：验证车道独立执行测试、演练、证据审计和 `91` 评分。
4. `收口段`：owner 统一合并、回写架构、给出下一步准入结论。

### 9.3 最快团队编排

若希望在最短周期内完成全部 step，推荐最少配置：

- `Owner / Architect`
- `Protocol & Routing Lane`
- `Provider & Control Plane Lane`
- `Data / Security / Observability Lane`
- `Delivery / Performance / Release Lane`
- `Verifier Lane`

这样可以保证：

- `04` 之后的主能力基本都有人负责
- 验证工作不被实现工作吞掉
- 最终放行结论由独立证据驱动，而不是由实现方自证

### 9.3.1 默认车道模板

若没有特殊组织限制，推荐每个波次默认采用以下车道模板：

- `Owner`
- `实现车道-1`
- `实现车道-2`
- `实现车道-3`
- `Verifier`
- `Doc-Lane`

波次 D 额外增加：

- `Release-Truth Lane`

### 9.4 并行执行的日内节奏

推荐每个工作日固定采用以下节奏：

1. 上午先由 owner 更新执行卡与阻塞状态。
2. 中午前各实现车道同步主写入范围是否有变更。
3. 下午由验证车道开始接收已完成子包并独立验证。
4. 收工前 owner 只做合并决策，不做大规模新实现。

## 10. 结论

真正高效的并行执行，不是把系统拆散，而是在不打破主脊柱的前提下，把不同写入范围和验证工作独立出来并行推进。
