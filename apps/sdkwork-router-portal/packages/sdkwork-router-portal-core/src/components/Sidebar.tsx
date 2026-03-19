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
import { usePortalAuthStore } from '../store/usePortalAuthStore';
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
  onLogout?: () => void;
  onOpenConfigCenter: () => void;
  workspace?: PortalWorkspaceSummary | null;
}) {
  const storedWorkspace = usePortalAuthStore((state) => state.workspace);
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

  const resolvedWorkspace = workspace ?? storedWorkspace;
  const resolvedSidebarWidth = clampSidebarWidth(sidebarWidth);
  const userInitials = getUserInitials(resolvedWorkspace);

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
      <aside className="flex h-full w-full flex-col overflow-visible border-r border-zinc-900/90 bg-[linear-gradient(180deg,_#13151a_0%,_#0b0c10_100%)] text-zinc-300 shadow-[18px_0_50px_rgba(9,9,11,0.16)]">
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
                  <div className="mb-3 px-3 text-[10px] font-semibold uppercase tracking-[0.22em] text-zinc-500">
                    {group.title}
                  </div>
                ) : (
                  <div className="mx-2 my-4 h-px bg-white/6" />
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
                              ? 'bg-white/[0.08] font-medium text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'
                              : 'text-zinc-400 hover:bg-white/[0.05] hover:text-zinc-200'
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
                                  isActive ? 'text-primary-400' : 'text-zinc-500 group-hover:text-zinc-300'
                                }`}
                              />
                              {!isSidebarCollapsed ? (
                                <span className="text-[14px] tracking-tight">{route.label}</span>
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

        <div className="flex flex-col gap-1 border-t border-white/5 p-3">
          <SidebarProfileDock
            isSidebarCollapsed={isSidebarCollapsed}
            onLogout={onLogout}
            onOpenConfigCenter={onOpenConfigCenter}
            userInitials={userInitials}
            workspace={resolvedWorkspace}
          />
        </div>
      </aside>

      <button
        className={`absolute right-1 top-5 z-30 flex h-8 w-8 items-center justify-center rounded-full bg-zinc-950 text-zinc-200 shadow-[0_10px_24px_rgba(9,9,11,0.26)] transition-all duration-200 dark:bg-zinc-900 ${
          showEdgeControls
            ? 'opacity-100 hover:scale-105 hover:bg-zinc-900'
            : 'pointer-events-none opacity-0'
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
