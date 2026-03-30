import { ChevronUp, CircleUserRound, LogIn, LogOut, Settings2 } from 'lucide-react';
import { useEffect, useRef, useState, type CSSProperties } from 'react';
import { useNavigate } from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { usePortalAuthStore } from '../store/usePortalAuthStore';

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

export function SidebarProfileDock({
  isSidebarCollapsed,
  onLogout,
  onOpenConfigCenter,
  sidebarWidth,
  userInitials,
  workspace,
}: {
  isSidebarCollapsed: boolean;
  onLogout?: () => void;
  onOpenConfigCenter: () => void;
  sidebarWidth: number;
  userInitials: string;
  workspace: PortalWorkspaceSummary | null;
}) {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const isAuthenticated = usePortalAuthStore((state) => state.isAuthenticated);
  const signOut = usePortalAuthStore((state) => state.signOut);
  const [open, setOpen] = useState(false);
  const [userMenuStyle, setUserMenuStyle] = useState<CSSProperties>();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const userControlRef = useRef<HTMLButtonElement | null>(null);
  const userMenuPanelRef = useRef<HTMLDivElement | null>(null);
  const profileName =
    workspace?.user.display_name?.trim()
    || workspace?.tenant.name
    || workspace?.project.name
    || t('Portal operator');
  const profileMeta = workspace?.user.email ?? workspace?.tenant.name ?? t('Portal workspace');

  useEffect(() => {
    if (!open) {
      setUserMenuStyle(undefined);
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target;

      if (!(target instanceof Node)) {
        return;
      }

      if (!containerRef.current?.contains(target)) {
        setOpen(false);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setOpen(false);
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const viewportPadding = 16;
    const menuGap = 12;

    const updateUserMenuPosition = () => {
      const userControl = userControlRef.current;
      if (!userControl) {
        return;
      }

      const triggerRect = userControl.getBoundingClientRect();
      const nextMenuWidth = Math.min(288, window.innerWidth - viewportPadding * 2);
      const measuredMenuHeight = userMenuPanelRef.current?.offsetHeight ?? 168;
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
  }, [isSidebarCollapsed, open, sidebarWidth]);

  const footerActionClass = `group relative flex items-center rounded-xl border border-white/8 bg-white/[0.04] text-zinc-300 transition-all duration-200 hover:bg-white/[0.07] hover:text-white ${
    isSidebarCollapsed ? 'mx-auto h-10 w-10 justify-center px-0' : 'w-full gap-3 px-3 py-2.5'
  }`;

  function handleLogout() {
    setOpen(false);
    void signOut().then(() => {
      onLogout?.();
      navigate('/login', { replace: true });
    });
  }

  return (
    <div className="relative grid gap-1" ref={containerRef}>
      <button
        type="button"
        data-slot="portal-sidebar-footer-settings"
        className={footerActionClass}
        onClick={() => {
          setOpen(false);
          onOpenConfigCenter();
        }}
        title={isSidebarCollapsed ? t('Settings') : undefined}
      >
        <Settings2 className="h-4 w-4 shrink-0 text-zinc-400 transition-colors group-hover:text-zinc-200" />
        {!isSidebarCollapsed ? <span className="text-sm font-medium">{t('Settings')}</span> : null}
      </button>

      <button
        ref={userControlRef}
        type="button"
        data-slot="portal-sidebar-user-control"
        aria-expanded={open}
        aria-haspopup="menu"
        className={`${footerActionClass} ${open ? 'border-white/12 bg-white/[0.08] text-white' : ''}`}
        onClick={() => {
          if (!isAuthenticated) {
            navigate('/login');
            return;
          }

          setOpen((current) => !current);
        }}
        title={isSidebarCollapsed ? profileName : undefined}
      >
        <div className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-primary-500/15 text-sm font-semibold text-white">
          {isAuthenticated ? userInitials : <CircleUserRound className="h-4 w-4 text-zinc-300" />}
        </div>
        {!isSidebarCollapsed ? (
          <>
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm font-semibold text-white">{profileName}</div>
              <div className="truncate text-xs text-zinc-500">{profileMeta}</div>
            </div>
            {isAuthenticated ? (
              <ChevronUp
                className={`h-4 w-4 shrink-0 text-zinc-500 transition-transform ${
                  open ? '' : 'rotate-180'
                }`}
              />
            ) : (
              <LogIn className="h-4 w-4 shrink-0 text-zinc-500 transition-colors group-hover:text-zinc-300" />
            )}
          </>
        ) : null}
      </button>

      {open ? (
        <div
          ref={userMenuPanelRef}
          className="fixed z-40"
          role="menu"
          style={userMenuStyle}
        >
          <div className="overflow-hidden rounded-3xl border border-white/10 bg-zinc-950/96 p-2 text-white shadow-[0_20px_48px_rgba(9,9,11,0.34)] backdrop-blur-xl">
            <div className="flex items-center gap-3 rounded-2xl border border-white/8 bg-white/[0.04] p-3">
              <div className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-primary-500/15 text-sm font-bold text-primary-200">
                {isAuthenticated ? userInitials : <CircleUserRound className="h-4 w-4 text-zinc-300" />}
              </div>
              <div className="min-w-0">
                <div className="truncate text-sm font-semibold text-white">{profileName}</div>
                <div className="truncate text-xs text-zinc-400">{profileMeta}</div>
              </div>
            </div>

            <button
              className="mt-2 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm text-rose-300 transition-colors hover:bg-rose-500/10 hover:text-rose-200"
              onClick={handleLogout}
              role="menuitem"
              type="button"
            >
              <LogOut className="h-4 w-4" />
              <span>{t('Sign out')}</span>
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
