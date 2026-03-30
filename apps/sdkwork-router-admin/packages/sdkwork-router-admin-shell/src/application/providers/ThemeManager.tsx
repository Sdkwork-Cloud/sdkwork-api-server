import { useEffect } from 'react';

import { useAdminAppStore } from 'sdkwork-router-admin-core';

export function ThemeManager() {
  const { themeMode, themeColor } = useAdminAppStore();

  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', themeColor);

    const applyMode = () => {
      if (
        themeMode === 'dark'
        || (themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
      ) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }
    };

    applyMode();

    if (themeMode === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => applyMode();
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }

    return undefined;
  }, [themeColor, themeMode]);

  return null;
}
