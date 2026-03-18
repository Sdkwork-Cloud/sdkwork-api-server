type TauriWindowController = {
  close?: () => Promise<void>;
  maximize?: () => Promise<void>;
  minimize?: () => Promise<void>;
  toggleMaximize?: () => Promise<void>;
};

type TauriWindowLike = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: unknown;
  isTauri?: boolean;
};

function resolveWindow(): TauriWindowLike | null {
  if (typeof window === 'undefined') {
    return null;
  }

  return window as TauriWindowLike;
}

export function isTauriDesktop(): boolean {
  const currentWindow = resolveWindow();
  return Boolean(
    currentWindow?.isTauri ||
      currentWindow?.__TAURI__ ||
      currentWindow?.__TAURI_INTERNALS__,
  );
}

async function getCurrentTauriWindow(): Promise<TauriWindowController | null> {
  if (!isTauriDesktop()) {
    return null;
  }

  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export async function minimizeWindow(): Promise<void> {
  const currentWindow = await getCurrentTauriWindow();
  await currentWindow?.minimize?.();
}

export async function maximizeWindow(): Promise<void> {
  const currentWindow = await getCurrentTauriWindow();

  if (!currentWindow) {
    return;
  }

  if (currentWindow.toggleMaximize) {
    await currentWindow.toggleMaximize();
    return;
  }

  await currentWindow.maximize?.();
}

export async function closeWindow(): Promise<void> {
  const currentWindow = await getCurrentTauriWindow();
  await currentWindow?.close?.();
}
