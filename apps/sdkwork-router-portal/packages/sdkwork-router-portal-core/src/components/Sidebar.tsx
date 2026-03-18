import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react';
import {
  Activity,
  Coins,
  Gauge,
  KeyRound,
  PanelLeftClose,
  PanelLeftOpen,
  ReceiptText,
  Route,
  UserRound,
  WalletCards,
  type LucideIcon,
} from 'lucide-react';
import { NavLink } from 'react-router-dom';
import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { resolvePortalPath } from '../application/router/routeManifest';
import {
  clampSidebarWidth,
  PORTAL_COLLAPSED_SIDEBAR_WIDTH,
  PORTAL_MIN_SIDEBAR_WIDTH,
} from '../lib/portalPreferences';
import { portalRoutes } from '../routes';
import { usePortalShellStore } from '../store/usePortalShellStore';
import { SidebarProfileDock } from './SidebarProfileDock';

const routeIcons: Record<PortalRouteKey, LucideIcon> = {
  dashboard: Gauge,
  routing: Route,
  'api-keys': KeyRound,
  usage: Activity,
  user: UserRound,
  credits: Coins,
  billing: WalletCards,
  account: ReceiptText,
};

const routeGroups: Array<{ title: string; items: PortalRouteKey[] }> = [
  { title: 'Workspace', items: ['dashboard', 'routing', 'usage'] },
  { title: 'Access', items: ['api-keys', 'user'] },
  { title: 'Commerce', items: ['credits', 'billing', 'account'] },
];

function getUserInitials(workspace: PortalWorkspaceSummary | null): string {
  const rawValue =
    workspace?.user.display_name?.trim() ||
    workspace?.user.email?.split('@')[0] ||
    workspace?.tenant.name ||
    'PW';

  const segments = rawValue
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2);

  return segments.map((segment) => segment[0]?.toUpperCase() ?? '').join('') || 'PW';
}

export function Sidebar({
  onLogout,
  onOpenConfigCenter,
  workspace,
}: {
  onLogout: () => void;
  onOpenConfigCenter: () => void;
  workspace: PortalWorkspaceSummary | null;
}) {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    sidebarWidth,
    toggleSidebar,
    setSidebarCollapsed,
    setSidebarWidth,
  } = usePortalShellStore();
  const [isHovered, setIsHovered] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const resizeStartXRef = useRef(0);
  const resizeStartWidthRef = useRef(0);

  const resolvedSidebarWidth = clampSidebarWidth(sidebarWidth);
  const userInitials = getUserInitials(workspace);

  useEffect(() => {
    if (resolvedSidebarWidth !== sidebarWidth) {
      setSidebarWidth(resolvedSidebarWidth);
    }
  }, [resolvedSidebarWidth, setSidebarWidth, sidebarWidth]);

  useEffect(() => {
    if (!isResizing) {
      return;
    }

    const previousCursor = document.body.style.cursor;
    const previousUserSelect = document.body.style.userSelect;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';

    const handlePointerMove = (event: PointerEvent) => {
      const nextWidth = clampSidebarWidth(
        resizeStartWidthRef.current + (event.clientX - resizeStartXRef.current),
      );
      setSidebarWidth(nextWidth);
    };

    const handlePointerUp = () => {
      setIsResizing(false);
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);

    return () => {
      document.body.style.cursor = previousCursor;
      document.body.style.userSelect = previousUserSelect;
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [isResizing, setSidebarWidth]);

  function startResize(event: ReactPointerEvent<HTMLDivElement>) {
    event.preventDefault();
    event.stopPropagation();

    const nextWidth = isSidebarCollapsed ? PORTAL_MIN_SIDEBAR_WIDTH : resolvedSidebarWidth;
    resizeStartXRef.current = event.clientX;
    resizeStartWidthRef.current = nextWidth;

    if (isSidebarCollapsed) {
      setSidebarCollapsed(false);
      setSidebarWidth(nextWidth);
    }

    setIsResizing(true);
  }

  const currentWidth = isSidebarCollapsed ? PORTAL_COLLAPSED_SIDEBAR_WIDTH : resolvedSidebarWidth;
  const showEdgeControls = isHovered || isResizing;

  return (
    <div
      className={`relative z-20 flex h-full shrink-0 ${isResizing ? '' : 'transition-[width] duration-200 ease-out'}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      style={{ width: currentWidth }}
    >
      <aside className="flex h-full w-full flex-col overflow-visible border-r border-[color:var(--portal-sidebar-border)] [background:var(--portal-sidebar-background)] text-[var(--portal-sidebar-text)] shadow-[18px_0_50px_rgba(9,9,11,0.18)]">
        <div className={`grid gap-3 border-b border-[color:var(--portal-sidebar-border)] ${isSidebarCollapsed ? 'px-3 py-4' : 'px-4 py-5'}`}>
          <div className="grid gap-1">
            {!isSidebarCollapsed ? (
              <>
                <span className="text-[10px] font-semibold uppercase tracking-[0.24em] text-[var(--portal-sidebar-muted)]">
                  Active workspace
                </span>
                <strong className="truncate text-base font-semibold tracking-tight text-[var(--portal-sidebar-text)]">
                  {workspace?.project.name ?? 'Loading workspace'}
                </strong>
                <span className="truncate text-xs text-[var(--portal-sidebar-muted)]">
                  {workspace?.user.email ?? 'Restoring session'}
                </span>
              </>
            ) : (
              <div className="mx-auto flex h-10 w-10 items-center justify-center rounded-2xl border border-[color:var(--portal-sidebar-border)] bg-[var(--portal-sidebar-surface)] text-sm font-semibold text-[var(--portal-sidebar-text)]">
                {userInitials}
              </div>
            )}
          </div>
        </div>

        <nav className={`scrollbar-hide flex-1 space-y-5 overflow-x-hidden overflow-y-auto ${isSidebarCollapsed ? 'px-2 py-4' : 'px-3 py-5'}`}>
          {routeGroups.map((group) => {
            const visibleRoutes = group.items
              .map((routeKey) => portalRoutes.find((route) => route.key === routeKey))
              .filter((route) => route && !hiddenSidebarItems.includes(route.key));

            if (!visibleRoutes.length) {
              return null;
            }

            return (
              <div key={group.title}>
                {!isSidebarCollapsed ? (
                  <div className="mb-3 px-3 text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--portal-sidebar-muted)]">
                    {group.title}
                  </div>
                ) : (
                  <div className="mx-2 my-4 h-px bg-[var(--portal-sidebar-border)]" />
                )}

                <div className="space-y-1">
                  {visibleRoutes.map((route) => {
                    if (!route) {
                      return null;
                    }

                    const Icon = routeIcons[route.key];

                    return (
                      <NavLink
                        className={({ isActive }) =>
                          `group relative flex items-center rounded-2xl transition-all duration-200 ${
                            isSidebarCollapsed ? 'mx-auto h-11 w-11 justify-center' : 'justify-between px-3 py-2.5'
                          } ${
                            isActive
                              ? 'bg-[var(--portal-sidebar-active)] font-medium text-[var(--portal-sidebar-text)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'
                              : 'text-[var(--portal-sidebar-muted)] hover:bg-[var(--portal-sidebar-hover)] hover:text-[var(--portal-sidebar-text)]'
                          }`
                        }
                        key={route.key}
                        title={isSidebarCollapsed ? route.label : undefined}
                        to={resolvePortalPath(route.key)}
                      >
                        {({ isActive }) => (
                          <>
                            {isActive && !isSidebarCollapsed ? (
                              <div className="absolute left-0 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-primary-500" />
                            ) : null}
                            <div className="flex items-center gap-3">
                              <Icon
                                className={`h-4 w-4 shrink-0 transition-colors ${
                                  isActive ? 'text-primary-300' : 'text-[var(--portal-sidebar-muted)] group-hover:text-[var(--portal-sidebar-text)]'
                                }`}
                              />
                              {!isSidebarCollapsed ? (
                                <div className="grid gap-0.5">
                                  <span className="text-[14px] tracking-tight">{route.label}</span>
                                  <small className="text-[11px] text-[var(--portal-sidebar-muted)]">{route.eyebrow}</small>
                                </div>
                              ) : null}
                            </div>
                          </>
                        )}
                      </NavLink>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </nav>

        <div className="border-t border-[color:var(--portal-sidebar-border)] px-3 pb-4 pt-3 [background:linear-gradient(180deg,rgba(255,255,255,0.01),rgba(255,255,255,0.04))]">
          <SidebarProfileDock
            isSidebarCollapsed={isSidebarCollapsed}
            onLogout={onLogout}
            onOpenConfigCenter={onOpenConfigCenter}
            userInitials={userInitials}
            workspace={workspace}
          />
        </div>
      </aside>

      <button
        className={`absolute right-1 top-5 z-30 flex h-8 w-8 items-center justify-center rounded-full [background:var(--portal-surface-contrast)] text-[var(--portal-text-on-contrast)] shadow-[0_10px_24px_rgba(9,9,11,0.26)] transition-all duration-200 ${
          showEdgeControls ? 'opacity-100 hover:scale-105 hover:brightness-110' : 'pointer-events-none opacity-0'
        }`}
        onClick={toggleSidebar}
        title={isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        type="button"
      >
        {isSidebarCollapsed ? <PanelLeftOpen className="h-4 w-4" /> : <PanelLeftClose className="h-4 w-4" />}
      </button>

      <div
        className="absolute inset-y-0 right-0 z-20 w-3 cursor-col-resize touch-none"
        data-slot="sidebar-resize-handle"
        onPointerDown={startResize}
      />
    </div>
  );
}
