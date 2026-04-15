import { ArrowRight, GitBranch, Globe, Lock, Mail, QrCode, Smartphone, User } from 'lucide-react';
import {
  useEffect,
  useState,
  type ComponentPropsWithoutRef,
  type ComponentType,
  type FormEvent,
  type ReactNode,
} from 'react';
import { Navigate, useLocation, useNavigate, useSearchParams } from 'react-router-dom';
import { Badge, Button, Input, Label } from '@sdkwork/ui-pc-react';

import { ADMIN_ROUTE_PATHS, useAdminI18n } from 'sdkwork-router-admin-core';

type AuthMode = 'login' | 'register' | 'forgot';

const DEFAULT_LOGIN_STATUS = 'Authenticate to open the super-admin workspace.';
const SSO_NOTICE =
  'Use the operator email and password flow for admin access. External SSO remains disabled in this workspace.';

function resolveDevLoginEmailHint() {
  if (!import.meta.env.DEV) {
    return '';
  }

  return String(import.meta.env.VITE_ADMIN_LOGIN_HINT_EMAIL ?? '').trim();
}

type AuthBadgeProps = {
  children?: ReactNode;
  className?: string;
  variant?: 'default' | 'secondary' | 'outline' | 'danger' | 'success' | 'warning';
};

const AuthBadge = Badge as unknown as ComponentType<AuthBadgeProps>;

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

function authCopy(mode: AuthMode) {
  switch (mode) {
    case 'register':
      return {
        title: 'Create operator access',
        description:
          'Request operator access and continue into the router control plane once an existing admin provisions your identity.',
        submitLabel: 'Request access',
        alternateLabel: 'Sign in',
        alternatePath: ADMIN_ROUTE_PATHS.LOGIN,
        modeLabel: 'Request access',
      };
    case 'forgot':
      return {
        title: 'Recover access',
        description:
          'Password reset links are not enabled for this workspace yet. Continue back to sign in with your operator email.',
        submitLabel: 'Back to login',
        alternateLabel: 'Create access',
        alternatePath: ADMIN_ROUTE_PATHS.REGISTER,
        modeLabel: 'Recovery',
      };
    case 'login':
    default:
      return {
        title: 'Welcome back',
        description: 'Sign in to continue to your router admin workspace.',
        submitLabel: 'Sign in',
        alternateLabel: 'Sign up',
        alternatePath: ADMIN_ROUTE_PATHS.REGISTER,
        modeLabel: 'Sign in',
      };
  }
}

function AuthTextInput({
  icon,
  inputClassName,
  style,
  type,
  ...props
}: Omit<ComponentPropsWithoutRef<'input'>, 'className'> & {
  icon: ReactNode;
  inputClassName?: string;
}) {
  return (
    <div className="relative block w-full">
      <span className="pointer-events-none absolute left-4 top-1/2 flex h-5 w-5 -translate-y-1/2 items-center justify-center text-zinc-400 dark:text-zinc-500">
        {icon}
      </span>
      <Input
        {...props}
        className={['h-10 pr-3', inputClassName].filter(Boolean).join(' ')}
        style={{ ...style, paddingLeft: '2.75rem' }}
        type={type ?? 'text'}
      />
    </div>
  );
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
  const copy = authCopy(mode);
  const devLoginEmailHint = resolveDevLoginEmailHint();
  const [email, setEmail] = useState(
    mode === 'login' ? devLoginEmailHint : '',
  );
  const [password, setPassword] = useState('');
  const [name, setName] = useState('');
  const [feedback, setFeedback] = useState('');
  const showDevAccessHint = import.meta.env.DEV && mode === 'login';

  useEffect(() => {
    const nextEmail = searchParams.get('email');
    if (nextEmail) {
      setEmail(nextEmail);
    }
  }, [searchParams]);

  useEffect(() => {
    setFeedback('');
  }, [mode]);

  useEffect(() => {
    if (!showDevAccessHint || !devLoginEmailHint) {
      return;
    }

    setEmail((current) => current.trim() || devLoginEmailHint);
  }, [devLoginEmailHint, showDevAccessHint]);

  function withRedirect(pathname: string, extra: Record<string, string> = {}) {
    const params = new URLSearchParams();

    if (redirectTarget !== ADMIN_ROUTE_PATHS.OVERVIEW) {
      params.set('redirect', redirectTarget);
    }

    Object.entries(extra).forEach(([key, value]) => {
      if (value) {
        params.set(key, value);
      }
    });

    const queryString = params.toString();
    return queryString ? `${pathname}?${queryString}` : pathname;
  }

  const visibleFeedback =
    feedback || (mode === 'login' && status && status !== DEFAULT_LOGIN_STATUS ? t(status) : '');

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (mode === 'login') {
      await onLogin({ email, password });
      return;
    }

    if (mode === 'register') {
      setFeedback(
        t(
          'Operator account requests stay inside the control plane. Ask an existing admin to provision {name} access from Users.',
          { name: name.trim() || email.trim() || t('your') },
        ),
      );
      return;
    }

    navigate(withRedirect(ADMIN_ROUTE_PATHS.LOGIN, { email }), { replace: true });
  }

  if (isAuthenticated) {
    return <Navigate to={redirectTarget} replace />;
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 p-4 dark:bg-zinc-950 sm:p-8">
      <div className="flex w-full max-w-4xl flex-col overflow-hidden rounded-3xl bg-white shadow-2xl dark:bg-zinc-900 md:flex-row">
        <div className="relative flex w-full flex-col items-center justify-center overflow-hidden bg-zinc-900 p-8 text-white dark:bg-black md:w-2/5">
          <div className="absolute inset-0 bg-gradient-to-br from-[var(--sdk-color-brand-primary)]/25 to-transparent" />
          <div className="relative z-10 flex flex-col items-center text-center">
            <AuthBadge
              className="mb-6 border-white/20 bg-white/10 text-white"
              variant="outline"
            >
              {t('Router Admin')}
            </AuthBadge>

            <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-2xl bg-[var(--sdk-color-brand-primary)] shadow-lg">
              <QrCode className="h-8 w-8 text-white" />
            </div>
            <h2 className="mb-2 text-2xl font-bold">{t('QR login')}</h2>
            <p className="mb-8 max-w-[220px] text-sm text-zinc-400">
              {t(
                'Open the SDKWork app and scan this code to continue without typing credentials.',
              )}
            </p>

            <div className="mb-6 rounded-2xl bg-white p-4 shadow-xl">
              <div className="flex h-48 w-48 items-center justify-center rounded-xl border-2 border-dashed border-zinc-300 bg-zinc-100">
                <QrCode className="h-24 w-24 text-zinc-400" />
              </div>
            </div>

            <div className="flex items-center gap-2 text-sm text-zinc-400">
              <Smartphone className="h-4 w-4" />
              <span>{t('Open app to scan')}</span>
            </div>
          </div>
        </div>

        <div className="w-full p-8 md:w-3/5 md:p-12">
          <div className="mx-auto max-w-md">
            <div className="mb-8">
              <div className="mb-3 flex items-center gap-3">
                <AuthBadge variant="secondary">
                  {t(mode === 'login' ? 'Operator session' : 'Workspace access')}
                </AuthBadge>
                <AuthBadge variant="outline">{t(copy.modeLabel)}</AuthBadge>
              </div>
              <h1 className="mb-2 text-3xl font-black tracking-tight text-zinc-900 dark:text-white">
                {t(copy.title)}
              </h1>
              <p className="text-zinc-500 dark:text-zinc-400">{t(copy.description)}</p>
            </div>

            <form className="space-y-5" onSubmit={handleSubmit}>
              {mode === 'register' ? (
                <div>
                  <Label className="mb-1.5 block text-zinc-700 dark:text-zinc-300">
                    {t('Name')}
                  </Label>
                  <AuthTextInput
                    autoComplete="name"
                    icon={<User className="h-5 w-5" />}
                    onChange={(event) => setName(event.target.value)}
                    placeholder={t('Workspace owner')}
                    required
                    type="text"
                    value={name}
                  />
                </div>
              ) : null}

              <div>
                <Label className="mb-1.5 block text-zinc-700 dark:text-zinc-300">
                  {t('Email')}
                </Label>
                <AuthTextInput
                  autoComplete="email"
                  icon={<Mail className="h-5 w-5" />}
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder={t('name@example.com')}
                  required
                  type="email"
                  value={email}
                />
              </div>

              {mode !== 'forgot' ? (
                <div>
                  <div className="mb-1.5 flex items-center justify-between">
                    <Label className="text-zinc-700 dark:text-zinc-300">{t('Password')}</Label>
                    {mode === 'login' ? (
                      <Button
                        className="h-auto rounded-none p-0 text-sm font-medium text-[var(--sdk-color-brand-primary)] shadow-none hover:bg-transparent hover:text-[var(--sdk-color-brand-primary-hover)]"
                        onClick={() =>
                          navigate(withRedirect(ADMIN_ROUTE_PATHS.FORGOT_PASSWORD, { email }))
                        }
                        type="button"
                        variant="ghost"
                      >
                        {t('Forgot password?')}
                      </Button>
                    ) : null}
                  </div>
                  <AuthTextInput
                    autoComplete={mode === 'register' ? 'new-password' : 'current-password'}
                    icon={<Lock className="h-5 w-5" />}
                    onChange={(event) => setPassword(event.target.value)}
                    placeholder={
                      mode === 'register' ? t('Create a password') : t('Enter your password')
                    }
                    required
                    type="password"
                    value={password}
                  />
                </div>
              ) : null}

              <Button
                className="h-auto w-full rounded-xl py-3 font-bold shadow-sm"
                loading={mode === 'login' ? loading : false}
                type="submit"
                variant="primary"
              >
                {t(mode === 'login' && loading ? 'Signing In...' : copy.submitLabel)}
                <ArrowRight className="h-4 w-4" />
              </Button>
            </form>

            {showDevAccessHint ? (
              <p className="mt-4 text-sm font-medium text-zinc-500 dark:text-zinc-400">
                {devLoginEmailHint
                  ? t(
                      'Local development uses identities from the active bootstrap profile. Email hint: {email}. Enter the matching password from your runtime configuration.',
                      { email: devLoginEmailHint },
                    )
                  : t(
                      'Local development uses identities from the active bootstrap profile. Enter the operator email and password provisioned by your runtime configuration.',
                    )}
              </p>
            ) : null}

            {visibleFeedback ? (
              <p
                className={[
                  'mt-4 text-sm font-medium',
                  mode === 'login' ? 'text-rose-500' : 'text-zinc-500 dark:text-zinc-400',
                ].join(' ')}
              >
                {visibleFeedback}
              </p>
            ) : null}

            {mode === 'login' ? (
              <div className="mt-8">
                <div className="relative">
                  <div className="absolute inset-0 flex items-center">
                    <div className="w-full border-t border-zinc-200 dark:border-zinc-800" />
                  </div>
                  <div className="relative flex justify-center text-sm">
                    <span className="bg-white px-2 text-zinc-500 dark:bg-zinc-900">
                      {t('Continue with')}
                    </span>
                  </div>
                </div>

                <div className="mt-6 grid grid-cols-2 gap-3">
                  <Button
                    className="h-auto w-full rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-700 shadow-sm transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800"
                    onClick={() => setFeedback(t(SSO_NOTICE))}
                    type="button"
                    variant="outline"
                  >
                    <GitBranch className="h-5 w-5" />
                    {t('GitHub')}
                  </Button>
                  <Button
                    className="h-auto w-full rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-700 shadow-sm transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800"
                    onClick={() => setFeedback(t(SSO_NOTICE))}
                    type="button"
                    variant="outline"
                  >
                    <Globe className="h-5 w-5" />
                    {t('Google')}
                  </Button>
                </div>
              </div>
            ) : null}

            <div className="mt-8 text-center text-sm text-zinc-600 dark:text-zinc-400">
              {mode === 'login' ? (
                <>
                  {t('No account?')}{' '}
                  <Button
                    className="h-auto rounded-none p-0 font-bold text-[var(--sdk-color-brand-primary)] shadow-none hover:bg-transparent hover:text-[var(--sdk-color-brand-primary-hover)]"
                    onClick={() => navigate(withRedirect(copy.alternatePath))}
                    type="button"
                    variant="ghost"
                  >
                    {t(copy.alternateLabel)}
                  </Button>
                </>
              ) : mode === 'register' ? (
                <>
                  {t('Already have an account?')}{' '}
                  <Button
                    className="h-auto rounded-none p-0 font-bold text-[var(--sdk-color-brand-primary)] shadow-none hover:bg-transparent hover:text-[var(--sdk-color-brand-primary-hover)]"
                    onClick={() => navigate(withRedirect(copy.alternatePath))}
                    type="button"
                    variant="ghost"
                  >
                    {t(copy.alternateLabel)}
                  </Button>
                </>
              ) : (
                <Button
                  className="mx-auto h-auto gap-1 rounded-none p-0 font-bold text-[var(--sdk-color-brand-primary)] shadow-none hover:bg-transparent hover:text-[var(--sdk-color-brand-primary-hover)]"
                  onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.LOGIN, { email }))}
                  type="button"
                  variant="ghost"
                >
                  <ArrowRight className="h-4 w-4 rotate-180" />
                  {t('Back to login')}
                </Button>
              )}
            </div>

            {mode === 'forgot' ? (
              <div className="mt-4 text-center">
                <Button
                  className="h-auto rounded-none p-0 text-sm font-medium text-[var(--sdk-color-brand-primary)] shadow-none hover:bg-transparent hover:text-[var(--sdk-color-brand-primary-hover)]"
                  onClick={() => navigate(withRedirect(ADMIN_ROUTE_PATHS.REGISTER))}
                  type="button"
                  variant="ghost"
                >
                  {t('Create account')}
                </Button>
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}
