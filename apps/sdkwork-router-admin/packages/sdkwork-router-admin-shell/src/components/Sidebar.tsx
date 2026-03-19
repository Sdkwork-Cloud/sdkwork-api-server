import {
  Activity,
  Blocks,
  Building2,
  ChevronUp,
  CircleUserRound,
  Gauge,
  LogOut,
  PanelLeftClose,
  PanelLeftOpen,
  ServerCog,
  Settings2,
  ShieldCheck,
  TicketPercent,
  Users,
} from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { NavLink, useLocation, useNavigate } from 'react-router-dom';

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
  const navigate = useNavigate();
  const location = useLocation();
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
  const [isUserMenuOpen, setIsUserMenuOpen] = useState(false);
  const resizeStartXRef = useRef(0);
  const resizeStartWidthRef = useRef(0);
  const userMenuRef = useRef<HTMLDivElement>(null);

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

  useEffect(() => {
    setIsUserMenuOpen(false);
  }, [isSidebarCollapsed, location.pathname, location.search]);

  useEffect(() => {
    if (!isUserMenuOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (!userMenuRef.current?.contains(event.target as Node)) {
        setIsUserMenuOpen(false);
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  }, [isUserMenuOpen]);

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
  const userControlTitle = isUserMenuOpen ? 'Close account controls' : 'Open account controls';

  const handleOpenSettings = () => {
    setIsUserMenuOpen(false);
    navigate(adminRoutePathByKey.settings);
  };

  const handleUserControlClick = () => {
    setIsUserMenuOpen((open) => !open);
  };

  return (
    <aside
      className={`adminx-shell-sidebar ${isSidebarResizing ? 'is-resizing' : ''}`}
      style={{ width: currentSidebarWidth }}
      onMouseEnter={() => setIsSidebarHovered(true)}
      onMouseLeave={() => setIsSidebarHovered(false)}
    >
      <div className="adminx-shell-sidebar-inner adminx-shell-sidebar-surface">
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
                            <motion.span
                              layoutId="adminx-sidebar-active-indicator"
                              className="adminx-shell-sidebar-link-rail"
                            />
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
          <NavLink
            to={adminRoutePathByKey.settings}
            title={isSidebarCollapsed ? 'Open settings center' : undefined}
            className={({ isActive }) =>
              `adminx-shell-sidebar-secondary-action adminx-shell-sidebar-footer-settings ${
                isActive ? 'is-active' : ''
              } ${isSidebarCollapsed ? 'is-collapsed' : ''}`.trim()
            }
          >
            {({ isActive }) => (
              <>
                {isActive && !isSidebarCollapsed ? (
                  <motion.span
                    layoutId="adminx-sidebar-active-indicator"
                    className="adminx-shell-sidebar-link-rail"
                  />
                ) : null}
                <div className="adminx-shell-sidebar-link-leading">
                  <Settings2 className="adminx-shell-sidebar-link-icon" />
                  {!isSidebarCollapsed ? (
                    <div className="adminx-shell-sidebar-secondary-copy">
                      <strong>Settings center</strong>
                      <span>Theme, rail, and canvas continuity</span>
                    </div>
                  ) : null}
                </div>
                {!isSidebarCollapsed ? (
                  <span className="adminx-shell-sidebar-link-badge">Shell</span>
                ) : null}
              </>
            )}
          </NavLink>

          <div ref={userMenuRef} className="adminx-shell-sidebar-user-surface">
            <AnimatePresence>
              {isUserMenuOpen ? (
                <motion.div
                  initial={{ opacity: 0, y: 10, scale: 0.98 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: 8, scale: 0.98 }}
                  transition={{ duration: 0.18, ease: 'easeOut' }}
                  className={`adminx-shell-sidebar-user-menu ${
                    isSidebarCollapsed ? 'is-collapsed' : ''
                  }`}
                >
                  <div className="adminx-shell-sidebar-user-menu-card adminx-shell-sidebar-profile">
                    <div className="adminx-shell-sidebar-avatar" aria-hidden="true">
                      {profileInitials}
                    </div>
                    <div className="adminx-shell-sidebar-profile-copy">
                      <strong>{profileName}</strong>
                      <span>{profileDetail}</span>
                    </div>
                  </div>

                  <div className="adminx-shell-sidebar-user-menu-status" title={status}>
                    <ShieldCheck className="adminx-shell-sidebar-link-icon" />
                    <span>{status}</span>
                  </div>

                  <button
                    type="button"
                    className="adminx-shell-sidebar-menu-action"
                    onClick={handleOpenSettings}
                  >
                    <Settings2 className="adminx-shell-sidebar-link-icon" />
                    <span>Open settings center</span>
                  </button>

                  <button
                    type="button"
                    className="adminx-shell-sidebar-menu-action adminx-shell-sidebar-footer-logout is-danger"
                    onClick={() => {
                      setIsUserMenuOpen(false);
                      void handleLogout();
                    }}
                  >
                    <LogOut className="adminx-shell-sidebar-link-icon" />
                    <span>Sign out</span>
                  </button>
                </motion.div>
              ) : null}
            </AnimatePresence>

            <button
              type="button"
              data-slot="sidebar-user-control"
              title={isSidebarCollapsed ? userControlTitle : undefined}
              className={`adminx-shell-sidebar-profile adminx-shell-sidebar-user-control ${
                isSidebarCollapsed ? 'is-collapsed' : ''
              } ${isUserMenuOpen ? 'is-open' : ''}`.trim()}
              onClick={handleUserControlClick}
            >
              <div className="adminx-shell-sidebar-avatar" aria-hidden="true">
                {sessionUser ? profileInitials : <CircleUserRound className="adminx-shell-sidebar-link-icon" />}
              </div>
              {!isSidebarCollapsed ? (
                <>
                  <div className="adminx-shell-sidebar-profile-copy">
                    <strong>{profileName}</strong>
                    <span>{profileDetail}</span>
                  </div>
                  <div className="adminx-shell-sidebar-user-summary">
                    <span title={status}>{status}</span>
                    <ChevronUp
                      className={`adminx-shell-sidebar-user-chevron ${
                        isUserMenuOpen ? '' : 'is-collapsed'
                      }`.trim()}
                    />
                  </div>
                </>
              ) : null}
            </button>
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
