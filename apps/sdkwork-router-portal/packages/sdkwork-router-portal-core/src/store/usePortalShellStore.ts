import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { PortalRouteKey, PortalThemeColor, PortalThemeMode } from 'sdkwork-router-portal-types';

import {
  clampSidebarWidth,
  PORTAL_DEFAULT_SIDEBAR_WIDTH,
  PORTAL_PREFERENCES_STORAGE_KEY,
} from '../lib/portalPreferences';
import { resolveAutoSidebarCollapsed } from '../lib/sidebarAutoCollapse';

type SidebarCollapsePreference = 'auto' | 'user';

interface PortalShellState {
  isSidebarCollapsed: boolean;
  sidebarWidth: number;
  sidebarCollapsePreference: SidebarCollapsePreference;
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

function createDefaultShellState() {
  return {
    isSidebarCollapsed: resolveAutoSidebarCollapsed(),
    sidebarWidth: PORTAL_DEFAULT_SIDEBAR_WIDTH,
    sidebarCollapsePreference: 'auto' as SidebarCollapsePreference,
    hiddenSidebarItems: [] as PortalRouteKey[],
    themeMode: 'system' as PortalThemeMode,
    themeColor: 'lobster' as PortalThemeColor,
  };
}

type PersistedPortalShellState = Pick<
  PortalShellState,
  | 'isSidebarCollapsed'
  | 'sidebarWidth'
  | 'sidebarCollapsePreference'
  | 'hiddenSidebarItems'
  | 'themeMode'
  | 'themeColor'
>;

function resolveSidebarCollapsePreference(
  nextState: Partial<PersistedPortalShellState>,
  currentState: PortalShellState,
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

export const usePortalShellStore = create<PortalShellState>()(
  persist(
    (set) => ({
      ...createDefaultShellState(),
      toggleSidebar: () =>
        set((state) => ({
          isSidebarCollapsed: !state.isSidebarCollapsed,
          sidebarCollapsePreference: 'user',
        })),
      setSidebarCollapsed: (isSidebarCollapsed) =>
        set({ isSidebarCollapsed, sidebarCollapsePreference: 'user' }),
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
      resetShellPreferences: () => set(createDefaultShellState()),
    }),
    {
      name: PORTAL_PREFERENCES_STORAGE_KEY,
      partialize: (state): PersistedPortalShellState => ({
        isSidebarCollapsed: state.isSidebarCollapsed,
        sidebarWidth: clampSidebarWidth(state.sidebarWidth),
        sidebarCollapsePreference: state.sidebarCollapsePreference,
        hiddenSidebarItems: state.hiddenSidebarItems,
        themeMode: state.themeMode,
        themeColor: state.themeColor,
      }),
      merge: (persistedState, currentState) => {
        const nextState = (persistedState as Partial<PersistedPortalShellState>) || {};
        const nextThemeColor = PORTAL_THEME_COLOR_IDS.includes(
          nextState.themeColor ?? currentState.themeColor,
        )
          ? nextState.themeColor ?? currentState.themeColor
          : currentState.themeColor;
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
          themeColor: nextThemeColor,
        };
      },
    },
  ),
);
