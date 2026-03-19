import { useLocation } from 'react-router-dom';

import { isAdminAuthPath, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { AppHeader } from '../../components/AppHeader';
import { Sidebar } from '../../components/Sidebar';
import { AppRoutes } from '../router/AppRoutes';

export function MainLayout() {
  const location = useLocation();
  const { authResolved, sessionUser } = useAdminWorkbench();
  const isAuthRoute = isAdminAuthPath(location.pathname);

  if (!authResolved && !sessionUser) {
    return (
      <div className="adminx-auth-stage">
        <div className="adminx-auth-atmosphere" aria-hidden="true">
          <div className="adminx-auth-atmosphere-top" />
        </div>
        <main className="adminx-auth-stage-main">
          <AppRoutes />
        </main>
      </div>
    );
  }

  if (isAuthRoute || !sessionUser) {
    return (
      <div className="adminx-auth-stage">
        <div className="adminx-auth-atmosphere" aria-hidden="true">
          <div className="adminx-auth-atmosphere-top" />
        </div>
        <main className="adminx-auth-stage-main">
          <AppRoutes />
        </main>
      </div>
    );
  }

  return (
    <div className="adminx-shell">
      <div className="adminx-shell-atmosphere" aria-hidden="true">
        <div className="adminx-shell-atmosphere-top" />
        <div className="adminx-shell-atmosphere-left" />
      </div>
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
