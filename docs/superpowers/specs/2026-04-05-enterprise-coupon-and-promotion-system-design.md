# 企业级优惠券与促销系统详细设计

> 状态：历史设计草案，已被 `docs/架构/166-*` 与 2026-04-10 的 legacy coupon 全量退场结果取代；以下内容仅保留为问题分析与演进背景。

**状态：** 基于 `sdkwork-api-router` 当前仓库实现与 2026-04-05 官方行业资料校对后的增强设计稿

**目标：** 将当前 `单表 coupon campaign + 剩余额度递减` 的轻量实现，升级为可商用、可审计、可回滚、可扩展、可对账、可运营的企业级优惠券与促销系统，并与现有订单、支付、账户、计费、退款、门户和后台体系形成完整闭环。

**适用范围：**

- 后台优惠券与促销管理
- 门户领券、验券、预占、下单、核销、退款、回滚
- 订单中心、支付中心、账户中心、营销增长、渠道归因
- SQLite 本地模式与 PostgreSQL 生产模式

## 1. 执行结论

当前仓库中的优惠券能力还不能视为商用级系统。

从当前代码看，优惠券仍然主要是以下模型：

- 一个 `CouponCampaign`
- 一个明文 `code`
- 一个 `remaining`
- 一个 `active`
- 一个 `discount_label`

它可以支撑简单活动或演示，但无法支撑以下商业现实：

- 一个模板发行多批多码
- 共享码与唯一券码并存
- 预览校验与正式核销分离
- 并发校验与下单时的库存占用
- 退款、取消、部分退货后的权益回滚
- 栈式叠加、互斥组、优先级和预算约束
- 渠道、伙伴、邀请、裂变与归因一体化
- 财务对账、营销预算、异常审计和防刷

正确方向不是继续强化 `ai_coupon_campaigns`，而是建立一个独立的营销内核，并通过兼容层逐步替换现有实现。

## 2. 行业对标结论

本设计对标了三类成熟系统的公开官方方案：

- Stripe Coupons / Promotion Codes
- Voucherify
- Talon.One

### 2.1 Stripe 给出的关键启发

基于 Stripe 官方文档，Stripe 清晰区分了 `coupon` 与 `promotion code`：

- coupon 定义折扣语义
- promotion code 是面向用户暴露的可输入代码
- 多个 promotion code 可以映射到同一个 coupon
- 支持产品范围限制、首单限制、最低金额、有效期、最大核销次数
- 支持多折扣叠加，但对叠加顺序、对象范围、更新语义有明确限制

对本系统的借鉴：

- 必须把“优惠定义”与“用户可输入券码”分离
- 必须显式建模适用范围、限制条件、叠加规则
- 必须允许一个模板映射多个外显码
- 必须把“可见码”和“底层 benefit 定义”解耦

### 2.2 Voucherify 给出的关键启发

基于 Voucherify 官方文档，Voucherify 的优势在于：

- 校验规则是独立的一等公民
- `validate` 与 `redeem` 是两个不同动作
- 支持 stackable discounts
- 支持 redemption rollback
- 支持 session lock，用来处理并发校验与核销
- 支持将预算、产品适用范围、受众、元数据、次数限制写成结构化规则

对本系统的借鉴：

- 必须将“验券”和“核销”拆开
- 必须引入 reservation / hold / session lock 机制
- 必须让 redemption 可回滚，而且回滚是显式业务动作
- 必须支持多券组合、分组互斥和父子核销链路
- 必须把 eligibility 规则结构化，而不是继续依赖 `discount_label` 文本推断

### 2.3 Talon.One 给出的关键启发

基于 Talon.One 官方文档，Talon.One 的关键思想是：

- 一切围绕 customer session / cart session 建模
- 会话有 `open / closed / cancelled / partially returned` 生命周期
- 关闭会话时触发正式促销效果
- 取消和部分退货会回滚相关效果与预算
- 促销不是单点 API，而是附着在购物车、订单、退货和归因上下文中的规则执行

对本系统的借鉴：

- 必须把优惠券应用过程绑定到 `quote / order / payment / refund / return` 生命周期
- 必须支持订单级与订单项级的部分回滚
- 必须让营销预算与退款、取消、部分退货联动
- 必须保存 session / order / payment 的上下文，以便追踪一次促销的完整生命周期

### 2.4 对标后的设计原则

综合三类平台，本系统应采用以下原则：

1. 定义与外显码分离
2. 校验、预占、核销、履约、回滚分离
3. 一切营销效果必须可审计、可回放、可回滚
4. 优惠券必须接入订单与支付生命周期，而不是只做表字段扣减
5. 营销预算、权益发放、财务记账必须联动

## 3. 当前系统现状与主要问题

### 3.1 当前代码现状

当前实现的核心位置：

- 当时的 legacy coupon 领域对象：已移除的历史 coupon domain crate
- 当时的 legacy coupon 应用逻辑：已移除的历史 coupon app crate
- 当前订单与门户优惠逻辑：`crates/sdkwork-api-app-commerce/src/lib.rs`
- 当前后台优惠券接口：`crates/sdkwork-api-interface-admin/src/lib.rs`
- 当前门户订单与对账联动：`crates/sdkwork-api-interface-portal/src/lib.rs`

### 3.2 当前模型缺陷

当前 `CouponCampaign` 只有：

- `id`
- `code`
- `discount_label`
- `audience`
- `remaining`
- `active`
- `note`
- `expires_on`
- `created_at_ms`

它缺失了真正商用系统需要的核心能力：

- 没有模板层
- 没有批次层
- 没有券码库存层
- 没有 claim 层
- 没有 reservation 层
- 没有 redemption 证据层
- 没有 rollback 层
- 没有预算层
- 没有验证规则层
- 没有叠加与互斥层
- 没有渠道与归因层

### 3.3 当前流程缺陷

当前主要问题包括：

1. `discount_label` 被用于推断折扣百分比，属于脆弱的字符串解析逻辑。
2. 订单预览与正式下单没有独立的券预占语义。
3. `remaining` 通过重写优惠券记录来递减，不具备并发安全和财务审计能力。
4. 没有 idempotency key，回调重放和重复提交可能造成重复消耗。
5. 没有 redemption record，无法精确追踪“谁在什么订单上用了哪张券，产出了什么权益”。
6. 退款路径并没有把优惠券效果作为显式可回滚对象管理。
7. 门户只能做简单输入券码，不支持我的券包、领取、锁定、释放、失效原因解释。
8. 后台只有 list/create/delete，远达不到运营系统要求。

### 3.4 当前商业风险

在生产环境中，这些缺陷会直接带来：

- 超发
- 漏发
- 重复核销
- 退款后预算未回收
- 订单失败但券已消耗
- 部分退款无法正确回滚
- 高并发下剩余额度异常
- 无法做营销活动复盘与财务对账

## 4. 目标系统边界

本设计将营销系统划分为六个边界清晰的子域：

1. 优惠模板内核
2. 活动与预算内核
3. 券码发行与库存内核
4. 校验与预占内核
5. 核销与履约内核
6. 回滚、退款与归因内核

### 4.1 优惠模板内核

负责定义：

- 优惠类型
- 适用对象
- 规则集合
- 叠加策略
- 生命周期

### 4.2 活动与预算内核

负责定义：

- 活动业务语义
- 时间窗口
- 渠道归因
- 预算与补贴上限
- 运营 owner

### 4.3 券码发行与库存内核

负责定义：

- 共享码
- 唯一码
- 批量导入
- 自动生成
- 发放渠道
- 库存状态

### 4.4 校验与预占内核

负责：

- 验证条件是否满足
- 试算折扣效果
- 锁定可用库存与预算
- 返回可解释失败原因

### 4.5 核销与履约内核

负责：

- 正式核销
- 生成不可变 redemption evidence
- 触发折扣生效或权益发放
- 触发账务侧联动

### 4.6 回滚、退款与归因内核

负责：

- 订单取消回滚
- 退款回滚
- 部分退货局部回滚
- 预算恢复
- 归因保留
- 审计保留

## 5. 目标产品设计

### 5.1 Admin 后台产品模块

应新增或重构为以下工作台：

- 模板中心 `Templates`
- 活动中心 `Campaigns`
- 预算中心 `Budgets`
- 批次中心 `Batches`
- 券码仓库 `Codes Vault`
- 领取与券包 `Claims and Wallet`
- 核销中心 `Redemptions`
- 回滚与异常 `Rollbacks and Exceptions`
- 邀请与裂变 `Referral Programs`
- 渠道归因 `Attribution`
- 风控审计 `Fraud and Audit`

每个模块的重点如下。

#### 模板中心

模板定义以下能力：

- 名称、编码、展示文案
- benefit 类型
- 适用对象
- 有效期与生效条件
- 是否需要 claim
- 是否允许 stack
- 互斥组
- 每用户、每账户、每项目上限

#### 活动中心

活动定义以下能力：

- 归属模板
- 目标人群
- 渠道
- 时间窗口
- 预算
- 投放目标
- owner
- 是否允许手工补发

#### 批次中心

支持：

- 生成唯一码批次
- 导入外部批次
- 共享码注册
- 码规则模板
- 导出掩码列表
- 全量禁用
- 分批失效

#### 券码仓库

支持：

- 按前缀、后缀、批次、状态、活动检索
- 查看码生命周期
- 查看 claim 主体
- 查看 redemption 链路
- block / void / rotate

#### 核销中心

支持：

- 查看校验失败原因分布
- 查看 reservation 与最终 redemption 差异
- 查看按订单、用户、项目的核销证据
- 发起回滚或补履约

#### 风控审计

支持：

- 同账号短时高频尝试
- 设备指纹聚集
- 同支付单多次核销尝试
- 渠道刷券
- budget burn outlier

### 5.2 Portal 门户产品能力

门户应提供：

- 验券预览
- 优惠试算
- 我的优惠券包
- 领取券
- 绑定邀请码
- 订单确认前预占
- 支付失败后释放
- 订单完成后自动核销
- 退款后展示优惠回滚结果

### 5.3 运维与财务能力

运维与财务侧需要：

- 预算消耗看板
- 活动 ROI
- 核销成功率
- 回滚率
- 退款冲销一致性
- 渠道补贴分摊
- 财务凭证导出

## 6. 领域模型设计

### 6.1 核心实体

建议引入以下领域对象。

#### CouponTemplateRecord

定义“优惠是什么”。

关键字段：

- `coupon_template_id`
- `tenant_id`
- `template_code`
- `display_name`
- `status`
- `benefit_kind`
- `distribution_kind`
- `claim_required`
- `stacking_policy`
- `exclusive_group`
- `starts_at_ms`
- `ends_at_ms`
- `max_total_redemptions`
- `max_redemptions_per_subject`
- `created_at_ms`
- `updated_at_ms`

#### CouponBenefitRuleRecord

定义优惠怎么生效。

支持的 `benefit_kind`：

- `percentage_discount`
- `fixed_amount_discount`
- `credit_grant`
- `quota_units_grant`
- `request_allowance_grant`
- `meter_allowance_grant`
- `gift_pack`
- `trial_activation_reward`

关键字段：

- `benefit_rule_id`
- `coupon_template_id`
- `apply_scope_kind`
- `product_scope_json`
- `plan_scope_json`
- `model_scope_json`
- `currency_scope_json`
- `discount_percent`
- `discount_amount_minor`
- `grant_quantity`
- `grant_asset_kind`
- `subsidy_cap_minor`
- `priority`

#### MarketingCampaignRecord

定义“活动如何运营”。

关键字段：

- `marketing_campaign_id`
- `coupon_template_id`
- `campaign_code`
- `display_name`
- `campaign_kind`
- `channel_source`
- `owner_user_id`
- `status`
- `starts_at_ms`
- `ends_at_ms`
- `audience_rule_json`
- `metadata_json`

#### MarketingCampaignBudgetRecord

定义预算约束。

关键字段：

- `budget_id`
- `marketing_campaign_id`
- `budget_kind`
- `currency_code`
- `hard_cap_minor`
- `soft_cap_minor`
- `reserved_minor`
- `consumed_minor`
- `remaining_minor`
- `reset_policy`
- `window_kind`
- `updated_at_ms`

#### CouponCodeBatchRecord

定义一批券码如何生成。

关键字段：

- `coupon_code_batch_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `batch_kind`
- `generation_mode`
- `code_prefix`
- `issued_count`
- `claimed_count`
- `reserved_count`
- `redeemed_count`
- `voided_count`
- `expires_at_ms`
- `created_at_ms`

#### CouponCodeRecord

定义可被输入或系统发放的具体券码。

关键字段：

- `coupon_code_id`
- `coupon_code_batch_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `normalized_code_hash`
- `display_code_prefix`
- `display_code_suffix`
- `code_kind`
- `status`
- `issued_to_subject_type`
- `issued_to_subject_id`
- `claim_subject_type`
- `claim_subject_id`
- `expires_at_ms`
- `issued_at_ms`
- `claimed_at_ms`
- `updated_at_ms`

#### CouponClaimRecord

定义“券已属于谁，但未必已使用”。

关键字段：

- `coupon_claim_id`
- `coupon_code_id`
- `coupon_template_id`
- `subject_type`
- `subject_id`
- `claim_channel`
- `claim_source`
- `claim_status`
- `claimed_at_ms`
- `expires_at_ms`

#### CouponReservationRecord

定义“验券后为订单暂时锁定的占用”。

关键字段：

- `coupon_reservation_id`
- `coupon_code_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `subject_type`
- `subject_id`
- `project_id`
- `quote_id`
- `order_id`
- `reservation_status`
- `budget_reserved_minor`
- `inventory_reserved_count`
- `idempotency_key`
- `expires_at_ms`
- `created_at_ms`
- `released_at_ms`

#### CouponRedemptionRecord

定义不可变核销证据。

关键字段：

- `coupon_redemption_id`
- `coupon_code_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `coupon_reservation_id`
- `tenant_id`
- `user_id`
- `account_id`
- `project_id`
- `quote_id`
- `order_id`
- `payment_order_id`
- `payment_event_id`
- `redemption_status`
- `subsidy_amount_minor`
- `currency_code`
- `pricing_adjustment_id`
- `benefit_lot_id`
- `idempotency_key`
- `redeemed_at_ms`

#### CouponRollbackRecord

定义核销回滚与退款回滚证据。

关键字段：

- `coupon_rollback_id`
- `coupon_redemption_id`
- `rollback_type`
- `rollback_scope`
- `reason_code`
- `refund_order_id`
- `returned_items_json`
- `restored_budget_minor`
- `restored_inventory_count`
- `rollback_status`
- `created_at_ms`

#### MarketingAttributionTouchRecord

定义一次营销触点。

关键字段：

- `marketing_touch_id`
- `marketing_campaign_id`
- `coupon_code_id`
- `source_kind`
- `source_code`
- `utm_source`
- `utm_medium`
- `utm_campaign`
- `partner_id`
- `referrer_user_id`
- `project_id`
- `order_id`
- `touch_type`
- `created_at_ms`

### 6.2 关键状态机

#### 券码状态机

- `draft`
- `issued`
- `claimable`
- `claimed`
- `reserved`
- `redeemed`
- `expired`
- `voided`
- `blocked`

#### reservation 状态机

- `pending`
- `active`
- `confirmed`
- `released`
- `expired`
- `cancelled`

#### redemption 状态机

- `pending`
- `applied`
- `fulfilled`
- `rolled_back`
- `failed`

#### rollback 状态机

- `pending`
- `completed`
- `partially_completed`
- `failed`

## 7. 核心业务流程设计

### 7.1 验券预览流程

目标：不改变库存与预算，只返回是否可用、折扣效果和失败原因。

流程：

1. 归一化用户输入券码
2. 哈希查找 `CouponCodeRecord`
3. 加载模板、规则、预算、用户画像、订单上下文
4. 执行 eligibility rules
5. 执行 stacking rules
6. 计算折扣或权益效果
7. 返回 `valid / invalid / inapplicable` 与诊断

输出：

- 可用性
- 原价 / 折后价 / 补贴额
- 奖励额度
- 不可用原因
- 是否需要 claim
- 是否需要 reservation

### 7.2 下单预占流程

目标：支付前锁住库存和预算，避免高并发超发。

流程：

1. 客户端提交 quote 与券码
2. 服务端调用验证逻辑
3. 通过后开启事务
4. 为券码建立 `CouponReservationRecord`
5. 为 campaign budget 建立 reserved amount
6. 为订单写入 `coupon_reservation_id`
7. 返回 reservation token 与过期时间

约束：

- 预占必须带 `idempotency_key`
- 重复提交返回同一 reservation
- 支付失败、超时、取消必须显式 release

### 7.3 订单确认与正式核销

目标：只有订单进入有效商业状态时才正式核销。

流程：

1. 订单确认或支付成功
2. 使用 `reservation_id + idempotency_key` 进入确认流程
3. 校验 reservation 仍然有效
4. 生成 `CouponRedemptionRecord`
5. 将券码状态从 `reserved` 切到 `redeemed`
6. 将 budget 从 `reserved` 转为 `consumed`
7. 写入价格调整或权益发放凭证
8. 发送 outbox event 给账务与分析侧

### 7.4 grant 型优惠的履约

grant 型优惠包括：

- 充值赠送额度
- 免费体验额度
- request quota grant
- meter allowance grant

处理规则：

- 核销记录先写
- 再写 benefit lot / quota grant record
- 再写 billing / ledger evidence
- 任一步失败则保持幂等重试，不得重复发放

### 7.5 退款与回滚

需要区分三类情况：

1. 支付失败或订单取消
2. 全额退款
3. 部分退款或部分退货

处理原则：

- discount 型：恢复预算、恢复可用次数、冲减 pricing adjustment
- grant 型：若尚未消费则回收权益；若已消费，进入人工审核或差额回收策略
- 部分退货：仅按受影响商品或比例局部回滚

### 7.6 支持 Talon.One 风格的 session 生命周期

建议引入会话状态：

- `open`
- `reserved`
- `closed`
- `cancelled`
- `partially_returned`

营销效果的确认点：

- `open` 只允许 validate
- `reserved` 允许 hold
- `closed` 才允许 confirm redemption
- `cancelled` 触发 full rollback
- `partially_returned` 触发 partial rollback

## 8. API 设计

## 8.1 Admin API

### 模板管理

- `POST /admin/marketing/templates`
- `GET /admin/marketing/templates`
- `GET /admin/marketing/templates/{template_id}`
- `PUT /admin/marketing/templates/{template_id}`
- `POST /admin/marketing/templates/{template_id}/activate`
- `POST /admin/marketing/templates/{template_id}/archive`

### 活动管理

- `POST /admin/marketing/campaigns`
- `GET /admin/marketing/campaigns`
- `GET /admin/marketing/campaigns/{campaign_id}`
- `PUT /admin/marketing/campaigns/{campaign_id}`
- `POST /admin/marketing/campaigns/{campaign_id}/pause`
- `POST /admin/marketing/campaigns/{campaign_id}/resume`

### 批次与券码管理

- `POST /admin/marketing/code-batches`
- `POST /admin/marketing/code-batches/{batch_id}/generate`
- `POST /admin/marketing/code-batches/{batch_id}/import`
- `GET /admin/marketing/code-batches/{batch_id}/codes`
- `POST /admin/marketing/codes/search`
- `POST /admin/marketing/codes/{code_id}/block`
- `POST /admin/marketing/codes/{code_id}/void`

### 预算与审计

- `GET /admin/marketing/budgets`
- `GET /admin/marketing/budgets/{campaign_id}`
- `GET /admin/marketing/redemptions`
- `GET /admin/marketing/redemptions/{redemption_id}`
- `POST /admin/marketing/redemptions/{redemption_id}/rollback`
- `GET /admin/marketing/rollbacks`

### 推荐的 Admin 请求示例

`POST /admin/marketing/templates`

```json
{
  "template_code": "WELCOME_CREDIT",
  "display_name": "Welcome Credit 100",
  "benefit_kind": "quota_units_grant",
  "distribution_kind": "shared_code",
  "claim_required": false,
  "stacking_policy": "exclusive",
  "exclusive_group": "welcome",
  "starts_at_ms": 1764547200000,
  "ends_at_ms": 1796083200000,
  "max_total_redemptions": 500000,
  "max_redemptions_per_subject": 1,
  "benefit_rules": [
    {
      "apply_scope_kind": "workspace_account",
      "grant_quantity": 100
    }
  ],
  "eligibility_rules": {
    "first_paid_order_only": false,
    "first_recharge_only": true,
    "project_scope": "all"
  }
}
```

### 兼容旧接口策略

当时的旧兼容接口：

- legacy admin coupon list route
- legacy admin coupon create route
- legacy admin coupon delete route

兼容策略：

- legacy admin coupon create route 继续存在，但内部改为生成：
  - 一个 `CouponTemplateRecord`
  - 一个 `MarketingCampaignRecord`
  - 一个 `CouponCodeBatchRecord`
  - 一个共享码 `CouponCodeRecord`
- 旧接口返回兼容视图，而不是直接操作未来的核心表语义

## 8.2 Portal API

### 验券与试算

- `POST /portal/marketing/validate`
- `POST /portal/marketing/stack/validate`
- `POST /portal/commerce/quotes`

### 预占与下单

- `POST /portal/marketing/reservations`
- `POST /portal/marketing/reservations/{reservation_id}/confirm`
- `POST /portal/marketing/reservations/{reservation_id}/release`
- `POST /portal/commerce/orders`

### 领券与我的券包

- `POST /portal/marketing/claims`
- `GET /portal/marketing/my-coupons`
- `GET /portal/marketing/redemptions`

### 退款与回滚查看

- `GET /portal/marketing/redemptions/{redemption_id}`
- `GET /portal/marketing/rollbacks`

### 推荐的 Portal 验券请求

`POST /portal/marketing/validate`

```json
{
  "project_id": "project_123",
  "user_id": "user_123",
  "quote_context": {
    "target_kind": "recharge_pack",
    "target_id": "pack-100k",
    "currency_code": "USD",
    "order_amount_minor": 9900,
    "items": [
      {
        "sku": "pack-100k",
        "quantity": 1,
        "amount_minor": 9900
      }
    ]
  },
  "redeemables": [
    {
      "type": "coupon_code",
      "code": "SPRING20"
    }
  ]
}
```

返回：

```json
{
  "valid": true,
  "reservation_required": true,
  "stack_result": {
    "discount_total_minor": 1980,
    "grant_total_units": 0
  },
  "applied_effects": [
    {
      "template_code": "SPRING_PROMO",
      "effect_kind": "percentage_discount",
      "discount_minor": 1980
    }
  ],
  "diagnostics": []
}
```

## 8.3 内部应用服务方法设计

建议新增 `sdkwork-api-app-marketing`，提供以下核心方法：

```rust
async fn validate_redeemables(
    store: &dyn MarketingStore,
    input: ValidateRedeemablesInput,
) -> Result<ValidateRedeemablesResult>;

async fn reserve_redeemables(
    store: &dyn MarketingStore,
    input: ReserveRedeemablesInput,
) -> Result<CouponReservationRecord>;

async fn confirm_redemption(
    store: &dyn MarketingStore,
    input: ConfirmRedemptionInput,
) -> Result<CouponRedemptionRecord>;

async fn release_reservation(
    store: &dyn MarketingStore,
    input: ReleaseReservationInput,
) -> Result<CouponReservationRecord>;

async fn rollback_redemption(
    store: &dyn MarketingStore,
    input: RollbackRedemptionInput,
) -> Result<CouponRollbackRecord>;

async fn sync_order_refund_to_coupon_rollbacks(
    store: &dyn MarketingStore,
    input: SyncOrderRefundInput,
) -> Result<Vec<CouponRollbackRecord>>;
```

这些方法必须具备：

- 幂等键
- 事务边界
- 可解释失败原因
- outbox event
- 指标埋点

## 9. 数据库设计

## 9.1 新表设计

建议新增以下表。

### 模板与规则

- `ai_marketing_coupon_template`
- `ai_marketing_coupon_benefit_rule`
- `ai_marketing_coupon_eligibility_rule`
- `ai_marketing_coupon_stack_rule`

### 活动与预算

- `ai_marketing_campaign`
- `ai_marketing_campaign_budget`
- `ai_marketing_campaign_budget_ledger`

### 券码与批次

- `ai_marketing_coupon_code_batch`
- `ai_marketing_coupon_code`
- `ai_marketing_coupon_claim`

### 预占、核销、回滚

- `ai_marketing_coupon_reservation`
- `ai_marketing_coupon_redemption`
- `ai_marketing_coupon_effect`
- `ai_marketing_coupon_rollback`

### 归因与裂变

- `ai_marketing_referral_program`
- `ai_marketing_referral_invite`
- `ai_marketing_attribution_touch`

### 集成保障

- `ai_marketing_outbox_event`
- `ai_marketing_idempotency_key`

## 9.2 关键字段与约束

### `ai_marketing_coupon_code`

建议字段：

- `coupon_code_id TEXT PRIMARY KEY`
- `coupon_template_id TEXT NOT NULL`
- `marketing_campaign_id TEXT`
- `coupon_code_batch_id TEXT`
- `normalized_code_hash TEXT NOT NULL`
- `display_code_prefix TEXT NOT NULL`
- `display_code_suffix TEXT NOT NULL`
- `code_kind TEXT NOT NULL`
- `status TEXT NOT NULL`
- `claim_subject_type TEXT`
- `claim_subject_id TEXT`
- `issued_at_ms INTEGER NOT NULL`
- `claimed_at_ms INTEGER`
- `expires_at_ms INTEGER`
- `updated_at_ms INTEGER NOT NULL`
- `version INTEGER NOT NULL DEFAULT 0`

唯一约束与索引：

- `UNIQUE(normalized_code_hash)`
- `INDEX(status, expires_at_ms, updated_at_ms DESC)`
- `INDEX(coupon_template_id, status, updated_at_ms DESC)`
- `INDEX(claim_subject_type, claim_subject_id, status)`

### `ai_marketing_coupon_reservation`

建议字段：

- `coupon_reservation_id TEXT PRIMARY KEY`
- `coupon_code_id TEXT NOT NULL`
- `order_id TEXT`
- `quote_id TEXT`
- `subject_type TEXT NOT NULL`
- `subject_id TEXT NOT NULL`
- `reservation_status TEXT NOT NULL`
- `budget_reserved_minor INTEGER NOT NULL DEFAULT 0`
- `inventory_reserved_count INTEGER NOT NULL DEFAULT 0`
- `idempotency_key TEXT NOT NULL`
- `expires_at_ms INTEGER NOT NULL`
- `created_at_ms INTEGER NOT NULL`
- `released_at_ms INTEGER`

唯一约束：

- `UNIQUE(idempotency_key)`

索引：

- `INDEX(coupon_code_id, reservation_status, expires_at_ms)`
- `INDEX(order_id, reservation_status)`
- `INDEX(subject_type, subject_id, created_at_ms DESC)`

### `ai_marketing_coupon_redemption`

建议字段：

- `coupon_redemption_id TEXT PRIMARY KEY`
- `coupon_code_id TEXT NOT NULL`
- `coupon_reservation_id TEXT`
- `coupon_template_id TEXT NOT NULL`
- `marketing_campaign_id TEXT`
- `order_id TEXT`
- `payment_order_id TEXT`
- `payment_event_id TEXT`
- `project_id TEXT`
- `account_id INTEGER`
- `redemption_status TEXT NOT NULL`
- `subsidy_amount_minor INTEGER NOT NULL DEFAULT 0`
- `currency_code TEXT`
- `pricing_adjustment_id TEXT`
- `benefit_lot_id INTEGER`
- `idempotency_key TEXT NOT NULL`
- `redeemed_at_ms INTEGER NOT NULL`

唯一约束：

- `UNIQUE(idempotency_key)`

索引：

- `INDEX(order_id, redeemed_at_ms DESC)`
- `INDEX(project_id, redeemed_at_ms DESC)`
- `INDEX(coupon_code_id, redeemed_at_ms DESC)`
- `INDEX(coupon_template_id, redeemed_at_ms DESC)`

### `ai_marketing_coupon_rollback`

建议字段：

- `coupon_rollback_id TEXT PRIMARY KEY`
- `coupon_redemption_id TEXT NOT NULL`
- `rollback_type TEXT NOT NULL`
- `rollback_scope TEXT NOT NULL`
- `reason_code TEXT NOT NULL`
- `refund_order_id TEXT`
- `returned_items_json TEXT`
- `restored_budget_minor INTEGER NOT NULL DEFAULT 0`
- `restored_inventory_count INTEGER NOT NULL DEFAULT 0`
- `rollback_status TEXT NOT NULL`
- `created_at_ms INTEGER NOT NULL`

索引：

- `INDEX(coupon_redemption_id, created_at_ms DESC)`
- `INDEX(refund_order_id, created_at_ms DESC)`

## 9.3 兼容层与迁移设计

现有表：

- `ai_coupon_campaigns`

迁移策略不是一次性删掉，而是三步走：

### 阶段 A：冻结兼容层

- 将 `ai_coupon_campaigns` 视为兼容输入视图
- 新建营销核心表
- 旧 `admin/coupons` API 继续可用

### 阶段 B：双写与适配

- 创建旧 coupon 时，同时生成新模板/活动/码
- 门户验券与下单逻辑逐步改走 marketing facade
- 旧查询接口通过 compatibility projector 生成旧视图

### 阶段 C：核心切换

- `portal/commerce/quotes` 改走 validate/reserve
- `portal/commerce/orders` 改走 confirm_redemption
- `refund` 和 `cancel` 改走 rollback
- `ai_coupon_campaigns` 最终只保留兼容读或迁移导出

## 10. 一致性、事务与幂等设计

### 10.1 事务原则

以下操作必须在单事务中完成：

- reservation 建立 + budget hold
- redemption 建立 + budget consume + code state 转移
- rollback 建立 + budget restore + code availability 恢复

### 10.2 幂等原则

以下动作必须强制要求 `idempotency_key`：

- reserve
- confirm redemption
- release reservation
- rollback redemption
- refund synchronization

### 10.3 并发控制

生产推荐 PostgreSQL 语义：

- 通过唯一索引和条件更新控制唯一核销
- 对 `coupon_code` 和 `campaign_budget` 使用乐观版本号或行锁
- reservation 过期扫描器异步释放

本地 SQLite 模式：

- 用唯一约束与状态检查模拟
- 不把 SQLite 作为高并发生产语义标准

## 11. 安全设计

### 11.1 券码安全

- 券码查找使用 `normalized_code_hash`
- 后台默认只显示 `prefix + suffix`
- 明文导出需要高级权限与审计日志
- 支持 blocklist

### 11.2 风控策略

第一阶段必须支持：

- 同用户短时失败次数限制
- 同项目短时验券频率限制
- 同 IP / device 指纹聚集告警
- 同支付单重复确认拦截
- 同退款单重复回滚拦截

### 11.3 权限模型

后台角色至少区分：

- `marketing_admin`
- `marketing_operator`
- `finance_auditor`
- `support_readonly`

## 12. 性能设计

### 12.1 读路径优化

验券是热点路径，建议：

- 模板规则快照缓存
- 活动与预算摘要缓存
- code hash 精确索引
- 订单试算结果短时缓存

### 12.2 写路径优化

核销是强一致路径，建议：

- reservation 与 redemption 分离
- 扣预算走 ledger + aggregate 双写
- 通过 outbox 异步投递分析与通知

### 12.3 高并发策略

高并发活动场景下：

- 共享码使用 campaign budget + total cap 双重约束
- 唯一码使用批次库存和状态位控制
- reservation 设置短 TTL
- 异步清理过期 reservation

## 13. 可观测性设计

必须新增以下指标：

- `sdkwork_marketing_validations_total`
- `sdkwork_marketing_validation_failures_total{reason}`
- `sdkwork_marketing_reservations_active`
- `sdkwork_marketing_reservations_expired_total`
- `sdkwork_marketing_redemptions_total{outcome}`
- `sdkwork_marketing_rollbacks_total{reason}`
- `sdkwork_marketing_budget_exhausted_total`
- `sdkwork_marketing_fraud_flags_total{type}`
- `sdkwork_marketing_validate_latency_ms`
- `sdkwork_marketing_redeem_latency_ms`

必须保留以下结构化日志字段：

- `coupon_template_id`
- `campaign_id`
- `coupon_code_id`
- `reservation_id`
- `redemption_id`
- `rollback_id`
- `order_id`
- `payment_event_id`
- `project_id`
- `subject_id`
- `idempotency_key`

## 14. 与现有商业系统的集成策略

### 14.1 与订单中心集成

订单记录需要增加：

- `coupon_reservation_id`
- `coupon_redemption_id`
- `discount_total_minor`
- `subsidy_total_minor`
- `marketing_campaign_id`

### 14.2 与支付系统集成

支付确认成功前：

- reservation 仍为 `active`

支付确认成功后：

- confirm redemption

支付失败或超时：

- release reservation

### 14.3 与账户和额度系统集成

grant 型优惠必须通过标准账户能力下发：

- 创建 benefit lot
- 写 ledger evidence
- 记录 order linkage

不能再通过“直接改 remaining units”之类的旁路逻辑完成。

### 14.4 与退款系统集成

退款系统应触发：

- 读取 order -> redemption linkage
- 评估是否允许 rollback
- 全额退款则 full rollback
- 部分退款则 partial rollback
- 已消费 grant 进入冲突处理策略

## 15. 推荐的落地步骤

### Phase 1：营销核心建模

- 新建 marketing domain 与 store
- 新建模板、活动、批次、码、reservation、redemption、rollback 表
- 保留旧 coupon 兼容接口

### Phase 2：验券与预占

- 把 portal quote 接入 `validate`
- 下单前引入 `reserve`
- 支付失败释放 reservation

### Phase 3：正式核销与账务联动

- 订单成功后 `confirm redemption`
- 统一下发折扣与 grant 型权益
- 建立 outbox 事件

### Phase 4：退款与回滚

- 支持 full rollback
- 支持 partial rollback
- 支持已消费权益的冲突处理

### Phase 5：后台运营与归因

- 模板/活动/批次/码/核销/预算工作台
- 风控与审计
- 裂变与邀请

## 16. 必须优先修复的现有问题

在正式建设营销内核前，现有实现至少要先避免以下风险：

1. 不再从 `discount_label` 解析折扣比例。
2. 不再用“重写 coupon 行并减 `remaining`”作为最终核销机制。
3. 不再让 quote 与 create order 共用非幂等逻辑。
4. 给优惠相关动作补 `idempotency_key`。
5. 给优惠核销补不可变 `redemption record`。

## 17. 本设计的最终判断

对标 Stripe、Voucherify、Talon.One 后，当前系统最重要的改造不是“再加几个 coupon 字段”，而是把优惠券系统升级成真正的营销执行内核。

这套内核必须满足以下终态：

- 一个模板可发行多批多码
- 校验、预占、核销、回滚全链路可追踪
- 与订单、支付、账户、退款、预算、归因形成完整闭环
- 在高并发、多活动、退款回滚、财务审计场景下依然可靠

## 18. 官方对标资料

以下资料为本稿采用的官方参考：

- Stripe Coupons and promotion codes: https://docs.stripe.com/billing/subscriptions/coupons
- Stripe Checkout discounts: https://docs.stripe.com/payments/checkout/discounts
- Voucherify Rollback Redemption: https://docs.voucherify.io/api-reference/redemptions/rollback-redemption
- Voucherify Validate Stackable Discounts: https://docs.voucherify.io/api-reference/validations/validate-stackable-discounts
- Voucherify Validation rule reference: https://docs.voucherify.io/optimize/validation-rules-reference
- Voucherify Discount types and effects: https://docs.voucherify.io/build/discount-types-and-effects
- Talon.One Customer session entity: https://docs.talon.one/docs/dev/concepts/entities/customer-sessions
- Talon.One Integration API: https://docs.talon.one/integration-api
- Talon.One Coupon and discount campaigns: https://docs.talon.one/docs/product/tutorials/coupons

## 19. 建议的后续文档

本设计之后，建议继续输出三份实施文档：

1. `优惠券营销内核数据库迁移计划`
2. `优惠券核销与退款回滚事务设计`
3. `Admin/Portal 优惠券产品交互与 API 实施计划`
