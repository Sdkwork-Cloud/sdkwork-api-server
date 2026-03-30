import { create } from 'zustand';
import { persist } from 'zustand/middleware';

import type { AdminSidebarItemKey, ThemeColor, ThemeMode } from 'sdkwork-router-admin-types';
import { resolveAutoSidebarCollapsed } from './sidebarAutoCollapse';

type SidebarCollapsePreference = 'auto' | 'user';

const DEFAULT_SIDEBAR_WIDTH = 252;
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 360;

function clampSidebarWidth(width: number): number {
  return Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, width));
}

interface AdminAppStore {
  isSidebarCollapsed: boolean;
  sidebarWidth: number;
  sidebarCollapsePreference: SidebarCollapsePreference;
  hiddenSidebarItems: AdminSidebarItemKey[];
  themeMode: ThemeMode;
  themeColor: ThemeColor;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSidebarWidth: (width: number) => void;
  toggleSidebarItem: (key: AdminSidebarItemKey) => void;
  setThemeMode: (themeMode: ThemeMode) => void;
  setThemeColor: (themeColor: ThemeColor) => void;
}

type PersistedAdminAppStore = Pick<
  AdminAppStore,
  | 'isSidebarCollapsed'
  | 'sidebarWidth'
  | 'sidebarCollapsePreference'
  | 'hiddenSidebarItems'
  | 'themeMode'
  | 'themeColor'
>;

function resolveSidebarCollapsePreference(
  nextState: Partial<PersistedAdminAppStore>,
  currentState: AdminAppStore,
): SidebarCollapsePreference {
  if (
    nextState.sidebarCollapsePreference === 'auto'
    || nextState.sidebarCollapsePreference === 'user'
  ) {
    return nextState.sidebarCollapsePreference;
  }

  if (typeof nextState.isSidebarCollapsed === 'boolean') {
    return 'user';
  }

  return currentState.sidebarCollapsePreference;
}

export const useAdminAppStore = create<AdminAppStore>()(
  persist(
    (set) => ({
      isSidebarCollapsed: resolveAutoSidebarCollapsed(),
      sidebarWidth: DEFAULT_SIDEBAR_WIDTH,
      sidebarCollapsePreference: 'auto',
      hiddenSidebarItems: [],
      themeMode: 'system',
      themeColor: 'lobster',
      toggleSidebar: () =>
        set((state) => ({
          isSidebarCollapsed: !state.isSidebarCollapsed,
          sidebarCollapsePreference: 'user',
        })),
      setSidebarCollapsed: (isSidebarCollapsed) =>
        set({ isSidebarCollapsed, sidebarCollapsePreference: 'user' }),
      setSidebarWidth: (sidebarWidth) => set({ sidebarWidth: clampSidebarWidth(sidebarWidth) }),
      toggleSidebarItem: (key) =>
        set((state) => ({
          hiddenSidebarItems: state.hiddenSidebarItems.includes(key)
            ? state.hiddenSidebarItems.filter((item) => item !== key)
            : [...state.hiddenSidebarItems, key],
        })),
      setThemeMode: (themeMode) => set({ themeMode }),
      setThemeColor: (themeColor) => set({ themeColor }),
    }),
    {
      name: 'sdkwork-router-admin-ui-store',
      partialize: (state): PersistedAdminAppStore => ({
        isSidebarCollapsed: state.isSidebarCollapsed,
        sidebarWidth: clampSidebarWidth(state.sidebarWidth),
        sidebarCollapsePreference: state.sidebarCollapsePreference,
        hiddenSidebarItems: state.hiddenSidebarItems,
        themeMode: state.themeMode,
        themeColor: state.themeColor,
      }),
      merge: (persistedState, currentState) => {
        const nextState = (persistedState as Partial<PersistedAdminAppStore>) || {};
        const sidebarCollapsePreference = resolveSidebarCollapsePreference(nextState, currentState);
        const isSidebarCollapsed =
          sidebarCollapsePreference === 'auto'
            ? resolveAutoSidebarCollapsed()
            : nextState.isSidebarCollapsed ?? currentState.isSidebarCollapsed;

        return {
          ...currentState,
          ...nextState,
          isSidebarCollapsed,
          sidebarCollapsePreference,
          sidebarWidth: clampSidebarWidth(nextState.sidebarWidth ?? currentState.sidebarWidth),
        };
      },
    },
  ),
);
