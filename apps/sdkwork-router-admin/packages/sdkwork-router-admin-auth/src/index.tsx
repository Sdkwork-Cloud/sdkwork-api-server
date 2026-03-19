import { useEffect, useState, type FormEvent } from 'react';
import {
  ArrowRight,
  Chrome,
  Github,
  Lock,
  Mail,
  QrCode,
  Smartphone,
  User,
} from 'lucide-react';
import { Navigate, useLocation, useNavigate, useSearchParams } from 'react-router-dom';

import { ADMIN_ROUTE_PATHS } from 'sdkwork-router-admin-core';

type AuthMode = 'login' | 'register' | 'forgot';

const DEFAULT_LOGIN_STATUS = 'Authenticate to open the super-admin workspace.';

function resolveAuthMode(pathname: string): AuthMode {
  if (pathname === ADMIN_ROUTE_PATHS.REGISTER) {
    return 'register';
  }

  if (pathname === ADMIN_ROUTE_PATHS.FORGOT_PASSWORD) {
    return 'forgot';
  }

  return 'login';
}

function resolveRedirectTarget(rawTarget: string | null) {
  if (!rawTarget || !rawTarget.startsWith('/')) {
    return ADMIN_ROUTE_PATHS.OVERVIEW;
  }

  if (
    rawTarget === ADMIN_ROUTE_PATHS.ROOT ||
    rawTarget === ADMIN_ROUTE_PATHS.AUTH ||
    rawTarget === ADMIN_ROUTE_PATHS.LOGIN ||
    rawTarget === ADMIN_ROUTE_PATHS.REGISTER ||
    rawTarget === ADMIN_ROUTE_PATHS.FORGOT_PASSWORD
  ) {
    return ADMIN_ROUTE_PATHS.OVERVIEW;
  }

  return rawTarget;
}

function titleByMode(mode: AuthMode) {
  if (mode === 'register') {
    return 'Create an account';
  }

  if (mode === 'forgot') {
    return 'Reset password';
  }

  return 'Welcome back';
}

function descriptionByMode(mode: AuthMode) {
  if (mode === 'register') {
    return 'Join us to start building amazing things.';
  }

  if (mode === 'forgot') {
    return 'Enter your email to receive a reset link.';
  }

  return 'Enter your details to access your account.';
}

function submitLabelByMode(mode: AuthMode, loading: boolean) {
  if (mode === 'login') {
    return loading ? 'Signing In...' : 'Sign In';
  }

  if (mode === 'register') {
    return 'Sign Up';
  }

  return 'Send Reset Link';
}

export function AdminLoginPage({
  status,
  loading,
  isAuthenticated,
  onLogin,
}: {
  status: string;
  loading: boolean;
  isAuthenticated: boolean;
  onLogin: (input: { email: string; password: string }) => Promise<void>;
}) {
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const mode = resolveAuthMode(location.pathname);
  const redirectTarget = resolveRedirectTarget(searchParams.get('redirect'));
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [name, setName] = useState('');
  const [modeNotice, setModeNotice] = useState('');

  useEffect(() => {
    const nextEmail = searchParams.get('email');
    if (nextEmail) {
      setEmail(nextEmail);
    }
  }, [searchParams]);

  useEffect(() => {
    setModeNotice('');
  }, [mode]);

  const withRedirect = (pathname: string) => {
    const params = new URLSearchParams();
    if (redirectTarget !== ADMIN_ROUTE_PATHS.OVERVIEW) {
      params.set('redirect', redirectTarget);
    }

    if (email.trim()) {
      params.set('email', email.trim());
    }

    const queryString = params.toString();
    return queryString ? `${pathname}?${queryString}` : pathname;
  };

  const authNotice = (() => {
    if (modeNotice) {
      return modeNotice;
    }

    if (mode === 'login' && status && status !== DEFAULT_LOGIN_STATUS) {
      return status;
    }

    return '';
  })();

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (mode === 'login') {
      await onLogin({ email, password });
      return;
    }

    if (mode === 'register') {
      setModeNotice(
        `Operator account requests stay inside the control plane. Ask an existing admin to provision ${email.trim() || name.trim() || 'your'} access from Users.`,
      );
      return;
    }

    setModeNotice(
      'Password resets are managed by an authenticated admin in Users. Contact the platform owner to rotate your credential safely.',
    );
  }

  if (isAuthenticated) {
    return <Navigate to={redirectTarget} replace />;
  }

  return (
    <div className="adminx-auth-frame">
      <div className="adminx-auth-aside">
        <div className="adminx-auth-aside-overlay" />
        <div className="adminx-auth-aside-inner">
          <div className="adminx-auth-qr-badge">
            <QrCode className="adminx-auth-qr-badge-icon" />
          </div>
          <h2 className="adminx-auth-qr-title">Scan to Login</h2>
          <p className="adminx-auth-qr-desc">
            Use the SDKWork mobile app to scan the QR code for instant access.
          </p>

          <div className="adminx-auth-qr-card">
            <div className="adminx-auth-qr-placeholder" aria-hidden="true">
              <QrCode className="adminx-auth-qr-placeholder-icon" />
            </div>
          </div>

          <div className="adminx-auth-qr-footer">
            <Smartphone className="adminx-auth-qr-footer-icon" />
            <span>Open SDKWork App</span>
          </div>
        </div>
      </div>

      <div className="adminx-auth-content">
        <div className="adminx-auth-content-inner">
          <div className="adminx-auth-heading">
            <h1 className="adminx-auth-title">{titleByMode(mode)}</h1>
            <p className="adminx-auth-subtitle">{descriptionByMode(mode)}</p>
          </div>

          <form onSubmit={handleSubmit} className="adminx-auth-form">
            {mode === 'register' ? (
              <div className="adminx-auth-form-group">
                <label className="adminx-auth-label">Full Name</label>
                <div className="adminx-auth-input-shell">
                  <div className="adminx-auth-input-icon-wrap">
                    <User className="adminx-auth-input-icon" />
                  </div>
                  <input
                    type="text"
                    value={name}
                    onChange={(event) => setName(event.target.value)}
                    placeholder="John Doe"
                    autoComplete="name"
                    required
                  />
                </div>
              </div>
            ) : null}

            <div className="adminx-auth-form-group">
              <label className="adminx-auth-label">Email Address</label>
              <div className="adminx-auth-input-shell">
                <div className="adminx-auth-input-icon-wrap">
                  <Mail className="adminx-auth-input-icon" />
                </div>
                <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder="you@example.com"
                  autoComplete="email"
                  required
                />
              </div>
            </div>

            {mode !== 'forgot' ? (
              <div className="adminx-auth-form-group">
                <div className="adminx-auth-label-row">
                  <label className="adminx-auth-label">Password</label>
                  {mode === 'login' ? (
                    <button
                      type="button"
                      onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.FORGOT_PASSWORD))}
                      className="adminx-auth-link-button"
                    >
                      Forgot password
                    </button>
                  ) : null}
                </div>
                <div className="adminx-auth-input-shell">
                  <div className="adminx-auth-input-icon-wrap">
                    <Lock className="adminx-auth-input-icon" />
                  </div>
                  <input
                    type="password"
                    value={password}
                    onChange={(event) => setPassword(event.target.value)}
                    placeholder="Enter your password"
                    autoComplete={mode === 'login' ? 'current-password' : 'new-password'}
                    required
                  />
                </div>
              </div>
            ) : null}

            <button
              type="submit"
              disabled={mode === 'login' ? loading : false}
              className="adminx-auth-primary-button"
            >
              <span>{submitLabelByMode(mode, loading)}</span>
              <ArrowRight className="adminx-auth-primary-button-icon" />
            </button>
          </form>

          {authNotice ? <div className="adminx-auth-inline-notice">{authNotice}</div> : null}

          {mode === 'login' ? (
            <div className="adminx-auth-provider-section">
              <div className="adminx-auth-divider">
                <div className="adminx-auth-divider-line" />
                <span>Or continue with</span>
              </div>

              <div className="adminx-auth-provider-grid">
                <button
                  type="button"
                  className="adminx-auth-provider-button"
                  onClick={() =>
                    setModeNotice(
                      'Use the operator email and password flow for admin access. External SSO remains disabled in this workspace.',
                    )
                  }
                >
                  <Github className="adminx-auth-provider-icon" />
                  <span>GitHub</span>
                </button>
                <button
                  type="button"
                  className="adminx-auth-provider-button"
                  onClick={() =>
                    setModeNotice(
                      'Use the operator email and password flow for admin access. External SSO remains disabled in this workspace.',
                    )
                  }
                >
                  <Chrome className="adminx-auth-provider-icon" />
                  <span>Google</span>
                </button>
              </div>
            </div>
          ) : null}

          <div className="adminx-auth-mode-switch">
            {mode === 'login' ? (
              <>
                <span>Don't have an account?</span>{' '}
                <button
                  type="button"
                  onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.REGISTER))}
                  className="adminx-auth-mode-link"
                >
                  Sign Up
                </button>
              </>
            ) : mode === 'register' ? (
              <>
                <span>Already have an account?</span>{' '}
                <button
                  type="button"
                  onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.LOGIN))}
                  className="adminx-auth-mode-link"
                >
                  Sign In
                </button>
              </>
            ) : (
              <button
                type="button"
                onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.LOGIN))}
                className="adminx-auth-back-link"
              >
                <ArrowRight className="adminx-auth-back-link-icon" />
                <span>Back to login</span>
              </button>
            )}
          </div>

          {mode === 'forgot' ? (
            <div className="adminx-auth-secondary-switch">
              <button
                type="button"
                onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.REGISTER))}
                className="adminx-auth-mode-link"
              >
                Sign Up
              </button>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
