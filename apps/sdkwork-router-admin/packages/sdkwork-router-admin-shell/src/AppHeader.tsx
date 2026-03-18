import type { ReactNode } from 'react';
import { Minus, Palette, RefreshCw, ShieldCheck, Square, X } from 'lucide-react';
import { useLocation, useNavigate } from 'react-router-dom';

import {
  adminRouteKeyFromPathname,
  adminRoutes,
  ADMIN_ROUTE_PATHS,
  useAdminAppStore,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';

import {
  closeWindow,
  isTauriDesktop,
  minimizeWindow,
  toggleMaximizeWindow,
} from './desktopWindow';

function BrandMark() {
  return (
    <div className="adminx-shell-brand-mark" aria-hidden="true">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M5 12h14" />
        <path d="M9 7l-4 5 4 5" />
        <path d="M15 7l4 5-4 5" />
        <path d="M12 5v14" />
      </svg>
    </div>
  );
}

function HeaderActionButton({
  title,
  onClick,
  className = '',
  children,
}: {
  title: string;
  onClick: () => void | Promise<void>;
  className?: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      data-tauri-drag-region="false"
      title={title}
      className={`adminx-shell-header-action ${className}`.trim()}
      onClick={() => void onClick()}
    >
      {children}
    </button>
  );
}

function DesktopWindowControls() {
  return (
    <div
      className="adminx-window-controls"
      data-tauri-drag-region="false"
    >
      <button
        type="button"
        title="Minimize window"
        data-tauri-drag-region="false"
        onClick={() => {
          void minimizeWindow();
        }}
      >
        <Minus />
      </button>
      <button
        type="button"
        title="Maximize window"
        data-tauri-drag-region="false"
        onClick={() => {
          void toggleMaximizeWindow();
        }}
      >
        <Square />
      </button>
      <button
        type="button"
        title="Close window"
        className="is-danger"
        data-tauri-drag-region="false"
        onClick={() => {
          void closeWindow();
        }}
      >
        <X />
      </button>
    </div>
  );
}

export function AppHeader() {
  const navigate = useNavigate();
  const location = useLocation();
  const { themeColor, themeMode } = useAdminAppStore();
  const { loading, refreshWorkspace, status } = useAdminWorkbench();
  const routeKey = adminRouteKeyFromPathname(location.pathname);
  const activeRoute = adminRoutes.find((route) => route.key === routeKey);
  const isDesktop = isTauriDesktop();

  return (
    <div className={`adminx-shell-header-wrap ${isDesktop ? 'is-desktop' : ''}`.trim()}>
      <header className="adminx-shell-header">
        <div
          className="adminx-shell-header-main"
          data-slot="app-header-leading"
          data-tauri-drag-region={isDesktop ? 'true' : undefined}
        >
          <div className="adminx-shell-brand">
            <BrandMark />
            <div className="adminx-shell-brand-copy">
              <span>SDKWork Router Admin</span>
              <strong>{activeRoute?.label ?? 'Super Admin Workspace'}</strong>
            </div>
          </div>

          <div
            className="adminx-shell-header-search"
            data-tauri-drag-region="false"
          >
            <HeaderActionButton
              title="Open shell settings"
              onClick={() => navigate(ADMIN_ROUTE_PATHS.SETTINGS)}
            >
              <Palette className="adminx-shell-meta-icon" />
              <span className="adminx-shell-header-search-label">Appearance</span>
              <span className="adminx-shell-header-search-shortcut">{themeColor}</span>
            </HeaderActionButton>
          </div>
        </div>

        <div
          className="adminx-shell-header-center"
          data-slot="app-header-center"
          data-tauri-drag-region={isDesktop ? 'true' : undefined}
        >
          <span className="adminx-shell-header-workspace">Workspace</span>
          <div className="adminx-shell-header-workspace-pill">
            <strong>{activeRoute?.eyebrow ?? 'Control Plane'}</strong>
            <span>
              {activeRoute?.detail
                ?? `Right-side operator canvas aligned to claw-studio in ${themeMode} mode.`}
            </span>
          </div>
        </div>

        <div
          className="adminx-shell-header-actions"
          data-slot="app-header-trailing"
          data-tauri-drag-region="false"
        >
          <div className="adminx-shell-meta-pill adminx-shell-meta-pill-status" title={status}>
            <ShieldCheck className="adminx-shell-meta-icon" />
            <span>{status}</span>
          </div>

          <HeaderActionButton
            title="Refresh workspace"
            onClick={() => refreshWorkspace()}
            className="adminx-shell-header-action-icon"
          >
            <RefreshCw className={loading ? 'is-spinning' : undefined} />
          </HeaderActionButton>
          {isDesktop ? <DesktopWindowControls /> : null}
        </div>
      </header>
    </div>
  );
}
