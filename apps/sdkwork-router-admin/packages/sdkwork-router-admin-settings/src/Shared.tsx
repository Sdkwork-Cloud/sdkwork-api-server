import type { ReactNode } from 'react';
import { ChevronRight } from 'lucide-react';

export function SettingsShellCard({
  children,
  className = '',
}: {
  children: ReactNode;
  className?: string;
}) {
  return <article className={`admin-shell-settings-card ${className}`.trim()}>{children}</article>;
}

export function SettingsSection({
  eyebrow,
  title,
  icon,
  children,
  className = '',
}: {
  eyebrow: string;
  title: string;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <SettingsShellCard className={className}>
      <div className="admin-shell-settings-card-head">
        <div>
          <span>{eyebrow}</span>
          <strong>{title}</strong>
        </div>
        {icon}
      </div>
      {children}
    </SettingsShellCard>
  );
}

export function SettingsNavButton({
  active,
  icon,
  label,
  detail,
  tabId,
  onClick,
}: {
  active: boolean;
  icon: ReactNode;
  label: string;
  detail: string;
  tabId: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      data-settings-tab={tabId}
      className={active ? 'is-active' : ''}
      onClick={onClick}
    >
      <span className="admin-shell-settings-tab-icon">{icon}</span>
      <div className="admin-shell-settings-tab-copy">
        <strong>{label}</strong>
        <span>{detail}</span>
      </div>
      <ChevronRight className="admin-shell-settings-tab-chevron" />
    </button>
  );
}
