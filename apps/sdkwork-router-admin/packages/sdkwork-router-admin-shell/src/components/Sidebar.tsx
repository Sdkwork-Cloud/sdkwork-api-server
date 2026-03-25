import {
  Activity,
  Blocks,
  Building2,
  ChevronUp,
  CircleUserRound,
  Gauge,
  LogOut,
  PanelLeftClose,
  ServerCog,
  Settings2,
  ShieldCheck,
  TicketPercent,
  Users,
} from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';
import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
import { NavLink, useLocation } from 'react-router-dom';

import {
  adminRoutePathByKey,
  adminRoutes,
  useAdminAppStore,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';
import { useAdminI18n } from 'sdkwork-router-admin-commons';

const COLLAPSED_SIDEBAR_WIDTH = 60;

const iconByRoute = {
  overview: Gauge,
  users: Users,
  tenants: Building2,
  coupons: TicketPercent,
  'api-keys': ShieldCheck,
  'route-config': Blocks,
  'model-mapping': Building2,
  'usage-records': Activity,
  catalog: Blocks,
  traffic: Activity,
  operations: ServerCog,
  settings: Settings2,
} as const;

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

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

export function Sidebar() {
  const { t } = useAdminI18n();
  const location = useLocation();
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, toggleSidebar } = useAdminAppStore();
  const { handleLogout, sessionUser } = useAdminWorkbench();
  const [isUserMenuOpen, setIsUserMenuOpen] = useState(false);
  const [userMenuStyle, setUserMenuStyle] = useState<CSSProperties>();
  const userSurfaceRef = useRef<HTMLDivElement>(null);
  const userControlRef = useRef<HTMLButtonElement>(null);
  const userMenuPanelRef = useRef<HTMLDivElement>(null);
  const currentSidebarWidth = isSidebarCollapsed ? COLLAPSED_SIDEBAR_WIDTH : sidebarWidth;

  useEffect(() => {
    setIsUserMenuOpen(false);
  }, [isSidebarCollapsed, location.pathname, location.search]);

  useEffect(() => {
    if (!isUserMenuOpen) {
      setUserMenuStyle(undefined);
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (!userSurfaceRef.current?.contains(event.target as Node)) {
        setIsUserMenuOpen(false);
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  }, [isUserMenuOpen]);

  useEffect(() => {
    if (!isUserMenuOpen) {
      return;
    }

    const viewportPadding = 16;
    const menuGap = 10;

    const updateUserMenuPosition = () => {
      const userControl = userControlRef.current;
      if (!userControl) {
        return;
      }

      const triggerRect = userControl.getBoundingClientRect();
      const nextMenuWidth = Math.min(280, window.innerWidth - viewportPadding * 2);
      const measuredMenuHeight = userMenuPanelRef.current?.offsetHeight ?? 176;
      const maxTop = Math.max(viewportPadding, window.innerHeight - measuredMenuHeight - viewportPadding);
      const preferredTop = isSidebarCollapsed
        ? triggerRect.bottom - measuredMenuHeight
        : triggerRect.top - measuredMenuHeight - menuGap;
      const preferredLeft = isSidebarCollapsed
        ? triggerRect.right + menuGap
        : triggerRect.right - nextMenuWidth;

      setUserMenuStyle({
        left: clamp(
          preferredLeft,
          viewportPadding,
          Math.max(viewportPadding, window.innerWidth - nextMenuWidth - viewportPadding),
        ),
        maxHeight: window.innerHeight - viewportPadding * 2,
        top: clamp(preferredTop, viewportPadding, maxTop),
        width: nextMenuWidth,
      });
    };

    const frameId = window.requestAnimationFrame(updateUserMenuPosition);

    window.addEventListener('resize', updateUserMenuPosition);
    window.addEventListener('scroll', updateUserMenuPosition, true);

    return () => {
      window.cancelAnimationFrame(frameId);
      window.removeEventListener('resize', updateUserMenuPosition);
      window.removeEventListener('scroll', updateUserMenuPosition, true);
    };
  }, [currentSidebarWidth, isSidebarCollapsed, isUserMenuOpen]);

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

  const profileName = sessionUser?.display_name ?? t('Router Admin');
  const profileDetail = sessionUser?.email ?? t('Control plane operator');
  const profileInitials = buildAvatarInitials(sessionUser?.display_name ?? sessionUser?.email);
  const userControlTitle = isUserMenuOpen ? t('Close account controls') : t('Open account controls');

  const renderActiveRail = !isSidebarCollapsed ? (
    <motion.div
      layoutId="adminx-sidebar-active-indicator"
      className="adminx-shell-sidebar-link-rail absolute left-0 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-primary-500"
    />
  ) : null;

  const footerActionClassName = ({
    active = false,
    open = false,
  }: {
    active?: boolean;
    open?: boolean;
  }) =>
    `adminx-shell-sidebar-footer-action group relative flex items-center rounded-xl border transition-all duration-200 ${
      isSidebarCollapsed ? 'mx-auto h-10 w-10 justify-center px-0' : 'w-full gap-3 px-3 py-2.5'
    } ${
      active || open
        ? 'is-active border-primary-500/20 bg-primary-500/10 font-medium text-primary-400'
        : 'border-white/8 bg-white/[0.04] text-zinc-300 hover:bg-white/[0.07] hover:text-white'
    } ${
      open ? 'is-open' : ''
    } ${
      isSidebarCollapsed ? 'is-collapsed' : ''
    }`;

  return (
    <motion.div
      initial={false}
      animate={{ width: currentSidebarWidth }}
      transition={{ type: 'spring', stiffness: 300, damping: 30, mass: 0.8 }}
      className="adminx-shell-sidebar relative z-20 flex min-h-0 shrink-0 self-stretch"
    >
      <aside className="adminx-shell-sidebar-inner adminx-shell-sidebar-surface flex min-h-0 w-full flex-col overflow-hidden border-r border-zinc-900 bg-zinc-950 bg-[linear-gradient(180deg,_#13151a_0%,_#0b0c10_100%)] text-zinc-300 shadow-[18px_0_50px_rgba(9,9,11,0.16)]">
        <div className={`relative flex flex-col pb-4 pt-6 ${isSidebarCollapsed ? 'items-center' : 'px-5'}`}>
          <div
            className={`flex w-full items-center overflow-hidden whitespace-nowrap ${isSidebarCollapsed ? 'justify-center' : 'gap-3'}`}
          >
            <button
              type="button"
              onClick={isSidebarCollapsed ? toggleSidebar : undefined}
              title={isSidebarCollapsed ? t('Expand sidebar') : undefined}
              aria-label={isSidebarCollapsed ? t('Expand sidebar') : undefined}
              tabIndex={isSidebarCollapsed ? 0 : -1}
              className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary-600 shadow-lg shadow-primary-900/20 ${
                isSidebarCollapsed ? 'cursor-pointer transition-colors hover:bg-primary-500' : ''
              }`}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="h-5 w-5 text-white"
              >
                <path d="M12 2v2" />
                <path d="M12 18v4" />
                <path d="M4.93 10.93l1.41 1.41" />
                <path d="M17.66 17.66l1.41 1.41" />
                <path d="M2 12h2" />
                <path d="M20 12h2" />
                <path d="M4.93 13.07l1.41-1.41" />
                <path d="M17.66 6.34l1.41-1.41" />
                <path d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                <path d="M12 6a6 6 0 0 1 6 6" />
                <path d="M12 18a6 6 0 0 1-6-6" />
              </svg>
            </button>

            {!isSidebarCollapsed ? (
              <>
                <div className="min-w-0">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500">
                    {t('Control plane')}
                  </div>
                  <div className="truncate text-xl font-bold tracking-tight text-white">
                    {t('SDKWork Router Admin')}
                  </div>
                </div>

                <button
                  type="button"
                  onClick={toggleSidebar}
                  className="absolute right-4 top-6 rounded-md p-1 text-zinc-500 transition-opacity hover:bg-white/5 hover:text-white"
                  title={t('Collapse sidebar')}
                >
                  <PanelLeftClose className="h-4 w-4" />
                </button>
              </>
            ) : null}
          </div>
        </div>

        <nav
          className={`adminx-shell-sidebar-nav scrollbar-hide mt-4 min-h-0 flex-1 space-y-6 overflow-x-hidden overflow-y-auto ${
            isSidebarCollapsed ? 'px-2 pb-4' : 'px-4 pb-5'
          }`}
        >
          {groupedRoutes.map(([group, routes]) => (
            <div key={group} className="adminx-shell-sidebar-group">
              {!isSidebarCollapsed ? (
                <div className="adminx-shell-sidebar-group-label mb-3 px-3 text-[11px] font-bold uppercase tracking-widest text-zinc-500">
                  {t(group)}
                </div>
              ) : (
                <div className="adminx-shell-sidebar-divider mx-2 my-4 h-px bg-zinc-800/50" />
              )}

              <div className="adminx-shell-sidebar-links space-y-1">
                {routes.map((route) => {
                  const Icon = iconByRoute[route.key];

                  return (
                    <NavLink
                      key={route.key}
                      to={adminRoutePathByKey[route.key]}
                      title={isSidebarCollapsed ? t(route.label) : undefined}
                      className={({ isActive }) =>
                        `adminx-shell-sidebar-link group relative flex items-center rounded-xl transition-all duration-200 ${
                          isSidebarCollapsed ? 'mx-auto h-10 w-10 justify-center' : 'justify-between px-3 py-2.5'
                        } ${
                          isActive
                            ? 'is-active bg-primary-500/10 font-medium text-primary-400'
                            : 'text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200'
                        } ${isSidebarCollapsed ? 'is-collapsed' : ''}`
                      }
                    >
                      {({ isActive }) => (
                        <>
                          {isActive ? renderActiveRail : null}
                          <div className="flex items-center gap-3">
                            <Icon
                              className={`adminx-shell-sidebar-link-icon h-4 w-4 shrink-0 transition-colors ${
                                isActive ? 'text-primary-400' : 'text-zinc-500 group-hover:text-zinc-300'
                              }`}
                            />
                            {!isSidebarCollapsed ? (
                              <span className="text-[14px] tracking-tight">{t(route.label)}</span>
                            ) : null}
                          </div>
                        </>
                      )}
                    </NavLink>
                  );
                })}
              </div>
            </div>
            ))}
        </nav>

        <div className="adminx-shell-sidebar-footer mt-auto flex flex-col gap-1 border-t border-zinc-900 p-4">
          <NavLink
            data-slot="sidebar-footer-settings"
            to={adminRoutePathByKey.settings}
            title={isSidebarCollapsed ? t('Settings') : undefined}
            className={({ isActive }) =>
              `${footerActionClassName({ active: isActive })} adminx-shell-sidebar-footer-settings`
            }
          >
            {({ isActive }) => (
              <>
                {isActive ? renderActiveRail : null}
                <div className="flex items-center gap-3">
                  <Settings2
                    className={`h-4 w-4 shrink-0 transition-colors ${
                      isActive ? 'text-primary-400' : 'text-zinc-400 group-hover:text-zinc-200'
                    }`}
                  />
                  {!isSidebarCollapsed ? (
                    <span className="text-sm font-medium">{t('Settings')}</span>
                  ) : null}
                </div>
              </>
            )}
          </NavLink>

          <div ref={userSurfaceRef} className="adminx-shell-sidebar-user-surface">
            <AnimatePresence>
              {isUserMenuOpen ? (
                <motion.div
                  ref={userMenuPanelRef}
                  initial={{ opacity: 0, y: 10, scale: 0.98 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: 8, scale: 0.98 }}
                  transition={{ duration: 0.18, ease: 'easeOut' }}
                  className={`adminx-shell-sidebar-user-menu ${
                    isSidebarCollapsed ? 'is-collapsed' : ''
                  }`}
                  style={userMenuStyle}
                >
                  <div className="adminx-shell-sidebar-user-menu-card">
                    <div className="adminx-shell-sidebar-avatar" aria-hidden="true">
                      {profileInitials}
                    </div>
                    <div className="adminx-shell-sidebar-profile-copy">
                      <strong>{profileName}</strong>
                      <span>{profileDetail}</span>
                    </div>
                  </div>

                  <button
                    type="button"
                    className="adminx-shell-sidebar-menu-action adminx-shell-sidebar-footer-logout is-danger"
                    onClick={() => {
                      setIsUserMenuOpen(false);
                      void handleLogout();
                    }}
                  >
                    <LogOut className="adminx-shell-sidebar-link-icon" />
                    <span>{t('Sign out')}</span>
                  </button>
                </motion.div>
              ) : null}
            </AnimatePresence>

            <button
              ref={userControlRef}
              type="button"
              data-slot="sidebar-user-control"
              title={isSidebarCollapsed ? userControlTitle : undefined}
              className={`${footerActionClassName({ open: isUserMenuOpen })} adminx-shell-sidebar-user-control adminx-shell-sidebar-footer-user`}
              onClick={() => setIsUserMenuOpen((open) => !open)}
            >
              <div className="adminx-shell-sidebar-avatar" aria-hidden="true">
                {sessionUser ? (
                  profileInitials
                ) : (
                  <CircleUserRound className="h-4 w-4 text-zinc-300" />
                )}
              </div>
              {!isSidebarCollapsed ? (
                <>
                  <div className="adminx-shell-sidebar-profile-copy min-w-0 flex-1">
                    <strong>{profileName}</strong>
                    <span>{profileDetail}</span>
                  </div>
                  <ChevronUp
                    className={`h-4 w-4 shrink-0 text-zinc-500 transition-transform ${
                      isUserMenuOpen ? '' : 'rotate-180'
                    }`}
                  />
                </>
              ) : null}
            </button>
          </div>
        </div>
      </aside>
    </motion.div>
  );
}
