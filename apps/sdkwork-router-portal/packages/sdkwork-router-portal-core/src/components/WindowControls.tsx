import { Minus, Square, X } from 'lucide-react';

import { closeWindow, maximizeWindow, minimizeWindow } from '../lib/desktop';

export function WindowControls() {
  return (
    <div
      className="flex h-full items-stretch border-l border-[color:var(--portal-titlebar-border)]"
      data-tauri-drag-region="false"
    >
      <button
        className="flex h-full w-11 items-center justify-center text-[var(--portal-text-muted-on-contrast)] transition-colors hover:[background:var(--portal-window-control-hover)] hover:text-[var(--portal-text-on-contrast)]"
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
        className="flex h-full w-11 items-center justify-center text-[var(--portal-text-muted-on-contrast)] transition-colors hover:[background:var(--portal-window-control-hover)] hover:text-[var(--portal-text-on-contrast)]"
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
        className="flex h-full w-11 items-center justify-center text-[var(--portal-text-muted-on-contrast)] transition-colors hover:[background:var(--portal-window-control-danger-hover)] hover:text-white"
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
