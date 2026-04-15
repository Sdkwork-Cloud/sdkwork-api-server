import './admin.css';

import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { ChannelRegistryPage } from 'sdkwork-api-channel';
import { appName } from 'sdkwork-api-core';
import {
  AdminApiError,
  changeAdminPassword,
  clearAdminSessionToken,
  getAdminMe,
  loginAdminUser,
  persistAdminSessionToken,
} from 'sdkwork-api-admin-sdk';
import { RouteSimulationPage } from 'sdkwork-api-routing';
import { RuntimeStatusPage } from 'sdkwork-api-runtime';
import type { AdminAuthSession, AdminUserProfile } from 'sdkwork-api-types';
import { RequestExplorerPage } from 'sdkwork-api-usage';
import { WorkspaceDashboard } from 'sdkwork-api-workspace';

const adminSections = [
  { id: 'overview', label: 'Overview', detail: 'Platform posture and security boundary' },
  { id: 'security', label: 'Security', detail: 'Operator identity and password rotation' },
  { id: 'workspace', label: 'Workspace', detail: 'Tenants, projects, and gateway keys' },
  { id: 'catalog', label: 'Catalog', detail: 'Channels, providers, and models' },
  { id: 'routing', label: 'Routing', detail: 'Simulation and routing decision logs' },
  { id: 'traffic', label: 'Traffic', detail: 'Usage, cost, and quota exposure' },
  { id: 'runtime', label: 'Runtime', detail: 'Extension health and live reload control' },
] as const;

function adminErrorMessage(error: unknown): string {
  if (error instanceof AdminApiError) {
    return error.message;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return 'Admin request failed.';
}

function formatCreatedAt(timestampMs: number): string {
  return new Date(timestampMs).toLocaleDateString();
}

function resolveDevLoginEmailHint(): string {
  if (!import.meta.env.DEV) {
    return '';
  }

  return String(import.meta.env.VITE_ADMIN_LOGIN_HINT_EMAIL ?? '').trim();
}

function AdminLoginPage({
  onAuthenticated,
}: {
  onAuthenticated: (session: AdminAuthSession) => void;
}) {
  const devLoginEmailHint = resolveDevLoginEmailHint();
  const showDevAccessHint = import.meta.env.DEV;
  const [email, setEmail] = useState(devLoginEmailHint);
  const [password, setPassword] = useState('');
  const [status, setStatus] = useState(
    'Use the operator email and password provisioned by your runtime configuration.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Establishing operator session...');

    try {
      const session = await loginAdminUser({ email, password });
      persistAdminSessionToken(session.token);
      setStatus('Operator session established. Loading control plane...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(adminErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className="admin-auth-shell">
      <div className="admin-auth-layout">
        <article className="admin-auth-story">
          <p className="admin-kicker">Admin Operator App</p>
          <h1>Run the control plane from a dedicated operator surface.</h1>
          <p className="admin-auth-text">
            This application is not a themed view of the portal. It owns a separate operator
            session, separate password lifecycle, and separate access boundary for platform
            governance.
          </p>
          <div className="admin-auth-badges">
            <span>Control-plane telemetry</span>
            <span>Routing and runtime supervision</span>
            <span>SQLite quick-start ready</span>
          </div>
        </article>

        <article className="admin-auth-card">
          <p className="admin-kicker">Operator Sign-In</p>
          <h2>Authenticate the admin application</h2>
          <p className="admin-status">{status}</p>
          <form className="admin-form" onSubmit={handleSubmit}>
            <label className="admin-field">
              <span>Email</span>
              <input
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                type="email"
                autoComplete="email"
                required
              />
            </label>
            <label className="admin-field">
              <span>Password</span>
              <input
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                type="password"
                autoComplete="current-password"
                required
              />
            </label>
            <button className="admin-primary-button" type="submit" disabled={submitting}>
              {submitting ? 'Signing in...' : 'Open control plane'}
            </button>
          </form>
          {showDevAccessHint ? (
            <div className="admin-note-card">
              <span>Local development</span>
              <span>
                {devLoginEmailHint
                  ? `Local development uses identities from the active bootstrap profile. Email hint: ${devLoginEmailHint}. Enter the matching password from your runtime configuration.`
                  : 'Local development uses identities from the active bootstrap profile. Enter the operator email and password provisioned by your runtime configuration.'}
              </span>
            </div>
          ) : null}
        </article>
      </div>
    </section>
  );
}

function AdminShell({
  adminUser,
  onLogout,
}: {
  adminUser: AdminUserProfile;
  onLogout: () => void;
}) {
  const devLoginEmailHint = resolveDevLoginEmailHint();
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [status, setStatus] = useState('Operator control plane is live.');
  const [submitting, setSubmitting] = useState(false);

  async function handleChangePassword(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (newPassword !== confirmPassword) {
      setStatus('New password confirmation does not match.');
      return;
    }

    setSubmitting(true);
    setStatus('Rotating operator password...');

    try {
      await changeAdminPassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setStatus('Admin password updated. Future logins must use the rotated credential.');
    } catch (error) {
      setStatus(adminErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="admin-app">
      <aside className="admin-sidebar">
        <div className="admin-brand">
          <p className="admin-kicker">SDKWork Admin</p>
          <h1>{appName}</h1>
          <p>Operator-only command center for tenant governance, routing control, and runtime health.</p>
        </div>

        <nav className="admin-nav" aria-label="Admin sections">
          {adminSections.map((section) => (
            <a key={section.id} href={`#${section.id}`}>
              <strong>{section.label}</strong>
              <span>{section.detail}</span>
            </a>
          ))}
        </nav>

        <div className="admin-identity-card">
          <span className="admin-identity-name">{adminUser.display_name}</span>
          <span>{adminUser.email}</span>
          <span>Created {formatCreatedAt(adminUser.created_at_ms)}</span>
        </div>

        <button className="admin-secondary-button" type="button" onClick={onLogout}>
          Logout
        </button>
      </aside>

      <main className="admin-content">
        <header id="overview" className="admin-hero">
          <div>
            <p className="admin-kicker">Operator Control Plane</p>
            <h2>Observe platform posture and intervene without leaving the admin boundary.</h2>
            <p className="admin-hero-text">
              The admin product is intentionally dense, operational, and telemetry-first. It is
              built for platform owners managing tenants, routing strategy, cost, and runtime
              safety.
            </p>
          </div>
          <div className="admin-hero-pills">
            <span>Session: sdkwork.admin.session-token</span>
            <span>Security domain: operator-only</span>
            <span>Preferred local store: SQLite</span>
          </div>
        </header>

        <section className="admin-summary-grid">
          <article className="admin-summary-card">
            <span>Mode</span>
            <strong>Control plane</strong>
            <p>Built for governance, observability, and platform-level changes.</p>
          </article>
          <article className="admin-summary-card">
            <span>Identity</span>
            <strong>{adminUser.email}</strong>
            <p>Operator credentials are isolated from portal end-user accounts.</p>
          </article>
          <article className="admin-summary-card">
            <span>Primary workflow</span>
            <strong>Supervise and govern</strong>
            <p>Review workspace registry, route selection, billing posture, and runtime health.</p>
          </article>
        </section>

        <section id="security" className="admin-section">
          <div className="admin-section-heading">
            <div>
              <p className="admin-kicker">Security</p>
              <h3>Rotate the operator password inside the admin domain</h3>
            </div>
            <p className="admin-status">{status}</p>
          </div>

          <div className="admin-two-column">
            <form className="admin-card admin-form" onSubmit={handleChangePassword}>
              <h4>Change password</h4>
              <label className="admin-field">
                <span>Current password</span>
                <input
                  value={currentPassword}
                  onChange={(event) => setCurrentPassword(event.target.value)}
                  type="password"
                  autoComplete="current-password"
                  required
                />
              </label>
              <label className="admin-field">
                <span>New password</span>
                <input
                  value={newPassword}
                  onChange={(event) => setNewPassword(event.target.value)}
                  type="password"
                  autoComplete="new-password"
                  required
                />
              </label>
              <label className="admin-field">
                <span>Confirm new password</span>
                <input
                  value={confirmPassword}
                  onChange={(event) => setConfirmPassword(event.target.value)}
                  type="password"
                  autoComplete="new-password"
                  required
                />
              </label>
              <button className="admin-primary-button" type="submit" disabled={submitting}>
                {submitting ? 'Updating password...' : 'Rotate password'}
              </button>
            </form>

            <article className="admin-card">
              <h4>Bootstrap identity guidance</h4>
              <ul className="admin-facts">
                <li>
                  <strong>Identity source</strong>
                  <span>Use the active bootstrap profile for local identities.</span>
                </li>
                <li>
                  <strong>Email hint</strong>
                  <span>
                    {devLoginEmailHint
                      ? `Optional UI hint from VITE_ADMIN_LOGIN_HINT_EMAIL: ${devLoginEmailHint}`
                      : 'Set VITE_ADMIN_LOGIN_HINT_EMAIL to prefill the operator email without exposing a password.'}
                  </span>
                </li>
                <li>
                  <strong>Password source</strong>
                  <span>Use the password provisioned by your runtime configuration.</span>
                </li>
                <li>
                  <strong>Access boundary</strong>
                  <span>Admin APIs only</span>
                </li>
              </ul>
            </article>
          </div>
        </section>

        <section id="workspace" className="admin-section">
          <div className="admin-section-heading">
            <div>
              <p className="admin-kicker">Workspace</p>
              <h3>Tenant and project governance</h3>
            </div>
            <p className="admin-section-copy">
              Review the control-plane registry of tenants, projects, and issued gateway keys.
            </p>
          </div>
          <WorkspaceDashboard />
        </section>

        <section id="catalog" className="admin-section">
          <div className="admin-section-heading">
            <div>
              <p className="admin-kicker">Catalog</p>
              <h3>Provider, channel, and model exposure</h3>
            </div>
            <p className="admin-section-copy">
              Inspect the operator-owned channel mesh that shapes what the gateway can expose.
            </p>
          </div>
          <ChannelRegistryPage />
        </section>

        <section id="routing" className="admin-section">
          <div className="admin-section-heading">
            <div>
              <p className="admin-kicker">Routing</p>
              <h3>Decision simulation and recent route outcomes</h3>
            </div>
            <p className="admin-section-copy">
              Validate provider selection before traffic moves and inspect recent decision posture.
            </p>
          </div>
          <RouteSimulationPage />
        </section>

        <section id="traffic" className="admin-section">
          <div className="admin-section-heading">
            <div>
              <p className="admin-kicker">Traffic</p>
              <h3>Usage ledger and billing posture</h3>
            </div>
            <p className="admin-section-copy">
              Track request volume, booked amount, and quota exhaustion across the platform.
            </p>
          </div>
          <RequestExplorerPage />
        </section>

        <section id="runtime" className="admin-section">
          <div className="admin-section-heading">
            <div>
              <p className="admin-kicker">Runtime</p>
              <h3>Managed extension runtime health</h3>
            </div>
            <p className="admin-section-copy">
              Reload runtimes, inspect health snapshots, and watch runtime family coverage.
            </p>
          </div>
          <RuntimeStatusPage />
        </section>
      </main>
    </div>
  );
}

export function AdminApp() {
  const [adminUser, setAdminUser] = useState<AdminUserProfile | null>(null);
  const [bootstrapped, setBootstrapped] = useState(false);

  useEffect(() => {
    let cancelled = false;

    void getAdminMe()
      .then((user) => {
        if (cancelled) {
          return;
        }
        setAdminUser(user);
        setBootstrapped(true);
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }
        if (error instanceof AdminApiError && error.status === 401) {
          clearAdminSessionToken();
        }
        setAdminUser(null);
        setBootstrapped(true);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  function handleAuthenticated(session: AdminAuthSession) {
    setAdminUser(session.user);
    setBootstrapped(true);
  }

  function handleLogout() {
    clearAdminSessionToken();
    setAdminUser(null);
  }

  if (!bootstrapped) {
    return (
      <section className="admin-auth-shell">
        <div className="admin-auth-card admin-loading-card">
          <p className="admin-kicker">Admin Bootstrap</p>
          <h2>Restoring operator session</h2>
          <p className="admin-status">Checking for a persisted admin token.</p>
        </div>
      </section>
    );
  }

  if (!adminUser) {
    return <AdminLoginPage onAuthenticated={handleAuthenticated} />;
  }

  return <AdminShell adminUser={adminUser} onLogout={handleLogout} />;
}
