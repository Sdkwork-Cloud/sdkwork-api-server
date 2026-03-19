import { useEffect } from 'react';

import { AppProviders } from '../providers/AppProviders';
import { AppRoutes } from '../router/AppRoutes';
import { subscribeToPortalSessionExpiry, usePortalAuthStore } from '../../store/usePortalAuthStore';

function PortalProductRuntime() {
  const authenticated = usePortalAuthStore((state) => state.isAuthenticated);
  const isBootstrapping = usePortalAuthStore((state) => state.isBootstrapping);
  const bootStatus = usePortalAuthStore((state) => state.bootstrapStatus);
  const workspace = usePortalAuthStore((state) => state.workspace);
  const dashboardSnapshot = usePortalAuthStore((state) => state.dashboardSnapshot);
  const signIn = usePortalAuthStore((state) => state.signIn);
  const register = usePortalAuthStore((state) => state.register);
  const hydrate = usePortalAuthStore((state) => state.hydrate);

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  useEffect(() => subscribeToPortalSessionExpiry(), []);

  return (
    <AppRoutes
      authenticated={authenticated}
      bootStatus={bootStatus}
      bootstrapped={!isBootstrapping}
      dashboardSnapshot={dashboardSnapshot}
      register={register}
      signIn={signIn}
      workspace={workspace}
    />
  );
}

export function PortalProductApp() {
  return (
    <AppProviders>
      <PortalProductRuntime />
    </AppProviders>
  );
}
