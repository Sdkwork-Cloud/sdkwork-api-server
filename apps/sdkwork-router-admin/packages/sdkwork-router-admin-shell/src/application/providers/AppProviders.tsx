import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { AdminI18nProvider } from 'sdkwork-router-admin-commons';

import { ThemeManager } from './ThemeManager';

function resolveBaseName(): string {
  const baseName = import.meta.env.BASE_URL ?? '/admin/';
  return baseName === '/' ? '/' : baseName.replace(/\/$/, '');
}

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <AdminI18nProvider>
      <BrowserRouter basename={resolveBaseName()}>
        <ThemeManager />
        {children}
      </BrowserRouter>
    </AdminI18nProvider>
  );
}
