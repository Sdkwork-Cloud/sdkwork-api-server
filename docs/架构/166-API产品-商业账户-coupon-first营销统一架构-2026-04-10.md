# API 产品 / 商业账户 / Coupon-First 营销统一架构

> 日期：2026-04-10
> 状态：active
> 范围：pricing、billing/account、commerce、marketing、admin app、portal app、public API

## 1. 文档目标

本文档用于定义 `sdkwork-api-router` 商业化主线的规范化架构，回答五个问题：

1. `coupon` 应该如何并入 `market`，同时保持一等业务语义，而不是被抽象成难维护的 generic promotion。
2. `API Product`、`Commercial Account`、`Coupon-First Marketing` 三层应该如何分责、如何建模、如何协作。
3. 当前实现哪些地方仍停留在兼容态，继续叠加能力会导致治理失控。
4. Admin、Portal 与未来 API 产品体系应该如何升级为商业化专业版本。
5. legacy coupon 体系应如何退场，避免长期双模型并存。

本文档是 coupon / marketing / commercial account / API product 方向的当前主参考，并补足 [133-控制平面与运营后台设计-2026-04-07](./133-控制平面与运营后台设计-2026-04-07.md) 中营销控制面的目标态定义缺口。

## 2. 当前业务阶段判断

### 2.1 这不是传统零售电商系统

当前仓库已经具备典型 API 商业化平台特征，而不是普通购物车电商：

- Portal 商业化对象已经包含 `subscription_plan`、`recharge_pack`、`custom_recharge`、`coupon_redemption`。
- 账务侧已经存在 `AccountBenefitLotRecord` 与 `AccountBenefitSourceType::Coupon`，说明 coupon 已不仅是下单折扣，还会进入账户权益账本。
- 营销侧已经存在 `PercentageOff`、`FixedAmountOff`、`GrantUnits` 三类 benefit。
- Settlement、ledger、hold、benefit lot、pricing lifecycle 已经进入真实业务边界。

结论：

- 当前主线应定义为“API 商业化平台”，而不是“电商活动插件”。
- `coupon` 必须同时支持“订单优惠”与“账户权益注入”两种效果。
- `market` 应作为商业化运营产品套件，而 `coupon` 必须作为其中的一等语义子域保留。

### 2.2 当前代码已经暴露出升级方向

当前实现并非完全空白，已经出现明确的演进信号：

- `sdkwork-api-domain-marketing` 已经包含 `CouponTemplateRecord`、`CouponCodeRecord`、`CouponReservationStatus`、`CouponRedemptionStatus`、`CouponRollbackStatus` 等规范化对象。
- legacy `sdkwork-api-domain-coupon` / `sdkwork-api-app-coupon` 已移除，coupon 语义真值只保留在 `sdkwork-api-domain-marketing` / `sdkwork-api-app-marketing`。
- Pricing 控制面已经形成 `clone / publish / schedule / retire` 的专业生命周期；marketing 已补齐 `coupon template / campaign` 的 `publish / schedule / retire`，但 `budget / code` 仍主要停留在 `status toggle`。

这说明真正的问题不是“要不要做 marketing/coupon”，而是“必须把已经长出来的商业化能力收敛成单一规范模型”。

## 3. 行业基线与应吸收的标准

### 3.1 API 产品 / 商业账户基线

API 商业化平台通常把“产品定义”“订阅/账户”“价格与访问控制”拆开建模，参考：

- [Azure API Management products](https://learn.microsoft.com/en-us/azure/api-management/api-management-howto-add-products)
- [Azure API Management subscriptions](https://learn.microsoft.com/en-us/azure/api-management/api-management-subscriptions)
- [Apigee API products](https://cloud.google.com/apigee/docs/api-platform/publish/what-api-product)
- [Stripe products and prices](https://docs.stripe.com/products-prices/how-products-and-prices-work)

抽象出的共同标准：

- `Product` 描述可售卖能力，不直接等于最终成交记录。
- `Price / Plan / Offer` 是可发布、可切换、可灰度的商业政策，不应混进账户状态。
- `Subscription / Account / Entitlement` 是客户已拥有的东西，不应与营销活动配置混在一起。

### 3.2 Coupon / 营销基线

成熟营销系统不会把 coupon 简化成一个 code 字符串，而是拆成定义、活动、分发、校验、核销、回滚等多个业务对象，参考：

- [Shopify discount types](https://help.shopify.com/en/manual/discounts/discount-types)
- [Shopify discount combinations](https://help.shopify.com/en/manual/discounts/discount-combinations)
- [commercetools cart discounts](https://docs.commercetools.com/merchant-center/cart-discounts)
- [commercetools discount codes](https://docs.commercetools.com/merchant-center/discount-codes)
- [Talon.One campaigns overview](https://docs.talon.one/docs/product/campaigns/overview)
- [Talon.One revising campaigns](https://docs.talon.one/docs/product/campaigns/revising-campaigns)
- [Voucherify validation rules and campaign limits](https://support.voucherify.io/article/529-validation-rules-campaign-limits)
- [Voucherify stacking rules](https://support.voucherify.io/article/604-stacking-rules)
- [Voucherify campaign approval process](https://support.voucherify.io/article/118-campaign-approval-process)
- [Voucherify distribution manager](https://support.voucherify.io/article/19-how-does-the-distribution-manager-work)

抽象出的共同标准：

1. `coupon` 是显式语义，不被 `promotion template` 抹平。
2. `campaign` 与 `coupon definition` 分离，活动负责投放与运营，券定义负责规则与权益。
3. `validation -> reservation -> redemption -> rollback` 是完整事务链，而不是一次性“核销”按钮。
4. stacking、budget、distribution、approval、revision 是营销系统的标配，不是“以后再说”的高级功能。
5. 自助门户与运营后台分离，但共享同一套规范化事实源。

## 4. 当前实现复核与问题清单

### 4.1 P0：双 coupon 模型并存问题已关闭

当前 coupon 只保留一套 canonical 模型：

- canonical：`crates/sdkwork-api-domain-marketing/src/lib.rs` 中的 coupon template / campaign / budget / code / reservation / redemption / rollback

以下 legacy 载体已完成移除：

- `crates/sdkwork-api-domain-coupon`
- `crates/sdkwork-api-app-coupon`
- `project_legacy_coupon_campaign(...)`

当前规则：

- `sdkwork-api-domain-marketing` 是 coupon 主模型的唯一演进方向。
- Admin / Portal / Commerce 不允许恢复 legacy coupon fallback、兼容投影真值、或 `/admin/coupons` 控制面。

### 4.2 P0：coupon 语义仍然不够规范化

当前 marketing 模型已经比 legacy 强很多，但仍有三处不够专业：

- `CouponTemplateRecord` 仍带有“模板导向”命名，缺少 `CouponDefinition` 语义。
- `eligible_target_kinds: Vec<String>` 仍偏向自由文本，难以支撑产品绑定、offer 绑定、能力绑定、价格计划绑定。
- `claimed_subject_scope` 只停留在 code 记录层，尚未提升为规范化的“issued coupon / owned coupon”语义。

这会导致两个典型问题：

- 运营同学只能看到“模板/码/活动”，看不到“券定义 / 分发 / 持有 / 核销 / 回滚”的清晰业务线。
- 后续接入 API product、commercial account、partner distribution 时，字段会继续失控。

### 4.3 P0：控制面生命周期远弱于 pricing

当前 marketing admin handler 主要是：

- `create_marketing_coupon_template_handler`
- `update_marketing_coupon_template_status_handler`
- `publish_marketing_coupon_template_handler`
- `schedule_marketing_coupon_template_handler`
- `retire_marketing_coupon_template_handler`
- `create_marketing_campaign_handler`
- `update_marketing_campaign_status_handler`
- `publish_marketing_campaign_handler`
- `schedule_marketing_campaign_handler`
- `retire_marketing_campaign_handler`
- `create_marketing_budget_handler`
- `update_marketing_budget_status_handler`
- `create_marketing_coupon_code_handler`
- `update_marketing_coupon_code_status_handler`

而 pricing 已经具备：

- `clone_canonical_pricing_plan_handler`
- `publish_canonical_pricing_plan_handler`
- `schedule_canonical_pricing_plan_handler`
- `retire_canonical_pricing_plan_handler`

当前差距已经从“完全没有语义生命周期”收敛为“生命周期不完整”：

- 仍缺 revision / clone / compare / approval。
- template 已具备 `activation_at_ms` 与 `schedule / publish / retire`，campaign 已具备 `start_at_ms` 驱动的 `schedule / publish / retire`，且 template / campaign 生命周期审计已持久化并可查询；但 budget / code 仍缺同级治理与审计能力。
- 仍缺 campaign 版本冻结与运行期引用。
- template / campaign 生命周期审计已沉淀为专门审计面，至少覆盖 `operator_id / request_id / reason / decision_reasons / requested_at_ms`；下一缺口集中在 `budget / code` 与 `revision / approval / compare / clone`。

如果继续沿用“创建后改状态”的工作方式，营销能力一旦进入大规模投放，必然失控。

### 4.4 P1：Portal 的商业语义混合了产品维度与交易维度

`PortalCommerceQuoteKind` 当前同时包含：

- `subscription_plan`
- `recharge_pack`
- `custom_recharge`
- `coupon_redemption`

前三个是产品/报价对象，最后一个更接近交易或结算动作，而不是产品种类。

这意味着当前模型把“卖什么”和“发生了什么交易”混在了一起。长期看应做规范化拆分：

- `ApiProductKind`：定义产品种类
- `CommercialTransactionKind`：定义订单、充值、券核销、调整等交易类型
- `QuoteKind`：仅保留报价行为视角

兼容期可以保留 `coupon_redemption` 作为 quote 流程入口，但不得继续把它当成产品目录的 canonical 分类。

### 4.5 P1：管理系统 legacy 兼容心智已清理

Admin 主工作流已经移除 “Legacy coupon compatibility” 与旧 coupon CRUD 入口，这意味着：

- 管理系统不再把旧体系视为长期并存对象。
- 运营工作台只围绕 template / campaign / budget / code / reservation / redemption / rollback 工作。
- coupon 语义不再被 legacy 心智污染。

后续约束不是“把 legacy 信息展示得更漂亮”，而是：

- 让运营工作台持续只围绕规范化对象工作。
- 兼容投影只作为后台迁移支撑，不进入主工作流。

### 4.6 P1：Portal 自助能力还不是“账户 + 营销”一体化体验

当前 Portal 已有：

- `marketing/coupon-validations`
- `marketing/coupon-reservations`
- `marketing/coupon-redemptions/confirm`
- `marketing/coupon-redemptions/rollback`
- `marketing/my-coupons`
- `billing/account/benefit-lots`

但视图仍偏“我的券”与“我的账本”分裂：

- 用户很难理解一张 coupon 是拿来抵扣订单，还是拿来注入权益。
- account benefit lot 与 coupon redemption 的关系尚未成为显式产品语义。
- 自助领取、持有、适用产品、核销结果、到账权益没有形成完整闭环。

## 5. 目标业务语义与分层

### 5.1 `Market` 的正确定位

面向产品与运营的上层产品套件应统一命名为 `Market`，但领域建模不能被这个上层命名抹平。

目标结构：

```text
Market Suite
|- API Product
|- Commercial Account
`- Coupon-First Marketing
```

定义如下：

- `Market`：运营产品套件名称，面向 Admin / Portal / API Product 管理。
- `Marketing`：商业化中的增长与补贴子域，负责投放、券、活动、预算、分发、核销、回滚。
- `Coupon`：`Marketing` 内的一等业务语义，不下沉为 generic code/template。

### 5.2 三层 canonical 架构

### A. API Product 层

负责回答“卖什么”：

- `ApiProduct`
- `ProductOffer`
- `PricingPlan`
- `PricingRate`
- `CatalogPublication`

职责：

- 定义产品与能力包
- 定义可售卖 offer
- 绑定价格与计费方式
- 发布到门户、自助 API、合作伙伴渠道

不负责：

- 账户余额
- 已持有权益
- coupon 持有与核销事务

### B. Commercial Account 层

负责回答“客户已经拥有什么、还能消费什么”：

- `CommercialAccount`
- `Subscription`
- `BalanceWallet`
- `AccountBenefitLot`
- `AccountHold`
- `SettlementRequest`

职责：

- 持有现金余额、订阅、权益包、冻结与清结算
- 表达账户可用权益
- 接收来自 purchase、coupon、grant 的权益注入

不负责：

- 定义营销活动
- 决定券适用规则

### C. Coupon-First Marketing 层

负责回答“如何发券、如何领券、如何用券、如何回滚”：

- `CouponDefinition`
- `CouponCampaign`
- `CouponBudget`
- `CouponBatch`
- `CouponPublication`
- `IssuedCoupon`
- `CouponCode`
- `CouponValidationSession`
- `CouponReservation`
- `CouponRedemption`
- `CouponRollback`

职责：

- 定义 coupon benefit 与 restrictions
- 管理活动、预算、分发、发行与持有
- 驱动 validate / reserve / confirm / rollback 事务
- 将核销结果注入 order discount 或 account benefit lot

### 5.3 coupon 的两种标准结算模式

coupon 在本系统中必须显式支持两种模式：

1. `checkout_discount`
2. `account_entitlement`

说明：

- `checkout_discount`：用于订阅、充值包、自定义充值等交易场景，表现为订单减免或折扣。
- `account_entitlement`：用于直接向商业账户发放额度、赠送 units、充值券、体验包等，表现为 `AccountBenefitLot` 的新增，且 `source_type = coupon`。

必要规则：

- 同一 `CouponDefinition` 必须显式声明支持哪种结算模式，不允许靠文案猜。
- `GrantUnits` 类型 coupon 默认优先对接 `account_entitlement`。
- 折扣型 coupon 不得直接写入账户账本，必须先经过 redemption 结果判定。

### 5.4 规范化术语表

| 术语 | 定义 | 归属层 |
| --- | --- | --- |
| `CouponDefinition` | 券本体规则，包含 benefit、restriction、stacking、适用范围、结算模式 | Marketing |
| `CouponCampaign` | 运营活动，定义投放周期、受众、预算、发布渠道 | Marketing |
| `CouponBudget` | 活动或券池预算约束，控制成本与耗尽策略 | Marketing |
| `CouponBatch` | 一次发行或码段生成批次，负责唯一性、数量与失效窗 | Marketing |
| `CouponPublication` | 某次投放配置，定义投放渠道、入口、文案、分发规则 | Marketing |
| `IssuedCoupon` | 发到具体用户 / 项目 / 工作区 / 账户名下的一张券 | Marketing |
| `CouponCode` | 供输入、分享、分发的可见 token，可为空 | Marketing |
| `CouponValidationSession` | 幂等校验会话，产出可追踪校验结果 | Marketing |
| `CouponReservation` | 订单或核销前的短时占用锁 | Marketing |
| `CouponRedemption` | 实际核销结果，连接订单或账户权益注入 | Marketing |
| `CouponRollback` | 撤销、退款、人工纠偏导致的逆向处理记录 | Marketing |
| `ApiProduct` | 对外售卖产品定义 | API Product |
| `CommercialAccount` | 客户资产承载体 | Commercial Account |
| `AccountBenefitLot` | 可消费权益包/额度包 | Commercial Account |

## 6. 规范化领域模型

### 6.1 `CouponDefinition` 应替代 `CouponTemplate`

建议把当前 `CouponTemplateRecord` 视为过渡命名，目标语义升级为 `CouponDefinition`。核心字段应包括：

- `definition_id`
- `tenant_id`
- `name`
- `benefit_spec`
- `restriction_spec`
- `stacking_policy`
- `settlement_mode`
- `target_bindings`
- `subject_scope`
- `approval_state`
- `revision`

其中 `target_bindings` 不应继续停留在 `eligible_target_kinds: Vec<String>`，而应升级为结构化绑定，例如：

- `product_kind`
- `product_id`
- `offer_id`
- `pricing_plan_id`
- `capability_code`

这一步是 coupon 与 API product 语义打通的关键。

### 6.2 `IssuedCoupon` 必须成为一等对象

当前系统已经有 `my-coupons` 概念，但底层仍偏向 code-centric。目标态中必须显式引入 `IssuedCoupon`：

- 支持 `user / project / workspace / account` 级持有关系
- 区分“已发放但未领取”“已领取未使用”“已占用”“已核销”“已过期”“已回滚”
- 让 Portal 与 Admin 都能围绕“券持有状态”工作，而不是只围绕“码状态”工作

规则：

- `CouponCode` 是分发介质，不是券本体。
- 没有 code 的 auto-claim / direct-grant 也必须能产生 `IssuedCoupon`。

### 6.3 `CouponCampaign` 与 `CouponPublication` 必须分离

活动与投放是两个不同问题：

- `CouponCampaign` 关心受众、周期、目标、预算、渠道策略。
- `CouponPublication` 关心具体入口与分发动作，例如 `portal_claim`、`admin_grant`、`partner_distribution`、`system_reward`。

这样才能支持：

- 同一 campaign 下多渠道投放
- 同一定义下多波次分发
- 运营复盘按 publication 粒度看效果

### 6.4 Validation / Reservation / Redemption / Rollback 必须是完整事务链

标准链路：

1. `validate`
2. `reserve`
3. `confirm redemption`
4. `rollback`（按退款、取消、人工调整触发）

关键原则：

- `validate` 负责可用性判断，不落最终业务结果。
- `reserve` 负责并发保护与短时占用。
- `confirm redemption` 才是最终生效点。
- `rollback` 必须显式记录原因、来源交易、逆向权益影响。

这条链必须同时兼容：

- 订单型抵扣
- 账户权益注入型核销

## 7. API Product 体系定义

### 7.1 规范化对象

未来 API 产品体系建议统一成以下对象：

- `ApiProduct`
- `ProductOffer`
- `PricingPlan`
- `PricingRate`
- `Quote`
- `Order`
- `Settlement`
- `EntitlementGrant`

其中：

- `ApiProduct` 决定卖什么
- `ProductOffer` 决定怎么卖
- `PricingPlan / PricingRate` 决定多少钱、按什么能力计费
- `Quote / Order` 决定一次交易
- `EntitlementGrant` 决定到账什么权益

### 7.2 当前模型需要修正的点

当前 `PortalCommerceQuoteKind` 把产品类型与交易结果混在一起。目标态建议调整为：

- `ApiProductKind = subscription_plan | recharge_pack | custom_recharge`
- `CommercialTransactionKind = product_purchase | coupon_redemption | manual_adjustment | settlement_request`
- `QuoteKind = product_purchase | coupon_redemption`

兼容期保留现有字段可以接受，但文档和新接口设计必须先按新语义收敛。

### 7.3 coupon 与 API product 的连接方式

`coupon` 不直接成为 `ApiProduct`，而是通过以下方式影响 API product 交易：

- 作用于 `ProductOffer` 的价格
- 作用于 `PricingPlan` 的优惠条件
- 作用于 `Order` 的实付金额
- 作用于 `CommercialAccount` 的到账权益

因此 coupon 与 API product 的关系应该是“可作用于产品交易与账户入账”，而不是“coupon 本身等于一种产品”。

## 8. 控制平面与管理系统变更

### 8.1 Admin 目标工作台

Admin 侧建议统一成 `Market` 套件，并拆出以下工作区：

- `API Products`
- `Pricing`
- `Coupons`
- `Campaigns`
- `Publications & Distribution`
- `Budgets`
- `Commercial Accounts`
- `Benefit Lots & Settlements`
- `Approvals & Change Calendar`

每个工作区都应支持专业化操作：

- `create`
- `clone`
- `revise`
- `compare`
- `submit for approval`
- `approve / reject`
- `schedule`
- `publish`
- `pause / resume`
- `retire / archive`

约束：

- 运营主界面不得继续展示 “Legacy coupon compatibility” 作为主工作流一部分。
- 兼容数据如需保留，只能进入后台迁移诊断视图，不进入日常运营页面。

### 8.2 Portal 目标能力

Portal 侧应围绕“产品购买 + 账户权益 + 券自助”构建，而不是把它们拆成互相无关的页面：

- 商品目录与 offer 浏览
- 价格试算与 coupon apply
- 我的 coupons / 已领取 / 可领取
- coupon 使用记录与回滚结果
- account balance / benefit lots / entitlement 明细
- 与 coupon 关联的到账权益说明

Portal 用户看到的不是“模板、批次、码”，而是：

- 这张券是什么
- 适用于哪些产品
- 是否已经归属到我的账户
- 是否已被占用 / 核销 / 回滚
- 用完后给账户带来了什么变化

### 8.3 Public API 目标面

未来外部 API 产品体系至少应定义以下面：

- `GET /market/products`
- `GET /market/offers`
- `POST /market/quotes`
- `POST /marketing/coupons/validate`
- `POST /marketing/coupons/reserve`
- `POST /marketing/coupons/confirm`
- `POST /marketing/coupons/rollback`
- `GET /commercial/account`
- `GET /commercial/account/benefit-lots`

要求：

- 幂等键、主体范围、tenant 边界必须标准化。
- 自助 API 与 Portal 必须共享同一套领域语义，不允许出现第二套“前端专用 coupon 逻辑”。

## 9. 生命周期、审批与变更安全

### 9.1 coupon 定义生命周期

建议标准化为：

`draft -> in_review -> approved -> scheduled -> published -> retired -> archived`

说明：

- `draft` 允许编辑
- `in_review` 冻结待审
- `approved` 可等待发布
- `scheduled` 有明确生效窗口
- `published` 已可投放
- `retired` 停止新增使用
- `archived` 仅保留审计

### 9.2 campaign / publication / budget 生命周期

- `CouponCampaign`：`draft -> approved -> scheduled -> active -> paused -> ended -> archived`
- `CouponPublication`：`draft -> generated -> published -> suspended -> closed`
- `CouponBudget`：`draft -> active -> exhausted -> closed`

关键规则：

- 运行中的 campaign 引用的是已冻结 revision，不引用可变草稿。
- budget exhaustion 应触发 publication 和 validation 的联动约束。
- template / campaign 的 publish / schedule / retire 必须留下持久审计，至少包含 `operator`、`reason`、`request id`、`decision reasons`、`requested_at_ms`，且可通过专门列表 API 查询。
- budget / code 在达到同级治理前，不得继续停留在仅 response 回显的审计形态。

### 9.3 对齐 pricing 的治理标准

pricing 已经证明，专业控制面不能停留在“状态开关”。marketing 也必须复制同等级治理能力：

- `clone`：用于版本分叉
- `revise`：用于草稿修订
- `compare`：用于变更审阅
- `schedule`：用于定时发布
- `publish`：用于显式生效
- `retire`：用于可审计退场

这不是锦上添花，而是商业化上线的基础设施。

## 10. 数据与存储建模原则

### 10.1 单一事实源

必须明确：

- marketing 规范模型是 coupon 领域事实源
- coupon 不再保留 legacy 兼容投影真值；如需保留历史信息，只允许保留文档、审计与迁移记录。
- Admin / Portal / Commerce 读模型可以不同，但不能各自维护一套真值

### 10.2 主数据与交易数据分离

以下对象属于主数据：

- `CouponDefinition`
- `CouponCampaign`
- `CouponBudget`
- `CouponBatch`
- `CouponPublication`

以下对象属于交易数据：

- `IssuedCoupon`
- `CouponValidationSession`
- `CouponReservation`
- `CouponRedemption`
- `CouponRollback`

规则：

- 主数据可 revision 化
- 交易数据必须可追溯且不可被草稿覆盖

### 10.3 幂等、审计与事件

营销体系必须默认具备：

- validate / reserve / confirm / rollback 全链路幂等键
- outbox 事件
- 审计日志
- 可重放的 redemption 结果

推荐事件：

- `coupon.definition.published`
- `coupon.issued`
- `coupon.reserved`
- `coupon.redeemed`
- `coupon.rolled_back`
- `account.entitlement.granted_from_coupon`

## 11. 存量系统升级与删除清单

### 11.1 已完成移除的旧路径

以下对象已经退出当前产品真值链路：

- `sdkwork-api-domain-coupon`
- `sdkwork-api-app-coupon`
- `project_legacy_coupon_campaign(...)` 及类似 legacy projection
- Admin 中的 `/admin/coupons` 与 legacy coupon compatibility 主视图

### 11.2 规范化落地顺序

### 阶段 A：先统一语义，不先拆物理模块

在不大规模拆 crate 的前提下，先收敛为：

- `catalog + billing/pricing + commerce` 共同承载 `API Product`
- `billing/accounts` 承载 `Commercial Account`
- `marketing` 承载 `Coupon-First Marketing`

这样可以先把语义统一，再决定是否拆出独立 `product` crate。

### 阶段 B：引入规范化对象并做兼容映射

优先补齐：

- `CouponDefinition`
- `IssuedCoupon`
- `CouponPublication`
- 结构化 `CouponTargetBinding`
- 明确 `CouponSettlementMode`

当前约束：

- `CouponTemplateRecord` 可作为短期 DTO 保留，但文档与新接口必须转向 `CouponDefinition`
- 不允许恢复 legacy `CouponCampaign`、`/admin/coupons`、或任何 coupon 兼容投影视图

### 阶段 C：迁移运行时与界面

- Admin 改成规范化 `Market` 工作台
- Portal 围绕账户与营销闭环重组页面
- Commerce 取消对 legacy coupon projection 的主路径依赖

### 阶段 D：legacy 已退场

当前阶段的硬约束是禁止重新引入：

- `sdkwork-api-domain-coupon`
- `sdkwork-api-app-coupon`
- Admin 兼容展示
- Portal / Commerce 的 fallback 适配逻辑

## 12. 分阶段实施路线

### Phase 1：语义冻结与文档对齐

输出：

- 统一术语
- 统一边界
- 禁止旧模块继续长新功能

### Phase 2：模型标准化

输出：

- `CouponDefinition / IssuedCoupon / CouponPublication / CouponSettlementMode / CouponTargetBinding`
- 产品维度与交易维度拆分

### Phase 3：控制面专业化

输出：

- marketing lifecycle 对齐 pricing lifecycle
- approval / schedule / publish / retire 全链路

### Phase 4：Portal 与 API 产品体系打通

输出：

- 产品目录 + coupon apply + entitlement arrival 一体化体验
- 对外 API 语义稳定

### Phase 5：legacy 删除

输出：

- 单一事实源
- 单一运营工作流
- 单一 Portal 自助语义

## 13. 验收标准

达到以下标准，才算进入商业化专业版本：

1. `coupon` 在 `market` 中是显式一等语义，而不是被 generic promotion 抹平。
2. `API Product`、`Commercial Account`、`Coupon-First Marketing` 三层边界清晰，概念不串层。
3. coupon 同时支持 `checkout_discount` 与 `account_entitlement` 两种结算模式。
4. `PortalCommerceQuoteKind` 不再长期混用产品类型与交易类型。
5. marketing 控制面具备 `clone / revise / compare / approval / schedule / publish / retire` 标准能力。
6. Admin 主工作台不再暴露 legacy coupon compatibility 主语义。
7. Portal 用户能够清楚理解“券归属、适用产品、核销结果、到账权益”的完整闭环。
8. runtime 主路径不再依赖 `sdkwork-api-domain-coupon` / `sdkwork-api-app-coupon` 作为真值来源。
9. 所有 coupon 核销、回滚、到账权益都可审计、可追溯、可幂等重放。

## 14. 最终结论

这次升级的核心不是把 `coupon` 从系统里抽象掉，而是把它从 legacy 兼容对象升级为商业化体系中的核心语义对象。

目标态应明确为：

- `Market` 是运营产品套件
- `API Product` 负责定义可售卖能力
- `Commercial Account` 负责承接客户已拥有资产
- `Coupon-First Marketing` 负责增长、补贴、发券、领券、核销、回滚

只有把这三层同时打通，后续账户体系、营销体系、API 产品体系、合作伙伴分发体系才能在同一条商业化主线上持续迭代，而不是继续围绕旧 coupon 兼容模型打补丁。
## 2026-04-10 S03 Loop Addendum

- `budget / code` semantic lifecycle governance is now closed on the admin canonical model:
  - budget primary actions: `activate / close`
  - code primary actions: `disable / restore`
- both aggregates now persist and expose durable lifecycle audit/query surfaces carrying:
  - `audit_id`
  - `operator_id`
  - `request_id`
  - `reason`
  - `decision_reasons`
  - `requested_at_ms`
- `/admin/marketing/budgets/{campaign_budget_id}/status` and `/admin/marketing/codes/{coupon_code_id}/status` remain compatibility routes only; semantic lifecycle is the primary governance contract
- budget activation is now headroom-aware and campaign-context-aware; code restore is expiry-aware and operator control no longer treats runtime-owned `reserved / redeemed` as normal toggle targets
- current next `S03` governance gap is narrowed to `revision / approval / compare / clone`, followed by wider portal/public/account convergence

## 2026-04-10 S03 Loop Addendum 2

- canonical `coupon template` revision governance is now closed on the admin model:
  - template additive fields:
    - `approval_state`
    - `revision`
    - `root_coupon_template_id`
    - `parent_coupon_template_id`
  - template primary revision actions:
    - `clone`
    - `compare`
    - `submit-for-approval`
    - `approve`
    - `reject`
- template lifecycle audit now also carries revision evidence:
  - `source_coupon_template_id`
  - `previous_approval_state / resulting_approval_state`
  - `previous_revision / resulting_revision`
- clone now creates a governed draft revision with explicit lineage; compare exposes field-level review; governed template `publish / schedule` is approval-aware
- raw template `create / status` remains compatibility authoring surface, but governed revision actions are now the preferred commercial contract
- current next `S03` governance gap is narrowed from template-side to campaign-side `revision / approval / compare / clone`, followed by wider portal/public/account convergence

## 2026-04-10 S03 Loop Addendum 3

- canonical `marketing campaign` revision governance is now closed on the admin model:
  - campaign additive fields:
    - `approval_state`
    - `revision`
    - `root_marketing_campaign_id`
    - `parent_marketing_campaign_id`
  - campaign primary revision actions:
    - `clone`
    - `compare`
    - `submit-for-approval`
    - `approve`
    - `reject`
- campaign lifecycle audit now also carries revision evidence:
  - `source_marketing_campaign_id`
  - `previous_approval_state / resulting_approval_state`
  - `previous_revision / resulting_revision`
- clone now creates a governed draft revision with explicit lineage; compare exposes field-level review; governed campaign `publish / schedule` is approval-aware
- raw campaign `create / status` remains compatibility authoring surface, but governed revision actions are now the preferred commercial contract
- current next `S03` governance gap is no longer campaign-side revision control; it shifts to wider portal/public/account convergence

## 2026-04-10 S03 Loop Addendum 4

- portal coupon outward semantics are now materially converged on the coupon-first model:
  - `/portal/marketing/my-coupons`
  - `/portal/marketing/reward-history`
- both portal read models now expose additive coupon semantics instead of only operational fragments:
  - `template`
  - `campaign`
  - `applicability`
  - `effect`
  - `ownership`
- outward coupon effect is now explicit as:
  - `checkout_discount`
  - `account_entitlement`
- portal wallet/history now answers more of the target questions from section `8.2`:
  - what the coupon is
  - what target kinds it applies to
  - whether it belongs to the current subject
  - whether it has moved into reservation/redemption/rollback state
- current next `S03` outward gap narrows from generic portal semantics to two sharper items:
  - per-redemption account benefit-lot arrival correlation for grant-style coupons
  - wider public API convergence on the same coupon-semantic contract

## 2026-04-10 S03 Loop Addendum 5

- portal reward-history now closes per-redemption account-arrival evidence for grant-style coupons
- outward reward-history adds additive `account_arrival`:
  - `order_id`
  - `account_id`
  - `benefit_lot_count`
  - `credited_quantity`
  - `benefit_lots`
- correlation is evidence-based only:
  - `CouponRedemptionRecord.order_id`
  - current workspace payable account
  - `AccountBenefitLotRecord.scope_json.order_id`
  - `AccountBenefitLotRecord.source_type = order`
- portal reward-history UI now distinguishes:
  - `account_entitlement + linked lots => Arrived to account`
  - `account_entitlement + no linked lots => No linked account lot evidence yet`
  - `checkout_discount => No account arrival for checkout discount`
- current next `S03` outward gap is reduced again:
  - wider public API convergence on the same coupon-semantic + account-arrival contract
  - performance/index optimization stays deferred until scale proves it necessary

## 2026-04-10 S06 Loop Addendum 1

- public gateway runtime and `/openapi.json` now converge on the `8.3` route matrix:
  - `GET /market/products`
  - `GET /market/offers`
  - `POST /market/quotes`
  - `POST /marketing/coupons/validate`
  - `POST /marketing/coupons/reserve`
  - `POST /marketing/coupons/confirm`
  - `POST /marketing/coupons/rollback`
  - `GET /commercial/account`
  - `GET /commercial/account/benefit-lots`
- outward public coupon responses now expose additive coupon semantics instead of only operational status:
  - `template`
  - `campaign`
  - `applicability`
  - `effect`
- public coupon effect is now explicit as:
  - `checkout_discount`
  - `account_entitlement`
- public commercial benefit-lot payload now exposes `scope_order_id`, making coupon redemption to account-arrival correlation outwardly inspectable
- gateway OpenAPI now publishes the same surface under tags:
  - `market`
  - `marketing`
  - `commercial`
- current next `S06` gap is no longer missing public runtime/doc routes; it narrows to:
  - account-scoped benefit-lot query, pagination, and index convergence
  - downstream SDK/client regeneration verification against the updated public gateway schema

## 2026-04-10 S06 Loop Addendum 2

- public commercial benefit-lot visibility now closes the scale follow-up from Addendum 1:
  - route query params:
    - `after_lot_id`
    - `limit`
  - response pagination evidence:
    - `page.limit`
    - `page.after_lot_id`
    - `page.next_after_lot_id`
    - `page.has_more`
    - `page.returned_count`
- account benefit-lot reads are now aligned to the storage boundary instead of route-side filtering:
  - `list_account_benefit_lots_for_account(account_id, after_lot_id, limit)`
- storage now carries a dedicated public traversal index:
  - `idx_ai_account_benefit_lot_account_lot`
- outward coupon/account-arrival semantics stay unchanged:
  - `scope_order_id` remains the correlation field for coupon redemption to account-arrival evidence
- current next `S06` gap is narrowed again:
  - downstream SDK/client regeneration verification against the updated public gateway schema

## 2026-04-10 S06 Loop Addendum 3

- public developer-facing API reference closure is now aligned with runtime and OpenAPI truth:
  - API reference center explicitly labels:
    - `market`
    - `marketing`
    - `commercial`
  - `docs/api-reference/gateway-api.md` now documents:
    - `GET /market/products`
    - `GET /market/offers`
    - `POST /market/quotes`
    - `POST /marketing/coupons/validate`
    - `POST /marketing/coupons/reserve`
    - `POST /marketing/coupons/confirm`
    - `POST /marketing/coupons/rollback`
    - `GET /commercial/account`
    - `GET /commercial/account/benefit-lots`
- public benefit-lot consumer docs now also publish:
  - `after_lot_id`
  - `next_after_lot_id`
  - `scope_order_id`
- current next `S06` gap is now reduced to one follow-up:
  - downstream SDK/client regeneration verification against the updated public gateway schema

## 2026-04-10 S07 Loop Addendum 1

- legacy coupon compatibility is no longer present on the main coupon runtime path:
  - `gateway_market`
  - portal marketing read / reserve flows
  - commerce coupon catalog and order coupon state flows
- canonical marketing code records are now the only coupon context source for those main runtime surfaces
- reserve-time atomic writes no longer persist projected legacy template / campaign objects
- main-path crate dependencies now drop legacy coupon app/domain coupling from:
  - `sdkwork-api-app-commerce`
  - `sdkwork-api-interface-portal`
- admin primary coupon detail no longer renders `Legacy coupon compatibility`
- the deeper `S07` exit work recorded here has been closed in `S07 Loop Addendum 2`

## 2026-04-10 S07 Loop Addendum 2

- deeper legacy coupon exit is now closed on the active control plane and storage/runtime boundary:
  - removed `sdkwork-api-domain-coupon`
  - removed `sdkwork-api-app-coupon`
  - removed `/admin/coupons`
  - removed legacy coupon storage/bootstrap contracts
- admin API reference and module-boundary docs now treat `marketing` as the only coupon governance surface
- remaining `legacy coupon` mentions in the repo are historical records under `docs/review`, `docs/step`, or `docs/superpowers`, not active runtime or active product surfaces

## 2026-04-10 S08 Loop Addendum 1

- integrated acceptance now has fresh evidence across:
  - admin control-plane source-contract and UX suites
  - portal commercial and marketing source-contract suites
  - admin legacy coupon route-removal backend proof
  - gateway public `market / marketing / commercial` runtime and OpenAPI proof
- the verified internal product truth across `API Product`, `Commercial Account`, and `Coupon-First Marketing` is currently converged on active admin, portal, and public gateway surfaces
- release readiness is not yet `go`:
  - `release-slo-governance` remains blocked by missing governed telemetry export / snapshot input
  - `release-window-snapshot` remains blocked on the current host because governed CLI replay cannot materialize release Git window truth
  - `release-sync-audit` remains blocked on the current host for the same release-truth boundary
- current commercialization posture is therefore:
  - internal product/runtime convergence: materially closed for the verified `S03/S06/S07` path
  - external release truth and final sign-off: not closed
- the next commercialization gate is no longer coupon/product model convergence; it is governed release evidence materialization plus final release-truth replay on a host that can prove remote refs

## 2026-04-10 S08 Loop Addendum 2

- governed latest-artifact replay now proves that the current remaining commercialization blocker is narrower than the earlier all-blocked gate:
  - `release-window-snapshot` is closed on the current host through a shell-git-governed artifact
  - `release-sync-audit` is not an opaque exec failure anymore; it is an explicit release `fail`
  - `release-slo-governance` remains the only still-blocked lane
- current release `fail` truth is concrete:
  - `sdkwork-api-router` is dirty and not branch-synced for release
  - `sdkwork-core` is not a standalone git root for the expected release boundary and its remote does not match the governed target repository
  - `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` all remain dirty
  - remote head verification is still not proven from the current host
- commercialization readiness therefore remains limited not by product-model closure, but by release-governance hygiene and live release evidence

## 2026-04-10 S08 Loop Addendum 3

- the governed release-window lane now carries an extra operational rule:
  - default latest-artifact replay is authoritative for blocked-host governance
  - therefore every later `S08` backwrite that changes workspace size must re-materialize `docs/release/release-window-snapshot-latest.json`
- the current refreshed governed window truth is:
  - `latestReleaseTag = release-2026-03-28-8`
  - `commitsSinceLatestRelease = 22`
  - `workingTreeEntryCount = 518`
- this refresh does not change business capability closure:
  - `API Product / Commercial Account / Coupon-First Marketing` remain internally converged on the verified admin, portal, and public surfaces
  - release readiness still fails on repository hygiene and missing governed live telemetry evidence, not on product-model drift

## 2026-04-10 S08 Loop Addendum 4

- the remaining telemetry blocker is now classified more precisely as an external observability handoff gap:
  - current shell injects no `SDKWORK_RELEASE_TELEMETRY_*` or `SDKWORK_SLO_GOVERNANCE_*` values
  - the repo carries no current `release-telemetry-export`, `release-telemetry-snapshot`, or `slo-governance` latest artifact
  - the documented local metrics endpoints on `127.0.0.1:8080/8081/8082` are not reachable from the current host
- this architecture fact matters because the repo already owns:
  - the governed telemetry export materializer
  - the telemetry snapshot derivation chain
  - the SLO evidence materializer and baseline
- what it still does not own locally is the truthful upstream release-time observability handoff:
  - real hosted control-plane metrics
  - truthful `supplemental.targets` for the non-availability SLO targets
- commercialization readiness therefore remains blocked by external release evidence supply plus release repo hygiene, not by missing coupon/account/product architecture work

## 2026-04-10 S08 Loop Addendum 5

- the earlier `8080 / 8081 / 8082` result is now understood as a raw-standalone boundary, not a proof that local observability is absent:
  - those raw defaults are still unreachable on this host
  - managed local services are simultaneously live on `127.0.0.1:9980 / 9981 / 9982`
- current managed observability truth is stronger:
  - `health` probes on `9980 / 9981 / 9982` all return `200`
  - authenticated `metrics` probes on `9980 / 9981 / 9982` all return Prometheus text
  - live managed metrics are therefore sufficient to materialize a truthful release telemetry export locally
- the actionable blocker has narrowed again:
  - export materialization succeeds from live managed `/metrics`
  - snapshot derivation then fails immediately on `gateway-non-streaming-success-rate`
  - this means the remaining observability gap is truthful non-availability SLO coverage, not the absence of availability telemetry
- in repository terms, the still-missing governed truth is:
  - truthful `supplemental.targets` for the remaining non-availability SLO targets
  - the default governed latest telemetry snapshot / SLO evidence writeback that depends on that complete target set
- commercialization readiness therefore remains blocked by incomplete release evidence coverage plus release repo hygiene, not by product-model incompleteness inside `API Product / Commercial Account / Coupon-First Marketing`

## 2026-04-11 S08 Loop Addendum 6

- the commercialization gate is now more precise again because a truthful local derived-target probe closes six non-availability SLO targets from live runtime actions:
  - `routing-simulation-p95-latency`: `20/20`, `p95Ms = 30`
  - `api-key-issuance-success-rate`: `3/3`
  - `runtime-rollout-success-rate`: `1/1`
  - `gateway-non-streaming-success-rate`: `3/3`
  - `gateway-streaming-completion-success-rate`: `3/3`
  - `billing-event-write-success-rate`: `6/6`
- this matters architecturally because the current repo can now prove more of the governed SLO baseline from the real local system without inventing second-source commercial truth
- the first missing governed snapshot target has therefore advanced:
  - previous first failure: `gateway-non-streaming-success-rate`
  - current first failure: `gateway-fallback-success-rate`
- the remaining release-evidence gap is now split into three concrete classes:
  - provider-level fallback / timeout evidence is still absent from live gateway metrics
  - canonical commercial billing kernel surfaces for holds, settlements, and pricing synchronization are still unavailable on the current runtime
  - portal commercial account truth is still not provisioned on the current live runtime
- correlated runtime evidence stays aligned with the product model:
  - gateway probe traffic still writes usage records and billing events into the same `Commercial Account` accounting chain
  - routing decision logs still originate from the canonical gateway path with `decision_source = gateway`
  - `fallback_reason = policy_candidate_unavailable` is observable in business records even though provider-level failover counters are still absent
- commercialization readiness therefore remains `no-go` for release reasons, not for product-model incompleteness:
  - `API Product / Commercial Account / Coupon-First Marketing` remain internally converged on the verified admin, portal, and public surfaces
  - release readiness is still blocked by incomplete truthful fallback / timeout evidence and by independent release-sync hygiene failure

## 2026-04-11 S08 Loop Addendum 7

- commercialization release evidence is now fully materialized on the current host:
  - the governed telemetry export, telemetry snapshot, and SLO evidence latest artifacts all exist under `docs/release`
  - `release-slo-governance` is no longer structurally blocked by missing telemetry input
- the final repo-local obstacle to that materialization was a release-tooling parser defect, not an architecture gap:
  - live admin Prometheus route labels can contain template braces such as `/admin/runtime-config/rollouts/{rollout_id}`
  - the governed telemetry snapshot parser now accepts that truthful live shape
- the commercialization evidence picture is therefore sharper than in Addendum 6:
  - gateway fallback viability is now truthfully closed by live stateful-provider execution:
    - `gateway-fallback-success-rate = 1`
    - `gateway-provider-timeout-budget = 0`
  - admin/commercial control-plane failure is now quantitative rather than inferential:
    - `admin-api-availability` fails because live admin counters include accumulated `501` responses
    - `account-hold-creation-success-rate = 0`
    - `request-settlement-finalize-success-rate = 0`
    - `pricing-lifecycle-synchronize-success-rate = 0`
- commercialization readiness therefore remains `no-go`, but the reason has improved:
  - it is no longer blocked by the inability to produce truthful release evidence
  - it is blocked by explicit release-governance failure on the commercial billing control plane plus independent release-sync hygiene failure

## 2026-04-11 S08 Loop Addendum 8

- current source now closes the admin standalone commercial billing gap directly:
  - the admin standalone runtime bootstraps `commercial_billing` through `build_admin_store_and_commercial_billing_from_config(...)`
  - that kernel is now injected into both runtime state and reload handles, matching the already-correct portal/product runtime pattern
- current source evidence is also stronger than "route exists":
  - a service-level regression test now proves the canonical admin commercial-control-plane routes execute successfully from the standalone runtime path
  - the same test proves admin Prometheus counters accumulate `status="200"` samples for those routes and leave the matching `status="501"` samples at `0`
- this changes the architectural diagnosis:
  - the remaining commercialization blocker is no longer an unresolved repo-local admin runtime capability hole
  - it is release-evidence freshness on a host that can replay the source-backed stack after this fix
- commercialization readiness still remains `no-go` in the current loop:
  - the present shell policy blocks the background child-process launch pattern required to replay the live source-backed stack from this session
  - existing governed latest artifacts therefore still represent the pre-fix live replay and cannot be promoted to post-fix commercialization truth without a new replay

## 2026-04-11 S08 Loop Addendum 9

- the commercialization architecture is now proven again from fresh governed live replay, not only from source-level or service-level tests:
  - the canonical admin commercial-control-plane routes now succeed on the replayed source-backed stack
  - the resulting evidence pack materializes both isolated and latest governed telemetry artifacts from the same live runtime truth
- this removes the last architecture-level ambiguity around commercialization closure:
  - `API Product / Commercial Account / Coupon-First Marketing` no longer fail on admin runtime capability
  - they no longer fail on telemetry completeness for the governed SLO baseline
  - `release-slo-governance` now passes all `14` targets from the current live replay
- the remaining release blocker now sits outside the product architecture itself:
  - `release-window-snapshot`: `pass`
  - `release-slo-governance`: `pass`
  - `release-sync-audit`: `fail`
- architecture consequence:
  - no new second-source coupon/account/admin/portal truth is introduced
  - the remaining `no-go` is release-governance hygiene across repositories, not commercialization model incompleteness inside this architecture

## 2026-04-11 S08 Loop Addendum 10

- the final remaining blocker is now explicitly classified as external to this commercialization architecture:
  - current repo-local product, account, coupon-first marketing, and control-plane capability closure is not the failing surface anymore
  - the failing surface is governed cross-repository release hygiene
- the strongest evidence is repository-boundary, not product-boundary:
  - current `sdkwork-api-router` branch has no upstream configured and remains dirty
  - current `sdkwork-core` audit resolves to a larger parent git root and a mismatched origin URL
- architecture conclusion:
  - this repo should not reopen commercial product implementation work while the release-sync blocker remains unchanged
  - the remaining `S08` `no-go` is outside the commercial product architecture and must be resolved at the release-governance boundary
