import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import {
  changePortalPassword,
  clearPortalSessionToken,
  createPortalApiKey,
  getPortalWorkspace,
  listPortalApiKeys,
  PortalApiError,
} from 'sdkwork-api-portal-sdk';
import type {
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  PortalWorkspaceSummary,
} from 'sdkwork-api-types';

interface PortalDashboardPageProps {
  onLogout: () => void;
  onNavigate: (path: string) => void;
}

const emptyWorkspace: PortalWorkspaceSummary = {
  user: {
    id: '',
    email: '',
    display_name: '',
    workspace_tenant_id: '',
    workspace_project_id: '',
    active: false,
    created_at_ms: 0,
  },
  tenant: {
    id: '',
    name: '',
  },
  project: {
    tenant_id: '',
    id: '',
    name: '',
  },
};

function portalErrorMessage(error: unknown): string {
  if (error instanceof PortalApiError) {
    return error.message;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return 'Portal request failed.';
}

export function PortalDashboardPage({ onLogout, onNavigate }: PortalDashboardPageProps) {
  const [workspace, setWorkspace] = useState<PortalWorkspaceSummary>(emptyWorkspace);
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [status, setStatus] = useState('Loading workspace and API key registry...');
  const [environment, setEnvironment] = useState('live');
  const [createdKey, setCreatedKey] = useState<CreatedGatewayApiKey | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordSubmitting, setPasswordSubmitting] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadPortalSnapshot() {
      try {
        const [workspaceSnapshot, keySnapshot] = await Promise.all([
          getPortalWorkspace(),
          listPortalApiKeys(),
        ]);

        if (cancelled) {
          return;
        }

        setWorkspace(workspaceSnapshot);
        setApiKeys(keySnapshot);
        setStatus('Workspace ready. Plaintext API keys are only shown once at creation time.');
      } catch (error) {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          onNavigate('/login');
          return;
        }

        setStatus(portalErrorMessage(error));
      }
    }

    void loadPortalSnapshot();

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleCreateKey(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus(`Issuing a ${environment} gateway key...`);

    try {
      const key = await createPortalApiKey(environment);
      const nextKeys = await listPortalApiKeys();
      setCreatedKey(key);
      setApiKeys(nextKeys);
      setStatus(`Gateway key issued for ${environment}. Copy the plaintext value before leaving.`);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleChangePassword(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (newPassword !== confirmPassword) {
      setStatus('New password confirmation does not match.');
      return;
    }

    setPasswordSubmitting(true);
    setStatus('Updating portal password...');

    try {
      await changePortalPassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setStatus('Portal password updated. Use the new password on your next sign-in.');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setPasswordSubmitting(false);
    }
  }

  function handleLogout() {
    clearPortalSessionToken();
    onLogout();
  }

  return (
    <section className="portal-shell">
      <header className="portal-hero">
        <div>
          <p className="portal-kicker">Developer Workspace</p>
          <h1>{workspace.user.display_name || 'Portal dashboard'}</h1>
          <p className="portal-hero-text">
            This application is optimized for self-service onboarding and credential lifecycle
            management. It keeps developer workflows separate from admin operator tooling.
          </p>
        </div>
        <div className="portal-hero-actions">
          <span className="portal-chip">Session: sdkwork.portal.session-token</span>
          <span className="portal-chip">Workspace: self-service</span>
          <button className="portal-secondary-button" type="button" onClick={handleLogout}>
            Logout
          </button>
        </div>
      </header>

      <section className="portal-guide-grid">
        <article className="portal-guide-card">
          <span>1. Confirm workspace</span>
          <p>Review the tenant and project assigned to your portal identity.</p>
        </article>
        <article className="portal-guide-card">
          <span>2. Issue a key</span>
          <p>Create a `live`, `test`, or `staging` API key for your client integration.</p>
        </article>
        <article className="portal-guide-card">
          <span>3. Call the gateway</span>
          <p>Use the issued key against your gateway `/v1/*` endpoints from an app or script.</p>
        </article>
      </section>

      <section className="portal-panel">
        <div className="portal-panel-heading">
          <div>
            <p className="portal-kicker">Workspace Identity</p>
            <h2>Your tenant, project, and login boundary</h2>
          </div>
          <p className="portal-status">{status}</p>
        </div>

        <div className="portal-detail-grid">
          <article className="portal-card">
            <h3>User</h3>
            <ul className="portal-facts">
              <li>
                <strong>{workspace.user.display_name || 'Pending'}</strong>
                <span>{workspace.user.email || 'No email loaded yet'}</span>
              </li>
              <li>
                <strong>User ID</strong>
                <span>{workspace.user.id || 'Unavailable'}</span>
              </li>
            </ul>
          </article>

          <article className="portal-card">
            <h3>Tenant</h3>
            <ul className="portal-facts">
              <li>
                <strong>{workspace.tenant.name || 'Pending tenant'}</strong>
                <span>{workspace.tenant.id || 'Unavailable'}</span>
              </li>
            </ul>
          </article>

          <article className="portal-card">
            <h3>Project</h3>
            <ul className="portal-facts">
              <li>
                <strong>{workspace.project.name || 'Pending project'}</strong>
                <span>{workspace.project.id || 'Unavailable'}</span>
              </li>
            </ul>
          </article>

          <article className="portal-card portal-code-card">
            <h3>Client usage</h3>
            <pre>{`Base URL: /v1/*
Authorization: Bearer <plaintext API key>
Environment: ${environment}`}</pre>
          </article>
        </div>
      </section>

      <section className="portal-panel portal-panel-accent">
        <div className="portal-panel-heading">
          <div>
            <p className="portal-kicker">Gateway Keys</p>
            <h2>Issue environment-scoped API keys</h2>
          </div>
          <p className="portal-status">
            Plaintext values are write-only. Copy them at creation time and store them securely.
          </p>
        </div>

        <div className="portal-layout-grid">
          <form className="portal-card portal-form" onSubmit={handleCreateKey}>
            <h3>Create API key</h3>
            <label className="portal-field">
              <span>Environment</span>
              <select value={environment} onChange={(event) => setEnvironment(event.target.value)}>
                <option value="live">live</option>
                <option value="test">test</option>
                <option value="staging">staging</option>
              </select>
            </label>
            <button className="portal-primary-button" type="submit" disabled={submitting}>
              {submitting ? 'Issuing key...' : 'Create key'}
            </button>
            {createdKey && (
              <div className="portal-note-card">
                <span>Plaintext key</span>
                <code>{createdKey.plaintext}</code>
              </div>
            )}
          </form>

          <article className="portal-card">
            <h3>Issued keys</h3>
            <ul className="portal-facts">
              {apiKeys.map((key) => (
                <li key={key.hashed_key}>
                  <strong>{key.environment}</strong>
                  <span>{key.hashed_key}</span>
                </li>
              ))}
              {!apiKeys.length && <li className="portal-empty">No portal keys issued yet.</li>}
            </ul>
          </article>
        </div>
      </section>

      <section className="portal-panel">
        <div className="portal-panel-heading">
          <div>
            <p className="portal-kicker">Account Security</p>
            <h2>Rotate your portal password</h2>
          </div>
          <p className="portal-status">
            Portal credentials are isolated from operator accounts used in the admin application.
          </p>
        </div>

        <div className="portal-layout-grid">
          <form className="portal-card portal-form" onSubmit={handleChangePassword}>
            <h3>Change password</h3>
            <label className="portal-field">
              <span>Current password</span>
              <input
                value={currentPassword}
                onChange={(event) => setCurrentPassword(event.target.value)}
                type="password"
                autoComplete="current-password"
                required
              />
            </label>
            <label className="portal-field">
              <span>New password</span>
              <input
                value={newPassword}
                onChange={(event) => setNewPassword(event.target.value)}
                type="password"
                autoComplete="new-password"
                required
              />
            </label>
            <label className="portal-field">
              <span>Confirm new password</span>
              <input
                value={confirmPassword}
                onChange={(event) => setConfirmPassword(event.target.value)}
                type="password"
                autoComplete="new-password"
                required
              />
            </label>
            <button
              className="portal-primary-button"
              type="submit"
              disabled={passwordSubmitting}
            >
              {passwordSubmitting ? 'Updating password...' : 'Update password'}
            </button>
          </form>

          <article className="portal-card">
            <h3>Bootstrap identity guidance</h3>
            <ul className="portal-facts">
              <li>
                <strong>Identity source</strong>
                <span>Use the active bootstrap profile or register a portal account.</span>
              </li>
              <li>
                <strong>Password source</strong>
                <span>Use the password provisioned by your runtime configuration.</span>
              </li>
              <li>
                <strong>Access boundary</strong>
                <span>Portal APIs only</span>
              </li>
            </ul>
          </article>
        </div>
      </section>
    </section>
  );
}
