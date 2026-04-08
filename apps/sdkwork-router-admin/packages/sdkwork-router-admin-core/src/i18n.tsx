import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { ADMIN_TRANSLATIONS as BASE_ADMIN_TRANSLATIONS } from './i18nTranslations';

// BEGIN CONTRACT TRANSLATIONS

const ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS: Record<string, string> = {
  "API key groups currently bound to reusable routing profiles.": "\u5f53\u524d\u5df2\u7ed1\u5b9a\u5230\u53ef\u590d\u7528\u8def\u7531\u914d\u7f6e\u7684 API \u5bc6\u94a5\u7ec4\u3002",
  "API key groups currently bound to a reusable routing profile.": "\u5f53\u524d\u5df2\u7ed1\u5b9a\u5230\u53ef\u590d\u7528\u8def\u7531\u914d\u7f6e\u7684 API \u5bc6\u94a5\u7ec4\u3002",
  "Define reusable policy groups for workspace-scoped key issuance, routing posture, and accounting defaults.": "\u5b9a\u4e49\u53ef\u590d\u7528\u7684\u7b56\u7565\u7ec4\uff0c\u7528\u4e8e\u7ba1\u7406\u5de5\u4f5c\u533a\u8303\u56f4\u5185\u7684\u5bc6\u94a5\u7b7e\u53d1\u3001\u8def\u7531\u6001\u52bf\u548c\u8bb0\u8d26\u9ed8\u8ba4\u503c\u3002",
  "Define the defaults that each bound API key should inherit from this group policy.": "\u5b9a\u4e49\u6bcf\u4e2a\u7ed1\u5b9a API \u5bc6\u94a5\u5e94\u4ece\u8be5\u7b56\u7565\u7ec4\u7ee7\u627f\u7684\u9ed8\u8ba4\u8bbe\u7f6e\u3002",
  "Capture reusable routing posture so API key groups and workspace policy can bind to a named profile instead of repeating provider order, latency, and health rules.": "\u6c89\u6dc0\u53ef\u590d\u7528\u7684\u8def\u7531\u7b56\u7565\u6001\u52bf\uff0c\u8ba9 API \u5bc6\u94a5\u7ec4\u548c\u5de5\u4f5c\u533a\u7b56\u7565\u53ef\u4ee5\u7ed1\u5b9a\u547d\u540d\u914d\u7f6e\uff0c\u65e0\u9700\u91cd\u590d\u7ef4\u62a4\u4f9b\u5e94\u5546\u987a\u5e8f\u3001\u65f6\u5ef6\u548c\u5065\u5eb7\u89c4\u5219\u3002",
  "Create route providers first, then return to build a reusable routing profile.": "\u8bf7\u5148\u521b\u5efa\u8def\u7531\u4f9b\u5e94\u5546\uff0c\u518d\u56de\u6765\u6784\u5efa\u53ef\u590d\u7528\u7684\u8def\u7531\u914d\u7f6e\u3002",
};

const ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS: Record<string, string> = {
  "API key groups": "API 密钥分组",
  "Define reusable policy groups for workspace-scoped key issuance, routing posture, and accounting defaults.": "定义可复用策略分组，用于工作区范围的密钥签发、路由态势和记账默认值。",
  "Search groups": "搜索分组",
  "Create group": "创建分组",
  "No groups match the current filter": "当前筛选条件下没有匹配的分组",
  "Broaden the query or create a new policy group for this workspace scope.": "放宽查询条件，或为该工作区范围创建新的策略分组。",
  "Pin the workspace boundary for this reusable key governance group.": "为该可复用密钥治理分组锁定工作区边界。",
  "Edit group": "编辑分组",
  "Leave empty to derive a slug from the group name.": "留空时将根据分组名称自动生成 slug。",
  "Policy defaults": "策略默认值",
  "Examples: chat,responses or images,audio.": "示例：chat,responses 或 images,audio。",
  "No accounting override": "无记账覆盖",
  "Bind to an active routing profile inside the same workspace scope when needed.": "需要时可绑定同一工作区范围内的生效路由配置。",
  "No routing profile override": "无路由配置覆盖",
  "Group updated. Review the refreshed policy state on the left.": "分组已更新。请在左侧查看刷新后的策略状态。",
  "Group created. Review the refreshed policy state on the left.": "分组已创建。请在左侧查看刷新后的策略状态。",
  "Group disabled. New key assignments now require a different policy group.": "分组已停用。新的密钥分配现在需要使用其他策略分组。",
  "Group enabled. Keys in this workspace scope can bind to it again.": "分组已启用。该工作区范围内的密钥现在可重新绑定到此分组。",
  "Group deleted. Review the refreshed policy inventory.": "分组已删除。请查看刷新后的策略清单。",
  "Failed to save API key group.": "保存 API 密钥分组失败。",
  "Failed to update API key group status.": "更新 API 密钥分组状态失败。",
  "Failed to delete API key group.": "删除 API 密钥分组失败。",
  "Delete API key group": "删除 API 密钥分组",
  "Delete group": "删除分组",
  "Delete {name}. Keys already bound to this group will need a new policy assignment before future updates.": "删除 {name}。已绑定此分组的密钥在后续更新前需要重新分配策略。",
  "Enable group": "启用分组",
  "Disable group": "停用分组",
  "Save group": "保存分组",
  "Slug": "Slug 标识",
  "Color": "颜色",
  "Default scope": "默认范围",
  "Routing profile": "路由配置",
  "Open": "打开",
  "No default scope": "无默认范围",
  "Accounting mode": "记账模式",
  "Routing profiles": "路由配置",
  "Capture reusable routing posture so API key groups and workspace policy can bind to a named profile instead of repeating provider order, latency, and health rules.": "沉淀可复用路由态势，让 API 密钥分组和工作区策略绑定命名配置，而不必重复维护供应商顺序、时延和健康规则。",
  "Search routing profiles": "搜索路由配置",
  "Create profile": "创建配置",
  "No routing profiles match the current filter": "当前筛选条件下没有匹配的路由配置",
  "Broaden the query or create the first reusable routing policy for this workspace.": "放宽查询条件，或为该工作区创建首个可复用路由策略。",
  "Creating a new profile from {name}. Adjust the scope, provider order, or routing constraints before saving.": "正在基于 {name} 创建新配置。请在保存前调整范围、供应商顺序或路由约束。",
  "Pin the workspace boundary and profile metadata before defining routing posture.": "在定义路由态势前先锁定工作区边界和配置元数据。",
  "Leave empty to derive a slug automatically from the profile name.": "留空时将根据配置名称自动生成 slug。",
  "Keep the new profile immediately selectable by API key groups after creation.": "让新配置在创建后即可被 API 密钥分组直接选择。",
  "Routing posture": "路由态势",
  "Set the route selection behavior, region preference, and SLO limits that the gateway should inherit from this profile.": "设置网关应从该配置继承的路由选择行为、地域偏好和 SLO 限制。",
  "Preferred region": "首选地域",
  "No preferred region": "无首选地域",
  "Max cost": "最大成本",
  "Max latency (ms)": "最大延迟（ms）",
  "Require healthy": "仅限健康",
  "Only keep healthy providers in the candidate set when this profile is applied.": "应用该配置时，仅保留健康的供应商进入候选集。",
  "Provider order": "供应商顺序",
  "Choose which providers belong to the profile, then arrange the fallback chain explicitly.": "选择哪些供应商属于该配置，并显式排列回退链路。",
  "Create route providers first, then return to build a reusable routing profile.": "请先创建路由供应商，再回来构建可复用路由配置。",
  "Default provider": "默认供应商",
  "Default": "默认",
  "Move up": "上移",
  "Move down": "下移",
  "Use as template": "作为模板使用",
  "Routing profile created. Review the refreshed policy inventory on the left.": "路由配置已创建。请在左侧查看刷新后的策略清单。",
  "Failed to create routing profile.": "创建路由配置失败。",
  "Save routing profile": "保存路由配置",
  "Auto": "自动",
  "Compiled snapshots": "编译快照",
  "Inspect the compiled routing evidence that the gateway produced after combining policy, project defaults, and API key group routing profile overlays.": "查看网关在组合策略、项目默认值和 API 密钥分组路由配置覆盖后生成的编译路由证据。",
  "All compiled route snapshots currently loaded into the admin workspace.": "当前已加载到管理工作区的全部编译路由快照。",
  "Applied routing profile": "已应用路由配置",
  "Snapshots that carry an applied routing profile id.": "携带已应用路由配置 ID 的快照。",
  "Bound groups": "已绑定分组",
  "Route keys": "路由键",
  "Distinct route keys represented across the compiled snapshot evidence set.": "编译快照证据集中覆盖的不同路由键。",
  "Search compiled snapshots": "搜索编译快照",
  "No applied routing profile": "无已应用路由配置",
  "All profiles": "全部配置",
  "No API key group scope": "无 API 密钥分组范围",
  "No ordered providers": "无排序供应商",
  "No default provider": "无默认供应商",
  "No compiled snapshots match the current filter": "当前筛选条件下没有匹配的编译快照",
  "Broaden the query or refresh the workspace after new routing decisions land.": "放宽查询条件，或在新的路由决策落地后刷新工作区。",
  "Compiled snapshot": "编译快照",
  "Fallback reason": "回退原因",
  "Request settlements": "请求结算",
  "{amount} charge": "{amount} 收费",
  "{amount} upstream": "{amount} 上游",
  "Active plans": "生效方案",
  "Audio sec": "音频秒",
  "Billing events": "计费事件",
  "Capability mix": "能力构成",
  "Captured credits": "已捕获额度",
  "Charge distribution across routed multimodal capabilities.": "路由多模态能力的收费分布。",
  "Commercial pricing plans and rates stay visible alongside usage analytics and billing events.": "商业定价方案与费率会与使用分析和计费事件一同保持可见。",
  "Compare platform credit, BYOK, and passthrough posture.": "对比平台额度、BYOK 和透传模式态势。",
  "Customer charge": "客户收费",
  "Event-level chargeback stays aligned with the active usage filters.": "事件级分摊会与当前使用筛选保持对齐。",
  "Export billing events CSV": "导出计费事件 CSV",
  "Group chargeback": "分组分摊",
  "Images": "图像",
  "Multimodal signals": "多模态信号",
  "Music sec": "音乐秒",
  "No accounting-mode breakdown is available yet.": "暂无记账模式拆分数据。",
  "No billing events match the current filters.": "当前筛选条件下没有匹配的计费事件。",
  "No capability billing mix is available yet.": "暂无能力计费构成数据。",
  "No recent billing events yet": "暂无近期计费事件",
  "None": "无",
  "Operators can audit profile application, compiled snapshots, and fallback posture without leaving usage review.": "运营无需离开使用复核即可审计配置应用、编译快照和回退态势。",
  "Operators can inspect hold-to-settlement posture without leaving multimodal usage review.": "运营无需离开多模态使用复核即可查看从冻结到结算的态势。",
  "Plans": "方案",
  "Pricing posture": "定价态势",
  "Rates": "费率",
  "Recent billing events": "近期计费事件",
  "Recent billing events appear once routed requests create billable multimodal traffic.": "当路由请求产生可计费的多模态流量后，近期计费事件会显示在此。",
  "Recent billing events keep multimodal chargeback, provider cost, and routing evidence in one operator review table.": "近期计费事件会在同一张运营复核表中呈现多模态分摊、供应商成本和路由证据。",
  "Routing evidence": "路由证据",
  "Signal": "信号",
  "Top API key groups by visible customer charge.": "按可见客户收费排序的 API 密钥分组。",
  "Total accounts": "账户总数",
  "Track token, image, audio, video, and music exposure from routed billing events.": "跟踪路由计费事件中的 Token、图像、音频、视频和音乐敞口。",
  "Ungrouped": "未分组",
  "Usage review now stays anchored to the canonical commercial account inventory.": "使用复核现在锚定在规范化商业账户清单之上。",
  "Video sec": "视频秒",
  "Window": "\u7a97\u53e3",
  "Policies": "\u7b56\u7565",
  "Live windows": "\u5b9e\u65f6\u7a97\u53e3",
  "Manage routing profiles": "\u7ba1\u7406\u8def\u7531\u914d\u7f6e",
  "Snapshot evidence": "\u5feb\u7167\u8bc1\u636e",
};

const ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS: Record<string, string> = {
  "Suspended": "\u5df2\u6682\u505c",
  "Manage groups": "\u7ba1\u7406\u5206\u7ec4",
  "Commercial governance": "\u5546\u4e1a\u6cbb\u7406",
  "Credit holds, settlement capture, and liability posture stay visible while governing API access.": "\u7ba1\u7406 API \u8bbf\u95ee\u65f6\uff0c\u4fe1\u7528\u51bb\u7ed3\u3001\u7ed3\u7b97\u6263\u6536\u4e0e\u8d23\u4efb\u6001\u52bf\u4ecd\u4f1a\u4fdd\u6301\u53ef\u89c1\u3002",
  "Operators can confirm that API key issuance is mapped onto live commercial account inventory.": "\u8fd0\u7ef4\u4eba\u5458\u53ef\u4ee5\u786e\u8ba4 API \u5bc6\u94a5\u7b7e\u53d1\u5df2\u6620\u5c04\u5230\u5f53\u524d\u5728\u7ebf\u7684\u5546\u4e1a\u8d26\u6237\u5e93\u5b58\u3002",
  "Group policy": "\u5206\u7ec4\u7b56\u7565",
  "API key lifecycle, route posture, and bootstrap workflows stay attached to the selected registry row.": "API \u5bc6\u94a5\u751f\u547d\u5468\u671f\u3001\u8def\u7531\u6001\u52bf\u4e0e\u5f15\u5bfc\u6d41\u7a0b\u4f1a\u6301\u7eed\u5173\u8054\u5230\u5f53\u524d\u9009\u4e2d\u7684\u767b\u8bb0\u884c\u3002",
  "Group defaults and inherited posture bound to this key.": "\u4e0e\u8be5\u5bc6\u94a5\u7ed1\u5b9a\u7684\u5206\u7ec4\u9ed8\u8ba4\u503c\u4e0e\u7ee7\u627f\u6001\u52bf\u3002",
  "API key group": "API \u5bc6\u94a5\u5206\u7ec4",
  "No group assigned": "\u672a\u5206\u914d\u5206\u7ec4",
  "No routing profile": "\u65e0\u8def\u7531\u914d\u7f6e",
  "No group": "\u65e0\u5206\u7ec4",
  "Direct key policy": "\u76f4\u63a5\u5bc6\u94a5\u7b56\u7565",
  "Compiled snapshots currently loaded from the routing evidence layer.": "\u5f53\u524d\u5df2\u4ece\u8def\u7531\u8bc1\u636e\u5c42\u52a0\u8f7d\u7684\u7f16\u8bd1\u5feb\u7167\u3002",
  "Snapshots carrying an applied routing profile id.": "\u643a\u5e26\u5df2\u5e94\u7528\u8def\u7531\u914d\u7f6e ID \u7684\u5feb\u7167\u6570\u91cf\u3002",
  "Review how routing profiles compile into route-key and capability evidence before changing provider posture.": "\u8c03\u6574\u63d0\u4f9b\u5546\u6001\u52bf\u524d\uff0c\u8bf7\u5148\u5ba1\u67e5\u8def\u7531\u914d\u7f6e\u5982\u4f55\u7f16\u8bd1\u4e3a\u8def\u7531\u952e\u4e0e\u80fd\u529b\u8bc1\u636e\u3002",
  "{snapshots} snapshots": "{snapshots} \u4e2a\u5feb\u7167",
  "No compiled routing evidence is available yet.": "\u6682\u65e0\u53ef\u7528\u7684\u7f16\u8bd1\u8def\u7531\u8bc1\u636e\u3002",
  "Routing impact": "\u8def\u7531\u5f71\u54cd",
  "Inspect how the selected provider participates in compiled snapshots, reusable routing profiles, and default-route posture.": "\u67e5\u770b\u6240\u9009\u63d0\u4f9b\u5546\u5982\u4f55\u53c2\u4e0e\u7f16\u8bd1\u5feb\u7167\u3001\u53ef\u590d\u7528\u8def\u7531\u914d\u7f6e\u4ee5\u53ca\u9ed8\u8ba4\u8def\u7531\u6001\u52bf\u3002",
  "Top affected routing profiles": "\u53d7\u5f71\u54cd\u6700\u5927\u7684\u8def\u7531\u914d\u7f6e",
  "{count} snapshots": "{count} \u4e2a\u5feb\u7167",
  "{count} groups": "{count} \u4e2a\u5206\u7ec4",
  "No compiled routing evidence currently references this provider through a reusable routing profile.": "\u5f53\u524d\u5c1a\u65e0\u7f16\u8bd1\u8def\u7531\u8bc1\u636e\u901a\u8fc7\u53ef\u590d\u7528\u8def\u7531\u914d\u7f6e\u5f15\u7528\u8be5\u63d0\u4f9b\u5546\u3002",
  "Routing evidence is empty": "\u8def\u7531\u8bc1\u636e\u4e3a\u7a7a",
  "Recent compiled snapshots": "\u6700\u8fd1\u7f16\u8bd1\u5feb\u7167",
  "Fallback path": "\u56de\u9000\u8def\u5f84",
};

const ADMIN_ZH_TRAFFIC_TRANSLATIONS: Record<string, string> = {
  "Billing events stay aligned with quota posture and remaining project headroom.": "\u8ba1\u8d39\u4e8b\u4ef6\u4f1a\u4e0e\u914d\u989d\u6001\u52bf\u548c\u9879\u76ee\u5269\u4f59\u4f59\u91cf\u4fdd\u6301\u5bf9\u9f50\u3002",
  "Billing events summarize project chargeback, request volume, and quota posture in one view.": "\u8ba1\u8d39\u4e8b\u4ef6\u4f1a\u5728\u540c\u4e00\u89c6\u56fe\u4e2d\u6c47\u603b\u9879\u76ee\u5206\u644a\u3001\u8bf7\u6c42\u91cf\u548c\u914d\u989d\u6001\u52bf\u3002",
  "Billing-event analytics stay visible across all traffic lenses.": "\u8ba1\u8d39\u4e8b\u4ef6\u5206\u6790\u4f1a\u5728\u6240\u6709\u6d41\u91cf\u89c6\u89d2\u4e2d\u6301\u7eed\u53ef\u89c1\u3002",
  "No accounting-mode mix is visible for this slice.": "\u5f53\u524d\u65f6\u95f4\u5207\u7247\u6682\u65e0\u53ef\u89c1\u7684\u8ba1\u8d39\u6a21\u5f0f\u5206\u5e03\u3002",
  "No billing events match the current filters": "\u5f53\u524d\u7b5b\u9009\u6761\u4ef6\u4e0b\u6ca1\u6709\u5339\u914d\u7684\u8ba1\u8d39\u4e8b\u4ef6",
  "No capability mix is visible for this slice.": "\u5f53\u524d\u65f6\u95f4\u5207\u7247\u6682\u65e0\u53ef\u89c1\u7684\u80fd\u529b\u5206\u5e03\u3002",
  "No fallback used": "\u672a\u4f7f\u7528\u56de\u9000",
  "No group chargeback data is visible for this slice.": "\u5f53\u524d\u65f6\u95f4\u5207\u7247\u6682\u65e0\u53ef\u89c1\u7684\u5206\u7ec4\u5206\u644a\u6570\u636e\u3002",
  "Not captured": "\u672a\u91c7\u96c6",
  "Platform credit, BYOK, and passthrough mix remain visible.": "\u5e73\u53f0\u989d\u5ea6\u3001BYOK \u548c\u900f\u4f20\u6a21\u5f0f\u7684\u5360\u6bd4\u4f1a\u6301\u7eed\u53ef\u89c1\u3002",
  "Provider selection, fallback evidence, compiled snapshots, and SLO posture remain visible for every routing decision.": "\u6bcf\u6b21\u8def\u7531\u51b3\u7b56\u90fd\u4f1a\u6301\u7eed\u5c55\u793a\u63d0\u4f9b\u5546\u9009\u62e9\u3001\u56de\u9000\u8bc1\u636e\u3001\u7f16\u8bd1\u5feb\u7167\u548c SLO \u6001\u52bf\u3002",
  "Top billed capabilities in the active time slice.": "\u5f53\u524d\u65f6\u95f4\u5207\u7247\u4e2d\u6536\u8d39\u6700\u9ad8\u7684\u80fd\u529b\u3002",
  "Try a broader query to inspect more billing events.": "\u8bf7\u5c1d\u8bd5\u66f4\u5bbd\u7684\u67e5\u8be2\u6761\u4ef6\u4ee5\u67e5\u770b\u66f4\u591a\u8ba1\u8d39\u4e8b\u4ef6\u3002",
  "Upstream cost": "\u4e0a\u6e38\u6210\u672c",
};

const ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS: Record<string, string> = {
  "Commercial accounts": "\u5546\u4e1a\u8d26\u6237",
  "Canonical payable accounts currently discoverable by the commercial control plane.": "\u5f53\u524d\u5546\u4e1a\u63a7\u5236\u9762\u53ef\u8bc6\u522b\u7684\u89c4\u8303\u5316\u5e94\u4ed8\u8d26\u6237\u3002",
  "Available balance": "\u53ef\u7528\u4f59\u989d",
  "Spendable credit still available across the commercial account inventory.": "\u5f53\u524d\u5546\u4e1a\u8d26\u6237\u6e05\u5355\u4e2d\u4ecd\u53ef\u4f7f\u7528\u7684\u4fe1\u7528\u989d\u5ea6\u3002",
  "Active accounts": "\u6d3b\u8dc3\u8d26\u6237",
  "Accounts currently able to receive holds and settlement capture.": "\u5f53\u524d\u53ef\u63a5\u6536\u51bb\u7ed3\u548c\u7ed3\u7b97\u6355\u83b7\u7684\u8d26\u6237\u3002",
  "Suspended accounts": "\u5df2\u6682\u505c\u8d26\u6237",
  "Accounts blocked from new commercial admission until operator review.": "\u5728\u8fd0\u8425\u590d\u6838\u524d\u7981\u6b62\u65b0\u7684\u5546\u4e1a\u51c6\u5165\u7684\u8d26\u6237\u3002",
  "Held balance": "\u51bb\u7ed3\u4f59\u989d",
  "Credit currently reserved by request admission and pending settlement flows.": "\u5f53\u524d\u88ab\u8bf7\u6c42\u51c6\u5165\u548c\u5f85\u7ed3\u7b97\u6d41\u7a0b\u9884\u7559\u7684\u4fe1\u7528\u989d\u5ea6\u3002",
  "Commercial accounts, settlement explorer, and pricing governance now live as a first-class admin module.": "\u5546\u4e1a\u8d26\u6237\u3001\u7ed3\u7b97\u5206\u6790\u548c\u5b9a\u4ef7\u6cbb\u7406\u73b0\u5df2\u6210\u4e3a\u4e00\u7ea7\u7ba1\u7406\u6a21\u5757\u3002",
  "Account posture keeps status, held balance, and admission readiness visible in one surface.": "\u8d26\u6237\u6001\u52bf\u4f1a\u5728\u540c\u4e00\u89c6\u56fe\u4e2d\u5c55\u793a\u72b6\u6001\u3001\u51bb\u7ed3\u4f59\u989d\u548c\u51c6\u5165\u5c31\u7eea\u5ea6\u3002",
};

const ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS: Record<string, string> = {
  "Commercial control plane": "商业控制台",
  "Operators can audit commercial accounts, request settlement posture, and pricing governance without leaving a dedicated module.": "运营可在独立模块中审计商业账户、请求结算态势和定价治理。",
  "Captured, released, and refunded request settlements ready for operator investigation.": "已捕获、已释放和已退款的请求结算记录，可供运营排查。",
  "Recent commerce orders stay linked to provider callbacks and operator-visible payment evidence.": "最近的商业订单持续关联支付回调和运营可见的支付证据。",
  "Live metric-rate rows currently shaping canonical commercial charging.": "当前生效的指标费率行正在塑造规范化商业计费。",
  "Active pricing plans": "生效中的定价方案",
  "Commercial pricing plans that are active and currently effective in the control plane.": "当前在控制台中已启用且生效的商业定价方案。",
  "Priced metrics": "已定价指标",
  "Distinct metric codes already governed by canonical pricing rates.": "已由规范化定价费率治理的不同指标编码。",
  "Primary plan": "主定价方案",
  "No active plan": "暂无生效方案",
  "The first active pricing plan remains the quickest operator reference point.": "首个生效的定价方案仍是运营最快的参考点。",
  "Charge unit": "计费单位",
  "Primary metered unit keeps settlement granularity explicit for operator review.": "主计费单位让运营复核时的结算粒度保持清晰。",
  "Billing method": "计费方式",
  "Settlement method shows whether the primary rate charges per unit, flat, or step-based.": "结算方式会说明主费率按量计费、固定计费还是阶梯计费。",
  "Display unit makes the commercial rate readable for token and multimodal pricing review.": "展示单位让 Token 与多模态定价费率更易于复核。",
  "Order audit detail opens the full order, payment, and coupon evidence stream for the selected order.": "订单审计详情会为所选订单展开完整的订单、支付和优惠券证据流。",
  "No order payment evidence yet": "暂无订单支付证据",
  "The right rail keeps the most recent commercial settlement evidence in view for rapid operator triage.": "右侧栏持续展示最新商业结算证据，便于快速运营分诊。",
  "Order audit detail": "订单审计详情",
  "Loading selected order": "正在加载所选订单",
  "Loading order audit evidence": "正在加载订单审计证据",
  "Order audit detail unavailable": "订单审计详情暂不可用",
  "Commercial order, checkout, and coupon evidence stay bundled here so operators can reconstruct fulfillment and refund posture without switching modules.": "商业订单、结账和优惠券证据会在此统一汇聚，便于运营无需切换模块即可还原履约与退款态势。",
  "Order audit detail keeps payment callbacks and coupon lifecycle evidence scoped to the selected order so reconciliation triage stays deterministic.": "订单审计详情会将支付回调与优惠券生命周期证据限定在所选订单内，确保对账分诊结果可确定。",
};

const ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS: Record<string, string> = {
  "Pricing governance": "定价治理",
  "Account": "账户",
  "Account #{id}": "账户 #{id}",
  "Request": "请求",
  "Request #{id}": "请求 #{id}",
  "Hold #{id}": "冻结 #{id}",
  "Order": "订单",
  "Order #{id}": "订单 #{id}",
  "Investigation": "排查",
  "View order audit": "查看订单审计",
  "Entry": "分录",
  "Credits": "额度",
  "Settlement": "结算",
  "Retail charge": "零售价",
  "Refund credits": "退款额度",
  "Provider cost": "供应商成本",
  "Event": "事件",
  "Processing": "处理状态",
  "Refund state": "退款状态",
  "Target kind": "目标类型",
  "List price": "标价",
  "Payable price": "应付价格",
  "Granted units": "发放额度",
  "Bonus units": "赠送额度",
  "Order status after": "后续订单状态",
  "Payment event id": "支付事件 ID",
  "Dedupe key": "幂等键",
  "No linked request": "无关联请求",
  "No linked hold": "无关联冻结",
  "No provider event id": "无供应商事件 ID",
  "No derived order status": "无推导订单状态",
  "No payment evidence": "无支付证据",
  "Pending evidence": "待补充证据",
  "Unlinked": "未关联",
  "Loading": "加载中",
  "n/a": "不适用",
  "Retail charge: {amount}": "零售价：{amount}",
  "Provider cost: {amount}": "供应商成本：{amount}",
  "Captured credits: {count}": "已捕获额度：{count}",
  "Commercial holds that still need capture, release, expiry, or operator intervention.": "仍需捕获、释放、到期处理或人工干预的商业冻结。",
  "Settlements already converted into captured commercial liability evidence.": "已转换为商业负债证据的已捕获结算。",
  "Rejected or failed provider callbacks stay visible before they drift into silent payment reconciliation gaps.": "被拒绝或失败的供应商回调会持续可见，避免其悄然演变为支付对账盲区。",
};

const ADMIN_ZH_MARKETING_TRANSLATIONS: Record<string, string> = {
  "Payment, coupon, and campaign evidence is being loaded for the selected order.": "\u6b63\u5728\u4e3a\u6240\u9009\u8ba2\u5355\u52a0\u8f7d\u652f\u4ed8\u3001\u4f18\u60e0\u5238\u548c\u6d3b\u52a8\u8bc1\u636e\u3002",
  "No coupon applied": "\u672a\u4f7f\u7528\u4f18\u60e0\u5238",
  "Coupon evidence chain": "\u4f18\u60e0\u5238\u8bc1\u636e\u94fe",
  "Reservation, redemption, rollback, code, template, and campaign evidence stays attached so discount posture can be audited together with payment callbacks.": "\u9884\u7559\u3001\u5151\u6362\u3001\u56de\u6eda\u3001\u5238\u7801\u3001\u6a21\u677f\u548c\u6d3b\u52a8\u8bc1\u636e\u4f1a\u7edf\u4e00\u4fdd\u7559\uff0c\u4fbf\u4e8e\u7ed3\u5408\u652f\u4ed8\u56de\u8c03\u4e00\u8d77\u5ba1\u8ba1\u4f18\u60e0\u72b6\u6001\u3002",
  "Reservation": "\u9884\u7559",
  "Redemption": "\u5151\u6362",
  "Rollback count": "\u56de\u6eda\u6b21\u6570",
  "No reservation evidence": "\u6682\u65e0\u9884\u7559\u8bc1\u636e",
  "No redemption evidence": "\u6682\u65e0\u5151\u6362\u8bc1\u636e",
  "No coupon code evidence": "\u6682\u65e0\u4f18\u60e0\u7801\u8bc1\u636e",
  "Coupon template": "\u4f18\u60e0\u5238\u6a21\u677f",
  "No template evidence": "\u6682\u65e0\u6a21\u677f\u8bc1\u636e",
  "Marketing campaign": "\u8425\u9500\u6d3b\u52a8",
  "No campaign evidence": "\u6682\u65e0\u6d3b\u52a8\u8bc1\u636e",
  "Coupon rollback timeline": "\u4f18\u60e0\u5238\u56de\u6eda\u65f6\u95f4\u7ebf",
  "Rollback evidence confirms whether coupon subsidy and inventory were restored during refund handling.": "\u56de\u6eda\u8bc1\u636e\u7528\u4e8e\u786e\u8ba4\u9000\u6b3e\u5904\u7406\u4e2d\u662f\u5426\u5df2\u6062\u590d\u4f18\u60e0\u8865\u8d34\u548c\u5e93\u5b58\u3002",
  "Restored budget": "\u6062\u590d\u9884\u7b97",
  "Restored inventory": "\u6062\u590d\u5e93\u5b58",
  "No coupon rollback evidence has been recorded for this order.": "\u8be5\u8ba2\u5355\u5c1a\u672a\u8bb0\u5f55\u4f18\u60e0\u5238\u56de\u6eda\u8bc1\u636e\u3002",
  "Template governance": "\u6a21\u677f\u6cbb\u7406",
  "{count} active templates": "{count} \u4e2a\u542f\u7528\u6a21\u677f",
  "Campaign budgets": "\u6d3b\u52a8\u9884\u7b97",
  "{count} active campaigns": "{count} \u4e2a\u6d3b\u8dc3\u6d3b\u52a8",
  "Code vault": "\u5238\u7801\u5e93",
  "{count} total codes": "{count} \u4e2a\u5238\u7801",
  "Redemption ledger": "\u5151\u6362\u53f0\u8d26",
  "{count} tracked redemptions": "{count} \u6761\u5df2\u8ddf\u8e2a\u5151\u6362",
  "Rollback trail": "\u56de\u6eda\u8f68\u8ff9",
  "{count} recorded rollbacks": "{count} \u6761\u5df2\u8bb0\u5f55\u56de\u6eda",
  "Activate budget": "\u542f\u7528\u9884\u7b97",
  "Activate campaign": "\u542f\u7528\u6d3b\u52a8",
  "Activate template": "\u542f\u7528\u6a21\u677f",
  "Archive template": "\u5f52\u6863\u6a21\u677f",
  "Budget status": "\u9884\u7b97\u72b6\u6001",
  "Campaign status": "\u6d3b\u52a8\u72b6\u6001",
  "Close budget": "\u5173\u95ed\u9884\u7b97",
  "Code locked by lifecycle": "\u5238\u7801\u53d7\u751f\u547d\u5468\u671f\u9501\u5b9a",
  "Code status": "\u5238\u7801\u72b6\u6001",
  "Disable code": "\u505c\u7528\u5238\u7801",
  "Enable code": "\u542f\u7528\u5238\u7801",
  "Governance controls": "\u6cbb\u7406\u63a7\u5236",
  "Legacy coupon compatibility": "\u65e7\u7248\u4f18\u60e0\u5238\u517c\u5bb9",
  "No budget linked": "\u672a\u5173\u8054\u9884\u7b97",
  "No campaign linked": "\u672a\u5173\u8054\u6d3b\u52a8",
  "No code linked": "\u672a\u5173\u8054\u5238\u7801",
  "No template linked": "\u672a\u5173\u8054\u6a21\u677f",
  "Pause campaign": "\u6682\u505c\u6d3b\u52a8",
  "Template status": "\u6a21\u677f\u72b6\u6001",
  "Template, campaign, budget, and code status controls let operators stop risk exposure or restore offers without editing the whole record.": "\u6a21\u677f\u3001\u6d3b\u52a8\u3001\u9884\u7b97\u548c\u5238\u7801\u72b6\u6001\u63a7\u5236\u53ef\u8ba9\u8fd0\u8425\u5728\u4e0d\u4fee\u6539\u6574\u6761\u8bb0\u5f55\u7684\u60c5\u51b5\u4e0b\u6b62\u635f\u6216\u6062\u590d\u4f18\u60e0\u3002",
  "This record remains available for the compatibility layer while canonical marketing templates, budgets, codes, and rollbacks are governed in the marketing workbench.": "\u8be5\u8bb0\u5f55\u4ecd\u4fdd\u7559\u7ed9\u517c\u5bb9\u5c42\u4f7f\u7528\uff0c\u89c4\u8303\u5316\u7684\u8425\u9500\u6a21\u677f\u3001\u9884\u7b97\u3001\u5238\u7801\u548c\u56de\u6eda\u7531\u8425\u9500\u5de5\u4f5c\u53f0\u7edf\u4e00\u6cbb\u7406\u3002",
  "Use this panel to review the historical coupon posture while the marketing control plane tracks the enterprise lifecycle around issuance and redemption.": "\u53ef\u5728\u6b64\u9762\u677f\u67e5\u770b\u5386\u53f2\u4f18\u60e0\u5238\u6001\u52bf\uff0c\u540c\u65f6\u7531\u8425\u9500\u63a7\u5236\u9762\u8ddf\u8e2a\u53d1\u653e\u548c\u5151\u6362\u76f8\u5173\u7684\u4f01\u4e1a\u7ea7\u751f\u547d\u5468\u671f\u3002",
  "missing": "\u7f3a\u5931",
};

const ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS: Record<string, string> = {
  "Settlement explorer": "\u7ed3\u7b97\u5206\u6790",
  "Settlement explorer highlights open holds, captured requests, and correction posture from canonical settlement records.": "\u7ed3\u7b97\u5206\u6790\u4f1a\u57fa\u4e8e\u89c4\u8303\u5316\u7ed3\u7b97\u8bb0\u5f55\u7a81\u51fa\u663e\u793a\u672a\u5b8c\u6210\u51bb\u7ed3\u3001\u5df2\u6355\u83b7\u8bf7\u6c42\u548c\u7ea0\u504f\u72b6\u6001\u3002",
  "Settlement ledger": "\u7ed3\u7b97\u53f0\u8d26",
  "Settlement ledger keeps capture and refund entries linked to request settlements so operators can audit credits, retail charge, and final correction posture without leaving the commercial module.": "\u7ed3\u7b97\u53f0\u8d26\u4f1a\u5c06\u6355\u83b7\u548c\u9000\u6b3e\u5206\u5f55\u4e0e\u8bf7\u6c42\u7ed3\u7b97\u5173\u8054\u8d77\u6765\uff0c\u4fbf\u4e8e\u8fd0\u8425\u5728\u5546\u4e1a\u6a21\u5757\u5185\u5ba1\u8ba1\u989d\u5ea6\u3001\u96f6\u552e\u6536\u8d39\u548c\u6700\u7ec8\u7ea0\u504f\u72b6\u6001\u3002",
  "Settlement ledger entries will appear here once commercial account history begins landing for the selected control-plane slice.": "\u6240\u9009\u63a7\u5236\u9762\u8303\u56f4\u5f00\u59cb\u5199\u5165\u5546\u4e1a\u8d26\u6237\u5386\u53f2\u540e\uff0c\u7ed3\u7b97\u53f0\u8d26\u5206\u5f55\u4f1a\u663e\u793a\u5728\u8fd9\u91cc\u3002",
  "No settlement ledger entries yet": "\u6682\u65e0\u7ed3\u7b97\u53f0\u8d26\u5206\u5f55",
  "Refund timeline": "\u9000\u6b3e\u65f6\u95f4\u7ebf",
  "Refund timeline isolates correction entries so support and finance can verify credited quantity, linked request, and refund cost posture at a glance.": "\u9000\u6b3e\u65f6\u95f4\u7ebf\u4f1a\u9694\u79bb\u5c55\u793a\u7ea0\u504f\u5206\u5f55\uff0c\u4fbf\u4e8e\u652f\u6301\u548c\u8d22\u52a1\u5feb\u901f\u6838\u5bf9\u5165\u8d26\u6570\u91cf\u3001\u5173\u8054\u8bf7\u6c42\u548c\u9000\u6b3e\u6210\u672c\u72b6\u6001\u3002",
  "Refund activity will appear here once commercial refunds are posted into the account ledger history.": "\u5546\u4e1a\u9000\u6b3e\u8fc7\u8d26\u5230\u8d26\u6237\u53f0\u8d26\u5386\u53f2\u540e\uff0c\u9000\u6b3e\u6d3b\u52a8\u4f1a\u663e\u793a\u5728\u8fd9\u91cc\u3002",
  "No refunds recorded yet": "\u6682\u65e0\u9000\u6b3e\u8bb0\u5f55",
  "Order payment audit": "\u8ba2\u5355\u652f\u4ed8\u5ba1\u8ba1",
  "Order payment audit keeps recent commercial orders linked to payment callbacks, provider evidence, and operator-visible processing posture without loading unbounded order history into the commercial module.": "\u8ba2\u5355\u652f\u4ed8\u5ba1\u8ba1\u4f1a\u5c06\u8fd1\u671f\u5546\u4e1a\u8ba2\u5355\u4e0e\u652f\u4ed8\u56de\u8c03\u3001\u4f9b\u5e94\u5546\u8bc1\u636e\u4ee5\u53ca\u9762\u5411\u8fd0\u8425\u7684\u5904\u7406\u72b6\u6001\u5173\u8054\u5c55\u793a\uff0c\u65e0\u9700\u5728\u5546\u4e1a\u6a21\u5757\u4e2d\u52a0\u8f7d\u65e0\u754c\u8ba2\u5355\u5386\u53f2\u3002",
  "Recent commerce orders will appear here once checkout, webhook, and settlement evidence starts landing in the commercial audit stream.": "\u7ed3\u8d26\u3001Webhook \u548c\u7ed3\u7b97\u8bc1\u636e\u5f00\u59cb\u5199\u5165\u5546\u4e1a\u5ba1\u8ba1\u6d41\u540e\uff0c\u8fd1\u671f\u5546\u4e1a\u8ba2\u5355\u4f1a\u663e\u793a\u5728\u8fd9\u91cc\u3002",
  "Order refund audit": "\u8ba2\u5355\u9000\u6b3e\u5ba1\u8ba1",
  "Order refund audit keeps explicit refund callbacks and refunded-order fallback evidence visible so operators can spot missing callback closure before it becomes a reconciliation blind spot.": "\u8ba2\u5355\u9000\u6b3e\u5ba1\u8ba1\u4f1a\u6301\u7eed\u663e\u793a\u660e\u786e\u7684\u9000\u6b3e\u56de\u8c03\u548c\u9000\u6b3e\u8ba2\u5355\u515c\u5e95\u8bc1\u636e\uff0c\u4fbf\u4e8e\u8fd0\u8425\u5728\u5b83\u6f14\u53d8\u4e3a\u5bf9\u8d26\u76f2\u70b9\u524d\u53d1\u73b0\u7f3a\u5931\u7684\u56de\u8c03\u95ed\u73af\u3002",
  "Refund audit rows will appear here once commercial orders begin entering explicit refund or refunded-order-state correction flows.": "\u5546\u4e1a\u8ba2\u5355\u5f00\u59cb\u8fdb\u5165\u663e\u5f0f\u9000\u6b3e\u6216\u9000\u6b3e\u8ba2\u5355\u72b6\u6001\u7ea0\u504f\u6d41\u7a0b\u540e\uff0c\u9000\u6b3e\u5ba1\u8ba1\u8bb0\u5f55\u4f1a\u663e\u793a\u5728\u8fd9\u91cc\u3002",
  "No refund evidence yet": "\u6682\u65e0\u9000\u6b3e\u8bc1\u636e",
  "Latest settlements": "\u6700\u65b0\u7ed3\u7b97",
  "No settlement evidence yet": "\u6682\u65e0\u7ed3\u7b97\u8bc1\u636e",
  "Latest settlements will appear here once request settlement records start landing from the canonical commercial kernel.": "\u89c4\u8303\u5316\u5546\u4e1a\u5185\u6838\u5f00\u59cb\u5199\u5165\u8bf7\u6c42\u7ed3\u7b97\u8bb0\u5f55\u540e\uff0c\u6700\u65b0\u7ed3\u7b97\u4f1a\u663e\u793a\u5728\u8fd9\u91cc\u3002",
  "Payment evidence timeline": "\u652f\u4ed8\u8bc1\u636e\u65f6\u95f4\u7ebf",
  "Provider callbacks remain ordered here so operators can verify settlement, rejection, and refund sequencing for the selected order.": "\u4f9b\u5e94\u5546\u56de\u8c03\u4f1a\u5728\u8fd9\u91cc\u6309\u987a\u5e8f\u4fdd\u7559\uff0c\u4fbf\u4e8e\u8fd0\u8425\u6838\u5bf9\u6240\u9009\u8ba2\u5355\u7684\u7ed3\u7b97\u3001\u62d2\u7edd\u548c\u9000\u6b3e\u65f6\u5e8f\u3002",
  "No payment evidence has been recorded for this order yet.": "\u8be5\u8ba2\u5355\u5c1a\u672a\u8bb0\u5f55\u652f\u4ed8\u8bc1\u636e\u3002",
  "Open holds": "\u672a\u5b8c\u6210\u51bb\u7ed3",
  "Captured settlements": "\u5df2\u6355\u83b7\u7ed3\u7b97",
  "Refunded settlements": "\u5df2\u9000\u6b3e\u7ed3\u7b97",
  "Rejected callbacks": "\u5df2\u62d2\u7edd\u56de\u8c03",
  "Refund posture keeps correction flows visible inside the settlement explorer.": "\u9000\u6b3e\u72b6\u6001\u8ba9\u7ea0\u504f\u6d41\u7a0b\u5728\u7ed3\u7b97\u5206\u6790\u4e2d\u4fdd\u6301\u53ef\u89c1\u3002",
};

const ADMIN_ZH_PRICING_TRANSLATIONS: Record<string, string> = {
  "A dedicated pricing module keeps settlement-facing pricing governance separate from catalog market prices.": "\u72ec\u7acb\u5b9a\u4ef7\u6a21\u5757\u5c06\u9762\u5411\u7ed3\u7b97\u7684\u5b9a\u4ef7\u6cbb\u7406\u4e0e\u76ee\u5f55\u5e02\u573a\u4ef7\u683c\u5206\u79bb\u3002",
  "Force lifecycle convergence when due planned versions should become active before the next automatic pricing read.": "\u5f53\u5230\u671f\u7684\u8ba1\u5212\u7248\u672c\u5e94\u5728\u4e0b\u4e00\u6b21\u81ea\u52a8\u8bfb\u53d6\u5b9a\u4ef7\u524d\u751f\u6548\u65f6\uff0c\u5f3a\u5236\u6267\u884c\u751f\u547d\u5468\u671f\u6536\u655b\u3002",
  "Plan code": "\u8ba1\u5212\u7f16\u7801",
  "Plan version": "\u8ba1\u5212\u7248\u672c",
  "Credit unit code": "\u4fe1\u7528\u5355\u4f4d\u7f16\u7801",
  "Saving...": "\u4fdd\u5b58\u4e2d...",
  "Create plan": "\u521b\u5efa\u8ba1\u5212",
  "Update plan": "\u66f4\u65b0\u8ba1\u5212",
  "Create new plan": "\u65b0\u5efa\u8ba1\u5212",
  "Immediate": "\u7acb\u5373\u751f\u6548",
  "Schedule plan": "\u8ba1\u5212\u53d1\u5e03",
  "Publish plan": "\u53d1\u5e03\u8ba1\u5212",
  "Retire plan": "\u505c\u7528\u8ba1\u5212",
  "Clone plan": "\u514b\u9686\u8ba1\u5212",
  "Edit plan": "\u7f16\u8f91\u8ba1\u5212",
  "Pricing rate composer": "\u5b9a\u4ef7\u8d39\u7387\u7f16\u8f91\u5668",
  "Create commercial pricing rows with explicit charge units, billing methods, rounding, and minimums.": "\u521b\u5efa\u5546\u4e1a\u5b9a\u4ef7\u8d39\u7387\u884c\uff0c\u660e\u786e\u8ba1\u8d39\u5355\u4f4d\u3001\u7ed3\u7b97\u65b9\u5f0f\u3001\u820d\u5165\u89c4\u5219\u548c\u6700\u5c0f\u503c\u3002",
  "Pricing plan": "\u5b9a\u4ef7\u8ba1\u5212",
  "No pricing plan available": "\u6682\u65e0\u53ef\u7528\u5b9a\u4ef7\u8ba1\u5212",
  "Metric code": "\u6307\u6807\u7f16\u7801",
  "Capability code": "\u80fd\u529b\u7f16\u7801",
  "Model code": "\u6a21\u578b\u7f16\u7801",
  "Provider code": "\u4f9b\u5e94\u5546\u7f16\u7801",
  "Quantity step": "\u8ba1\u8d39\u6b65\u957f",
  "Unit price": "\u5355\u4ef7",
  "Display unit": "\u5c55\u793a\u5355\u4f4d",
  "Minimum billable quantity": "\u6700\u5c0f\u8ba1\u8d39\u6570\u91cf",
  "Minimum charge": "\u6700\u4f4e\u6536\u8d39",
  "Rounding": "\u820d\u5165",
  "Rounding increment": "\u820d\u5165\u589e\u91cf",
  "Included quantity": "\u5305\u542b\u6570\u91cf",
  "Priority": "\u4f18\u5148\u7ea7",
  "Create pricing rate": "\u521b\u5efa\u5b9a\u4ef7\u8d39\u7387",
  "Update rate": "\u66f4\u65b0\u8d39\u7387",
  "Create new rate": "\u65b0\u5efa\u8d39\u7387",
  "Edit rate": "\u7f16\u8f91\u8d39\u7387",
  "Pricing governance keeps commercial plan activation and metric-rate coverage visible for operator review.": "\u5b9a\u4ef7\u6cbb\u7406\u4f1a\u6301\u7eed\u5c55\u793a\u5546\u4e1a\u5957\u9910\u542f\u7528\u548c\u6309\u6307\u6807\u8d39\u7387\u8986\u76d6\u60c5\u51b5\uff0c\u4fbf\u4e8e\u8fd0\u8425\u590d\u6838\u3002",
  "Pricing plans, charge units, and billing methods are maintained here for token, image, audio, video, and music APIs.": "\u6b64\u5904\u7ef4\u62a4 Token\u3001\u56fe\u50cf\u3001\u97f3\u9891\u3001\u89c6\u9891\u548c\u97f3\u4e50 API \u7684\u5b9a\u4ef7\u8ba1\u5212\u3001\u8ba1\u8d39\u5355\u4f4d\u4e0e\u7ed3\u7b97\u65b9\u5f0f\u3002",
  "Operators define versioned commercial plans before adding rate rows.": "\u8fd0\u8425\u9700\u5148\u5b9a\u4e49\u5e26\u7248\u672c\u7684\u5546\u4e1a\u5957\u9910\uff0c\u518d\u65b0\u589e\u8d39\u7387\u884c\u3002",
  "Create a pricing plan before maintaining pricing rates.": "\u8bf7\u5148\u521b\u5efa\u5b9a\u4ef7\u8ba1\u5212\uff0c\u518d\u7ef4\u62a4\u5b9a\u4ef7\u8d39\u7387\u3002",
  "Pricing plans and rates define the commercial surface that gateway access policies must honor.": "\u5b9a\u4ef7\u8ba1\u5212\u548c\u8d39\u7387\u5171\u540c\u5b9a\u4e49\u4e86\u7f51\u5173\u8bbf\u95ee\u7b56\u7565\u5fc5\u987b\u9075\u5faa\u7684\u5546\u4e1a\u9762\u89c4\u3002",
  "Pricing plans": "定价方案",
  "Versioned commercial plan headers available to operators.": "提供给运营查看的版本化商业方案头信息。",
  "Charge units": "计费单位",
  "Distinct units already represented in canonical pricing rows.": "规范化定价行中已覆盖的不同计费单位。",
  "Billing methods": "计费方式",
  "Settlement methods visible in active pricing definitions.": "当前生效定价定义中可见的结算方式。",
  "Due planned versions": "到期待生效版本",
  "Planned versions already inside their effective window and eligible for lifecycle convergence.": "已进入生效窗口且可执行生命周期收敛的计划版本。",
  "Pricing rates": "定价费率",
  "Token pricing and media pricing rows currently maintained.": "当前维护中的 Token 定价和媒体定价行。",
  "Synchronize lifecycle": "同步生命周期",
  "Synchronizing...": "同步中...",
  "Last sync activated {planCount} plan versions and {rateCount} pricing rows.": "上次同步已激活 {planCount} 个方案版本和 {rateCount} 条定价费率。",
  "Last sync skipped {count} due planned versions because no rate rows were attached.": "上次同步跳过了 {count} 个到期计划版本，因为未附带费率行。",
  "Last sync found no due planned versions that required lifecycle changes.": "上次同步未发现需要生命周期变更的到期计划版本。",
  "Token pricing": "Token 定价",
  "Token pricing stays explicit for input, output, and cache-related usage.": "Token 定价会明确区分输入、输出和缓存相关用量。",
  "Media pricing": "媒体定价",
  "Media pricing covers images, audio, video, and music with modality-native units.": "媒体定价按模态原生单位覆盖图像、音频、视频和音乐。",
  "Charge units define what quantity gets billed in the commercial settlement layer.": "计费单位定义商业结算层按什么数量计费。",
  "Billing methods stay standardized so settlement logic can evolve without schema churn.": "计费方式保持标准化，让结算逻辑演进时无需频繁调整模式。",
  "Input token": "输入 Token",
  "Output token": "输出 Token",
  "Cache read token": "缓存读取 Token",
  "Cache write token": "缓存写入 Token",
  "Request": "请求",
  "Image": "图像",
  "Audio second": "音频秒",
  "Audio minute": "音频分钟",
  "Video second": "视频秒",
  "Video minute": "视频分钟",
  "Music track": "音乐曲目",
  "Character": "字符",
  "Storage MB day": "存储 MB·天",
  "Tool call": "工具调用",
  "Unit": "单位",
  "Prompt and ingestion token pricing.": "提示词与摄取 Token 的定价。",
  "Completion and generation token pricing.": "输出与生成 Token 的定价。",
  "Read-side cached token pricing.": "读取侧缓存 Token 的定价。",
  "Write-side cache population pricing.": "写入侧缓存填充的定价。",
  "Flat request admission or invocation pricing.": "固定的请求准入或调用定价。",
  "Per-image generation pricing.": "按每张图像生成计费。",
  "Per-second audio processing pricing.": "按每秒音频处理计费。",
  "Minute-based audio processing pricing.": "按分钟音频处理计费。",
  "Per-second video generation pricing.": "按每秒视频生成计费。",
  "Minute-based video generation pricing.": "按分钟视频生成计费。",
  "Per-track music generation pricing.": "按每首音乐生成计费。",
  "Per-character text or OCR pricing.": "按每字符文本或 OCR 计费。",
  "Storage footprint pricing over time.": "按时间计量的存储占用定价。",
  "Per tool or function invocation pricing.": "按每次工具或函数调用计费。",
  "Fallback commercial unit when no specialized unit applies.": "无专用单位时使用的回退商业单位。",
  "Per unit": "按单位",
  "Flat": "固定",
  "Step": "阶梯",
  "Included then per unit": "先包含后按单位",
  "Quantity times unit price.": "数量乘以单价。",
  "One fixed charge per matched operation.": "每次匹配操作收取固定费用。",
  "Charge by normalized quantity steps.": "按标准化数量阶梯收费。",
  "Burn included usage before overage pricing.": "先消耗包含额度，再按超额计费。",
  "No rounding": "不舍入",
  "Round up": "向上取整",
  "Round down": "向下取整",
  "Round half up": "四舍五入",
  "{count} x {unit}": "{count} 个 {unit}",
  "USD / 1M input tokens": "美元 / 百万输入 Token",
  "USD / image": "美元 / 图像",
  "USD / input token": "美元 / 输入 Token",
  "USD / music track": "美元 / 音乐曲目",
  "USD / request": "美元 / 请求",
};

const ADMIN_ZH_MISC_CONTRACT_TRANSLATIONS: Record<string, string> = {
  'Commercial': '\u5546\u4e1a',
  'Revenue': '\u8425\u6536',
  'Commercial accounts, settlement explorer, and pricing governance': '\u5546\u4e1a\u8d26\u6237\u3001\u7ed3\u7b97\u5206\u6790\u4e0e\u5b9a\u4ef7\u6cbb\u7406',
  'Pricing': '\u5b9a\u4ef7',
  'Finops': '\u8d22\u52a1\u8fd0\u8425',
  'Pricing plans, charge units, and billing method governance': '\u5b9a\u4ef7\u65b9\u6848\u3001\u8ba1\u8d39\u5355\u5143\u4e0e\u8d26\u5355\u65b9\u5f0f\u6cbb\u7406',
  'Example: #2563eb': '\u793a\u4f8b\uff1a#2563eb',
  'Example: /v1/chat/completions': '\u793a\u4f8b\uff1a/v1/chat/completions',
  'Example: retail-pro': '\u793a\u4f8b\uff1aretail-pro',
  'Example: Retail Pro': '\u793a\u4f8b\uff1aRetail Pro',
  'Example: USD': '\u793a\u4f8b\uff1aUSD',
  'Example: credit': '\u793a\u4f8b\uff1acredit',
  'Example: token.input': '\u793a\u4f8b\uff1atoken.input',
  'Example: responses': '\u793a\u4f8b\uff1aresponses',
  'Example: gpt-4.1': '\u793a\u4f8b\uff1agpt-4.1',
  'Example: provider-openai-official': '\u793a\u4f8b\uff1aprovider-openai-official',
  'Example: Retail text input pricing': '\u793a\u4f8b\uff1aRetail text input pricing',
  'Draft': '\u8349\u7a3f',
  'Planned': '\u5df2\u8ba1\u5212',
  Shell: '\u5916\u58f3',
  'sk-router-live-demo': 'sk-router-live-demo',
  'Authorization: Bearer {token}': 'Authorization: Bearer {token}',
};

type AdminTranslationLocale = 'en-US' | 'zh-CN';

const ADMIN_TRANSLATIONS: Record<AdminTranslationLocale, Record<string, string>> = {
  'en-US': BASE_ADMIN_TRANSLATIONS['en-US'],
  'zh-CN': {
    ...BASE_ADMIN_TRANSLATIONS['zh-CN'],
    ...ADMIN_ZH_MISC_CONTRACT_TRANSLATIONS,
    ...ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS,
    ...ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS,
    ...ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS,
    ...ADMIN_ZH_TRAFFIC_TRANSLATIONS,
    ...ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS,
    ...ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS,
    ...ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS,
    ...ADMIN_ZH_MARKETING_TRANSLATIONS,
    ...ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS,
    ...ADMIN_ZH_PRICING_TRANSLATIONS,
  },
};

// END CONTRACT TRANSLATIONS

export type AdminLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, unknown>;

type AdminI18nContextValue = {
  formatCurrency: (value: number, fractionDigits?: number) => string;
  formatDateTime: (value?: number | null) => string;
  formatNumber: (value: number) => string;
  locale: AdminLocale;
  setLocale: (locale: AdminLocale) => void;
  t: (text: string, values?: TranslationValues) => string;
};

const ADMIN_I18N_STORAGE_KEY = 'sdkwork-router-admin.locale.v2';
const AdminI18nContext = createContext<AdminI18nContextValue | null>(null);

export const ADMIN_LOCALE_OPTIONS: Array<{ id: AdminLocale; label: string }> = [
  { id: 'en-US', label: 'English' },
  { id: 'zh-CN', label: 'Simplified Chinese' },
];

let activeAdminLocale: AdminLocale = 'en-US';


function interpolate(text: string, values?: TranslationValues) {
  if (!values) {
    return text;
  }

  return Object.entries(values).reduce(
    (result, [key, value]) => result.replaceAll(`{${key}}`, String(value)),
    text,
  );
}

function resolveTranslation(locale: AdminLocale, text: string) {
  return ADMIN_TRANSLATIONS[locale][text] ?? text;
}

function normalizeLocale(value: string | null | undefined): AdminLocale {
  if (!value) {
    return 'en-US';
  }

  return value.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

function resolveInitialLocale(): AdminLocale {
  if (typeof window === 'undefined') {
    return activeAdminLocale;
  }

  try {
    const persisted = window.localStorage.getItem(ADMIN_I18N_STORAGE_KEY);
    if (persisted) {
      return normalizeLocale(persisted);
    }
  } catch {
    // Ignore storage access failures and fall back to browser locale.
  }

  return normalizeLocale(window.navigator.language);
}

export function translateAdminText(text: string, values?: TranslationValues) {
  return interpolate(resolveTranslation(activeAdminLocale, text), values);
}

export function formatAdminDateTime(value?: number | null) {
  if (!value) {
    return '-';
  }

  return new Intl.DateTimeFormat(activeAdminLocale, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}

export function formatAdminNumber(value: number) {
  return new Intl.NumberFormat(activeAdminLocale).format(value);
}

export function formatAdminCurrency(value: number, fractionDigits = 2) {
  return new Intl.NumberFormat(activeAdminLocale, {
    currency: 'USD',
    maximumFractionDigits: fractionDigits,
    minimumFractionDigits: fractionDigits,
    style: 'currency',
  }).format(value);
}

export function AdminI18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<AdminLocale>(resolveInitialLocale);

  useEffect(() => {
    activeAdminLocale = locale;

    if (typeof document !== 'undefined') {
      document.documentElement.lang = locale;
    }

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(ADMIN_I18N_STORAGE_KEY, locale);
      } catch {
        // Ignore storage write failures.
      }
    }
  }, [locale]);

  const value = useMemo<AdminI18nContextValue>(
    () => ({
      formatCurrency: (value, fractionDigits) =>
        new Intl.NumberFormat(locale, {
          currency: 'USD',
          maximumFractionDigits: fractionDigits ?? 2,
          minimumFractionDigits: fractionDigits ?? 2,
          style: 'currency',
        }).format(value),
      formatDateTime: (value) => {
        if (!value) {
          return '-';
        }

        return new Intl.DateTimeFormat(locale, {
          dateStyle: 'medium',
          timeStyle: 'short',
        }).format(new Date(value));
      },
      formatNumber: (value) => new Intl.NumberFormat(locale).format(value),
      locale,
      setLocale,
      t: (text, values) => interpolate(resolveTranslation(locale, text), values),
    }),
    [locale],
  );

  return <AdminI18nContext.Provider value={value}>{children}</AdminI18nContext.Provider>;
}

export function useAdminI18n(): AdminI18nContextValue {
  const context = useContext(AdminI18nContext);

  if (!context) {
    throw new Error('Admin i18n hooks must be used inside AdminI18nProvider.');
  }

  return context;
}



