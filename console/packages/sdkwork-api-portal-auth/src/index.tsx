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

interface PortalAuthShellProps {
  eyebrow: string;
  title: string;
  blurb: string;
  status: string;
  highlights: string[];
  children: ReactNode;
}

function PortalAuthShell({
  eyebrow,
  title,
  blurb,
  status,
  highlights,
  children,
}: PortalAuthShellProps) {
  return (
    <section className="portal-auth-shell">
      <div className="portal-auth-layout">
        <article className="portal-auth-hero">
          <p className="portal-kicker">{eyebrow}</p>
          <h1>{title}</h1>
          <p className="portal-auth-text">{blurb}</p>
          <ul className="portal-highlight-list">
            {highlights.map((highlight) => (
              <li key={highlight}>{highlight}</li>
            ))}
          </ul>
        </article>

        <article className="portal-auth-card">
          <p className="portal-status">{status}</p>
          {children}
        </article>
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

function resolveDevLoginEmailHint(): string {
  if (!import.meta.env.DEV) {
    return '';
  }

  return String(import.meta.env.VITE_PORTAL_LOGIN_HINT_EMAIL ?? '').trim();
}

export function PortalRegisterPage({ onAuthenticated, onNavigate }: PortalAuthPageProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [status, setStatus] = useState(
    'Create a self-service workspace and receive a portal session immediately.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Provisioning workspace and issuing your first session...');

    try {
      const session = await registerPortalUser({
        email,
        password,
        display_name: displayName,
      });
      persistPortalSessionToken(session.token);
      setStatus('Workspace created. Redirecting to the developer dashboard...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <PortalAuthShell
      eyebrow="Developer Portal"
      title="Create a workspace for your team or prototype."
      blurb="The portal is a developer product with its own account model, onboarding flow, and API key lifecycle. It does not reuse operator sessions from the admin console."
      status={status}
      highlights={[
        'Create a tenant and project automatically',
        'Issue environment-scoped API keys',
        'Keep end-user credentials isolated from admin operators',
      ]}
    >
      <h2>Register workspace</h2>
      <form className="portal-form" onSubmit={handleSubmit}>
        <label className="portal-field">
          <span>Display name</span>
          <input
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
            placeholder="Portal User"
            autoComplete="name"
            required
          />
        </label>
        <label className="portal-field">
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
        <label className="portal-field">
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
        <div className="portal-form-actions">
          <button className="portal-primary-button" type="submit" disabled={submitting}>
            {submitting ? 'Creating workspace...' : 'Create workspace'}
          </button>
          <button
            className="portal-secondary-button"
            type="button"
            onClick={() => onNavigate('/login')}
          >
            Already have an account
          </button>
        </div>
      </form>
    </PortalAuthShell>
  );
}

export function PortalLoginPage({ onAuthenticated, onNavigate }: PortalAuthPageProps) {
  const devLoginEmailHint = resolveDevLoginEmailHint();
  const showDevAccessHint = import.meta.env.DEV;
  const [email, setEmail] = useState(devLoginEmailHint);
  const [password, setPassword] = useState('');
  const [status, setStatus] = useState(
    'Sign in to inspect your workspace and issue environment-specific API keys.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Authenticating developer workspace session...');

    try {
      const session = await loginPortalUser({ email, password });
      persistPortalSessionToken(session.token);
      setStatus('Portal session restored. Redirecting to your workspace...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <PortalAuthShell
      eyebrow="Portal Sign-In"
      title="Open your developer workspace."
      blurb="Use the portal for onboarding, workspace inspection, API key issuance, and password rotation. Operator-only admin workflows stay outside this application."
      status={status}
      highlights={[
        'Dedicated portal account boundary',
        'Workspace overview and API key management',
        'Bootstrap-profile guidance for local development',
      ]}
    >
      <h2>Sign in</h2>
      <form className="portal-form" onSubmit={handleSubmit}>
        <label className="portal-field">
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
        <label className="portal-field">
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
        <div className="portal-form-actions">
          <button className="portal-primary-button" type="submit" disabled={submitting}>
            {submitting ? 'Signing in...' : 'Open workspace'}
          </button>
          <button
            className="portal-secondary-button"
            type="button"
            onClick={() => onNavigate('/register')}
          >
            Create account
          </button>
        </div>
        {showDevAccessHint ? (
          <div className="portal-note-card">
            <span>Local development</span>
            <span>
              {devLoginEmailHint
                ? `Local development uses identities from the active bootstrap profile. Email hint: ${devLoginEmailHint}. Enter the matching password from your runtime configuration.`
                : 'Local development uses identities from the active bootstrap profile. Enter the portal email and password provisioned by your runtime configuration.'}
            </span>
          </div>
        ) : null}
      </form>
    </PortalAuthShell>
  );
}
