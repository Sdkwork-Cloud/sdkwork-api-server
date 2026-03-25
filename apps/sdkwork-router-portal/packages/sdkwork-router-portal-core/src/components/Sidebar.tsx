import {
  Activity,
  Coins,
  Gauge,
  KeyRound,
  PanelLeftClose,
  ReceiptText,
  Route,
  UserRound,
  WalletCards,
  type LucideIcon,
} from 'lucide-react';
import { motion } from 'motion/react';
import { NavLink } from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { resolvePortalPath } from '../application/router/routeManifest';
import { PORTAL_COLLAPSED_SIDEBAR_WIDTH } from '../lib/portalPreferences';
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
    workspace?.user.display_name?.trim()
    || workspace?.user.email?.split('@')[0]
    || workspace?.tenant.name
    || 'PW';

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
  const { t } = usePortalI18n();
  const storedWorkspace = usePortalAuthStore((state) => state.workspace);
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, toggleSidebar } =
    usePortalShellStore();

  const resolvedWorkspace = workspace ?? storedWorkspace;
  const userInitials = getUserInitials(resolvedWorkspace);
  const currentWidth = isSidebarCollapsed ? PORTAL_COLLAPSED_SIDEBAR_WIDTH : sidebarWidth;

  return (
    <motion.div
      initial={false}
      animate={{ width: currentWidth }}
      transition={{ type: 'spring', stiffness: 300, damping: 30, mass: 0.8 }}
      className="relative z-20 flex h-full shrink-0"
    >
      <aside className="flex h-full w-full flex-col overflow-hidden border-r border-zinc-900/90 bg-zinc-950 bg-[linear-gradient(180deg,_#13151a_0%,_#0b0c10_100%)] text-zinc-300 shadow-[18px_0_50px_rgba(9,9,11,0.16)]">
        <div className={`relative flex flex-col pb-4 pt-6 ${isSidebarCollapsed ? 'items-center' : 'px-5'}`}>
          <div
            className={`flex w-full items-center overflow-hidden whitespace-nowrap ${isSidebarCollapsed ? 'justify-center' : 'gap-3'}`}
          >
            <button
              type="button"
              onClick={toggleSidebar}
              title={isSidebarCollapsed ? t('Expand sidebar') : undefined}
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
                    {t('Developer portal')}
                  </div>
                  <div className="truncate text-[19px] font-bold tracking-tight text-white">
                    {t('SDKWork Router')}
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

        <nav className={`scrollbar-hide flex-1 space-y-5 overflow-x-hidden overflow-y-auto ${isSidebarCollapsed ? 'px-2 pb-4' : 'px-3 pb-5'}`}>
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
                    {t(group.title)}
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
                        key={route.key}
                        title={isSidebarCollapsed ? t(route.label) : undefined}
                        to={resolvePortalPath(route.key)}
                        className={({ isActive }) =>
                          `group relative flex items-center rounded-xl transition-all duration-200 ${
                            isSidebarCollapsed ? 'mx-auto h-10 w-10 justify-center' : 'justify-between px-3 py-2.5'
                          } ${
                            isActive
                              ? 'bg-primary-500/10 font-medium text-primary-400'
                              : 'text-zinc-400 hover:bg-white/[0.05] hover:text-zinc-200'
                          }`
                        }
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
    </motion.div>
  );
}
