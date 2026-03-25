import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { PortalI18nProvider } from 'sdkwork-router-portal-commons';

import { ThemeManager } from './ThemeManager';

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <PortalI18nProvider>
      <BrowserRouter basename="/portal">
        <ThemeManager />
        {children}
      </BrowserRouter>
    </PortalI18nProvider>
  );
}
