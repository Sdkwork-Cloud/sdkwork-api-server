import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

import { cn } from 'sdkwork-router-admin-commons';

export function SettingsShellCard({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <article
      className={cn(
        'overflow-hidden rounded-[1.5rem] border border-zinc-200/80 bg-white shadow-sm dark:border-zinc-800/80 dark:bg-zinc-900',
        className,
      )}
    >
      {children}
    </article>
  );
}

export function SettingsSection({
  eyebrow,
  title,
  description,
  actions,
  children,
  className,
}: {
  eyebrow?: string;
  title: string;
  description?: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <SettingsShellCard className={className}>
      <div className="border-b border-zinc-100 bg-zinc-50/50 px-6 py-5 dark:border-zinc-800/80 dark:bg-zinc-900/50">
        <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div className="space-y-1">
            {eyebrow ? (
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {eyebrow}
              </p>
            ) : null}
            <h3 className="text-[15px] font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
              {title}
            </h3>
            {description ? (
              <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>
            ) : null}
          </div>
          {actions ? <div className="flex flex-wrap gap-3">{actions}</div> : null}
        </div>
      </div>
      <div className="p-6">{children}</div>
    </SettingsShellCard>
  );
}

export function SettingsInfoCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: ReactNode;
  detail?: string;
}) {
  return (
    <div className="rounded-xl border border-zinc-200 bg-zinc-50/90 p-4 dark:border-zinc-800 dark:bg-zinc-900/80">
      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
        {label}
      </div>
      <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">{value}</div>
      {detail ? (
        <div className="mt-1 text-xs leading-6 text-zinc-500 dark:text-zinc-400">{detail}</div>
      ) : null}
    </div>
  );
}

export function SettingsNavButton({
  active,
  icon,
  label,
  tabId,
  onClick,
}: {
  active: boolean;
  icon: LucideIcon;
  label: string;
  tabId: string;
  onClick: () => void;
}) {
  const Icon = icon;

  return (
    <button
      type="button"
      data-settings-tab={tabId}
      className={`flex w-full items-center gap-3 rounded-xl border px-3 py-2.5 text-[14px] font-medium transition-all duration-200 ${
        active
          ? 'border-zinc-200/50 bg-white text-primary-600 shadow-sm dark:border-zinc-700/50 dark:bg-zinc-800 dark:text-primary-400'
          : 'border-transparent text-zinc-600 hover:bg-zinc-200/50 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800/50 dark:hover:text-zinc-100'
      }`}
      onClick={onClick}
    >
      <Icon
        className={
          active
            ? 'h-4 w-4 text-primary-500 dark:text-primary-400'
            : 'h-4 w-4 text-zinc-400 dark:text-zinc-500'
        }
      />
      {label}
    </button>
  );
}
