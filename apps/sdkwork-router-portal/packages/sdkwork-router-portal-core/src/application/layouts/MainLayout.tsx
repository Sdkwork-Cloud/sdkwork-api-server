import { useState, type ReactNode } from 'react';
import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { AppHeader } from '../../components/AppHeader';
import { ConfigCenter } from '../../components/ConfigCenter';
import { ShellStatus } from '../../components/ShellStatus';
import { Sidebar } from '../../components/Sidebar';

export function MainLayout({
  activeRoute,
  children,
  onLogout,
  pulseDetail,
  pulseStatus,
  pulseTitle,
  pulseTone,
  workspace,
}: {
  activeRoute: PortalRouteKey;
  children: ReactNode;
  onLogout: () => void;
  pulseDetail: string;
  pulseStatus: string;
  pulseTitle: string;
  pulseTone: 'accent' | 'positive' | 'warning';
  workspace: PortalWorkspaceSummary | null;
}) {
  const [configCenterOpen, setConfigCenterOpen] = useState(false);

  return (
    <div className="relative flex h-screen flex-col overflow-hidden [background:var(--portal-shell-background)] font-sans text-[var(--portal-text-primary)] transition-colors duration-300">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-x-0 top-0 h-40 bg-[radial-gradient(circle_at_top,rgb(var(--portal-accent-rgb)_/_0.16),transparent_68%)]" />
        <div className="absolute inset-y-0 left-0 w-80 bg-[radial-gradient(circle_at_left,var(--portal-shell-sidebar-glow),transparent_72%)]" />
      </div>

      <AppHeader />

      <div className="relative z-10 flex min-h-0 flex-1 overflow-hidden">
        <Sidebar
          onLogout={onLogout}
          onOpenConfigCenter={() => setConfigCenterOpen(true)}
          workspace={workspace}
        />

        <main className="scrollbar-hide relative z-10 min-w-0 flex-1 overflow-auto bg-[var(--portal-content-background)]">
          <div className="mx-auto flex min-h-full w-full max-w-[1600px] flex-col gap-6 px-4 py-5 md:px-6">
            <ShellStatus
              activeRoute={activeRoute}
              pulseDetail={pulseDetail}
              pulseStatus={pulseStatus}
              pulseTitle={pulseTitle}
              pulseTone={pulseTone}
              workspace={workspace}
            />
            {children}
          </div>
        </main>
      </div>

      <ConfigCenter onOpenChange={setConfigCenterOpen} open={configCenterOpen} />
    </div>
  );
}
