import * as DialogPrimitive from '@radix-ui/react-dialog';
import * as LabelPrimitive from '@radix-ui/react-label';
import { Slot } from '@radix-ui/react-slot';
import * as TabsPrimitive from '@radix-ui/react-tabs';
import { cva, type VariantProps } from 'class-variance-authority';
import { clsx, type ClassValue } from 'clsx';
import { X } from 'lucide-react';
import {
  forwardRef,
  type ComponentPropsWithoutRef,
  type ElementRef,
  type ReactNode,
} from 'react';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
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
        <span className="sr-only">Close</span>
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
  return `$${amount.toFixed(2)}`;
}

export function formatUnits(units: number): string {
  return new Intl.NumberFormat('en-US').format(units);
}

export function formatDateTime(timestamp: number): string {
  if (!timestamp) {
    return 'Pending';
  }

  return new Intl.DateTimeFormat('en-US', {
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
