import { Sparkles } from 'lucide-react';

import { isTauriDesktop } from '../lib/desktop';
import { WindowControls } from './WindowControls';

function BrandMark() {
  return (
    <div className="flex h-9 w-9 items-center justify-center rounded-2xl bg-[linear-gradient(135deg,rgb(var(--portal-accent-strong-rgb)_/_0.98),rgb(var(--portal-accent-rgb)_/_0.9))] text-white shadow-[0_14px_30px_rgb(var(--portal-accent-rgb)_/_0.28)]">
      <Sparkles className="h-4 w-4" />
    </div>
  );
}

export function AppHeader() {
  const desktopMode = isTauriDesktop();

  return (
    <div className="relative z-20 border-b border-[color:var(--portal-titlebar-border)] [background:var(--portal-surface-contrast)] text-[var(--portal-text-on-contrast)] shadow-[0_18px_44px_rgba(3,7,18,0.18)]">
      <header
        className={`mx-auto flex h-14 w-full max-w-[1680px] items-center justify-between ${desktopMode ? 'pl-4 pr-0' : 'px-4'}`.trim()}
      >
        <div
          className="flex min-w-0 items-center gap-3"
          data-tauri-drag-region={desktopMode ? 'true' : undefined}
        >
          <BrandMark />
          <div className="min-w-0">
            <span className="block truncate text-[10px] font-semibold uppercase tracking-[0.26em] text-[var(--portal-text-muted-on-contrast)]">
              SDKWork Router
            </span>
            <strong className="block truncate text-sm font-semibold tracking-[0.01em] text-[var(--portal-text-on-contrast)]">
              Portal Workspace
            </strong>
          </div>
        </div>

        {desktopMode ? <WindowControls /> : null}
      </header>
    </div>
  );
}
