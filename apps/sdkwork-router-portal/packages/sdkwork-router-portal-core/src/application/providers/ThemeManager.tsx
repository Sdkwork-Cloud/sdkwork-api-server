import { useEffect } from 'react';

import { usePortalShellStore } from '../../store/usePortalShellStore';

export function ThemeManager() {
  const { isSidebarCollapsed, themeMode, themeColor } = usePortalShellStore();

  useEffect(() => {
    const root = document.documentElement;

    const applyTheme = () => {
      root.setAttribute('data-theme', themeColor);
      root.setAttribute('data-theme-mode', themeMode);
      root.setAttribute('data-sidebar-collapsed', String(isSidebarCollapsed));

      const resolvedDark =
        themeMode === 'dark'
        || (themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

      root.classList.toggle('dark', resolvedDark);
      root.classList.toggle('theme-dark', resolvedDark);
      root.classList.toggle('theme-light', !resolvedDark);
      root.style.colorScheme = resolvedDark ? 'dark' : 'light';
    };

    applyTheme();

    if (themeMode === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => applyTheme();
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }

    return undefined;
  }, [isSidebarCollapsed, themeColor, themeMode]);

  return null;
}
