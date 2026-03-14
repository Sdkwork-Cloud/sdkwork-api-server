import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import {
  clearPortalSessionToken,
  createPortalApiKey,
  getPortalWorkspace,
  listPortalApiKeys,
  PortalApiError,
} from 'sdkwork-api-portal-sdk';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord, PortalWorkspaceSummary } from 'sdkwork-api-types';

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
  const [status, setStatus] = useState('Loading portal workspace...');
  const [environment, setEnvironment] = useState('live');
  const [createdKey, setCreatedKey] = useState<CreatedGatewayApiKey | null>(null);
  const [submitting, setSubmitting] = useState(false);

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
        setStatus('Portal workspace loaded. Plaintext API keys are only shown at creation time.');
      } catch (error) {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          onNavigate('/portal/login');
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

  function handleLogout() {
    clearPortalSessionToken();
    onLogout();
  }

  return (
    <section className="portal-shell">
      <header className="portal-hero">
        <div>
          <p className="eyebrow">Developer Portal</p>
          <h2>{workspace.user.display_name || 'Workspace dashboard'}</h2>
          <p className="portal-hero-text">
            Self-service registration, portal login, browser access, and embedded Tauri access all
            converge on the same `/portal/*` API boundary.
          </p>
        </div>
        <div className="portal-actions">
          <button className="button-secondary" type="button" onClick={() => onNavigate('/admin')}>
            Open admin shell
          </button>
          <button className="button-primary" type="button" onClick={handleLogout}>
            Logout
          </button>
        </div>
      </header>

      <section className="panel panel-highlight">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">Workspace</p>
            <h2>Portal-owned tenant and project</h2>
          </div>
          <p className="status">{status}</p>
        </div>

        <div className="detail-grid">
          <article className="detail-card">
            <h3>User</h3>
            <ul className="compact-list">
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

          <article className="detail-card">
            <h3>Tenant</h3>
            <ul className="compact-list">
              <li>
                <strong>{workspace.tenant.name || 'Pending tenant'}</strong>
                <span>{workspace.tenant.id || 'Unavailable'}</span>
              </li>
            </ul>
          </article>

          <article className="detail-card">
            <h3>Project</h3>
            <ul className="compact-list">
              <li>
                <strong>{workspace.project.name || 'Pending project'}</strong>
                <span>{workspace.project.id || 'Unavailable'}</span>
              </li>
            </ul>
          </article>
        </div>
      </section>

      <section className="panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">Gateway Keys</p>
            <h2>Issue environment-specific API keys</h2>
          </div>
          <p className="status">
            Generated plaintext keys are write-only. The list below shows hashed registry records
            only.
          </p>
        </div>

        <div className="portal-grid">
          <form className="detail-card portal-key-form" onSubmit={handleCreateKey}>
            <h3>Create key</h3>
            <label className="field">
              <span>Environment</span>
              <select value={environment} onChange={(event) => setEnvironment(event.target.value)}>
                <option value="live">live</option>
                <option value="test">test</option>
                <option value="staging">staging</option>
              </select>
            </label>
            <button className="button-primary" type="submit" disabled={submitting}>
              {submitting ? 'Issuing key...' : 'Create API key'}
            </button>
            {createdKey && (
              <div className="generated-key">
                <span>Plaintext key</span>
                <code>{createdKey.plaintext}</code>
              </div>
            )}
          </form>

          <article className="detail-card">
            <h3>Issued keys</h3>
            <ul className="compact-list">
              {apiKeys.map((key) => (
                <li key={key.hashed_key}>
                  <strong>{key.environment}</strong>
                  <span>{key.hashed_key}</span>
                </li>
              ))}
              {!apiKeys.length && <li className="empty">No portal keys issued yet.</li>}
            </ul>
          </article>
        </div>
      </section>
    </section>
  );
}
