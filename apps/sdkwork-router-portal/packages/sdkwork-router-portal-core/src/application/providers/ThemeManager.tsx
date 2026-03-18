import { useEffect } from 'react';

import { usePortalShellStore } from '../../store/usePortalShellStore';

export function ThemeManager() {
  const { themeMode, themeColor } = usePortalShellStore();

  useEffect(() => {
    const root = document.documentElement;

    const applyTheme = () => {
      root.setAttribute('data-theme', themeColor);

      if (
        themeMode === 'dark' ||
        (themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
      ) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }
    };

    applyTheme();

    if (themeMode === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => applyTheme();
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }

    return undefined;
  }, [themeColor, themeMode]);

  return null;
}
