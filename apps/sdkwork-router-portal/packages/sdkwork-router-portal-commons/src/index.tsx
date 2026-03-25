import * as DialogPrimitive from '@radix-ui/react-dialog';
import * as LabelPrimitive from '@radix-ui/react-label';
import { Slot } from '@radix-ui/react-slot';
import * as TabsPrimitive from '@radix-ui/react-tabs';
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
    'Search offers or ledger': '搜索优惠或账本',
    'View mode': '视图模式',
    Offers: '优惠',
    Ledger: '账本',
    'Create API key': '创建 API Key',
    'Search API keys': '搜索 API Key',
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
    'Open app to scan': '打开应用扫码',
    'Create your workspace access and continue into the portal shell.': '创建你的工作区访问权限并继续进入门户壳层。',
    'Password reset links are not enabled for the current portal backend. Continue back to sign in with your workspace email.': '当前门户后端未启用密码重置链接，请返回并使用工作区邮箱登录。',
    'Sign in to continue to your portal workspace.': '登录后继续进入你的门户工作区。',
    Name: '姓名',
    Email: '邮箱',
    Password: '密码',
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

  useEffect(() => {
    activePortalLocale = locale;

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

export function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot : 'button';

  return (
    <Comp
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}

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
    className={cn('fixed inset-0 z-50 bg-[var(--portal-overlay)] backdrop-blur-sm', className)}
    {...props}
  />
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

export const DialogContent = forwardRef<
  ElementRef<typeof DialogPrimitive.Content>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Content>
>(({ className, children, ...props }, ref) => (
  <DialogPortal>
    <DialogOverlay />
    <DialogPrimitive.Content
      ref={ref}
      className={cn(
        `fixed left-1/2 top-1/2 z-50 grid w-[min(720px,calc(100%-2rem))] -translate-x-1/2 -translate-y-1/2 gap-4 rounded-3xl border ${portalBorder} bg-[var(--portal-overlay-surface)] p-6 ${portalShadowStrong}`,
        className,
      )}
      {...props}
    >
      {children}
      <DialogPrimitive.Close className={`absolute right-4 top-4 rounded-md p-2 ${portalTextMuted} transition hover:bg-[var(--portal-hover-surface)] hover:text-[var(--portal-text-primary)]`}>
        <X className="h-4 w-4" />
        <span className="sr-only">{translatePortalText('Close')}</span>
      </DialogPrimitive.Close>
    </DialogPrimitive.Content>
  </DialogPortal>
));
DialogContent.displayName = DialogPrimitive.Content.displayName;

export function DialogHeader({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('flex flex-col gap-1.5', className)} {...props} />;
}

export function DialogFooter({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('flex flex-wrap items-center justify-end gap-3', className)} {...props} />;
}

export const DialogTitle = forwardRef<
  ElementRef<typeof DialogPrimitive.Title>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Title ref={ref} className={cn(`text-xl font-semibold ${portalText}`, className)} {...props} />
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;

export const DialogDescription = forwardRef<
  ElementRef<typeof DialogPrimitive.Description>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Description ref={ref} className={cn(`text-sm ${portalTextSecondary}`, className)} {...props} />
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

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
    className={cn(`flex h-11 w-full rounded-xl border ${portalBorder} bg-[var(--portal-surface-elevated)] px-3 py-2 text-sm ${portalText} outline-none transition placeholder:text-[var(--portal-text-muted)] focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20`, className)}
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
    className={cn(`flex h-11 w-full rounded-xl border ${portalBorder} bg-[var(--portal-surface-elevated)] px-3 py-2 text-sm ${portalText} outline-none transition focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20`, className)}
    {...props}
  />
));
Select.displayName = 'Select';

export const Checkbox = forwardRef<
  HTMLInputElement,
  ComponentPropsWithoutRef<'input'>
>(({ className, ...props }, ref) => (
  <input
    ref={ref}
    className={cn(`h-4 w-4 rounded border ${portalBorder} bg-[var(--portal-surface-background)] text-primary-500`, className)}
    type="checkbox"
    {...props}
  />
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
}: {
  label: string;
  children: ReactNode;
  hint?: string;
}) {
  return (
    <label className="grid gap-2">
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
      <span className="relative block w-full">
        <SearchIcon className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-400 dark:text-zinc-500" />
        <Input
          className={cn('pl-11', inputClassName)}
          {...props}
        />
      </span>
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
}: {
  children: ReactNode;
  onClick?: () => void;
  tone?: 'primary' | 'secondary' | 'ghost';
  type?: 'button' | 'submit';
  disabled?: boolean;
}) {
  const variant = tone === 'primary' ? 'default' : tone === 'ghost' ? 'ghost' : 'secondary';
  return (
    <Button disabled={disabled} onClick={onClick} type={type ?? 'button'} variant={variant}>
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
  empty: string;
  getKey: (row: T, index: number) => string;
}) {
  return (
    <div className={`overflow-hidden rounded-2xl border ${portalBorder} ${portalSurfaceElevated}`}>
      <div className="overflow-x-auto">
        <table className={`min-w-full divide-y ${portalBorder} text-sm`}>
          <thead className="bg-[var(--portal-hover-surface)]">
            <tr>
              {columns.map((column) => (
                <th
                  className={`whitespace-nowrap px-4 py-3 text-left text-xs font-semibold uppercase tracking-[0.18em] ${portalTextMuted}`}
                  key={column.key}
                >
                  {column.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className={`divide-y ${portalBorder}`}>
            {rows.length ? rows.map((row, index) => (
              <tr className="transition hover:bg-[var(--portal-hover-surface)]" key={getKey(row, index)}>
                {columns.map((column) => (
                  <td className={`px-4 py-3 align-top ${portalTextSecondary}`} key={column.key}>
                    {column.render(row)}
                  </td>
                ))}
              </tr>
            )) : (
              <tr>
                <td className={`px-4 py-8 text-center text-sm ${portalTextMuted}`} colSpan={columns.length}>
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

export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat(activePortalLocale, {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount);
}

export function formatUnits(units: number): string {
  return new Intl.NumberFormat(activePortalLocale).format(units);
}

export function formatDateTime(timestamp: number): string {
  if (!timestamp) {
    return translatePortalText('Pending');
  }

  return new Intl.DateTimeFormat(activePortalLocale, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp));
}

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
