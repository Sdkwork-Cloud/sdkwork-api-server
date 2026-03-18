import { LogOut, Settings2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

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
  onLogout: () => void;
  onOpenConfigCenter: () => void;
  userInitials: string;
  workspace: PortalWorkspaceSummary | null;
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const profileName = resolveProfileName(workspace);
  const profileMeta = resolveProfileMeta(workspace);
  const workspaceContext = resolveWorkspaceContext(workspace);

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
    onLogout();
  }

  return (
    <div className="relative" ref={containerRef}>
      <button
        aria-expanded={open}
        aria-haspopup="menu"
        className={`group flex w-full items-center rounded-[24px] border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-surface)] text-left text-[var(--portal-sidebar-text)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] transition duration-200 hover:[background:var(--portal-sidebar-dock-hover)] ${
          isSidebarCollapsed ? 'relative mx-auto h-12 w-12 justify-center' : 'gap-3 px-3 py-3'
        }`}
        onClick={() => setOpen((current) => !current)}
        title={isSidebarCollapsed ? `${profileName} settings` : undefined}
        type="button"
      >
        <div className="relative flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-[color:rgb(var(--portal-accent-rgb)_/_0.22)] text-sm font-semibold text-white shadow-[0_14px_30px_rgb(var(--portal-accent-rgb)_/_0.18)]">
          {userInitials}
          {isSidebarCollapsed ? (
            <span className="absolute -bottom-1 -right-1 flex h-5 w-5 items-center justify-center rounded-full border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-panel)] text-[var(--portal-text-on-contrast)] shadow-[0_10px_24px_rgba(3,7,18,0.26)]">
              <Settings2 className="h-3 w-3" />
            </span>
          ) : null}
        </div>

        {!isSidebarCollapsed ? (
          <>
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm font-semibold text-[var(--portal-sidebar-text)]">
                {workspace?.user.display_name ?? profileName}
              </div>
              <div className="truncate text-xs text-[var(--portal-sidebar-muted)]">
                {workspace?.user.email ?? profileMeta}
              </div>
            </div>
            <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-surface)] text-[var(--portal-sidebar-dock-muted)] transition-colors group-hover:text-[var(--portal-sidebar-text)]">
              <Settings2 className="h-4 w-4" />
            </div>
          </>
        ) : null}
      </button>

      {open ? (
        <div
          className={`absolute z-40 ${isSidebarCollapsed ? 'bottom-0 left-full ml-3 w-72' : 'bottom-full left-0 right-0 mb-3'}`}
          role="menu"
        >
          <div className="overflow-hidden rounded-[28px] border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-panel)] p-2 text-[var(--portal-text-on-contrast)] shadow-[var(--portal-shadow-strong)]">
            <div className="rounded-[22px] border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-panel-accent)] px-4 py-4">
              <div className="flex items-center gap-3">
                <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl bg-[color:rgb(var(--portal-accent-rgb)_/_0.22)] text-sm font-semibold text-white shadow-[0_14px_30px_rgb(var(--portal-accent-rgb)_/_0.18)]">
                  {userInitials}
                </div>
                <div className="min-w-0">
                  <div className="truncate text-sm font-semibold text-[var(--portal-text-on-contrast)]">
                    {workspace?.user.display_name ?? profileName}
                  </div>
                  <div className="truncate text-xs text-[var(--portal-sidebar-dock-muted)]">
                    {workspace?.user.email ?? profileMeta}
                  </div>
                </div>
              </div>
              <div className="mt-3 rounded-2xl border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-surface)] px-3 py-2">
                <div className="truncate text-[11px] font-semibold uppercase tracking-[0.22em] text-[var(--portal-sidebar-dock-muted)]">
                  Active workspace
                </div>
                <div className="truncate text-sm font-semibold text-[var(--portal-text-on-contrast)]">
                  {workspace?.project.name ?? 'Portal Workspace'}
                </div>
                <div className="truncate text-xs text-[var(--portal-sidebar-dock-muted)]">
                  {workspace?.tenant.name ?? workspaceContext}
                </div>
              </div>
            </div>

            <div className="mt-2 grid gap-1">
              <button
                className="flex w-full items-center gap-3 rounded-[20px] border border-transparent px-3 py-3 text-left text-sm font-medium text-[var(--portal-text-on-contrast)] transition-colors hover:border-[color:var(--portal-sidebar-dock-border)] hover:[background:var(--portal-sidebar-dock-hover)]"
                onClick={handleOpenConfigCenter}
                role="menuitem"
                type="button"
              >
                <span className="flex h-9 w-9 items-center justify-center rounded-2xl border border-[color:var(--portal-sidebar-dock-border)] [background:var(--portal-sidebar-dock-surface)] text-[var(--portal-sidebar-dock-muted)]">
                  <Settings2 className="h-4 w-4" />
                </span>
                <span className="grid gap-0.5">
                  <span>Settings</span>
                  <span className="text-xs font-normal text-[var(--portal-sidebar-dock-muted)]">
                    Theme, sidebar, and shell preferences
                  </span>
                </span>
              </button>

              <button
                className="flex w-full items-center gap-3 rounded-[20px] border border-transparent px-3 py-3 text-left text-sm font-medium text-[var(--portal-text-on-contrast)] transition-colors hover:border-[color:var(--portal-danger-border)] hover:[background:var(--portal-danger-hover)]"
                onClick={handleLogout}
                role="menuitem"
                type="button"
              >
                <span className="flex h-9 w-9 items-center justify-center rounded-2xl border border-[color:var(--portal-danger-border)] [background:var(--portal-danger-soft)] text-[var(--portal-danger-text)]">
                  <LogOut className="h-4 w-4" />
                </span>
                <span className="grid gap-0.5">
                  <span>Sign out</span>
                  <span className="text-xs font-normal text-[var(--portal-sidebar-dock-muted)]">
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
