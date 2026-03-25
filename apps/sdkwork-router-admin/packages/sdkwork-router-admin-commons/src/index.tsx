import * as DialogPrimitive from '@radix-ui/react-dialog';
import * as LabelPrimitive from '@radix-ui/react-label';
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';
import { clsx, type ClassValue } from 'clsx';
import { Search as SearchIcon, X } from 'lucide-react';
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

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

export type AdminLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, string | number>;

type AdminI18nContextValue = {
  locale: AdminLocale;
  setLocale: (locale: AdminLocale) => void;
  t: (text: string, values?: TranslationValues) => string;
  formatDateTime: (value?: number | null) => string;
  formatNumber: (value: number) => string;
  formatCurrency: (value: number, fractionDigits?: number) => string;
};

const ADMIN_I18N_STORAGE_KEY = 'sdkwork-router-admin.locale.v1';

const adminMessages: Record<Exclude<AdminLocale, 'en-US'>, Record<string, string>> = {
  'zh-CN': {
    'Close dialog': '关闭对话框',
    'More filters': '更多筛选',
    'Hide filters': '收起筛选',
    Language: '语言',
    English: '英文',
    'Simplified Chinese': '简体中文',
    'Synchronizing operator workspace...': '正在同步管理员工作区...',
    'Restoring theme, session, and live control-plane state.': '正在恢复主题、会话与控制平面实时状态。',
    Overview: '总览',
    Users: '用户',
    Tenants: '租户',
    Coupons: '优惠券',
    'Api Key': 'API Key',
    'Route Config': '路由配置',
    'Model Mapping': '模型映射',
    'Usage Records': '使用记录',
    Catalog: '目录',
    Traffic: '流量',
    Operations: '运维',
    Settings: '设置',
    Shell: '壳层',
    'Settings center': '设置中心',
    'Open settings center': '打开设置中心',
    'Close account controls': '关闭账户控制',
    'Open account controls': '打开账户控制',
    'Sign out': '退出登录',
    'Control plane operator': '控制平面操作员',
    'Router Admin': '路由管理后台',
    'Theme, rail, and canvas continuity': '主题、侧栏与画布连续性',
    'Search usage': '搜索使用记录',
    'Refresh workspace': '刷新工作区',
    'Export CSV': '导出 CSV',
    'Export usage CSV': '导出使用 CSV',
    'Clear filters': '清空筛选',
    'Api key': 'API Key',
    'Time range': '时间范围',
    'All time': '全部时间',
    'Last 24 hours': '最近 24 小时',
    'Last 7 days': '最近 7 天',
    'Last 30 days': '最近 30 天',
    'All API keys': '全部 API Key',
    'project, model, provider...': '项目、模型、提供商...',
    'Invalid custom range': '自定义时间范围无效',
    'End date must be on or after start date.': '结束日期必须晚于或等于开始日期。',
    Project: '项目',
    Provider: '提供商',
    Model: '模型',
    'Input tokens': '输入 Token',
    'Output tokens': '输出 Token',
    'Total tokens': '总 Token',
    Units: '计量单位',
    Amount: '金额',
    Created: '创建时间',
    'No usage rows can be shown until the custom date range is corrected.': '修正自定义时间范围后才可显示使用数据。',
    'No usage records match the current gateway filter.': '当前筛选条件下没有匹配的使用记录。',
    'Page {page} of {total} · {count} records': '第 {page} / {total} 页 · 共 {count} 条记录',
    'Previous page': '上一页',
    'Next page': '下一页',
    'Search logs and usage': '搜索日志与使用记录',
    'project, model, provider, route, reason...': '项目、模型、提供商、路由、原因...',
    'View mode': '视图模式',
    'Usage and routing': '使用与路由',
    'Usage only': '仅使用',
    'Routing only': '仅路由',
    'Recent window': '最近窗口',
    'Export routing CSV': '导出路由 CSV',
    'User traffic leaderboard': '用户流量榜',
    'Portal user': '门户用户',
    'Project hotspots': '项目热点',
    'Search operations': '搜索运维项',
    'Provider health': '提供商健康',
    'Managed runtimes': '受管运行时',
    Requests: '请求数',
    Tokens: 'Token 数',
    Status: '状态',
    'Usage records': '使用记录',
    'Billing summary by project': '按项目汇总账单',
    Entries: '条目数',
    Remaining: '剩余额度',
    Quota: '配额',
    'Routing decision logs': '路由决策日志',
    Capability: '能力',
    'Route key': '路由键',
    Strategy: '策略',
    Reason: '原因',
    'Selected provider': '选中提供商',
    active: '启用',
    disabled: '禁用',
    exhausted: '耗尽',
    healthy: '健康',
    'No portal users match the current filter.': '当前筛选条件下没有匹配的门户用户。',
    'No hotspot projects match the current filter.': '当前筛选条件下没有匹配的热点项目。',
    'No usage records match the current filter.': '当前筛选条件下没有匹配的使用记录。',
    'No billing records available.': '暂无账单记录。',
    'No routing decision logs match the current filter.': '当前筛选条件下没有匹配的路由决策日志。',
    'No provider health data available.': '暂无提供商健康数据。',
    'No runtime statuses available.': '暂无运行时状态数据。',
    'control plane settings center': '控制平面设置中心',
    General: '通用',
    'This workspace keeps operator preferences, shell posture, and control plane continuity aligned with claw-studio while preserving router-admin workflows.': '该工作区在保留 router-admin 工作流的同时，使操作员偏好、壳层姿态与控制平面连续性与 claw-studio 保持一致。',
    'The left rail remains the navigation source of truth and the right canvas remains the only content display region for every admin page.': '左侧侧边栏仍是导航真源，右侧画布仍是所有管理员页面唯一的内容展示区。',
    Workspace: '工作区',
    'live shell summary': '实时壳层摘要',
    Operator: '操作员',
    'Theme posture': '主题姿态',
    'Theme color': '主题色',
    'Theme mode': '主题模式',
    Light: '浅色',
    'Bright shell with frosted content panes.': '明亮壳层，搭配磨砂内容面板。',
    Dark: '深色',
    'Claw-style low-glare shell with higher contrast.': 'Claw 风格低眩光壳层，拥有更高对比度。',
    System: '跟随系统',
    'Follow the device preference automatically.': '自动跟随设备主题偏好。',
    Accent: '强调色',
    default: '默认',
    'accent preset': '强调预设',
    'Shell status': '壳层状态',
    Navigation: '导航',
    Behavior: '行为',
    'Sidebar behavior': '侧边栏行为',
    'Expanded sidebar': '展开侧边栏',
    'Keep labels visible across the full left rail.': '在完整左侧导航栏中持续显示标签。',
    'Collapsed sidebar': '折叠侧边栏',
    'Reduce the rail to icon-only navigation without changing the canvas.': '将侧栏收缩为仅图标导航，同时保持画布不变。',
    'sidebar and canvas posture': '侧边栏与画布姿态',
    'Sidebar state': '侧边栏状态',
    'Sidebar width': '侧边栏宽度',
    'Sidebar mode': '侧边栏模式',
    'Hidden nav items': '隐藏导航项',
    'Visible routes': '可见路由',
    'Content region': '内容区域',
    collapsed: '折叠',
    expanded: '展开',
    'sidebar visibility': '侧边栏可见性',
    'right canvas': '右侧画布',
    Appearance: '外观',
    'shell continuity': '壳层连续性',
    Continuity: '连续性',
    'shell posture': '壳层姿态',
    'workspace persistence': '工作区持久化',
    'Theme preferences, sidebar width, hidden entries, and collapse state are persisted so the control-plane workspace reopens with the same shell posture.': '主题偏好、侧边栏宽度、隐藏入口和折叠状态都会持久化保存，确保控制平面重新打开时仍保持相同的壳层姿态。',
    'The layout stays split into a claw-style left navigation rail and a single right content region, keeping product behavior and visual framing consistent.': '布局始终维持为 claw 风格左侧导航栏与单一右侧内容区域，保证产品行为和视觉框架持续一致。',
    'Appearance, navigation, and workspace sections now live in a real settings center instead of a standalone preferences panel.': '外观、导航与工作区分区已进入真正的设置中心，而不再是独立偏好面板。',
    'Every shell preference persists so the control plane reopens with the same workspace and operator posture.': '所有壳层偏好都会持久化保存，确保控制平面重新打开时仍保持相同的工作区与操作员姿态。',
    'Language and locale': '语言与区域',
    'Choose the operator workspace language. Dates, numbers, and shared shell copy follow this setting immediately.': '选择操作员工作区语言。日期、数字和共享壳层文案会立即跟随该设置切换。',
    'Authenticate to open the super-admin workspace.': '登录后即可进入超级管理员工作区。',
    'Create an account': '创建账户',
    'Reset password': '重置密码',
    'Welcome back': '欢迎回来',
    'Join us to start building amazing things.': '加入我们，开始构建出色的产品。',
    'Enter your email to receive a reset link.': '输入邮箱以接收重置链接。',
    'Enter your details to access your account.': '输入你的信息以访问账户。',
    'Signing In...': '登录中...',
    'Sign In': '登录',
    'Send Reset Link': '发送重置链接',
    'Scan to Login': '扫码登录',
    'Use the SDKWork mobile app to scan the QR code for instant access.': '使用 SDKWork 移动端扫描二维码，即可快速登录。',
    'Open SDKWork App': '打开 SDKWork 应用',
    'Full Name': '姓名',
    'John Doe': '张三',
    'Email Address': '邮箱地址',
    'you@example.com': 'you@example.com',
    Password: '密码',
    'Forgot password': '忘记密码',
    'Enter your password': '输入你的密码',
    'Or continue with': '或继续使用',
    GitHub: 'GitHub',
    Google: 'Google',
    "Don't have an account?": '还没有账户？',
    'Sign Up': '注册',
    'Already have an account?': '已有账户？',
    'Back to login': '返回登录',
    your: '你的',
    'Operator account requests stay inside the control plane. Ask an existing admin to provision {name} access from Users.': '操作员账户申请仍在控制平面内处理。请联系现有管理员在“用户”页为 {name} 开通访问。',
    'Password resets are managed by an authenticated admin in Users. Contact the platform owner to rotate your credential safely.': '密码重置由已登录的管理员在“用户”页处理。请联系平台负责人安全轮换凭据。',
    'Use the operator email and password flow for admin access. External SSO remains disabled in this workspace.': '管理员访问请使用邮箱和密码登录，本工作区暂未启用外部 SSO。',
  },
};

const adminSupplementalZhMessages: Record<string, string> = {
  'Control Plane': '控制平面',
  'Workspace Ops': '工作区运营',
  'API Router': 'API 路由',
  'Routing Mesh': '路由网格',
  System: '系统',
  'Global health, alerts, and operator shortcuts': '全局健康、告警与操作员快捷入口',
  'Operator and portal user management': '操作员与门户用户管理',
  'Tenants, projects, and gateway keys': '租户、项目与网关密钥',
  'Campaign and discount code operations': '活动与优惠码运营',
  'Api key registry with key-level route posture and usage visibility': 'API Key 注册表，覆盖密钥级路由姿态与使用可见性',
  'Channel and upstream route registry in the claw apirouter model': '沿用 claw apirouter 模型的渠道与上游路由注册表',
  'Overlay model mapping rules for gateway clients and OpenClaw': '面向网关客户端与 OpenClaw 的模型映射规则',
  'Request history, token volume, and CSV export by Api key': '按 API Key 查看请求历史、Token 用量与 CSV 导出',
  'Channels, providers, and model exposure': '渠道、提供商与模型暴露面',
  'Usage, billing, and request-log visibility': '使用量、计费与请求日志可见性',
  'Health snapshots, reloads, and runtime posture': '健康快照、重载与运行时姿态',
  'Theme mode, theme color, and sidebar preferences': '主题模式、主题色与侧栏偏好',
  'SDKWork Router Admin': 'SDKWork 路由管理后台',
  'Control plane': '控制平面',
  'Expand sidebar': '展开侧栏',
  'Collapse sidebar': '收起侧栏',
  'Operator Workspace': '操作员工作区',
  Search: '搜索',
  Control: '控制',
  Identity: '身份',
  Growth: '增长',
  Key: '密钥',
  Route: '路由',
  Mapping: '映射',
  Usage: '使用',
  Mesh: '网格',
  Audit: '审计',
  Runtime: '运行时',
  Preferences: '偏好',
  light: '浅色',
  dark: '深色',
  system: '跟随系统',
  'tech-blue': '科技蓝',
  lobster: '龙虾红',
  'green-tech': '科技绿',
  zinc: '锌灰',
  violet: '紫罗兰',
  rose: '玫瑰粉',
  'Open workspace search': '打开工作区搜索',
  'Right-side operator canvas aligned to claw-studio in {mode} mode.': '右侧操作员画布在 {mode} 模式下与 claw-studio 保持一致。',
  'Operator workspace language, shell posture, and persistence defaults.': '操作员工作区语言、壳层姿态与持久化默认项。',
  'Workspace posture': '工作区姿态',
  'Current shell posture for the control plane workspace.': '当前控制平面工作区的壳层姿态。',
  'Theme mode and accent color stay synchronized across header, sidebar, and page surfaces.': '主题模式与强调色会在顶部栏、侧栏与页面表面之间保持同步。',
  'Choose how the shell follows light, dark, or system appearance.': '选择壳层如何跟随浅色、深色或系统外观。',
  'Theme color updates accent surfaces without changing the claw-style shell contract.': '主题色会更新强调表面，但不会改变 claw 风格的壳层契约。',
  'Sidebar visibility and left-rail posture stay aligned with claw-studio.': '侧栏可见性与左侧导轨姿态持续与 claw-studio 保持一致。',
  'Keep the left rail expanded or collapse it into icon-only navigation.': '保持左侧导轨展开，或将其折叠为仅图标导航。',
  'Show or hide modules while keeping the left navigation rail compact and stable.': '在保持左侧导航导轨紧凑稳定的同时显示或隐藏模块。',
  'Shell posture, persistence, and canvas continuity for the control plane workspace.': '控制平面工作区的壳层姿态、持久化与画布连续性。',
  'Keep the left navigation rail and the right canvas in a single consistent shell contract.': '让左侧导航导轨与右侧画布保持在同一套一致的壳层契约中。',
  'Theme, sidebar, and hidden-navigation preferences reopen with the same shell posture on the next launch.': '下次启动时，主题、侧栏与隐藏导航偏好会以相同的壳层姿态恢复。',
};

Object.assign(adminMessages['zh-CN'], adminSupplementalZhMessages);

export const ADMIN_LOCALE_OPTIONS: Array<{ id: AdminLocale; label: string }> = [
  { id: 'en-US', label: 'English' },
  { id: 'zh-CN', label: 'Simplified Chinese' },
];

let activeAdminLocale: AdminLocale = 'en-US';

function interpolate(text: string, values?: TranslationValues): string {
  if (!values) {
    return text;
  }

  return Object.entries(values).reduce(
    (result, [key, value]) => result.replaceAll(`{${key}}`, String(value)),
    text,
  );
}

function normalizeLocale(value: string | null | undefined): AdminLocale {
  if (!value) {
    return 'en-US';
  }

  return value.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

function translateForLocale(locale: AdminLocale, text: string, values?: TranslationValues): string {
  const translated = locale === 'en-US' ? text : adminMessages[locale][text] ?? text;
  return interpolate(translated, values);
}

function formatDateTimeForLocale(locale: AdminLocale, value?: number | null): string {
  if (!value) {
    return '-';
  }

  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function formatNumberForLocale(locale: AdminLocale, value: number): string {
  return new Intl.NumberFormat(locale).format(value);
}

function formatCurrencyForLocale(
  locale: AdminLocale,
  value: number,
  fractionDigits = 2,
): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  }).format(value);
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

const AdminI18nContext = createContext<AdminI18nContextValue | null>(null);

export function translateAdminText(text: string, values?: TranslationValues): string {
  return translateForLocale(activeAdminLocale, text, values);
}

export function formatAdminDateTime(value?: number | null): string {
  return formatDateTimeForLocale(activeAdminLocale, value);
}

export function formatAdminNumber(value: number): string {
  return formatNumberForLocale(activeAdminLocale, value);
}

export function formatAdminCurrency(value: number, fractionDigits = 2): string {
  return formatCurrencyForLocale(activeAdminLocale, value, fractionDigits);
}

export function AdminI18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<AdminLocale>(resolveInitialLocale);

  useEffect(() => {
    activeAdminLocale = locale;

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(ADMIN_I18N_STORAGE_KEY, locale);
      } catch {
        // Ignore storage write failures.
      }
    }

    if (typeof document !== 'undefined') {
      document.documentElement.lang = locale;
    }
  }, [locale]);

  const value = useMemo<AdminI18nContextValue>(
    () => ({
      locale,
      setLocale,
      t: (text, values) => translateForLocale(locale, text, values),
      formatDateTime: (value) => formatDateTimeForLocale(locale, value),
      formatNumber: (value) => formatNumberForLocale(locale, value),
      formatCurrency: (value, fractionDigits) =>
        formatCurrencyForLocale(locale, value, fractionDigits),
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
    <div className="adminx-hero">
      <div>
        <p className="adminx-eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="adminx-hero-detail">{detail}</p>
      </div>
      {actions ? <div className="adminx-hero-actions">{actions}</div> : null}
    </div>
  );
}

export function PageToolbar({
  title,
  detail,
  actions,
  children,
  compact = false,
}: {
  title?: string;
  detail?: string;
  actions?: ReactNode;
  children?: ReactNode;
  compact?: boolean;
}) {
  const hasCopy = Boolean(title || detail);

  if (compact) {
    return (
      <section className="adminx-page-toolbar is-compact">
        <div className="adminx-page-toolbar-row">
          {hasCopy ? (
            <div className="adminx-page-toolbar-copy">
              {title ? <h2>{title}</h2> : null}
              {detail ? <p>{detail}</p> : null}
            </div>
          ) : null}
          {actions ? <div className="adminx-page-toolbar-actions">{actions}</div> : null}
          {children ? <div className="adminx-page-toolbar-body">{children}</div> : null}
        </div>
      </section>
    );
  }

  return (
    <section className="adminx-page-toolbar">
      {hasCopy || actions ? (
        <div className="adminx-page-toolbar-head">
          {hasCopy ? (
            <div className="adminx-page-toolbar-copy">
              {title ? <h2>{title}</h2> : null}
              {detail ? <p>{detail}</p> : null}
            </div>
          ) : null}
          {actions ? <div className="adminx-page-toolbar-actions">{actions}</div> : null}
        </div>
      ) : null}
      {children ? <div className="adminx-page-toolbar-body">{children}</div> : null}
    </section>
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
    <section className="adminx-surface">
      <div className="adminx-surface-heading">
        <div>
          <h2>{title}</h2>
          {detail ? <p>{detail}</p> : null}
        </div>
        {actions ? <div className="adminx-surface-actions">{actions}</div> : null}
      </div>
      {children}
    </section>
  );
}

export function StatCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="adminx-stat-card">
      <span>{label}</span>
      <strong>{value}</strong>
      <p>{detail}</p>
    </article>
  );
}

export function Pill({
  tone,
  children,
}: {
  tone?: 'default' | 'live' | 'seed' | 'danger';
  children: ReactNode;
}) {
  return <span className={`adminx-pill adminx-pill-${tone ?? 'default'}`}>{children}</span>;
}

export function DataTable<T>({
  columns,
  rows,
  empty,
  getKey,
}: {
  columns: Array<{ key: string; label: string; render: (row: T) => ReactNode }>;
  rows: T[];
  empty: string;
  getKey: (row: T, index: number) => string;
}) {
  return (
    <div className="adminx-table-wrap">
      <table className="adminx-table">
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key}>{column.label}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={getKey(row, index)} className="adminx-table-row">
              {columns.map((column) => (
                <td key={column.key}>{column.render(row)}</td>
              ))}
            </tr>
          ))}
          {!rows.length ? (
            <tr>
              <td colSpan={columns.length} className="adminx-empty">
                {empty}
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}

export const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-2xl text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/30 focus-visible:ring-offset-2 focus-visible:ring-offset-white disabled:pointer-events-none disabled:opacity-50 dark:focus-visible:ring-offset-zinc-950',
  {
    variants: {
      variant: {
        default:
          'border border-transparent bg-primary-600 text-white shadow-[0_18px_38px_rgba(var(--theme-primary-rgb),0.26)] hover:bg-primary-500 hover:shadow-[0_22px_44px_rgba(var(--theme-primary-rgb),0.34)]',
        secondary:
          'border border-[var(--admin-border)] bg-[var(--admin-bg-panel)] text-[var(--admin-text)] hover:border-primary-500/20 hover:bg-primary-500/10',
        ghost:
          'text-[var(--admin-text-muted)] hover:bg-zinc-500/10 hover:text-[var(--admin-text)]',
        destructive:
          'border border-rose-500/20 bg-rose-500 text-white shadow-[0_18px_38px_rgba(225,29,72,0.22)] hover:bg-rose-400 hover:shadow-[0_22px_44px_rgba(225,29,72,0.3)]',
      },
      size: {
        default: 'h-11 px-4 py-2',
        sm: 'h-10 rounded-xl px-3',
        lg: 'h-12 px-5',
        icon: 'h-11 w-11 p-0',
      },
    },
    defaultVariants: {
      variant: 'secondary',
      size: 'default',
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

export const Input = forwardRef<HTMLInputElement, ComponentPropsWithoutRef<'input'>>(
  ({ className, type, ...props }, ref) => (
    <input
      ref={ref}
      type={type}
      className={cn(
        'flex h-11 w-full rounded-xl border border-[var(--admin-border)] bg-[var(--admin-bg-input)] px-3 py-2 text-sm text-[var(--admin-text)] shadow-sm transition placeholder:text-[var(--admin-text-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/20 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-zinc-950',
        className,
      )}
      {...props}
    />
  ),
);
Input.displayName = 'Input';

export const Select = forwardRef<HTMLSelectElement, ComponentPropsWithoutRef<'select'>>(
  ({ className, ...props }, ref) => (
    <select
      ref={ref}
      className={cn(
        'flex h-11 w-full rounded-xl border border-[var(--admin-border)] bg-[var(--admin-bg-input)] px-3 py-2 text-sm text-[var(--admin-text)] shadow-sm transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/20 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-zinc-950',
        className,
      )}
      {...props}
    />
  ),
);
Select.displayName = 'Select';

export const Label = forwardRef<
  ElementRef<typeof LabelPrimitive.Root>,
  ComponentPropsWithoutRef<typeof LabelPrimitive.Root>
>(({ className, ...props }, ref) => (
  <LabelPrimitive.Root
    ref={ref}
    className={cn('text-sm font-medium text-[var(--admin-text)]', className)}
    {...props}
  />
));
Label.displayName = LabelPrimitive.Root.displayName;

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
    <label className={cn('adminx-toolbar-field', className)}>
      <span className="adminx-toolbar-field-label">{label}</span>
      <span className={cn('adminx-toolbar-field-control', controlClassName)}>{children}</span>
    </label>
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
      className={cn('adminx-toolbar-field-search', className)}
      controlClassName="adminx-toolbar-search-control"
    >
      <span className="adminx-toolbar-search-input">
        <SearchIcon className="adminx-toolbar-search-icon" />
        <Input
          className={cn('adminx-toolbar-search-input-element', inputClassName)}
          {...props}
        />
      </span>
    </ToolbarField>
  );
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
  tone?: 'primary' | 'secondary' | 'danger';
  type?: 'button' | 'submit';
  disabled?: boolean;
  className?: string;
}) {
  const variant = tone === 'primary'
    ? 'default'
    : tone === 'danger'
      ? 'destructive'
      : 'secondary';

  return (
    <Button
      className={cn(className)}
      disabled={disabled}
      onClick={onClick}
      type={type ?? 'button'}
      variant={variant}
    >
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
  const { t } = useAdminI18n();

  return (
    <div className="adminx-toolbar-disclosure">
      <InlineButton onClick={() => setOpen((current) => !current)}>
        {open ? closeLabel ?? t('Hide filters') : openLabel ?? t('More filters')}
      </InlineButton>
      {open ? <div className="adminx-toolbar-disclosure-panel">{children}</div> : null}
    </div>
  );
}

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
    className={cn('adminx-dialog-backdrop', className)}
    {...props}
  />
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

const dialogSizeClassNames = {
  small: 'adminx-dialog-panel-small',
  medium: 'adminx-dialog-panel-medium',
  large: 'adminx-dialog-panel-large',
} as const;

export const DialogContent = forwardRef<
  ElementRef<typeof DialogPrimitive.Content>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
    size?: keyof typeof dialogSizeClassNames;
    showCloseButton?: boolean;
  }
>(({ className, children, size = 'medium', showCloseButton = true, ...props }, ref) => (
  <DialogPortal>
    <DialogOverlay />
    <DialogPrimitive.Content
      ref={ref}
      className={cn('adminx-dialog-panel', dialogSizeClassNames[size], className)}
      {...props}
    >
      {children}
      {showCloseButton ? (
        <DialogPrimitive.Close
          className="adminx-dialog-close"
          aria-label={translateAdminText('Close dialog')}
        >
          <X className="h-4 w-4" />
        </DialogPrimitive.Close>
      ) : null}
    </DialogPrimitive.Content>
  </DialogPortal>
));
DialogContent.displayName = DialogPrimitive.Content.displayName;

export function DialogHeader({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cn('adminx-dialog-header', className)}>{children}</div>;
}

export const DialogTitle = forwardRef<
  ElementRef<typeof DialogPrimitive.Title>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, children, ...props }, ref) => (
  <DialogPrimitive.Title
    ref={ref}
    className={cn('adminx-dialog-title', className)}
    {...props}
  >
    {children}
  </DialogPrimitive.Title>
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;

export const DialogDescription = forwardRef<
  ElementRef<typeof DialogPrimitive.Description>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, children, ...props }, ref) => (
  <DialogPrimitive.Description
    ref={ref}
    className={cn('adminx-dialog-description', className)}
    {...props}
  >
    {children}
  </DialogPrimitive.Description>
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

export function DialogFooter({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cn('adminx-dialog-actions', className)}>{children}</div>;
}

export function AdminDialog({
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
    <>
      <DialogHeader>
        <div className="adminx-dialog-copy">
          <DialogTitle>{title}</DialogTitle>
          {detail ? <DialogDescription>{detail}</DialogDescription> : null}
        </div>
      </DialogHeader>
      <div className="adminx-dialog-body">{children}</div>
      {actions ? <DialogFooter>{actions}</DialogFooter> : null}
    </>
  );
}

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
    <label className={cn('adminx-field', className)}>
      <span>{label}</span>
      {children}
      {hint ? <small className="adminx-field-hint">{hint}</small> : null}
    </label>
  );
}

export function ConfirmDialog({
  open,
  title,
  detail,
  confirmLabel,
  onClose,
  onConfirm,
}: {
  open: boolean;
  title: string;
  detail: string;
  confirmLabel: string;
  onClose: () => void;
  onConfirm: () => Promise<void> | void;
}) {
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!open) {
      setIsSubmitting(false);
    }
  }, [open]);

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !isSubmitting) {
          onClose();
        }
      }}
    >
      <DialogContent size="small">
        <AdminDialog
          title={title}
          detail={detail}
          actions={(
            <>
              <InlineButton disabled={isSubmitting} onClick={onClose}>
                Cancel
              </InlineButton>
              <InlineButton
                tone="danger"
                disabled={isSubmitting}
                onClick={() => {
                  setIsSubmitting(true);
                  void Promise.resolve(onConfirm()).finally(() => {
                    setIsSubmitting(false);
                  });
                }}
              >
                {isSubmitting ? 'Working...' : confirmLabel}
              </InlineButton>
            </>
          )}
        >
          <div className="adminx-confirm-dialog">
            <p>{detail}</p>
          </div>
        </AdminDialog>
      </DialogContent>
    </Dialog>
  );
}
