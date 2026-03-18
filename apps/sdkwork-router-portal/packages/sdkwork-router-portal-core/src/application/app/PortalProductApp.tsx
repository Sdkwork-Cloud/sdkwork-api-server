import { useEffect, useState } from 'react';
import { formatUnits } from 'sdkwork-router-portal-commons';
import {
  clearPortalSessionToken,
  getPortalDashboard,
  getPortalMe,
  getPortalWorkspace,
  onPortalSessionExpired,
  portalErrorMessage,
  PortalApiError,
  readPortalSessionToken,
} from 'sdkwork-router-portal-portal-api';
import type {
  PortalAuthSession,
  PortalDashboardSummary,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';

import { AppProviders } from '../providers/AppProviders';
import { AppRoutes } from '../router/AppRoutes';

function buildWorkspacePulse(snapshot: PortalDashboardSummary | null): {
  detail: string;
  title: string;
  tone: 'accent' | 'positive' | 'warning';
} {
  if (!snapshot) {
    return {
      title: 'Restoring workspace pulse',
      detail: 'Traffic, credential, and quota posture are loading for the active project.',
      tone: 'accent',
    };
  }

  if (snapshot.billing_summary.exhausted) {
    return {
      title: 'Billing needs attention',
      detail: 'Visible quota is exhausted, so launch posture should be reviewed before traffic expands.',
      tone: 'warning',
    };
  }

  if (snapshot.api_key_count === 0) {
    return {
      title: 'Credential setup incomplete',
      detail: 'No project API key is visible yet. Issue a key before clients start sending traffic.',
      tone: 'warning',
    };
  }

  if (snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Ready for the first request',
      detail: 'The workspace is provisioned, but telemetry has not started flowing through the project yet.',
      tone: 'accent',
    };
  }

  return {
    title: 'Workspace is healthy',
    detail: `Telemetry, access, and visible quota are aligned across ${formatUnits(snapshot.usage_summary.total_requests)} recorded requests.`,
    tone: 'positive',
  };
}

function PortalProductRuntime() {
  const [authenticated, setAuthenticated] = useState(false);
  const [bootstrapped, setBootstrapped] = useState(false);
  const [bootStatus, setBootStatus] = useState('Checking for an existing portal session token.');
  const [workspace, setWorkspace] = useState<PortalWorkspaceSummary | null>(null);
  const [dashboardSnapshot, setDashboardSnapshot] = useState<PortalDashboardSummary | null>(null);
  const [pulseStatus, setPulseStatus] = useState('Workspace status will appear after sign-in.');

  useEffect(
    () =>
      onPortalSessionExpired(() => {
        clearPortalSessionToken();
        setAuthenticated(false);
        setWorkspace(null);
        setDashboardSnapshot(null);
        setBootStatus('Your portal session expired. Sign in again to continue.');
        setPulseStatus('Workspace status is unavailable until the next sign-in.');
        setBootstrapped(true);
      }),
    [],
  );

  useEffect(() => {
    const token = readPortalSessionToken();
    if (!token) {
      setAuthenticated(false);
      setWorkspace(null);
      setDashboardSnapshot(null);
      setPulseStatus('Sign in to load the current workspace status.');
      setBootstrapped(true);
      return;
    }

    let cancelled = false;
    setBootStatus('Refreshing workspace identity and navigation context.');

    void Promise.all([getPortalMe(token), getPortalWorkspace(token)])
      .then(([, nextWorkspace]) => {
        if (cancelled) {
          return;
        }

        setAuthenticated(true);
        setWorkspace(nextWorkspace);
        setBootstrapped(true);
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          setAuthenticated(false);
          setWorkspace(null);
          setBootstrapped(true);
          setBootStatus('The saved portal session is no longer valid. Please sign in again.');
          return;
        }

        setAuthenticated(false);
        setBootstrapped(true);
        setBootStatus(portalErrorMessage(error));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const token = readPortalSessionToken();
    if (!token || !authenticated) {
      setDashboardSnapshot(null);
      return;
    }

    let cancelled = false;
    setPulseStatus('Refreshing traffic, billing, and credential posture.');

    void getPortalDashboard(token)
      .then((snapshot) => {
        if (cancelled) {
          return;
        }

        setDashboardSnapshot(snapshot);
        setPulseStatus('Workspace status is synced with the latest dashboard snapshot.');
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          setAuthenticated(false);
          setDashboardSnapshot(null);
          setWorkspace(null);
          setPulseStatus('Your portal session expired. Sign in again to restore workspace status.');
          return;
        }

        setPulseStatus(portalErrorMessage(error));
      });

    return () => {
      cancelled = true;
    };
  }, [authenticated]);

  async function handleAuthenticated(session: PortalAuthSession) {
    setAuthenticated(true);
    setBootstrapped(true);
    setBootStatus('Hydrating workspace after sign-in.');
    setPulseStatus('Restoring workspace status after sign-in.');

    try {
      const nextWorkspace = await getPortalWorkspace(session.token);
      setWorkspace(nextWorkspace);
    } catch (error) {
      setBootStatus(portalErrorMessage(error));
    }
  }

  function handleLogout() {
    clearPortalSessionToken();
    setAuthenticated(false);
    setWorkspace(null);
    setDashboardSnapshot(null);
    setPulseStatus('Workspace status will appear after the next sign-in.');
  }

  const workspacePulse = buildWorkspacePulse(dashboardSnapshot);

  return (
    <AppRoutes
      authenticated={authenticated}
      bootStatus={bootStatus}
      bootstrapped={bootstrapped}
      dashboardSnapshot={dashboardSnapshot}
      onAuthenticated={handleAuthenticated}
      onLogout={handleLogout}
      pulseDetail={workspacePulse.detail}
      pulseStatus={pulseStatus}
      pulseTitle={workspacePulse.title}
      pulseTone={workspacePulse.tone}
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
