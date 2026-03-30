import { useEffect, type ReactNode } from 'react';
import { Search } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import { useAdminI18n } from 'sdkwork-router-admin-commons';

import { ROUTE_PATHS } from '../application/router/routePaths';
import { isTauriDesktop } from '../desktopWindow';

function BrandMark() {
  return (
    <div className="adminx-shell-brand-mark" aria-hidden="true">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
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
    </div>
  );
}

function HeaderActionButton({
  title,
  onClick,
  className = '',
  children,
}: {
  title: string;
  onClick: () => void | Promise<void>;
  className?: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      data-tauri-drag-region="false"
      title={title}
      className={`adminx-shell-header-action flex h-9 items-center justify-center rounded-2xl bg-zinc-950/[0.045] px-3 text-zinc-600 transition-colors hover:bg-zinc-950/[0.08] hover:text-zinc-950 dark:bg-white/[0.06] dark:text-zinc-300 dark:hover:bg-white/[0.12] dark:hover:text-white ${className}`.trim()}
      onClick={() => void onClick()}
    >
      {children}
    </button>
  );
}

export function AppHeader() {
  const navigate = useNavigate();
  const { t } = useAdminI18n();
  const isDesktop = isTauriDesktop();

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        navigate(ROUTE_PATHS.SETTINGS);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [navigate]);

  return (
    <div
      className={`adminx-shell-header-wrap relative z-30 bg-white/72 backdrop-blur-xl dark:bg-zinc-950/78 ${
        isDesktop ? 'is-desktop' : ''
      }`.trim()}
    >
      <header
        className={`adminx-shell-header relative flex h-12 items-center ${
          isDesktop ? 'pl-3 pr-0 sm:pl-4' : 'px-3 sm:px-4'
        }`.trim()}
      >
        <div
          className="adminx-shell-header-main flex min-w-0 flex-1 items-center gap-3"
          data-slot="app-header-leading"
          data-tauri-drag-region={isDesktop ? 'true' : undefined}
        >
          <div className="adminx-shell-brand flex min-w-0 items-center gap-3">
            <BrandMark />
            <div className="adminx-shell-brand-copy min-w-0">
              <div className="truncate text-[11px] font-semibold uppercase leading-none tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                {t('Control plane')}
              </div>
              <div className="truncate text-sm font-semibold leading-none text-zinc-950 dark:text-zinc-50">
                {t('SDKWork Router Admin')}
              </div>
            </div>
          </div>

          <div
            className="adminx-shell-header-search ml-4"
            data-slot="app-header-search"
            data-tauri-drag-region="false"
          >
            <HeaderActionButton
              title={t('Open workspace search')}
              onClick={() => navigate(ROUTE_PATHS.SETTINGS)}
              className="gap-2 px-2.5"
            >
              <Search className="h-4 w-4" />
              <span className="adminx-shell-header-search-label hidden text-xs font-medium md:inline">
                {t('Search')}
              </span>
              <span className="adminx-shell-header-search-shortcut hidden rounded-full bg-zinc-950/[0.06] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-zinc-500 dark:bg-white/[0.08] dark:text-zinc-400 md:inline">
                Ctrl K
              </span>
            </HeaderActionButton>
          </div>
        </div>
      </header>
    </div>
  );
}
