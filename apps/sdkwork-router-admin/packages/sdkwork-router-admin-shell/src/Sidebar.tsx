import {
  Activity,
  Blocks,
  Building2,
  Gauge,
  LogOut,
  PanelLeftClose,
  PanelLeftOpen,
  ServerCog,
  Settings2,
  TicketPercent,
  Users,
} from 'lucide-react';
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { NavLink } from 'react-router-dom';

import {
  adminRoutePathByKey,
  adminRoutes,
  useAdminAppStore,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';

const COLLAPSED_SIDEBAR_WIDTH = 72;
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 360;

const iconByRoute = {
  overview: Gauge,
  users: Users,
  tenants: Building2,
  coupons: TicketPercent,
  catalog: Blocks,
  traffic: Activity,
  operations: ServerCog,
  settings: Settings2,
} as const;

function clampSidebarWidth(width: number) {
  return Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, width));
}

function buildAvatarInitials(value?: string | null) {
  if (!value) {
    return 'RA';
  }

  const normalized = value.trim();
  if (!normalized) {
    return 'RA';
  }

  const parts = normalized.split(/\s+/).filter(Boolean);
  if (parts.length > 1) {
    return `${parts[0][0] ?? ''}${parts[1][0] ?? ''}`.toUpperCase();
  }

  return normalized.slice(0, 2).toUpperCase();
}

export function Sidebar() {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    setSidebarCollapsed,
    setSidebarWidth,
    sidebarWidth,
    toggleSidebar,
  } = useAdminAppStore();
  const { handleLogout, sessionUser, status } = useAdminWorkbench();
  const [isSidebarHovered, setIsSidebarHovered] = useState(false);
  const [isSidebarResizing, setIsSidebarResizing] = useState(false);
  const resizeStartXRef = useRef(0);
  const resizeStartWidthRef = useRef(0);

  const resolvedSidebarWidth = clampSidebarWidth(sidebarWidth);

  useEffect(() => {
    if (resolvedSidebarWidth !== sidebarWidth) {
      setSidebarWidth(resolvedSidebarWidth);
    }
  }, [resolvedSidebarWidth, setSidebarWidth, sidebarWidth]);

  useEffect(() => {
    if (!isSidebarResizing) {
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
      setIsSidebarResizing(false);
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);

    return () => {
      document.body.style.cursor = previousCursor;
      document.body.style.userSelect = previousUserSelect;
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [isSidebarResizing, setSidebarWidth]);

  const startSidebarResize = useCallback(
    (event: ReactPointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      event.stopPropagation();

      const nextWidth = isSidebarCollapsed ? MIN_SIDEBAR_WIDTH : resolvedSidebarWidth;
      resizeStartXRef.current = event.clientX;
      resizeStartWidthRef.current = nextWidth;

      if (isSidebarCollapsed) {
        setSidebarCollapsed(false);
        setSidebarWidth(nextWidth);
      }

      setIsSidebarResizing(true);
    },
    [isSidebarCollapsed, resolvedSidebarWidth, setSidebarCollapsed, setSidebarWidth],
  );

  const groupedRoutes = useMemo(() => {
    const routeGroups = new Map<string, typeof adminRoutes>();

    for (const route of adminRoutes.filter(
      (item) => item.key !== 'settings' && !hiddenSidebarItems.includes(item.key),
    )) {
      const group = route.group ?? 'Workspace';
      if (!routeGroups.has(group)) {
        routeGroups.set(group, []);
      }

      routeGroups.get(group)?.push(route);
    }

    return [...routeGroups.entries()];
  }, [hiddenSidebarItems]);

  const currentSidebarWidth = isSidebarCollapsed ? COLLAPSED_SIDEBAR_WIDTH : resolvedSidebarWidth;
  const showEdgeAffordances = isSidebarHovered || isSidebarResizing;
  const profileName = sessionUser?.display_name ?? 'Router Admin';
  const profileDetail = sessionUser?.email ?? 'Control plane operator';
  const profileInitials = buildAvatarInitials(sessionUser?.display_name ?? sessionUser?.email);

  return (
    <aside
      className={`adminx-shell-sidebar ${isSidebarResizing ? 'is-resizing' : ''}`}
      style={{ width: currentSidebarWidth }}
      onMouseEnter={() => setIsSidebarHovered(true)}
      onMouseLeave={() => setIsSidebarHovered(false)}
    >
      <div className="adminx-shell-sidebar-inner">
        <nav className="adminx-shell-sidebar-nav">
          {groupedRoutes.map(([group, routes]) => (
            <div key={group} className="adminx-shell-sidebar-group">
              {!isSidebarCollapsed ? (
                <div className="adminx-shell-sidebar-group-label">{group}</div>
              ) : (
                <div className="adminx-shell-sidebar-divider" />
              )}

              <div className="adminx-shell-sidebar-links">
                {routes.map((route) => {
                  const Icon = iconByRoute[route.key];

                  return (
                    <NavLink
                      key={route.key}
                      to={adminRoutePathByKey[route.key]}
                      title={isSidebarCollapsed ? route.label : undefined}
                      className={({ isActive }) =>
                        `adminx-shell-sidebar-link ${isActive ? 'is-active' : ''} ${
                          isSidebarCollapsed ? 'is-collapsed' : ''
                        }`
                      }
                    >
                      {({ isActive }) => (
                        <>
                          {isActive && !isSidebarCollapsed ? (
                            <span className="adminx-shell-sidebar-link-rail" />
                          ) : null}
                          <div className="adminx-shell-sidebar-link-leading">
                            <Icon className="adminx-shell-sidebar-link-icon" />
                            {!isSidebarCollapsed ? (
                              <div className="adminx-shell-sidebar-link-copy">
                                <strong>{route.label}</strong>
                              </div>
                            ) : null}
                          </div>
                          {!isSidebarCollapsed ? (
                            <span className="adminx-shell-sidebar-link-badge">
                              {route.eyebrow}
                            </span>
                          ) : null}
                        </>
                      )}
                    </NavLink>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>

        <div className="adminx-shell-sidebar-footer">
          <div className={`adminx-shell-sidebar-profile ${isSidebarCollapsed ? 'is-collapsed' : ''}`}>
            <div className="adminx-shell-sidebar-avatar" aria-hidden="true">
              {profileInitials}
            </div>
            {!isSidebarCollapsed ? (
              <div className="adminx-shell-sidebar-profile-copy">
                <strong>{profileName}</strong>
                <span>{profileDetail}</span>
              </div>
            ) : null}
          </div>

          <div className={`adminx-shell-sidebar-footer-actions ${isSidebarCollapsed ? 'is-collapsed' : ''}`}>
            {!isSidebarCollapsed ? (
              <div className="adminx-shell-sidebar-profile-status" title={status}>
                {status}
              </div>
            ) : null}
            <div className="adminx-shell-sidebar-footer-buttons">
              <NavLink
                to={adminRoutePathByKey.settings}
                title="Settings"
                className={({ isActive }) =>
                  `adminx-shell-sidebar-footer-settings ${isActive ? 'is-active' : ''}`
                }
              >
                <Settings2 className="adminx-shell-sidebar-link-icon" />
              </NavLink>
              <button
                type="button"
                title="Sign out"
                className="adminx-shell-sidebar-footer-logout"
                onClick={() => {
                  void handleLogout();
                }}
              >
                <LogOut className="adminx-shell-sidebar-link-icon" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <button
        type="button"
        className={`adminx-shell-sidebar-toggle ${showEdgeAffordances ? 'is-visible' : ''}`}
        onClick={toggleSidebar}
        title={isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      >
        {isSidebarCollapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
      </button>

      <div
        data-slot="sidebar-resize-handle"
        onPointerDown={startSidebarResize}
        className="adminx-shell-sidebar-resize-handle"
      />
    </aside>
  );
}
