import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { PortalRouteKey, PortalThemeColor, PortalThemeMode } from 'sdkwork-router-portal-types';

import {
  clampSidebarWidth,
  PORTAL_DEFAULT_SIDEBAR_WIDTH,
  PORTAL_PREFERENCES_STORAGE_KEY,
} from '../lib/portalPreferences';

interface PortalShellState {
  isSidebarCollapsed: boolean;
  sidebarWidth: number;
  hiddenSidebarItems: PortalRouteKey[];
  themeMode: PortalThemeMode;
  themeColor: PortalThemeColor;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSidebarWidth: (width: number) => void;
  toggleSidebarItem: (itemId: PortalRouteKey) => void;
  setThemeMode: (themeMode: PortalThemeMode) => void;
  setThemeColor: (themeColor: PortalThemeColor) => void;
  resetShellPreferences: () => void;
}

const PORTAL_THEME_COLOR_IDS: PortalThemeColor[] = [
  'tech-blue',
  'lobster',
  'green-tech',
  'zinc',
  'violet',
  'rose',
];

const defaultShellState = {
  isSidebarCollapsed: false,
  sidebarWidth: PORTAL_DEFAULT_SIDEBAR_WIDTH,
  hiddenSidebarItems: [] as PortalRouteKey[],
  themeMode: 'system' as PortalThemeMode,
  themeColor: 'lobster' as PortalThemeColor,
};

export const usePortalShellStore = create<PortalShellState>()(
  persist(
    (set) => ({
      ...defaultShellState,
      toggleSidebar: () =>
        set((state) => ({
          isSidebarCollapsed: !state.isSidebarCollapsed,
        })),
      setSidebarCollapsed: (isSidebarCollapsed) => set({ isSidebarCollapsed }),
      setSidebarWidth: (sidebarWidth) =>
        set({
          sidebarWidth: clampSidebarWidth(sidebarWidth),
        }),
      toggleSidebarItem: (itemId) =>
        set((state) => ({
          hiddenSidebarItems: state.hiddenSidebarItems.includes(itemId)
            ? state.hiddenSidebarItems.filter((candidate) => candidate !== itemId)
            : [...state.hiddenSidebarItems, itemId],
        })),
      setThemeMode: (themeMode) => set({ themeMode }),
      setThemeColor: (themeColor) => set({ themeColor }),
      resetShellPreferences: () => set(defaultShellState),
    }),
    {
      name: PORTAL_PREFERENCES_STORAGE_KEY,
      partialize: (state) => ({
        isSidebarCollapsed: state.isSidebarCollapsed,
        sidebarWidth: clampSidebarWidth(state.sidebarWidth),
        hiddenSidebarItems: state.hiddenSidebarItems,
        themeMode: state.themeMode,
        themeColor: state.themeColor,
      }),
      merge: (persistedState, currentState) => {
        const nextState = (persistedState as Partial<PortalShellState>) || {};
        const nextThemeColor = PORTAL_THEME_COLOR_IDS.includes(
          nextState.themeColor ?? currentState.themeColor,
        )
          ? nextState.themeColor ?? currentState.themeColor
          : currentState.themeColor;

        return {
          ...currentState,
          ...nextState,
          sidebarWidth: clampSidebarWidth(nextState.sidebarWidth ?? currentState.sidebarWidth),
          themeColor: nextThemeColor,
        };
      },
    },
  ),
);
