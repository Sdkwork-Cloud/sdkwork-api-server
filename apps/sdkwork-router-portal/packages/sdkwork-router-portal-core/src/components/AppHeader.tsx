import { usePortalI18n } from 'sdkwork-router-portal-commons';

import { isTauriDesktop } from '../lib/desktop';
import { WindowControls } from './WindowControls';

function BrandMark() {
  return (
    <div className="flex h-7 w-7 items-center justify-center rounded-xl bg-primary-600">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        className="h-4 w-4 text-white"
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

export function AppHeader() {
  const { t } = usePortalI18n();
  const desktopMode = isTauriDesktop();

  return (
    <div className="relative z-30 bg-white/72 backdrop-blur-xl dark:bg-zinc-950/78">
      <header className={`relative flex h-12 items-center ${desktopMode ? 'pl-3 pr-0 sm:pl-4' : 'px-3 sm:px-4'}`}>
        <div
          className="flex min-w-0 flex-1 items-center gap-3"
          data-slot="app-header-leading"
          data-tauri-drag-region
        >
          <div className="flex min-w-0 items-center gap-3">
            <BrandMark />
            <div className="min-w-0">
              <div className="truncate text-sm font-semibold leading-none text-zinc-950 dark:text-zinc-50">
                {t('SDKWork Router')}
              </div>
            </div>
          </div>
        </div>

        <div
          className="ml-auto flex h-full items-stretch justify-end"
          data-slot="app-header-trailing"
          data-tauri-drag-region="false"
        >
          {desktopMode ? <WindowControls /> : null}
        </div>
      </header>
    </div>
  );
}
