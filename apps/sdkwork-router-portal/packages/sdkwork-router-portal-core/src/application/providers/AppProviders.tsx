import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { ThemeManager } from './ThemeManager';

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <BrowserRouter basename="/portal">
      <ThemeManager />
      {children}
    </BrowserRouter>
  );
}
