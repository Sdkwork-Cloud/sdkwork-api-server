# S06 Portal与Public-API产品化收口 - 2026-04-10

## 1. 架构对齐

- 对齐文档：
  - `docs/架构/166-*`
  - `docs/架构/133-*`
  - `docs/架构/03-*`
  - `docs/step/100-*`
- 对齐能力：
  - Portal 自助商业化闭环
  - Public API 语义稳定
  - 单一 OpenAPI / TS / UI 契约
  - product + coupon + account 一体化体验
- 本轮必须兑现：
  - Portal 与 Public API 基于同一套商业化主模型输出。
  - `QuoteKind / TransactionKind / ApiProductKind` 不再继续自由文本扩散。
  - coupon 适用、核销、到账权益在自助面可解释。
- 本轮显式后延：
  - partner/channel 门户
  - 独立 `portal-commerce` 物理拆包
- 完成后应兑现：
  - `166` 中 Portal 一体化体验与 Public API 标准面，不再分裂出第二套自助语义。
- 执行后必须回写：
  - `docs/架构/166-*`
  - `docs/架构/133-*`
  - `docs/review/*`

## 2. Current / Target / Deferred

- Current：
  - Portal 体验分散在 `billing / credits / recharge / settlements / account` 多包中，当前仓库并不存在独立 `portal-commerce` 包。
  - Rust / OpenAPI 侧仍残留自由文本商业语义，TS 类型相对更前但未形成单一事实源。
  - `my-coupons`、`benefit-lots`、下单报价、充值入口尚未形成统一自助心智。
  - Portal 营销自助仍偏 user/code 视角，`Account` 范围能力不足。
- Target：
  - 保留现有 Portal 包布局，但统一 route taxonomy、repository contract、TS types 与页面语义。
  - Portal 用户能在同一套产品化流程内完成产品浏览、试算、用券、到账权益查看与历史追踪。
  - Public API 对外稳定暴露 `market / marketing / commercial-account` 语义，不再复制第二套前端专用 coupon 逻辑。
- Deferred：
  - 更完整的合作伙伴分发入口
  - 更复杂的 BFF 组合编排

## 3. 设计

- 不默认新增 `sdkwork-router-portal-commerce` 包；先在现有 `billing / credits / recharge / settlements / account` 上完成语义收口。
- Portal 是自助体验层，不拥有独立商业真值；所有商业语义必须来自 backend canonical contract。
- `billing` 作为购买与订单中心，`credits` 作为券与权益自助中心，`recharge` 作为充值与补能入口，`settlements` 作为账务历史，`account` 作为主体上下文。
- 对外 API 与 Portal API 必须复用同一组 typed enum / union，不允许 Rust、OpenAPI、TS、UI 各自发明常量。

## 4. 实施落地规划

- 主写入范围：
  - `crates/sdkwork-api-interface-portal/src/commerce.rs`
  - `crates/sdkwork-api-interface-portal/src/billing.rs`
  - `crates/sdkwork-api-interface-portal/src/marketing.rs`
  - `crates/sdkwork-api-interface-portal/src/marketing_handlers.rs`
  - `crates/sdkwork-api-interface-portal/src/marketing_types.rs`
  - `crates/sdkwork-api-interface-portal/src/openapi.rs`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/*`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/*`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/*`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-settlements/src/*`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/*`
- 实施顺序：
  1. 冻结 Portal / Public API 术语、路由面与 typed contract。
  2. 收敛 Rust -> OpenAPI -> TS types 的单向生成与校验链。
  3. 改造 portal repositories，统一报价、用券、到账权益读取语义。
  4. 重组页面与入口说明，补齐 `Account` 范围自助能力。
  5. 产出证据、回写架构与发布前体验说明。

## 5. 串行与并行

- Step 属性：可与 `S05` 并行；本步内部先 contract，再 repository/types，再页面。
- 共享文件 Owner：
  - `crates/sdkwork-api-interface-portal/src/openapi.rs`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`

| 车道 | 子域 | 写集 | 前置 | 交付物 |
| --- | --- | --- | --- | --- |
| `L1` | backend portal/public contracts | `interface-portal/*` | 无 | typed API / OpenAPI contract |
| `L2` | shared portal types + sdk facade | `portal-api`、`portal-types` | `L1` | TS contract 与 repository facade |
| `L3` | purchase/recharge surfaces | `portal-billing`、`portal-recharge` | `L2` | 产品购买与充值体验 |
| `L4` | coupon/account surfaces | `portal-credits`、`portal-account`、`portal-settlements` | `L2` | 领券、用券、到账权益与历史 |
| `LV` | verification/docs | tests、review、docs | `L2-L4` | 契约证据与体验验收 |

## 6. 测试计划

- Rust：
  - `cargo test -p sdkwork-api-interface-portal portal_commerce -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal openapi_route -- --nocapture`
- TypeScript / 前端：
  - `pnpm --dir apps/sdkwork-router-portal typecheck`
  - `node --test apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - `node --test apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`
  - `node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- 契约核验：
  - `rg -n "subscription_plan|coupon_redemption|benefit-lots|my-coupons|Account" crates/sdkwork-api-interface-portal apps/sdkwork-router-portal -g "*.rs" -g "*.ts" -g "*.tsx"`

## 7. 结果验证

- 主证据：
  - Portal 可以解释“这是什么产品、用了哪张券、到账了什么权益、写入了哪个账户”。
  - Public API 与 Portal API 对同一商业行为输出一致 contract。
- 辅证据：
  - route、OpenAPI、TS types、repository、页面文案使用同一套语义词典。
- 架构回写：
  - `166` 第 4.6、8.2、8.3 与 `133` 中自助面能力口径同步
- 通过条件：
  - Portal 不再维护第二套 coupon / quote 逻辑。
  - `Account` 范围的自助领取、持有、到账查看可走通。

## 8. 检查点

- `CPC06-1`：Portal / Public API typed contract 冻结并与 OpenAPI 对齐。
- `CPC06-2`：产品购买、券使用、到账权益、自助历史形成一体化体验。
- `CPC06-3`：现有多包布局下完成语义收口，未引入新的 Portal 真值分叉。

## 9. 风险与回退

- 风险：
  - Portal 包布局分散，容易在页面层重新复制业务判断。
  - Rust / OpenAPI / TS / UI 常量漂移会导致前后端兼容事故。
- 灰度：
  - 保留旧入口 URL 或 facade，先统一 contract 与 repository，再替换页面编排。
- 回退触发条件：
  - Portal 前后端 contract 仍无法稳定对齐。
  - 页面改造后用户无法定位券归属或到账权益。
- 回退动作：
  - 回退新页面编排，保留统一 contract/types 与 repository 收口成果。

## 10. 完成定义

- Portal 与 Public API 已围绕同一商业化主模型提供可解释、可测试、可迭代的自助能力。
- `166` 第 8.2、8.3、13 中自助侧能力已具备主实现与验证入口。

## 11. 下一步准入

- 准入结论：`conditional-go`
- 允许启动的下一步：`S07`，前提是 `S05` 同步完成并完成共享 contract 收口
- 仍需阻塞的项：
  - 任何继续在 Portal 页面层手写第二套 coupon / quote 规则的改动
