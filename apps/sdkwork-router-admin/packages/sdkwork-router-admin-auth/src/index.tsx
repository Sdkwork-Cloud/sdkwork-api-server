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
import { Button, Label, LeadingIconInput, useAdminI18n } from 'sdkwork-router-admin-commons';

import { ADMIN_ROUTE_PATHS } from 'sdkwork-router-admin-core';

type AuthMode = 'login' | 'register' | 'forgot';

const DEFAULT_LOGIN_STATUS = 'Authenticate to open the super-admin workspace.';
const DEV_ADMIN_CREDENTIALS = {
  email: 'admin@sdkwork.local',
  password: 'ChangeMe123!',
};

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
  const { t } = useAdminI18n();
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const mode = resolveAuthMode(location.pathname);
  const redirectTarget = resolveRedirectTarget(searchParams.get('redirect'));
  const [email, setEmail] = useState(
    import.meta.env.DEV && mode === 'login' ? DEV_ADMIN_CREDENTIALS.email : '',
  );
  const [password, setPassword] = useState(
    import.meta.env.DEV && mode === 'login' ? DEV_ADMIN_CREDENTIALS.password : '',
  );
  const [name, setName] = useState('');
  const [modeNotice, setModeNotice] = useState('');
  const showDevCredentials = import.meta.env.DEV && mode === 'login';

  useEffect(() => {
    const nextEmail = searchParams.get('email');
    if (nextEmail) {
      setEmail(nextEmail);
    }
  }, [searchParams]);

  useEffect(() => {
    setModeNotice('');
  }, [mode]);

  useEffect(() => {
    if (!showDevCredentials) {
      return;
    }

    setEmail((current) => current.trim() || DEV_ADMIN_CREDENTIALS.email);
    setPassword((current) => current || DEV_ADMIN_CREDENTIALS.password);
  }, [showDevCredentials]);

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
      return t(status);
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
        t(
          'Operator account requests stay inside the control plane. Ask an existing admin to provision {name} access from Users.',
          { name: email.trim() || name.trim() || t('your') },
        ),
      );
      return;
    }

    setModeNotice(
      t(
        'Password resets are managed by an authenticated admin in Users. Contact the platform owner to rotate your credential safely.',
      ),
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
          <h2 className="adminx-auth-qr-title">{t('Scan to Login')}</h2>
          <p className="adminx-auth-qr-desc">
            {t('Use the SDKWork mobile app to scan the QR code for instant access.')}
          </p>

          <div className="adminx-auth-qr-card">
            <div className="adminx-auth-qr-placeholder" aria-hidden="true">
              <QrCode className="adminx-auth-qr-placeholder-icon" />
            </div>
          </div>

          <div className="adminx-auth-qr-footer">
            <Smartphone className="adminx-auth-qr-footer-icon" />
            <span>{t('Open SDKWork App')}</span>
          </div>
        </div>
      </div>

      <div className="adminx-auth-content">
        <div className="adminx-auth-content-inner">
          <div className="adminx-auth-heading">
            <h1 className="adminx-auth-title">{t(titleByMode(mode))}</h1>
            <p className="adminx-auth-subtitle">{t(descriptionByMode(mode))}</p>
          </div>

          <form onSubmit={handleSubmit} className="adminx-auth-form">
            {mode === 'register' ? (
              <div className="adminx-auth-form-group">
                <Label className="adminx-auth-label">{t('Full Name')}</Label>
                <LeadingIconInput
                  className="adminx-auth-input-shell"
                  icon={<User className="adminx-auth-input-icon" />}
                  iconClassName="text-[#a1a1aa]"
                  inputClassName="adminx-auth-input-element"
                  type="text"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder={t('John Doe')}
                  autoComplete="name"
                  required
                />
              </div>
            ) : null}

            <div className="adminx-auth-form-group">
              <Label className="adminx-auth-label">{t('Email Address')}</Label>
              <LeadingIconInput
                className="adminx-auth-input-shell"
                icon={<Mail className="adminx-auth-input-icon" />}
                iconClassName="text-[#a1a1aa]"
                inputClassName="adminx-auth-input-element"
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder={t('you@example.com')}
                autoComplete="email"
                required
              />
            </div>

            {mode !== 'forgot' ? (
              <div className="adminx-auth-form-group">
                <div className="adminx-auth-label-row">
                  <Label className="adminx-auth-label">{t('Password')}</Label>
                  {mode === 'login' ? (
                    <Button
                      type="button"
                      onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.FORGOT_PASSWORD))}
                      className="adminx-auth-link-button h-auto rounded-none px-0 py-0 shadow-none hover:bg-transparent"
                      variant="ghost"
                    >
                      {t('Forgot password')}
                    </Button>
                  ) : null}
                </div>
                <LeadingIconInput
                  className="adminx-auth-input-shell"
                  icon={<Lock className="adminx-auth-input-icon" />}
                  iconClassName="text-[#a1a1aa]"
                  inputClassName="adminx-auth-input-element"
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  placeholder={t('Enter your password')}
                  autoComplete={mode === 'login' ? 'current-password' : 'new-password'}
                  required
                />
              </div>
            ) : null}

            <Button
              type="submit"
              disabled={mode === 'login' ? loading : false}
              className="adminx-auth-primary-button h-auto rounded-xl px-4 py-3"
              variant="default"
            >
              <span>{t(submitLabelByMode(mode, loading))}</span>
              <ArrowRight className="adminx-auth-primary-button-icon" />
            </Button>
          </form>

          {showDevCredentials ? (
            <div className="adminx-auth-inline-notice">
              {t('Local dev credentials are prefilled: {email} / {password}.', DEV_ADMIN_CREDENTIALS)}
            </div>
          ) : null}

          {authNotice ? <div className="adminx-auth-inline-notice">{authNotice}</div> : null}

          {mode === 'login' ? (
            <div className="adminx-auth-provider-section">
              <div className="adminx-auth-divider">
                <div className="adminx-auth-divider-line" />
                <span>{t('Or continue with')}</span>
              </div>

              <div className="adminx-auth-provider-grid">
                <Button
                  type="button"
                  className="adminx-auth-provider-button"
                  variant="secondary"
                  onClick={() =>
                    setModeNotice(
                      'Use the operator email and password flow for admin access. External SSO remains disabled in this workspace.',
                    )
                  }
                >
                  <Github className="adminx-auth-provider-icon" />
                  <span>{t('GitHub')}</span>
                </Button>
                <Button
                  type="button"
                  className="adminx-auth-provider-button"
                  variant="secondary"
                  onClick={() =>
                    setModeNotice(
                      'Use the operator email and password flow for admin access. External SSO remains disabled in this workspace.',
                    )
                  }
                >
                  <Chrome className="adminx-auth-provider-icon" />
                  <span>{t('Google')}</span>
                </Button>
              </div>
            </div>
          ) : null}

          <div className="adminx-auth-mode-switch">
            {mode === 'login' ? (
              <>
                <span>{t("Don't have an account?")}</span>{' '}
                <Button
                  type="button"
                  onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.REGISTER))}
                  className="adminx-auth-mode-link h-auto rounded-none px-0 py-0 shadow-none hover:bg-transparent"
                  variant="ghost"
                >
                  {t('Sign Up')}
                </Button>
              </>
            ) : mode === 'register' ? (
              <>
                <span>{t('Already have an account?')}</span>{' '}
                <Button
                  type="button"
                  onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.LOGIN))}
                  className="adminx-auth-mode-link h-auto rounded-none px-0 py-0 shadow-none hover:bg-transparent"
                  variant="ghost"
                >
                  {t('Sign In')}
                </Button>
              </>
            ) : (
              <Button
                type="button"
                onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.LOGIN))}
                className="adminx-auth-back-link h-auto rounded-none px-0 py-0 shadow-none hover:bg-transparent"
                variant="ghost"
              >
                <ArrowRight className="adminx-auth-back-link-icon" />
                <span>{t('Back to login')}</span>
              </Button>
            )}
          </div>

          {mode === 'forgot' ? (
            <div className="adminx-auth-secondary-switch">
              <Button
                type="button"
                onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.REGISTER))}
                className="adminx-auth-mode-link h-auto rounded-none px-0 py-0 shadow-none hover:bg-transparent"
                variant="ghost"
              >
                {t('Sign Up')}
              </Button>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
