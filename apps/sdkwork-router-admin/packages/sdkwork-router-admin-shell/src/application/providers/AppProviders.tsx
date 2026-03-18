import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { ThemeManager } from './ThemeManager';

function resolveBaseName(): string {
  const baseName = import.meta.env.BASE_URL ?? '/admin/';
  return baseName === '/' ? '/' : baseName.replace(/\/$/, '');
}

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <BrowserRouter basename={resolveBaseName()}>
      <ThemeManager />
      {children}
    </BrowserRouter>
  );
}
