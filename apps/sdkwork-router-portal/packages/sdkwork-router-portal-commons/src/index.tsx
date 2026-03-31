import * as CheckboxPrimitive from '@radix-ui/react-checkbox';
import * as DialogPrimitive from '@radix-ui/react-dialog';
import * as LabelPrimitive from '@radix-ui/react-label';
import { Slot } from '@radix-ui/react-slot';
import * as TabsPrimitive from '@radix-ui/react-tabs';
import { cva, type VariantProps } from 'class-variance-authority';
import { clsx, type ClassValue } from 'clsx';
import { Check, Search as SearchIcon, X } from 'lucide-react';
import {
  createContext,
  forwardRef,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ComponentPropsWithoutRef,
  type ElementRef,
  type ReactNode,
} from 'react';
import { twMerge } from 'tailwind-merge';

import { setActivePortalFormatLocale } from './format-core';
import { setActivePortalCoreLocale } from './i18n-core';

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

export type PortalLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, string | number>;

type PortalI18nContextValue = {
  locale: PortalLocale;
  setLocale: (locale: PortalLocale) => void;
  t: (text: string, values?: TranslationValues) => string;
};

const PORTAL_I18N_STORAGE_KEY = 'sdkwork-router-portal.locale.v1';

const portalMessages: Record<Exclude<PortalLocale, 'en-US'>, Record<string, string>> = {
  'zh-CN': {
    Close: '关闭',
    'More filters': '更多筛选',
    'Hide filters': '收起筛选',
    Language: '语言',
    English: '英文',
    'Simplified Chinese': '简体中文',
    Pending: '待处理',
    'Search usage': '搜索使用记录',
    'Time range': '时间范围',
    Refresh: '刷新',
    'Review billing': '查看账单',
    'Manage keys': '管理密钥',
    'Last 24 hours': '最近 24 小时',
    'Last 7 days': '最近 7 天',
    'Last 30 days': '最近 30 天',
    'All time': '全部时间',
    Settings: '设置',
    'Portal workspace settings': '门户工作区设置',
    'Theme, sidebar, and shell preferences': '主题、侧边栏与壳层偏好',
    'Sign out': '退出登录',
    'End this portal session on the current device': '在当前设备结束本次门户会话',
    'Search settings...': '搜索设置...',
    'No settings found.': '未找到匹配设置。',
    Appearance: '外观',
    'Theme mode and Theme color': '主题模式与主题颜色',
    Navigation: '导航',
    'Sidebar behavior and Sidebar navigation': '侧边栏行为与侧边栏导航',
    Workspace: '工作区',
    'Language and workspace preferences': '语言与工作区偏好',
    'Language and locale': '语言与区域',
    'Choose the portal workspace language. Shared shell copy and locale-aware formatting update immediately.': '选择门户工作区语言。共享壳层文案和区域格式会立即同步更新。',
    'Theme mode': '主题模式',
    'Theme mode stays synchronized across header, sidebar, content surfaces, and dialogs.': '主题模式会在顶部栏、侧边栏、内容面板与弹窗之间保持同步。',
    'Theme color': '主题颜色',
    'Theme color updates the accent surfaces without changing the claw-style shell contract.': '主题颜色会更新强调色表面，同时不改变 claw 风格壳层契约。',
    'Sidebar behavior': '侧边栏行为',
    'Keep the left rail aligned with claw-studio while preserving the portal route set.': '在保留门户路由集的同时，使左侧导航栏与 claw-studio 保持一致。',
    'Show or hide workspace modules while keeping the left rail compact and stable.': '在保持左侧导航栏紧凑稳定的同时，显示或隐藏工作区模块。',
    'Reset shell preferences': '重置壳层偏好',
    'Workspace preferences': '工作区偏好',
    'Keep workspace identity and shell reset controls in one place.': '将工作区身份信息与壳层重置控制统一收纳在同一处。',
    'Portal workspace': '门户工作区',
    'Awaiting workspace session': '等待工作区会话',
    'Portal tenant': '门户租户',
    'Portal operator': '门户操作员',
    Operator: '操作员',
    Light: '浅色',
    Dark: '深色',
    System: '跟随系统',
    'Search ledger': '搜索账本',
    'Financial account': '财务账户',
    'Financial account posture will appear after the portal loads billing summary and ledger evidence.': '门户加载账单摘要和账本凭证后，这里会显示财务账户状态。',
    'Preparing account': '正在准备账户',
    'No ledger entries recorded yet.': '暂无账本条目。',
    'No ledger entries for this slice': '当前视图下没有账本条目',
    'Open credits': '打开额度',
    'Open usage': '打开使用记录',
    'Open account': '打开账户',
    'Search offers or ledger': '搜索优惠或账本',
    'View mode': '视图模式',
    'Offer state': '优惠状态',
    'All offers': '全部优惠',
    'Eligible offers': '可领取优惠',
    'Expiring soon': '即将到期',
    'Archived offers': '已归档优惠',
    'Redeem now': '立即兑换',
    'Redeeming...': '兑换中...',
    Offers: '优惠',
    Ledger: '账本',
    'Loading preview...': '正在加载预览...',
    'Checkout preview': '结算预览',
    'Create checkout': '创建结算',
    'Creating checkout...': '正在创建结算...',
    'Refresh preview': '刷新预览',
    'Search order lifecycle': '搜索订单生命周期',
    'Order lane': '订单视图',
    'All orders': '全部订单',
    'Pending payment queue': '待支付队列',
    'Failed payment': '支付失败',
    'Order timeline': '订单时间线',
    'Decision support': '决策支持',
    'Order workbench': '订单工作台',
    'Loading session...': '正在加载会话...',
    'Open session': '打开会话',
    'Settling...': '结算中...',
    'Settle order': '结算订单',
    'Canceling...': '取消中...',
    'Cancel order': '取消订单',
    'No checkout methods remain': '已无可用结算方式',
    'Applying settlement...': '正在应用结算结果...',
    'Simulate provider settlement': '模拟提供商结算',
    'Applying failure...': '正在应用失败结果...',
    'Simulate provider failure': '模拟提供商失败',
    'Applying cancel...': '正在应用取消结果...',
    'Simulate provider cancel': '模拟提供商取消',
    'No checkout session selected': '尚未选择结算会话',
    'No pending payment orders for this slice': '当前视图下没有待支付订单',
    'No failed payment orders for this slice': '当前视图下没有支付失败订单',
    'No timeline orders for this slice': '当前视图下没有时间线订单',
    'No orders for this slice': '当前视图下没有订单',
    'Total requests': '总请求数',
    'Total tokens': '总 Token 数',
    'Total spend': '总消费金额',
    'Average latency': '平均耗时',
    Channel: '通道',
    Model: '模型',
    'Previous page': '上一页',
    'Next page': '下一页',
    'Create API key': '创建 API Key',
    'Search API keys': '搜索 API Key',
    'Search gateway evidence': '搜索网关证据',
    'Search routing evidence': '搜索路由证据',
    'Workbench lane': '工作台视图',
    'Operational focus': '运营焦点',
    'Loading the router product command center and current launch posture...': '正在加载路由产品指挥中心和当前启动姿态...',
    'The portal now exposes compatibility, deployment modes, runtime evidence, and commercial runway as one operator-facing product surface.': '门户现已将兼容性、部署模式、运行证据和商业跑道整合为一个面向运营的产品界面。',
    'The command center could not load the current gateway posture.': '指挥中心暂时无法加载当前网关姿态。',
    'The command center is showing the latest compatibility, runtime, and commercial posture.': '指挥中心当前展示的是最新的兼容性、运行态与商业姿态。',
    'The command center could not refresh the current gateway posture.': '指挥中心暂时无法刷新当前网关姿态。',
    'Restarting the embedded desktop runtime and refreshing live service posture...': '正在重启内嵌桌面运行时并刷新实时服务姿态...',
    'Desktop runtime restarted successfully and the command center has been refreshed with the latest service posture.': '桌面运行时已成功重启，指挥中心也已刷新为最新服务姿态。',
    'Desktop runtime restart failed before the command center could refresh.': '在指挥中心刷新前，桌面运行时重启失败。',
    'The command center will appear once the portal finishes assembling the product-facing router view.': '门户完成组装面向产品的路由视图后，这里将显示指挥中心。',
    'Preparing gateway command center': '正在准备网关指挥中心',
    'Refreshing the full command center posture...': '正在刷新完整指挥中心姿态...',
    'Refreshing command center...': '正在刷新指挥中心...',
    'Refresh command center': '刷新指挥中心',
    'Refreshing service health and gateway evidence...': '正在刷新服务健康与网关证据...',
    'Refreshing service health...': '正在刷新服务健康...',
    'Refresh service health': '刷新服务健康',
    'Gateway posture': '网关姿态',
    'Command workbench': '指挥工作台',
    'Launch readiness': '上线就绪度',
    'Critical blockers and watchpoints stay visible before launch traffic expands.': '关键阻塞项和观察点会在流量扩张前持续可见。',
    'Desktop runtime': '桌面运行时',
    'Desktop runtime cards keep the local bind story visible while Restart desktop runtime remains intentionally narrow.': '桌面运行时卡片会持续展示本地绑定姿态，而“重启桌面运行时”操作保持为明确且收敛的单一动作。',
    'Deployment playbooks': '部署作战手册',
    'Mode switchboard': '模式切换面板',
    'Keep the product launch path readable whether the router is running on one machine or transitioning into a hosted topology.': '无论路由器运行在单机还是正在迁移到托管拓扑，产品上线路径都保持清晰可读。',
    'Topology playbooks': '拓扑手册',
    'Promote runtime documentation into executable rollout playbooks that operators can apply immediately.': '将运行时说明提升为可执行的发布手册，方便运营人员立即落地。',
    'Commercial runway': '商业化跑道',
    'Commerce catalog': '商业目录',
    'Active membership, recharge packs, and coupon campaigns remain visible as backend product inventory instead of drifting into frontend-only launch copy.': '生效中的会员、充值包和优惠券活动会作为后端商品目录持续可见，而不是漂移成仅前端文案。',
    'Launch actions': '启动动作',
    'Open API Keys, Open Routing, and Open Billing are the three fastest actions for turning this command center into a real launch workflow.': '打开 API Keys、Routing 和 Billing 是把这个指挥中心转化为真实上线工作流的三个最快动作。',
    'Loading routing posture...': '正在加载路由姿态...',
    'Routing workbench is synced with the latest project posture, provider order, and decision evidence.': '路由工作台已同步最新的项目姿态、Provider 顺序和决策证据。',
    'Preset applied locally. Save posture when the updated routing shape looks right.': '预设已在本地应用。确认新的路由形态正确后再保存姿态。',
    'Active preset': '当前预设',
    'Apply preset': '应用预设',
    'Provider order changed locally. Save posture to publish the new fallback order.': 'Provider 顺序已在本地调整。保存姿态后才会发布新的回退顺序。',
    'Move up': '上移',
    'Move down': '下移',
    'Default provider updated locally. Save posture to publish the change.': '默认 Provider 已在本地更新。保存姿态后才会发布变更。',
    'Set default': '设为默认',
    'Saving routing preferences for this project...': '正在保存当前项目的路由偏好...',
    'Routing posture saved. The workbench now reflects the updated provider order and guardrails.': '路由姿态已保存。工作台现已反映新的 Provider 顺序和护栏设置。',
    'Previewing the active route...': '正在预览当前路由...',
    'Preview updated with the current routing posture and added to the evidence stream.': '预览已按当前路由姿态更新，并已加入证据流。',
    'Routing posture will appear once the portal finishes loading project summary, provider options, and decision evidence.': '门户完成加载项目摘要、Provider 选项和决策证据后，将显示路由姿态。',
    'Preparing routing workbench': '正在准备路由工作台',
    'Edit routing posture': '编辑路由姿态',
    'Save posture after adjusting profile label, strategy, regional preference, and reliability guardrails.': '调整配置标签、策略、区域偏好和可靠性护栏后再保存姿态。',
    'Routing profile label': '路由配置标签',
    Strategy: '策略',
    'Predictable order': '可预测顺序',
    'Traffic distribution': '流量分配',
    'Reliability guardrails': '可靠性护栏',
    'Regional preference': '区域偏好',
    'Max cost': '最大成本',
    'Max latency ms': '最大延迟毫秒',
    'Preferred region': '偏好区域',
    Auto: '自动',
    'Default provider': '默认 Provider',
    'Auto fallback': '自动回退',
    'Require healthy providers': '要求 Provider 健康可用',
    'Reliability guardrails bias routing toward healthy, lower-risk providers before traffic leaves the workspace.': '在流量离开工作区前，可靠性护栏会优先选择更健康、风险更低的 Provider。',
    Cancel: '取消',
    'Saving...': '保存中...',
    'Save posture': '保存姿态',
    'Preview route': '预览路由',
    'Preview route inputs are stored separately from the saved posture so operators can test scenarios before traffic shifts.': '预览路由输入与已保存姿态分开存储，方便操作人员在切换流量前测试不同场景。',
    Capability: '能力',
    'Requested model': '请求模型',
    'Requested region': '请求区域',
    'Selection seed': '选择种子',
    'Optional deterministic seed': '可选的确定性种子',
    'Running preview...': '预览运行中...',
    'Run preview': '运行预览',
    'Edit posture': '编辑姿态',
    'Validate with a key': '用密钥验证',
    'Routing posture': '路由姿态',
    'Routing workbench': '路由工作台',
    'Routing workbench keeps Provider roster, Preset catalog, and Evidence stream inside one operator table while edit and preview actions stay inside focused dialogs.': '路由工作台将 Provider 名录、预设目录和证据流集中到一张运营表格中，同时把编辑和预览动作保留在聚焦弹窗内。',
    'Guardrail posture keeps cost, latency, regional preference, and the latest routing signals readable before you publish changes.': '护栏姿态会在你发布变更前保持成本、延迟、区域偏好以及最新路由信号的可读性。',
    'Guardrail posture': '护栏姿态',
    'Latest routing signals': '最新路由信号',
    'Preview and live traces stay adjacent to guardrails so posture changes remain explainable without secondary tabs.': '预览与实时轨迹会紧邻护栏展示，无需次级标签页也能解释姿态变化。',
    '{count} signals': '{count} 条信号',
    'No routing signals yet': '暂无路由信号',
    'Run a preview or wait for live traffic to collect routing signals.': '运行一次预览，或等待真实流量进入后在这里查看路由信号。',
    'Preview outcome keeps the selected provider, fallback path, and provider assessments visible before traffic posture is saved.': '在保存流量姿态之前，预览结果会持续展示所选 Provider、回退路径以及 Provider 评估。',
    'Preview outcome': '预览结果',
    'Candidate assessments': '候选评估',
    'Selection evidence stays operationally readable so support teams can validate health, latency, and policy posture before rollout.': '选择证据会保持运营可读，方便支持团队在发布前核验健康度、延迟和策略姿态。',
    'Degraded fallback': '降级回退',
    'Guardrails applied': '已应用护栏',
    'Preview only': '仅预览',
    'Not available': '不可用',
    'The preview did not expose additional assessment detail for this provider.': '本次预览未返回该 Provider 的更多评估细节。',
    'Run a preview to inspect provider-level candidate assessments.': '运行一次预览以查看 Provider 级别的候选评估。',
    'No preview assessments yet': '暂无预览评估',
    'Clear filters': '清空筛选',
    'Active posture': '当前策略姿态',
    'Preview model': '预览模型',
    'Evidence entries': '证据条目',
    'Routing strategy is translated into user-facing posture language instead of raw enum names.': '路由策略会转换成面向用户的姿态语言，而不是直接暴露原始枚举名。',
    'The provider chosen by the latest routing preview.': '最近一次路由预览选中的 Provider。',
    'Selection reason': '选择原因',
    'Top-ranked eligible provider': '排名最高的可用 Provider',
    'The current preview explains why the selected provider won the route.': '当前预览会解释所选 Provider 为什么胜出。',
    'Candidate path': '候选路径',
    'No candidates': '无候选项',
    'Candidate order remains visible so fallback posture is explainable.': '候选顺序保持可见，以便解释回退姿态。',
    'SLO posture': 'SLO 姿态',
    'No active guardrails': '当前无生效护栏',
    'Matched policy {policyId}.': '命中策略 {policyId}。',
    'No routing policy matched the current preview inputs.': '当前预览输入未命中任何路由策略。',
    'Unknown provider': '未知 Provider',
    Unavailable: '不可用',
    unknown: '未知',
    healthy: '健康',
    unhealthy: '异常',
    'Platform fallback': '平台回退',
    'Adaptive routing': '自适应路由',
    Open: '开放',
    'Max latency': '最大延迟',
    'Selected provider': '已选 Provider',
    'Selection evidence is available from the current routing trace.': '可从当前路由轨迹查看选择证据。',
    'No matched policy': '未命中策略',
    'Priority #{priority}': '优先级 #{priority}',
    Default: '默认',
    Ordered: '有序',
    'Default provider stays available as the stable fallback when several providers remain eligible.': '当多个 Provider 仍可用时，默认 Provider 会作为稳定回退持续可用。',
    'Ordered providers keep deterministic failover readable for operators and support teams.': '有序 Provider 让运维与支持团队能够清晰理解确定性故障切换。',
    '{source} used {strategy}{regionSuffix}.': '{source} 使用了 {strategy}{regionSuffix}。',
    ' in {region}': '，区域 {region}',
    '{visible} of {total} rows visible': '显示 {visible} / {total} 行',
    'Focus: {focus}': '焦点：{focus}',
    All: '全部',
    'Preset catalog': '预设目录',
    'Provider roster': 'Provider 名录',
    'Evidence stream': '证据流',
    'Available presets': '可用预设',
    Preview: '预览',
    Degraded: '降级',
    Subject: '主题',
    'Operational detail': '运营详情',
    'Preset catalog converts backend routing strategy enums into product choices that an operator can apply without reading implementation details.': '预设目录将后端路由策略枚举转换为产品化选项，让操作人员无需阅读实现细节也能直接应用。',
    'No routing presets in this slice': '当前视图下没有路由预设',
    'Adjust the operational focus or search to reveal a different routing preset.': '调整运营焦点或搜索条件以查看其他路由预设。',
    'All presets': '全部预设',
    Available: '可用',
    'Routing signal': '路由信号',
    'Selection detail': '选择详情',
    Trace: '追踪',
    'Evidence stream keeps preview and live routing traces on one operational table instead of splitting them across tabs.': '证据流将预览与在线路由轨迹保留在一张运营表格中，而不是拆到多个标签页。',
    'No routing evidence in this slice': '当前视图下没有路由证据',
    'Run a preview or send live traffic and routing evidence will appear here.': '运行一次预览或发送真实流量后，路由证据会出现在这里。',
    'All evidence': '全部证据',
    'Preview traces': '预览轨迹',
    'Live traces': '在线轨迹',
    Guardrailed: '已加护栏',
    'Channel and order': '通道与顺序',
    'Routing role': '路由角色',
    'Provider roster keeps ordered fallback, default provider, and channel coverage inside one workbench so operations can adjust posture without digging through forms.': 'Provider 名录将顺序回退、默认 Provider 和通道覆盖集中在一个工作台中，方便运营人员无需深入表单也能调整姿态。',
    'No providers in this slice': '当前视图下没有 Provider',
    'Routing provider options will appear once the project summary is available.': '项目摘要可用后，这里会显示路由 Provider 选项。',
    'All providers': '全部 Provider',
    'Ordered providers': '有序 Provider',
    Live: '生产',
    Staging: '预发布',
    Test: '测试',
    'Service health': '服务健康',
    'Compatibility routes': '兼容路由',
    'Rate-limit policies': '限流策略',
    'Rate-limit windows': '限流窗口',
    'Verification commands': '验证命令',
    'All routes': '全部路由',
    'Direct gateway': '直连网关',
    'Translated routes': '转译路由',
    'Desktop setup': '桌面配置',
    Tool: '工具',
    'Route family': '路由族',
    'Execution truth': '执行形态',
    'Operator outcome': '运维结果',
    'No compatibility routes in this slice': '当前视图下没有兼容路由',
    'Adjust the workbench lane or search to reveal a different protocol family.': '调整工作台视图或搜索条件以查看其他协议族。',
    'Compatibility routes keep Codex, Claude Code, Gemini, and OpenClaw onboarding on one shared gateway posture.': '兼容路由会将 Codex、Claude Code、Gemini 与 OpenClaw 的接入统一到同一个网关姿态中。',
    'All policies': '全部策略',
    Enabled: '启用',
    Disabled: '禁用',
    Policy: '策略',
    Scope: '范围',
    Limit: '限制',
    'Operator notes': '运维备注',
    'Project rate-limit policy posture was last checked {checkedAt}.': '项目限流策略姿态最近检查于 {checkedAt}。',
    'No rate-limit policies in this slice': '当前视图下没有限流策略',
    'The workspace does not currently expose a matching project-scoped rate-limit policy.': '当前工作区暂未暴露匹配的项目级限流策略。',
    'No operator notes were attached to this policy.': '该策略未附带运维备注。',
    'All windows': '全部窗口',
    'Within limit': '未超限',
    'Over limit': '已超限',
    Window: '窗口',
    'Window detail': '窗口详情',
    'Live rate-limit windows were last checked {checkedAt}.': '实时限流窗口最近检查于 {checkedAt}。',
    'No live windows in this slice': '当前视图下没有实时窗口',
    'Live rate-limit pressure will appear here once gateway activity is present.': '一旦网关产生流量，这里就会显示实时限流压力。',
    'All commands': '全部命令',
    'OpenAI-compatible': 'OpenAI 兼容',
    'Anthropic Messages': 'Anthropic Messages',
    Gemini: 'Gemini',
    Check: '检查项',
    Protocol: '协议',
    'Verification command': '验证命令',
    'Verification commands turn gateway activation into an executable launch checklist instead of static documentation.': '验证命令会把网关启用流程变成可执行的上线清单，而不是静态文档。',
    'No verification commands in this slice': '当前视图下没有验证命令',
    'Change the focus or search to reveal another verification route family.': '切换焦点或搜索条件以查看更多验证路由族。',
    'All services': '全部服务',
    Healthy: '健康',
    Unreachable: '不可达',
    Service: '服务',
    'Health route': '健康检查路由',
    'Runtime signal': '运行时信号',
    'Operator detail': '运维详情',
    'Live service health was last checked {checkedAt}.': '实时服务健康最近检查于 {checkedAt}。',
    'No service health checks in this slice': '当前视图下没有服务健康检查',
    'Refresh service health to pull the latest runtime evidence into the command workbench.': '刷新服务健康即可将最新运行证据拉入指挥工作台。',
    'project-wide': '项目级',
    'No latency sample': '暂无延迟样本',
    'Updated {checkedAt}': '更新于 {checkedAt}',
    'Started {checkedAt}': '开始于 {checkedAt}',
    '{windowSeconds}s window · ends {endsAt}': '{windowSeconds} 秒窗口 · 结束于 {endsAt}',
    'Ready to run': '可立即运行',
    'Mode switchboard and topology playbooks keep the path from desktop mode to hosted server mode explicit.': '模式切换面板与拓扑手册会清晰展示从桌面模式到托管服务模式的迁移路径。',
    'Commerce catalog and launch actions keep access, routing, and billing runway on one commercial surface.': '商业目录与启动动作会将访问、路由与计费跑道统一呈现在同一商业化界面。',
    'Keep labels auditable for incident review, ownership review, and future rotation.': '保持标签可审计，便于事件复盘、归属核查与后续轮换。',
    'Key label': '密钥标签',
    'Production rollout': '生产发布',
    'Choose which workspace boundary this key should protect.': '选择该密钥应保护的工作区边界。',
    'Environment boundary': '环境边界',
    'Examples: canary, partner, sandbox-eu': '示例：canary、partner、sandbox-eu',
    'Custom environment': '自定义环境',
    'Gateway key mode': '网关密钥模式',
    'Choose whether Portal generates the secret or stores a custom plaintext value for this workspace boundary.': '选择由 Portal 生成密钥，还是为该工作区边界保存自定义明文值。',
    'System generated': '系统生成',
    'Let Portal create a one-time plaintext secret that is stored in write-only mode.': '由 Portal 创建一次性明文密钥，并以只写方式存储。',
    'Custom key': '自定义密钥',
    'Provide an exact plaintext value when rollout coordination requires a predefined credential.': '当发布协同要求预定义凭证时，提供精确的明文值。',
    'Portal-managed key': 'Portal 托管密钥',
    'Portal will generate a one-time plaintext secret, persist only the hashed value, and reveal the plaintext once after creation.': 'Portal 将生成一次性明文密钥，只持久化哈希值，并在创建后仅展示一次明文。',
    'A one-time plaintext key will be revealed after creation.': '创建后将展示一次性明文密钥。',
    'Paste the exact plaintext value that should be stored in write-only mode.': '粘贴需要以只写模式存储的精确明文值。',
    'API key': 'API 密钥',
    'Optional. Leave empty to keep this key active until you revoke it.': '可选。留空则该密钥会一直有效，直到你手动撤销。',
    'Expires at': '过期时间',
    'Add operator context, ownership, or rollout details for future review.': '补充操作背景、归属人或发布说明，便于后续审查。',
    Notes: '备注',
    'Operator-managed migration key': '运维迁移密钥',
    'Creating API key...': '正在创建 API Key...',
    'Recommended key setup starts with Key label ownership, any needed Custom environment override, and the Lifecycle policy that matches the rollout plan.': '推荐的密钥创建流程应先明确标签归属、必要的自定义环境覆盖，以及与发布计划匹配的生命周期策略。',
    'Usage method': '使用方式',
    'Use this key for the {environment} environment boundary and keep rollout verification inside the same workspace posture.': '将此密钥用于 {environment} 环境边界，并在同一工作区姿态内完成发布验证。',
    'Copy plaintext': '复制明文',
    'Portal endpoint': 'Portal 端点',
    'Authorization header': '授权请求头',
    'Plaintext unavailable. Rotate this key to obtain a new one-time secret.': '当前无法获取明文。请轮换该密钥以获得新的单次明文。',
    'This credential expires on {date}.': '该凭证将于 {date} 到期。',
    'This credential has no expiry. Keep revocation ownership explicit.': '该凭证未设置到期时间，请明确撤销责任归属。',
    'How to use this key': '如何使用此密钥',
    'Use this key for the {environment} environment boundary and keep rollout verification inside the same workspace posture. If the plaintext is no longer visible, create a replacement instead of depending on the UI as secret storage.': '将此密钥用于 {environment} 环境边界，并在同一工作区姿态内完成发布验证。如果明文已不可见，请创建替代密钥，而不要依赖 UI 作为密钥存储。',
    'Quick setup': '快速配置',
    'Apply setup directly on this device for Codex, Claude Code, OpenCode, Gemini, or OpenClaw, or copy the generated snippets into your preferred environment.': '可直接在当前设备为 Codex、Claude Code、OpenCode、Gemini 或 OpenClaw 应用配置，或复制生成的片段到你的目标环境。',
    'OpenClaw instances': 'OpenClaw 实例',
    'Loading local instances...': '正在加载本地实例...',
    'No OpenClaw instances were detected on this machine.': '当前机器未检测到 OpenClaw 实例。',
    'Applying...': '应用中...',
    'Apply setup': '应用配置',
    Status: '状态',
    'Quickstart snippet': '快速开始片段',
    Never: '永不',
    'Not yet': '尚未',
    'Revoked from gateway traffic': '已从网关流量中撤销',
    'Gateway traffic observed': '已观测到网关流量',
    'Ready for first authenticated request': '已可用于首次鉴权请求',
    'Copy key': '复制密钥',
    'Write-only': '只写',
    'Portal managed': 'Portal 托管',
    Source: '来源',
    Environment: '环境',
    Usage: '使用情况',
    'Last authenticated use': '最近鉴权使用',
    Active: '启用',
    Inactive: '停用',
    'Created at': '创建时间',
    Actions: '操作',
    Disable: '停用',
    Enable: '启用',
    Delete: '删除',
    'No API keys yet': '暂无 API 密钥',
    'Create your first key to connect a client or service to the SDKWork Router gateway.': '创建第一把密钥，以便将客户端或服务接入 SDKWork Router 网关。',
    'Loading issued keys...': '正在加载已签发密钥...',
    'Credential inventory is synced with the latest project key state.': '凭证清单已同步到最新的项目密钥状态。',
    'Key label is required so credentials remain auditable after creation.': '必须填写密钥标签，以确保创建后仍可审计。',
    'Custom environment is required when the custom environment option is selected.': '选择自定义环境时，必须填写自定义环境名称。',
    'Expires at must be a valid date before the credential can be created.': '创建凭证前，过期时间必须是有效日期。',
    'Custom key mode requires a plaintext key before the credential can be created.': '自定义密钥模式下，创建凭证前必须提供明文密钥。',
    'Registering a custom {environment} key for this workspace...': '正在为当前工作区登记自定义 {environment} 密钥...',
    'Issuing a Portal-managed {environment} key for this workspace...': '正在为当前工作区签发 Portal 托管的 {environment} 密钥...',
    'Custom key stored for {environment}. Verify the plaintext value before leaving this page.': '已为 {environment} 保存自定义密钥。离开页面前请先核对明文值。',
    'Portal-managed key issued for {environment}. Copy the plaintext secret before leaving this page.': '已为 {environment} 签发 Portal 托管密钥。离开页面前请先复制明文密钥。',
    'Plaintext key copied to clipboard.': '明文密钥已复制到剪贴板。',
    'Clipboard copy is unavailable in this browser context.': '当前浏览器上下文不支持复制到剪贴板。',
    'Restoring {label}...': '正在恢复 {label}...',
    'Revoking {label}...': '正在撤销 {label}...',
    '{label} is active again and can authenticate gateway traffic.': '{label} 已重新启用，可继续鉴权网关流量。',
    '{label} has been revoked and will no longer authenticate requests.': '{label} 已被撤销，不再允许鉴权请求。',
    'Deleting {label}...': '正在删除 {label}...',
    '{label} was deleted from this workspace.': '{label} 已从当前工作区删除。',
    'Plaintext Api key is no longer visible on this device. Create a replacement first.': '当前设备已无法查看该 API Key 的明文。请先创建替代密钥。',
    'Select at least one OpenClaw instance before applying setup.': '应用配置前，至少选择一个 OpenClaw 实例。',
    'Applying {label} setup...': '正在应用 {label} 配置...',
    'Applied setup to {count} OpenClaw instance(s).': '已将配置应用到 {count} 个 OpenClaw 实例。',
    'Applied setup and wrote {count} file(s).': '配置已应用，并写入 {count} 个文件。',
    Config: '配置',
    Auth: '认证',
    'Provider manifest': 'Provider 清单',
    'The newest plaintext secret is still available in this session, so you can validate the request shape before closing the page.': '当前会话内仍可查看最新明文密钥，可在关闭页面前先验证请求格式。',
    'This key is already stored in write-only mode. If you need the plaintext again, rotate it by creating a replacement credential.': '该密钥已进入只写模式。如果需要再次获取明文，请通过创建替代凭证来轮换。',
    'Use the current Api key directly against the SDKWork Router gateway without introducing a second credential boundary. Codex stays on the OpenAI-compatible responses stack.': '直接使用当前 API Key 访问 SDKWork Router 网关，无需引入第二层凭证边界。Codex 保持在 OpenAI 兼容的 responses 栈上。',
    'Claude Code uses the Anthropic-compatible route exposed by the gateway, keeps the same Api key, and preserves anthropic-version plus anthropic-beta headers on the relay path.': 'Claude Code 使用网关暴露的 Anthropic 兼容路由，沿用同一把 API Key，并保留 relay 路径上的 anthropic-version 与 anthropic-beta 请求头。',
    'OpenCode uses the OpenAI-compatible provider block and the same routed Api key.': 'OpenCode 使用 OpenAI 兼容 provider 配置块，并复用同一路由 API Key。',
    'Gemini CLI uses the official GOOGLE_GEMINI_BASE_URL plus GEMINI_API_KEY_AUTH_MECHANISM=bearer setup while keeping this Api key as the only secret.': 'Gemini CLI 使用官方 GOOGLE_GEMINI_BASE_URL 与 GEMINI_API_KEY_AUTH_MECHANISM=bearer 配置，并保持当前 API Key 作为唯一密钥。',
    'OpenClaw writes a provider manifest into the selected local instances and points them at the routed gateway endpoint.': 'OpenClaw 会将 provider 清单写入所选本地实例，并指向路由后的网关端点。',
    'Redeem credits': '兑换额度',
    'Coupon code': '优惠码',
    WELCOME100: 'WELCOME100',
    'Preview redemption': '预览兑换结果',
    'Preview offer': '预览优惠',
    'Seed coupons can be replaced by a live redemption backend without changing the page contract.': '种子优惠券后续可被真实兑换后端替换，而无需改变页面契约。',
    'No offers match the current filter.': '当前筛选条件下没有匹配的优惠。',
    'No offers for this slice': '当前视图下没有优惠',
    'No points ledger entries recorded yet.': '暂无积分账本记录。',
    'No ledger entries yet': '暂无账本记录',
    'Welcome back': '欢迎回来',
    'Create account': '创建账户',
    'Recover access': '恢复访问',
    'Sign in': '登录',
    'Sign up': '注册',
    'Back to login': '返回登录',
    'Continue with': '继续使用',
    'QR login': '扫码登录',
    'Open the desktop app and scan this code to continue without typing credentials.': '打开桌面应用并扫描此二维码，无需输入凭证即可继续。',
    'Local dev credentials are prefilled: {email} / {password}.': '本地开发环境已预填测试账号：{email} / {password}。',
    'Open app to scan': '打开应用扫码',
    'Create your workspace access and continue into the portal shell.': '创建你的工作区访问权限并继续进入门户壳层。',
    'Password reset links are not enabled for the current portal backend. Continue back to sign in with your workspace email.': '当前门户后端未启用密码重置链接，请返回并使用工作区邮箱登录。',
    'Sign in to continue to your portal workspace.': '登录后继续进入你的门户工作区。',
    Name: '姓名',
    'Workspace owner': '工作区负责人',
    'name@example.com': 'name@example.com',
    Email: '邮箱',
    Password: '密码',
    'Create a password': '创建密码',
    'Enter your password': '输入你的密码',
    'Forgot password?': '忘记密码？',
    'Loading...': '加载中...',
    'No account?': '没有账户？',
    'Already have an account?': '已有账户？',
  },
};

const portalSupplementalZhMessages: Record<string, string> = {
  Dashboard: '总览',
  Routing: '路由',
  'API Keys': 'API 密钥',
  Usage: '使用记录',
  User: '用户',
  Credits: '额度',
  Billing: '账单',
  Account: '账户',
  Overview: '概览',
  Control: '控制',
  Credentials: '凭证',
  Telemetry: '遥测',
  Identity: '身份',
  Points: '积分',
  Financial: '财务',
  Access: '访问',
  Commerce: '商业',
  'Traffic, routing, access, and spend at a glance': '流量、路由、访问与支出总览',
  'Default strategy, failover posture, and route evidence': '默认策略、故障转移姿态与路由证据',
  'Issue, inspect, and rotate project keys': '签发、检查并轮换项目密钥',
  'Requests, models, providers, and spend telemetry': '请求、模型、提供商与支出遥测',
  'Profile, security, and personal access settings': '资料、安全与个人访问设置',
  'Quota posture, redemption, and remaining units': '配额姿态、兑换与剩余单位',
  'Plans, recharge packs, and billing recovery': '套餐、充值包与账单恢复',
  'Cash balance, ledger visibility, and payment posture': '现金余额、账本可见性与支付姿态',
  'Expand sidebar': '展开侧栏',
  'Collapse sidebar': '收起侧栏',
  'Tech Blue': '科技蓝',
  Lobster: '龙虾红',
  'Green Tech': '科技绿',
  Zinc: '锌灰',
  Violet: '紫罗兰',
  Rose: '玫瑰粉',
  'SDKWork Router': 'SDKWork 路由',
  'Developer portal': '开发者门户',
};

Object.assign(portalMessages['zh-CN'], portalSupplementalZhMessages);

export const PORTAL_LOCALE_OPTIONS: Array<{ id: PortalLocale; label: string }> = [
  { id: 'en-US', label: 'English' },
  { id: 'zh-CN', label: 'Simplified Chinese' },
];

let activePortalLocale: PortalLocale = 'en-US';

function interpolate(text: string, values?: TranslationValues): string {
  if (!values) {
    return text;
  }

  return Object.entries(values).reduce(
    (result, [key, value]) => result.replaceAll(`{${key}}`, String(value)),
    text,
  );
}

function normalizeLocale(value: string | null | undefined): PortalLocale {
  if (!value) {
    return 'en-US';
  }

  return value.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

function translateForLocale(locale: PortalLocale, text: string, values?: TranslationValues): string {
  const translated = locale === 'en-US' ? text : portalMessages[locale][text] ?? text;
  return interpolate(translated, values);
}

function resolveInitialLocale(): PortalLocale {
  if (typeof window === 'undefined') {
    return activePortalLocale;
  }

  try {
    const persisted = window.localStorage.getItem(PORTAL_I18N_STORAGE_KEY);
    if (persisted) {
      return normalizeLocale(persisted);
    }
  } catch {
    // Ignore storage access failures and fall back to browser locale.
  }

  return normalizeLocale(window.navigator.language);
}

const PortalI18nContext = createContext<PortalI18nContextValue | null>(null);

export function translatePortalText(text: string, values?: TranslationValues): string {
  return translateForLocale(activePortalLocale, text, values);
}

export function PortalI18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<PortalLocale>(resolveInitialLocale);

  activePortalLocale = locale;
  setActivePortalCoreLocale(locale);
  setActivePortalFormatLocale(locale);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(PORTAL_I18N_STORAGE_KEY, locale);
      } catch {
        // Ignore storage write failures.
      }
    }

    if (typeof document !== 'undefined') {
      document.documentElement.lang = locale;
    }
  }, [locale]);

  const value = useMemo<PortalI18nContextValue>(
    () => ({
      locale,
      setLocale,
      t: (text, values) => translateForLocale(locale, text, values),
    }),
    [locale],
  );

  return <PortalI18nContext.Provider value={value}>{children}</PortalI18nContext.Provider>;
}

export function usePortalI18n(): PortalI18nContextValue {
  const context = useContext(PortalI18nContext);

  if (!context) {
    throw new Error('Portal i18n hooks must be used inside PortalI18nProvider.');
  }

  return context;
}

const portalBorder = 'border-[color:var(--portal-border-color)]';
const portalText = 'text-[var(--portal-text-primary)]';
const portalTextSecondary = 'text-[var(--portal-text-secondary)]';
const portalTextMuted = 'text-[var(--portal-text-muted)]';
const portalContrastText = 'text-[var(--portal-text-on-contrast)]';
const portalContrastMuted = 'text-[var(--portal-text-muted-on-contrast)]';
const portalSurface = 'bg-[var(--portal-surface-background)]';
const portalSurfaceElevated = 'bg-[var(--portal-surface-elevated)]';
const portalSurfaceContrast = '[background:var(--portal-surface-contrast)]';
const portalShadowSoft = 'shadow-[var(--portal-shadow-soft)]';
const portalShadowStrong = 'shadow-[var(--portal-shadow-strong)]';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-2xl text-sm font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/35 disabled:pointer-events-none disabled:opacity-50',
  {
    variants: {
      variant: {
        default: 'bg-primary-600 text-white shadow-[0_16px_30px_rgb(var(--portal-accent-rgb)_/_0.22)] hover:bg-primary-500',
        secondary: `border ${portalBorder} ${portalSurface} ${portalTextSecondary} hover:bg-[var(--portal-hover-surface)] hover:text-[var(--portal-text-primary)]`,
        ghost: `${portalTextSecondary} hover:bg-[var(--portal-hover-surface)] hover:text-[var(--portal-text-primary)]`,
        destructive: 'bg-rose-500 text-white hover:bg-rose-400',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 px-3',
        lg: 'h-11 px-5',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
);

const badgeVariants = cva(
  'inline-flex items-center rounded-full border px-2.5 py-1 text-xs font-medium tracking-wide',
  {
    variants: {
      variant: {
        default: `border ${portalBorder} bg-[var(--portal-hover-surface)] ${portalTextSecondary}`,
        accent: 'border-primary-500/25 bg-primary-500/10 text-primary-200 dark:text-primary-100',
        positive: 'border-emerald-400/20 bg-emerald-400/10 text-emerald-200 dark:text-emerald-100',
        warning: 'border-amber-400/20 bg-amber-400/10 text-amber-200 dark:text-amber-100',
        seed: 'border-fuchsia-400/20 bg-fuchsia-400/10 text-fuchsia-200 dark:text-fuchsia-100',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
);

export interface ButtonProps
  extends ComponentPropsWithoutRef<'button'>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';

    return (
      <Comp
        className={cn(buttonVariants({ variant, size }), className)}
        ref={ref}
        {...props}
      />
    );
  },
);
Button.displayName = 'Button';

export function Badge({
  className,
  variant,
  children,
}: {
  className?: string;
  variant?: VariantProps<typeof badgeVariants>['variant'];
  children: ReactNode;
}) {
  return <span className={cn(badgeVariants({ variant }), className)}>{children}</span>;
}

export const Card = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<'div'>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(`rounded-3xl border ${portalBorder} ${portalSurface} ${portalText} ${portalShadowSoft}`, className)}
    {...props}
  />
));
Card.displayName = 'Card';

export const CardHeader = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<'div'>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('flex flex-col gap-1.5 p-6', className)} {...props} />
));
CardHeader.displayName = 'CardHeader';

export const CardTitle = forwardRef<
  HTMLParagraphElement,
  ComponentPropsWithoutRef<'h3'>
>(({ className, ...props }, ref) => (
  <h3 ref={ref} className={cn(`text-lg font-semibold tracking-tight ${portalText}`, className)} {...props} />
));
CardTitle.displayName = 'CardTitle';

export const CardDescription = forwardRef<
  HTMLParagraphElement,
  ComponentPropsWithoutRef<'p'>
>(({ className, ...props }, ref) => (
  <p ref={ref} className={cn(`text-sm ${portalTextSecondary}`, className)} {...props} />
));
CardDescription.displayName = 'CardDescription';

export const CardContent = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<'div'>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('px-6 pb-6', className)} {...props} />
));
CardContent.displayName = 'CardContent';

export const CardFooter = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<'div'>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('flex items-center px-6 pb-6 pt-2', className)} {...props} />
));
CardFooter.displayName = 'CardFooter';

export const Dialog = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
export const DialogClose = DialogPrimitive.Close;

export const DialogPortal = DialogPrimitive.Portal;

export const DialogOverlay = forwardRef<
  ElementRef<typeof DialogPrimitive.Overlay>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Overlay
    ref={ref}
    className={cn('fixed inset-0 z-40 bg-[var(--portal-overlay)] backdrop-blur-sm', className)}
    {...props}
  />
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

const dialogSizeClassNames = {
  small: 'max-w-md',
  medium: 'max-w-2xl',
  large: 'max-w-4xl',
} as const;

export interface DialogContentProps
  extends ComponentPropsWithoutRef<typeof DialogPrimitive.Content> {
  size?: keyof typeof dialogSizeClassNames;
  showCloseButton?: boolean;
}

export const DialogContent = forwardRef<
  ElementRef<typeof DialogPrimitive.Content>,
  DialogContentProps
>(({ className, children, size = 'medium', showCloseButton = true, ...props }, ref) => (
  <DialogPortal>
    <DialogOverlay />
    <DialogPrimitive.Content
      ref={ref}
      className={cn(
        `fixed left-1/2 top-1/2 z-[60] grid w-[calc(100%-2rem)] max-h-[calc(100dvh-2rem)] -translate-x-1/2 -translate-y-1/2 gap-4 overflow-y-auto rounded-[28px] border ${portalBorder} bg-[var(--portal-overlay-surface)] p-6 ${portalShadowStrong} focus:outline-none`,
        dialogSizeClassNames[size],
        className,
      )}
      {...props}
    >
      {children}
      {showCloseButton ? (
        <DialogPrimitive.Close asChild>
          <DialogIconCloseButton
            className="absolute right-4 top-4"
            label={translatePortalText('Close')}
          />
        </DialogPrimitive.Close>
      ) : null}
    </DialogPrimitive.Content>
  </DialogPortal>
));
DialogContent.displayName = DialogPrimitive.Content.displayName;

export function DialogHeader({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('flex flex-col gap-1.5 text-center sm:text-left', className)} {...props} />;
}

export function DialogFooter({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('flex flex-col-reverse gap-2 sm:flex-row sm:justify-end', className)} {...props} />;
}

function DialogIconCloseButton({
  label,
  className,
}: {
  label: string;
  className?: string;
}) {
  return (
    <Button
      aria-label={label}
      className={cn(
        `${portalTextMuted} hover:bg-[var(--portal-hover-surface)] hover:text-[var(--portal-text-primary)]`,
        className,
      )}
      size="icon"
      type="button"
      variant="ghost"
    >
      <X className="h-4 w-4" />
      <span className="sr-only">{label}</span>
    </Button>
  );
}

export const DialogTitle = forwardRef<
  ElementRef<typeof DialogPrimitive.Title>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Title ref={ref} className={cn(`text-lg font-semibold tracking-tight ${portalText}`, className)} {...props} />
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;

export const DialogDescription = forwardRef<
  ElementRef<typeof DialogPrimitive.Description>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Description ref={ref} className={cn(`text-sm ${portalTextSecondary}`, className)} {...props} />
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export function Modal({
  isOpen,
  onClose,
  title,
  description,
  children,
  footer,
  className,
}: ModalProps) {
  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <DialogContent
        size="small"
        showCloseButton={false}
        className={cn(`max-w-md border ${portalBorder} p-0`, className)}
      >
        <DialogHeader className={cn(`flex-row items-start justify-between gap-4 border-b ${portalBorder} px-6 py-5 text-left`)}>
          <div className="grid gap-1.5">
            <DialogTitle className="text-xl font-semibold tracking-tight">{title}</DialogTitle>
            {description ? <DialogDescription>{description}</DialogDescription> : null}
          </div>
          <DialogClose asChild>
            <DialogIconCloseButton label={translatePortalText('Close')} />
          </DialogClose>
        </DialogHeader>
        <div className="overflow-y-auto p-6">{children}</div>
        {footer ? (
          <DialogFooter className={cn(`border-t ${portalBorder} px-6 py-5`)}>
            {footer}
          </DialogFooter>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

export const Tabs = TabsPrimitive.Root;

export const TabsList = forwardRef<
  ElementRef<typeof TabsPrimitive.List>,
  ComponentPropsWithoutRef<typeof TabsPrimitive.List>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.List
    ref={ref}
    className={cn(`inline-flex h-11 items-center gap-1 rounded-xl border ${portalBorder} bg-[var(--portal-hover-surface)] p-1`, className)}
    {...props}
  />
));
TabsList.displayName = TabsPrimitive.List.displayName;

export const TabsTrigger = forwardRef<
  ElementRef<typeof TabsPrimitive.Trigger>,
  ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Trigger
    ref={ref}
    className={cn(`inline-flex items-center justify-center rounded-lg px-3 py-2 text-sm font-medium ${portalTextMuted} transition data-[state=active]:bg-[var(--portal-surface-background)] data-[state=active]:text-[var(--portal-text-primary)] data-[state=active]:shadow-sm`, className)}
    {...props}
  />
));
TabsTrigger.displayName = TabsPrimitive.Trigger.displayName;

export const TabsContent = forwardRef<
  ElementRef<typeof TabsPrimitive.Content>,
  ComponentPropsWithoutRef<typeof TabsPrimitive.Content>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Content
    ref={ref}
    className={cn('mt-5 outline-none', className)}
    {...props}
  />
));
TabsContent.displayName = TabsPrimitive.Content.displayName;

export const Input = forwardRef<
  HTMLInputElement,
  ComponentPropsWithoutRef<'input'>
>(({ className, ...props }, ref) => (
  <input
    ref={ref}
    className={cn(`flex h-11 w-full rounded-xl border ${portalBorder} bg-[var(--portal-surface-elevated)] px-3 py-2 text-sm ${portalText} shadow-sm outline-none transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-[var(--portal-text-muted)] focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 disabled:cursor-not-allowed disabled:opacity-50`, className)}
    {...props}
  />
));
Input.displayName = 'Input';

export const Select = forwardRef<
  HTMLSelectElement,
  ComponentPropsWithoutRef<'select'>
>(({ className, ...props }, ref) => (
  <select
    ref={ref}
    className={cn(`flex h-11 w-full appearance-none rounded-xl border ${portalBorder} bg-[var(--portal-surface-elevated)] px-3 py-2 text-sm ${portalText} shadow-sm outline-none transition-colors focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 disabled:cursor-not-allowed disabled:opacity-50`, className)}
    {...props}
  />
));
Select.displayName = 'Select';

export const Textarea = forwardRef<
  HTMLTextAreaElement,
  ComponentPropsWithoutRef<'textarea'>
>(({ className, ...props }, ref) => (
  <textarea
    ref={ref}
    className={cn(`flex min-h-[96px] w-full rounded-xl border ${portalBorder} bg-[var(--portal-surface-elevated)] px-3 py-2 text-sm ${portalText} shadow-sm outline-none transition-colors placeholder:text-[var(--portal-text-muted)] focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 disabled:cursor-not-allowed disabled:opacity-50`, className)}
    {...props}
  />
));
Textarea.displayName = 'Textarea';

type PortalCheckboxEvent = {
  target: { checked: boolean };
  currentTarget: { checked: boolean };
};

export const Checkbox = forwardRef<
  ElementRef<typeof CheckboxPrimitive.Root>,
  Omit<ComponentPropsWithoutRef<typeof CheckboxPrimitive.Root>, 'onChange' | 'onCheckedChange'> & {
    onChange?: (event: PortalCheckboxEvent) => void;
    onCheckedChange?: (checked: boolean) => void;
  }
>(({ className, onChange, onCheckedChange, ...props }, ref) => (
  <CheckboxPrimitive.Root
    ref={ref}
    className={cn(`peer flex h-4 w-4 shrink-0 items-center justify-center rounded border ${portalBorder} bg-[var(--portal-surface-background)] text-primary-500 shadow-sm ring-offset-white transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-primary-600 data-[state=checked]:bg-primary-600 data-[state=checked]:text-white dark:focus-visible:ring-offset-zinc-950 dark:data-[state=checked]:border-primary-500 dark:data-[state=checked]:bg-primary-500`, className)}
    onCheckedChange={(checked) => {
      const resolvedChecked = checked === true;
      onCheckedChange?.(resolvedChecked);
      onChange?.({
        target: { checked: resolvedChecked },
        currentTarget: { checked: resolvedChecked },
      });
    }}
    {...props}
  >
    <CheckboxPrimitive.Indicator className="flex items-center justify-center text-current">
      <Check className="h-4 w-4" />
    </CheckboxPrimitive.Indicator>
  </CheckboxPrimitive.Root>
));
Checkbox.displayName = 'Checkbox';

export const Label = forwardRef<
  ElementRef<typeof LabelPrimitive.Root>,
  ComponentPropsWithoutRef<typeof LabelPrimitive.Root>
>(({ className, ...props }, ref) => (
  <LabelPrimitive.Root
    ref={ref}
    className={cn(`text-sm font-medium ${portalText}`, className)}
    {...props}
  />
));
Label.displayName = LabelPrimitive.Root.displayName;

export function FormField({
  label,
  children,
  hint,
  className,
}: {
  label: string;
  children: ReactNode;
  hint?: string;
  className?: string;
}) {
  return (
    <label className={cn('grid gap-2', className)}>
      <Label>{label}</Label>
      {children}
      {hint ? <span className={`text-xs ${portalTextMuted}`}>{hint}</span> : null}
    </label>
  );
}

export function ToolbarField({
  label,
  children,
  className,
  controlClassName,
}: {
  label: string;
  children: ReactNode;
  className?: string;
  controlClassName?: string;
}) {
  return (
    <label className={cn('flex min-w-0 max-w-full items-center gap-3', className)}>
      <span className={cn(`shrink-0 whitespace-nowrap text-[11px] font-semibold uppercase tracking-[0.18em] ${portalTextMuted}`)}>
        {label}
      </span>
      <span className={cn('min-w-0 flex-1', controlClassName)}>{children}</span>
    </label>
  );
}

export function ToolbarInline({
  children,
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return (
    <div
      className={cn('flex min-w-0 flex-nowrap items-end gap-3 overflow-x-auto', className)}
      {...props}
    >
      {children}
    </div>
  );
}

export function SearchInput({
  className,
  inputClassName,
  iconClassName,
  style,
  type,
  ...props
}: Omit<ComponentPropsWithoutRef<'input'>, 'className'> & {
  className?: string;
  inputClassName?: string;
  iconClassName?: string;
}) {
  return (
    <LeadingIconInput
      className={cn('portalx-search-input', className)}
      icon={<SearchIcon className="h-4 w-4" />}
      iconClassName={iconClassName}
      inputClassName={cn('portalx-search-input-element', inputClassName)}
      style={style}
      type={type}
      {...props}
    />
  );
}

export function LeadingIconInput({
  className,
  inputClassName,
  iconClassName,
  icon,
  style,
  type,
  ...props
}: Omit<ComponentPropsWithoutRef<'input'>, 'className'> & {
  className?: string;
  inputClassName?: string;
  iconClassName?: string;
  icon: ReactNode;
}) {
  return (
    <span className={cn('relative block w-full', className)}>
      <span
        className={cn(
          'pointer-events-none absolute left-4 top-1/2 flex h-5 w-5 -translate-y-1/2 items-center justify-center text-zinc-400 dark:text-zinc-500',
          iconClassName,
        )}
      >
        {icon}
      </span>
      <Input
        className={inputClassName}
        style={{ ...style, paddingLeft: '2.75rem' }}
        type={type ?? 'text'}
        {...props}
      />
    </span>
  );
}

export function ToolbarSearchField({
  label,
  className,
  inputClassName,
  ...props
}: ComponentPropsWithoutRef<'input'> & {
  label: string;
  className?: string;
  inputClassName?: string;
}) {
  return (
    <ToolbarField
      label={label}
      className={cn('flex-1 basis-[24rem]', className)}
      controlClassName="min-w-0"
    >
      <SearchInput inputClassName={inputClassName} {...props} />
    </ToolbarField>
  );
}

export function SectionHero({
  eyebrow,
  title,
  detail,
  actions,
}: {
  eyebrow: string;
  title: string;
  detail: string;
  actions?: ReactNode;
}) {
  return (
    <Card className={`border-[color:var(--portal-contrast-border)] ${portalSurfaceContrast} ${portalContrastText} ${portalShadowStrong}`}>
      <CardContent className="flex flex-col gap-6 px-6 py-6 md:flex-row md:items-start md:justify-between">
        <div className="space-y-3">
          <p className="text-xs font-semibold uppercase tracking-[0.24em] text-primary-200/80">{eyebrow}</p>
          <h1 className={`text-3xl font-semibold tracking-tight ${portalContrastText} md:text-4xl`}>{title}</h1>
          <p className={`max-w-3xl text-sm leading-6 ${portalContrastMuted} md:text-base`}>{detail}</p>
        </div>
        {actions ? <div className="flex flex-wrap gap-3">{actions}</div> : null}
      </CardContent>
    </Card>
  );
}

export function Surface({
  title,
  detail,
  actions,
  children,
}: {
  title: string;
  detail?: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Card>
      <CardHeader className={`flex flex-col gap-4 border-b ${portalBorder} pb-4 md:flex-row md:items-start md:justify-between`}>
        <div className="space-y-1">
          <CardTitle>{title}</CardTitle>
          {detail ? <CardDescription>{detail}</CardDescription> : null}
        </div>
        {actions ? <div className="flex flex-wrap gap-2">{actions}</div> : null}
      </CardHeader>
      <CardContent className="pt-6">{children}</CardContent>
    </Card>
  );
}

export function MetricCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <Card className={`rounded-2xl ${portalSurfaceElevated}`}>
      <CardContent className="space-y-3 p-5">
        <span className={`text-xs font-semibold uppercase tracking-[0.2em] ${portalTextMuted}`}>{label}</span>
        <strong className={`block text-3xl font-semibold tracking-tight ${portalText}`}>{value}</strong>
        <p className={`text-sm ${portalTextSecondary}`}>{detail}</p>
      </CardContent>
    </Card>
  );
}

export function Pill({
  tone,
  children,
}: {
  tone?: 'default' | 'accent' | 'positive' | 'warning' | 'seed';
  children: ReactNode;
}) {
  const variant = tone === 'accent' || tone === 'positive' || tone === 'warning' || tone === 'seed'
    ? tone
    : 'default';
  return <Badge variant={variant}>{children}</Badge>;
}

export function InlineButton({
  children,
  onClick,
  tone,
  type,
  disabled,
  className,
}: {
  children: ReactNode;
  onClick?: () => void;
  tone?: 'primary' | 'secondary' | 'ghost';
  type?: 'button' | 'submit';
  disabled?: boolean;
  className?: string;
}) {
  const variant = tone === 'primary' ? 'default' : tone === 'ghost' ? 'ghost' : 'secondary';
  return (
    <Button className={className} disabled={disabled} onClick={onClick} type={type ?? 'button'} variant={variant}>
      {children}
    </Button>
  );
}

export function ToolbarDisclosure({
  children,
  defaultOpen = false,
  openLabel,
  closeLabel,
}: {
  children: ReactNode;
  defaultOpen?: boolean;
  openLabel?: string;
  closeLabel?: string;
}) {
  const [open, setOpen] = useState(defaultOpen);
  const { t } = usePortalI18n();

  return (
    <div className="flex min-w-full flex-col gap-3">
      <div>
        <InlineButton onClick={() => setOpen((current) => !current)} tone="secondary">
          {open ? closeLabel ?? t('Hide filters') : openLabel ?? t('More filters')}
        </InlineButton>
      </div>
      {open ? <div className="grid gap-3">{children}</div> : null}
    </div>
  );
}

export function EmptyState({
  title,
  detail,
}: {
  title: string;
  detail: string;
}) {
  return (
    <div className={`rounded-2xl border border-dashed ${portalBorder} ${portalSurfaceElevated} p-6 text-center`}>
      <strong className={`block text-base font-semibold ${portalText}`}>{title}</strong>
      <p className={`mt-2 text-sm ${portalTextSecondary}`}>{detail}</p>
    </div>
  );
}

export function DataTable<T>({
  columns,
  rows,
  empty,
  getKey,
}: {
  columns: Array<{ key: string; label: string; render: (row: T) => ReactNode }>;
  rows: T[];
  empty: ReactNode;
  getKey: (row: T, index: number) => string;
}) {
  return (
    <div
      data-slot="table-container"
      className="overflow-hidden rounded-[28px] border border-zinc-200/80 bg-white/92 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70"
    >
      <div className="overflow-x-auto">
        <table
          data-slot="table"
          className="min-w-full border-separate border-spacing-0 text-sm"
        >
          <thead
            data-slot="table-header"
            className="bg-zinc-50/90 dark:bg-zinc-900/80"
          >
            <tr data-slot="table-header-row">
              {columns.map((column) => (
                <th
                  data-slot="table-head"
                  className="sticky top-0 z-10 whitespace-nowrap border-b border-zinc-200/80 bg-zinc-50/95 px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-900/95 dark:text-zinc-400"
                  key={column.key}
                >
                  {column.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody data-slot="table-body" className="bg-transparent">
            {rows.length ? rows.map((row, index) => (
              <tr
                className="transition-colors hover:bg-zinc-50/80 dark:hover:bg-zinc-900/70"
                data-slot="table-row"
                key={getKey(row, index)}
              >
                {columns.map((column) => (
                  <td
                    className="border-t border-zinc-200/70 px-4 py-4 align-top text-zinc-600 dark:border-zinc-800/80 dark:text-zinc-300"
                    data-slot="table-cell"
                    key={column.key}
                  >
                    {column.render(row)}
                  </td>
                ))}
              </tr>
            )) : (
              <tr data-slot="table-empty-row">
                <td
                  className="border-t border-zinc-200/70 px-4 py-9 text-center text-sm text-zinc-500 dark:border-zinc-800/80 dark:text-zinc-400"
                  colSpan={columns.length}
                  data-slot="table-empty"
                >
                  {empty}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export { formatCurrency, formatDateTime, formatUnits } from './format-core';

export async function copyText(value: string): Promise<boolean> {
  if (!value) {
    return false;
  }

  try {
    await globalThis.navigator?.clipboard?.writeText(value);
    return true;
  } catch {
    return false;
  }
}
