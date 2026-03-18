import { useLocation } from 'react-router-dom';

import { ADMIN_ROUTE_PATHS, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { AppHeader } from './AppHeader';
import { AppRoutes } from './AppRoutes';
import { Sidebar } from './Sidebar';

export function MainLayout() {
  const location = useLocation();
  const { authResolved, sessionUser } = useAdminWorkbench();
  const isAuthRoute = location.pathname === ADMIN_ROUTE_PATHS.LOGIN;

  if (!authResolved && !sessionUser) {
    return (
      <div className="adminx-auth-stage">
        <AppRoutes />
      </div>
    );
  }

  if (isAuthRoute || !sessionUser) {
    return (
      <div className="adminx-auth-stage">
        <AppRoutes />
      </div>
    );
  }

  return (
    <div className="adminx-shell">
      <div className="adminx-shell-background" />
      <AppHeader />
      <div className="adminx-shell-body">
        <Sidebar />
        <main className="adminx-shell-main">
          <AppRoutes />
        </main>
      </div>
    </div>
  );
}
