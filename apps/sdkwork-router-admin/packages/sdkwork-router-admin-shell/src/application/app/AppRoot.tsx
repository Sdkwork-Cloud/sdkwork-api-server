import { AdminWorkbenchProvider } from 'sdkwork-router-admin-core';

import { MainLayout } from '../layouts/MainLayout';
import { AppProviders } from '../providers/AppProviders';

export function AppRoot() {
  return (
    <AppProviders>
      <AdminWorkbenchProvider>
        <MainLayout />
      </AdminWorkbenchProvider>
    </AppProviders>
  );
}
