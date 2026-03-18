import type { PortalThemeColor, PortalThemeMode } from 'sdkwork-router-portal-types';

export const PORTAL_PREFERENCES_STORAGE_KEY = 'sdkwork-router-portal.preferences.v1';

export const PORTAL_COLLAPSED_SIDEBAR_WIDTH = 76;
export const PORTAL_DEFAULT_SIDEBAR_WIDTH = 252;
export const PORTAL_MIN_SIDEBAR_WIDTH = 224;
export const PORTAL_MAX_SIDEBAR_WIDTH = 360;

export const PORTAL_THEME_MODE_OPTIONS: Array<{ id: PortalThemeMode; label: string }> = [
  { id: 'light', label: 'Light' },
  { id: 'dark', label: 'Dark' },
  { id: 'system', label: 'System' },
];

export const PORTAL_THEME_COLOR_OPTIONS: Array<{
  id: PortalThemeColor;
  label: string;
  previewClassName: string;
}> = [
  { id: 'tech-blue', label: 'Tech Blue', previewClassName: 'bg-sky-500' },
  { id: 'lobster', label: 'Lobster', previewClassName: 'bg-red-500' },
  { id: 'green-tech', label: 'Green Tech', previewClassName: 'bg-emerald-500' },
  { id: 'zinc', label: 'Zinc', previewClassName: 'bg-zinc-500' },
  { id: 'violet', label: 'Violet', previewClassName: 'bg-violet-500' },
  { id: 'rose', label: 'Rose', previewClassName: 'bg-rose-500' },
];

export function clampSidebarWidth(sidebarWidth: number): number {
  return Math.max(PORTAL_MIN_SIDEBAR_WIDTH, Math.min(PORTAL_MAX_SIDEBAR_WIDTH, sidebarWidth));
}
