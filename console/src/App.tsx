import './App.css';

import { useEffect, useState } from 'react';
import { ChannelRegistryPage } from 'sdkwork-api-channel';
import { appName } from 'sdkwork-api-core';
import { PortalLoginPage, PortalRegisterPage } from 'sdkwork-api-portal-auth';
import {
  clearPortalSessionToken,
  getPortalMe,
  PortalApiError,
  readPortalSessionToken,
} from 'sdkwork-api-portal-sdk';
import { PortalDashboardPage } from 'sdkwork-api-portal-user';
import { RouteSimulationPage } from 'sdkwork-api-routing';
import { RuntimeStatusPage } from 'sdkwork-api-runtime';
import type { PortalAuthSession, PortalUserProfile } from 'sdkwork-api-types';
import { RequestExplorerPage } from 'sdkwork-api-usage';
import { WorkspaceDashboard } from 'sdkwork-api-workspace';

type AppRoute = '/portal/register' | '/portal/login' | '/portal/dashboard' | '/admin';

function normalizeRoute(hash: string): AppRoute {
  const candidate = hash.replace(/^#/, '') || '/portal/dashboard';

  if (
    candidate === '/portal/register' ||
    candidate === '/portal/login' ||
    candidate === '/portal/dashboard' ||
    candidate === '/admin'
  ) {
    return candidate;
  }

  return '/portal/dashboard';
}

function writeRoute(route: AppRoute): void {
  window.location.hash = route;
}

function ShellNav({
  activeRoute,
  onNavigate,
  portalUser,
}: {
  activeRoute: AppRoute;
  onNavigate: (route: AppRoute) => void;
  portalUser: PortalUserProfile | null;
}) {
  return (
    <nav className="shell-nav">
      <button
        className={activeRoute.startsWith('/portal') ? 'nav-pill is-active' : 'nav-pill'}
        type="button"
        onClick={() => onNavigate(portalUser ? '/portal/dashboard' : '/portal/login')}
      >
        Portal
      </button>
      <button
        className={activeRoute === '/admin' ? 'nav-pill is-active' : 'nav-pill'}
        type="button"
        onClick={() => onNavigate('/admin')}
      >
        Admin
      </button>
    </nav>
  );
}

function AdminConsoleView({
  onNavigate,
  portalUser,
}: {
  onNavigate: (route: AppRoute) => void;
  portalUser: PortalUserProfile | null;
}) {
  return (
    <main className="app-shell">
      <header className="hero">
        <div className="hero-copy">
          <p className="eyebrow">SDKWork API Gateway Console</p>
          <h1>{appName}</h1>
          <p className="hero-text">
            Operate the gateway control plane, inspect routing and usage, and share the same
            browser-accessible console with the public self-service portal.
          </p>
          <div className="hero-cta-row">
            <button className="button-secondary" type="button" onClick={() => onNavigate('/portal/dashboard')}>
              {portalUser ? 'Back to portal dashboard' : 'Open public portal'}
            </button>
          </div>
        </div>
        <aside className="hero-aside">
          <span>Axum</span>
          <span>Portal + Admin</span>
          <span>Browser + Tauri</span>
          <span>OpenAI API</span>
        </aside>
      </header>

      <WorkspaceDashboard />
      <ChannelRegistryPage />
      <RouteSimulationPage />
      <RequestExplorerPage />
      <RuntimeStatusPage />
    </main>
  );
}

export function App() {
  const [route, setRoute] = useState<AppRoute>(() => normalizeRoute(window.location.hash));
  const [portalUser, setPortalUser] = useState<PortalUserProfile | null>(null);
  const [portalBootstrapped, setPortalBootstrapped] = useState(false);

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(normalizeRoute(window.location.hash));
    };

    window.addEventListener('hashchange', handleHashChange);
    if (!window.location.hash) {
      writeRoute('/portal/dashboard');
    }

    return () => {
      window.removeEventListener('hashchange', handleHashChange);
    };
  }, []);

  useEffect(() => {
    const token = readPortalSessionToken();
    if (!token) {
      setPortalUser(null);
      setPortalBootstrapped(true);
      if (route === '/portal/dashboard') {
        writeRoute('/portal/login');
      }
      return;
    }

    let cancelled = false;
    void getPortalMe(token)
      .then((user) => {
        if (cancelled) {
          return;
        }
        setPortalUser(user);
        setPortalBootstrapped(true);
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          setPortalUser(null);
          setPortalBootstrapped(true);
          if (route !== '/portal/register') {
            writeRoute('/portal/login');
          }
          return;
        }

        setPortalUser(null);
        setPortalBootstrapped(true);
      });

    return () => {
      cancelled = true;
    };
  }, [route]);

  function navigate(nextRoute: AppRoute) {
    if (normalizeRoute(window.location.hash) !== nextRoute) {
      writeRoute(nextRoute);
      return;
    }
    setRoute(nextRoute);
  }

  function handleAuthenticated(session: PortalAuthSession) {
    setPortalUser(session.user);
    navigate('/portal/dashboard');
  }

  function handleLogout() {
    setPortalUser(null);
    navigate('/portal/login');
  }

  const shellContent = (() => {
    if (route === '/admin') {
      return <AdminConsoleView onNavigate={navigate} portalUser={portalUser} />;
    }

    if (!portalBootstrapped) {
      return (
        <main className="app-shell">
          <section className="auth-shell">
            <div className="auth-card">
              <p className="eyebrow">Portal Bootstrap</p>
              <h2>Restoring browser and desktop session</h2>
              <p className="status">
                Checking whether a previously issued public portal token is available.
              </p>
            </div>
          </section>
        </main>
      );
    }

    if (route === '/portal/register') {
      return (
        <main className="app-shell">
          <PortalRegisterPage
            onAuthenticated={handleAuthenticated}
            onNavigate={(path) => navigate(normalizeRoute(`#${path}`))}
          />
        </main>
      );
    }

    if (route === '/portal/login') {
      return (
        <main className="app-shell">
          <PortalLoginPage
            onAuthenticated={handleAuthenticated}
            onNavigate={(path) => navigate(normalizeRoute(`#${path}`))}
          />
        </main>
      );
    }

    return (
      <main className="app-shell">
        <PortalDashboardPage
          onLogout={handleLogout}
          onNavigate={(path) => navigate(normalizeRoute(`#${path}`))}
        />
      </main>
    );
  })();

  return (
    <>
      <ShellNav activeRoute={route} onNavigate={navigate} portalUser={portalUser} />
      {shellContent}
    </>
  );
}
