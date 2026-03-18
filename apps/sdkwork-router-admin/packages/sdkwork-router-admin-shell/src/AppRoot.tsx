import { AdminWorkbenchProvider } from 'sdkwork-router-admin-core';

import { AppProviders } from './AppProviders';
import { MainLayout } from './MainLayout';

export function AppRoot() {
  return (
    <AppProviders>
      <AdminWorkbenchProvider>
        <MainLayout />
      </AdminWorkbenchProvider>
    </AppProviders>
  );
}
