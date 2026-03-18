import { useEffect } from 'react';

import { useAdminAppStore } from 'sdkwork-router-admin-core';

export function ThemeManager() {
  const { themeMode, themeColor, isSidebarCollapsed } = useAdminAppStore();

  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', themeColor);
    root.setAttribute('data-theme-mode', themeMode);
    root.setAttribute('data-sidebar-collapsed', String(isSidebarCollapsed));

    const applyMode = () => {
      const resolvedDark =
        themeMode === 'dark'
        || (themeMode === 'system'
          && window.matchMedia('(prefers-color-scheme: dark)').matches);

      root.classList.toggle('dark', resolvedDark);
      root.classList.toggle('theme-dark', resolvedDark);
      root.classList.toggle('theme-light', !resolvedDark);
      root.style.colorScheme = resolvedDark ? 'dark' : 'light';
    };

    applyMode();

    if (themeMode === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => applyMode();
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }
  }, [isSidebarCollapsed, themeColor, themeMode]);

  return null;
}
