import { Sparkles } from 'lucide-react';

import { isTauriDesktop } from '../lib/desktop';
import { WindowControls } from './WindowControls';

function BrandMark() {
  return (
    <div className="flex h-7 w-7 items-center justify-center rounded-xl bg-primary-600">
      <Sparkles className="h-4 w-4 text-white" />
    </div>
  );
}

export function AppHeader() {
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
                SDKWork Router
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
