import { ChevronUp, CircleUserRound, LogIn, LogOut, Settings2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { usePortalAuthStore } from '../store/usePortalAuthStore';

function resolveProfileName(workspace: PortalWorkspaceSummary | null): string {
  return (
    workspace?.user.display_name?.trim() ||
    workspace?.tenant.name ||
    workspace?.project.name ||
    'Portal operator'
  );
}

function resolveProfileMeta(workspace: PortalWorkspaceSummary | null): string {
  return workspace?.user.email ?? 'Workspace settings';
}

function resolveWorkspaceContext(workspace: PortalWorkspaceSummary | null): string {
  const projectName = workspace?.project.name ?? 'Portal Workspace';
  const tenantName = workspace?.tenant.name ?? 'Portal tenant';

  return `${projectName} / ${tenantName}`;
}

export function SidebarProfileDock({
  isSidebarCollapsed,
  onLogout,
  onOpenConfigCenter,
  userInitials,
  workspace,
}: {
  isSidebarCollapsed: boolean;
  onLogout?: () => void;
  onOpenConfigCenter: () => void;
  userInitials: string;
  workspace: PortalWorkspaceSummary | null;
}) {
  const navigate = useNavigate();
  const isAuthenticated = usePortalAuthStore((state) => state.isAuthenticated);
  const signOut = usePortalAuthStore((state) => state.signOut);
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const profileName = resolveProfileName(workspace);
  const profileMeta = resolveProfileMeta(workspace);
  const workspaceContext = resolveWorkspaceContext(workspace);
  const triggerTitle = isAuthenticated
    ? open
      ? `${profileName} menu close`
      : `${profileName} menu open`
    : 'Sign in';

  useEffect(() => {
    if (!open) {
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

  function handleOpenConfigCenter() {
    setOpen(false);
    onOpenConfigCenter();
  }

  function handleLogout() {
    setOpen(false);
    void signOut().then(() => {
      onLogout?.();
      navigate('/login', { replace: true });
    });
  }

  return (
    <div className="relative" ref={containerRef}>
      <button
        aria-expanded={open}
        aria-haspopup="menu"
        className={`group relative flex w-full items-center rounded-2xl border border-white/8 bg-white/[0.04] text-zinc-300 transition-all duration-200 hover:bg-white/[0.07] hover:text-white ${
          isSidebarCollapsed
            ? 'mx-auto h-11 w-11 justify-center px-0'
            : 'gap-3 px-2.5 py-2.5'
        }`}
        onClick={() => {
          if (!isAuthenticated) {
            navigate('/login');
            return;
          }

          setOpen((current) => !current);
        }}
        title={isSidebarCollapsed ? triggerTitle : undefined}
        type="button"
      >
        <div className="flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-2xl bg-white/[0.08] text-sm font-semibold text-white">
          {isAuthenticated ? userInitials : <CircleUserRound className="h-4 w-4 text-zinc-300" />}
        </div>

        {!isSidebarCollapsed ? (
          <>
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm font-semibold text-white">
                {workspace?.user.display_name ?? profileName}
              </div>
              <div className="truncate text-xs text-zinc-500">
                {workspace?.user.email ?? profileMeta}
              </div>
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
          className={`absolute z-40 ${isSidebarCollapsed ? 'bottom-0 left-full ml-3 w-72' : 'bottom-full left-0 right-0 mb-3'}`}
          role="menu"
        >
          <div className="overflow-hidden rounded-3xl border border-white/10 bg-zinc-950/96 p-2 text-white shadow-[0_20px_48px_rgba(9,9,11,0.34)] backdrop-blur-xl">
            <div className="rounded-2xl border border-white/8 bg-white/[0.04] p-3">
              <div className="flex items-center gap-3">
                <div className="flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-2xl bg-primary-500/15 text-sm font-bold text-primary-200">
                  {isAuthenticated ? userInitials : <CircleUserRound className="h-4 w-4 text-zinc-300" />}
                </div>
                <div className="min-w-0">
                  <div className="truncate text-sm font-semibold text-white">
                    {workspace?.user.display_name ?? profileName}
                  </div>
                  <div className="truncate text-xs text-zinc-400">
                    {workspace?.user.email ?? profileMeta}
                  </div>
                </div>
              </div>
              <div className="mt-3 rounded-2xl border border-white/8 bg-white/[0.04] px-3 py-2">
                <div className="truncate text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-400">
                  Active workspace
                </div>
                <div className="truncate text-sm font-semibold text-white">
                  {workspace?.project.name ?? 'Portal Workspace'}
                </div>
                <div className="truncate text-xs text-zinc-400">
                  {workspace?.tenant.name ?? workspaceContext}
                </div>
              </div>
            </div>

            <div className="mt-2 grid gap-1">
              <button
                className="flex w-full items-center gap-3 rounded-2xl px-3 py-2.5 text-left text-sm text-zinc-300 transition-colors hover:bg-white/[0.06] hover:text-white"
                onClick={handleOpenConfigCenter}
                role="menuitem"
                type="button"
              >
                <Settings2 className="h-4 w-4 text-zinc-500" />
                <span className="grid gap-0.5 text-left">
                  <span>Settings</span>
                  <span className="text-xs font-normal text-zinc-400">
                    Theme, sidebar, and shell preferences
                  </span>
                </span>
              </button>

              <button
                className="mt-1 flex w-full items-center gap-3 rounded-2xl px-3 py-2.5 text-left text-sm text-rose-300 transition-colors hover:bg-rose-500/10 hover:text-rose-200"
                onClick={handleLogout}
                role="menuitem"
                type="button"
              >
                <LogOut className="h-4 w-4" />
                <span className="grid gap-0.5 text-left">
                  <span>Sign out</span>
                  <span className="text-xs font-normal text-zinc-400">
                    End this portal session on the current device
                  </span>
                </span>
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
