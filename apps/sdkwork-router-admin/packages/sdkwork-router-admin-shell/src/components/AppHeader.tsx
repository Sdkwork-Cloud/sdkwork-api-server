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
      className={`adminx-shell-header-action ${className}`.trim()}
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
    <div className={`adminx-shell-header-wrap ${isDesktop ? 'is-desktop' : ''}`.trim()}>
      <header className="adminx-shell-header">
        <div
          className="adminx-shell-header-main"
          data-slot="app-header-leading"
          data-tauri-drag-region={isDesktop ? 'true' : undefined}
        >
          <div className="adminx-shell-brand">
            <BrandMark />
            <div className="adminx-shell-brand-copy">
              <span>{t('Control plane')}</span>
              <strong>{t('SDKWork Router Admin')}</strong>
            </div>
          </div>

          <div
            className="adminx-shell-header-search"
            data-slot="app-header-search"
            data-tauri-drag-region="false"
          >
            <HeaderActionButton
              title={t('Open workspace search')}
              onClick={() => navigate(ROUTE_PATHS.SETTINGS)}
            >
              <Search className="adminx-shell-meta-icon" />
              <span className="adminx-shell-header-search-label">{t('Search')}</span>
              <span className="adminx-shell-header-search-shortcut">Ctrl K</span>
            </HeaderActionButton>
          </div>
        </div>
      </header>
    </div>
  );
}
