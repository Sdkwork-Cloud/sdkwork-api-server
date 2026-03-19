import { Minus, Square, X } from 'lucide-react';

import { closeWindow, maximizeWindow, minimizeWindow } from '../lib/desktop';

export function WindowControls() {
  return (
    <div className="flex h-full items-stretch" data-tauri-drag-region="false">
      <button
        className="flex h-full w-11 items-center justify-center text-zinc-500 transition-colors hover:bg-zinc-950/[0.06] hover:text-zinc-950 dark:text-zinc-300 dark:hover:bg-white/[0.1] dark:hover:text-white"
        data-tauri-drag-region="false"
        onClick={() => {
          void minimizeWindow();
        }}
        title="minimizeWindow"
        type="button"
      >
        <Minus className="h-4 w-4" />
      </button>
      <button
        className="flex h-full w-11 items-center justify-center text-zinc-500 transition-colors hover:bg-zinc-950/[0.06] hover:text-zinc-950 dark:text-zinc-300 dark:hover:bg-white/[0.1] dark:hover:text-white"
        data-tauri-drag-region="false"
        onClick={() => {
          void maximizeWindow();
        }}
        title="maximizeWindow"
        type="button"
      >
        <Square className="h-3.5 w-3.5" />
      </button>
      <button
        className="flex h-full w-11 items-center justify-center text-zinc-500 transition-colors hover:bg-rose-500 hover:text-white dark:text-zinc-300"
        data-tauri-drag-region="false"
        onClick={() => {
          void closeWindow();
        }}
        title="closeWindow"
        type="button"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}
