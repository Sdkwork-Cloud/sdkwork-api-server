import { useState } from 'react';
import type { FormEvent, ReactNode } from 'react';
import {
  loginPortalUser,
  persistPortalSessionToken,
  PortalApiError,
  registerPortalUser,
} from 'sdkwork-api-portal-sdk';
import type { PortalAuthSession } from 'sdkwork-api-types';

interface PortalAuthPageProps {
  onAuthenticated: (session: PortalAuthSession) => void;
  onNavigate: (path: string) => void;
}

interface PortalAuthCardProps {
  eyebrow: string;
  title: string;
  blurb: string;
  status: string;
  children: ReactNode;
}

function PortalAuthCard({ eyebrow, title, blurb, status, children }: PortalAuthCardProps) {
  return (
    <section className="auth-shell">
      <div className="auth-card">
        <p className="eyebrow">{eyebrow}</p>
        <h2>{title}</h2>
        <p className="auth-blurb">{blurb}</p>
        <p className="status">{status}</p>
        {children}
      </div>
    </section>
  );
}

function portalErrorMessage(error: unknown): string {
  if (error instanceof PortalApiError) {
    return error.message;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return 'Portal request failed.';
}

export function PortalRegisterPage({ onAuthenticated, onNavigate }: PortalAuthPageProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [status, setStatus] = useState(
    'Create a self-service workspace to obtain gateway API keys.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Provisioning workspace and issuing your first portal session...');

    try {
      const session = await registerPortalUser({
        email,
        password,
        display_name: displayName,
      });
      persistPortalSessionToken(session.token);
      setStatus('Workspace ready. Redirecting to the dashboard...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <PortalAuthCard
      eyebrow="Public Portal"
      title="Register a developer workspace"
      blurb="This public surface is separate from operator-only admin APIs. Each account receives a dedicated tenant, project, and self-service key lifecycle."
      status={status}
    >
      <form className="auth-form" onSubmit={handleSubmit}>
        <label className="field">
          <span>Display name</span>
          <input
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
            placeholder="Portal User"
            autoComplete="name"
            required
          />
        </label>
        <label className="field">
          <span>Email</span>
          <input
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            placeholder="portal@example.com"
            type="email"
            autoComplete="email"
            required
          />
        </label>
        <label className="field">
          <span>Password</span>
          <input
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            placeholder="At least 8 characters"
            type="password"
            autoComplete="new-password"
            required
          />
        </label>
        <div className="auth-actions">
          <button className="button-primary" type="submit" disabled={submitting}>
            {submitting ? 'Creating workspace...' : 'Register'}
          </button>
          <button
            className="button-secondary"
            type="button"
            onClick={() => onNavigate('/portal/login')}
          >
            Existing account
          </button>
        </div>
      </form>
    </PortalAuthCard>
  );
}

export function PortalLoginPage({ onAuthenticated, onNavigate }: PortalAuthPageProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [status, setStatus] = useState(
    'Sign in to inspect your workspace and issue environment-specific gateway keys.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Authenticating portal session...');

    try {
      const session = await loginPortalUser({ email, password });
      persistPortalSessionToken(session.token);
      setStatus('Portal session restored. Redirecting to the dashboard...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <PortalAuthCard
      eyebrow="Portal Sign-In"
      title="Return to your API workspace"
      blurb="The browser and Tauri desktop shell use the same public portal boundary. Sign in once and manage keys from either host."
      status={status}
    >
      <form className="auth-form" onSubmit={handleSubmit}>
        <label className="field">
          <span>Email</span>
          <input
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            placeholder="portal@example.com"
            type="email"
            autoComplete="email"
            required
          />
        </label>
        <label className="field">
          <span>Password</span>
          <input
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            placeholder="Your portal password"
            type="password"
            autoComplete="current-password"
            required
          />
        </label>
        <div className="auth-actions">
          <button className="button-primary" type="submit" disabled={submitting}>
            {submitting ? 'Signing in...' : 'Login'}
          </button>
          <button
            className="button-secondary"
            type="button"
            onClick={() => onNavigate('/portal/register')}
          >
            Create account
          </button>
        </div>
      </form>
    </PortalAuthCard>
  );
}
