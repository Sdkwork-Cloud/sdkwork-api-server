import { create } from 'zustand';
import { persist } from 'zustand/middleware';

import type { AdminSidebarItemKey, ThemeColor, ThemeMode } from 'sdkwork-router-admin-types';

const CLAW_SIDEBAR_WIDTH = 240;

function clampSidebarWidth(_: number): number {
  return CLAW_SIDEBAR_WIDTH;
}

interface AdminAppStore {
  isSidebarCollapsed: boolean;
  sidebarWidth: number;
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

export const useAdminAppStore = create<AdminAppStore>()(
  persist(
    (set) => ({
      isSidebarCollapsed: false,
      sidebarWidth: CLAW_SIDEBAR_WIDTH,
      hiddenSidebarItems: [],
      themeMode: 'system',
      themeColor: 'lobster',
      toggleSidebar: () =>
        set((state) => ({ isSidebarCollapsed: !state.isSidebarCollapsed })),
      setSidebarCollapsed: (isSidebarCollapsed) => set({ isSidebarCollapsed }),
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
      merge: (persistedState, currentState) => {
        const nextState = (persistedState as Partial<AdminAppStore>) || {};
        return {
          ...currentState,
          ...nextState,
          sidebarWidth: clampSidebarWidth(nextState.sidebarWidth ?? currentState.sidebarWidth),
        };
      },
    },
  ),
);
