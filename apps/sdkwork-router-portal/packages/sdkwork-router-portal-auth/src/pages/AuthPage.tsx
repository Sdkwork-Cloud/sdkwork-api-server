import { ArrowRight, Chrome, Github, Lock, Mail, QrCode, Smartphone, User } from 'lucide-react';
import { useEffect, useState, type FormEvent } from 'react';
import { useLocation, useNavigate, useSearchParams } from 'react-router-dom';
import { Button, Label, LeadingIconInput, usePortalI18n } from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import type { PortalAuthMode, PortalAuthPageProps } from '../types';

const DEV_PORTAL_CREDENTIALS = {
  email: 'portal@sdkwork.local',
  password: 'ChangeMe123!',
};

function resolveAuthMode(pathname: string): PortalAuthMode {
  if (pathname === '/register') {
    return 'register';
  }

  if (pathname === '/forgot-password') {
    return 'forgot-password';
  }

  return 'login';
}

function resolveRedirectTarget(rawTarget: string | null) {
  if (!rawTarget || !rawTarget.startsWith('/')) {
    return '/dashboard';
  }

  if (
    rawTarget === '/auth' ||
    rawTarget === '/login' ||
    rawTarget === '/register' ||
    rawTarget === '/forgot-password'
  ) {
    return '/dashboard';
  }

  return rawTarget;
}

function authCopy(mode: PortalAuthMode) {
  switch (mode) {
    case 'register':
      return {
        title: 'Create account',
        description: 'Create your workspace access and continue into the portal shell.',
        submitLabel: 'Create account',
        alternateLabel: 'Sign in',
        alternatePath: '/login',
      };
    case 'forgot-password':
      return {
        title: 'Recover access',
        description:
          'Password reset links are not enabled for the current portal backend. Continue back to sign in with your workspace email.',
        submitLabel: 'Back to login',
        alternateLabel: 'Create account',
        alternatePath: '/register',
      };
    case 'login':
    default:
      return {
        title: 'Welcome back',
        description: 'Sign in to continue to your portal workspace.',
        submitLabel: 'Sign in',
        alternateLabel: 'Sign up',
        alternatePath: '/register',
      };
  }
}

export function AuthPage({ signIn, register }: PortalAuthPageProps) {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const mode = resolveAuthMode(location.pathname);
  const redirectTarget = resolveRedirectTarget(searchParams.get('redirect'));
  const copy = authCopy(mode);
  const [email, setEmail] = useState(
    import.meta.env.DEV && mode === 'login' ? DEV_PORTAL_CREDENTIALS.email : '',
  );
  const [password, setPassword] = useState(
    import.meta.env.DEV && mode === 'login' ? DEV_PORTAL_CREDENTIALS.password : '',
  );
  const [name, setName] = useState('');
  const [feedback, setFeedback] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const showDevCredentials = import.meta.env.DEV && mode === 'login';

  useEffect(() => {
    const nextEmail = searchParams.get('email');
    if (nextEmail) {
      setEmail(nextEmail);
    }
  }, [searchParams]);

  useEffect(() => {
    if (!showDevCredentials) {
      return;
    }

    setEmail((current) => current.trim() || DEV_PORTAL_CREDENTIALS.email);
    setPassword((current) => current || DEV_PORTAL_CREDENTIALS.password);
  }, [showDevCredentials]);

  function withRedirect(pathname: string, extra: Record<string, string> = {}) {
    const params = new URLSearchParams();

    if (redirectTarget !== '/dashboard') {
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

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (submitting) {
      return;
    }

    setSubmitting(true);
    setFeedback('');

    try {
      if (mode === 'login') {
        await signIn({ email, password });
        navigate(redirectTarget, { replace: true });
        return;
      }

      if (mode === 'register') {
        await register({ name, email, password });
        navigate(redirectTarget, { replace: true });
        return;
      }

      navigate(withRedirect('/login', { email }), { replace: true });
    } catch (error) {
      setFeedback(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 p-4 dark:bg-zinc-950 sm:p-8">
      <div className="flex w-full max-w-4xl flex-col overflow-hidden rounded-3xl bg-white shadow-2xl dark:bg-zinc-900 md:flex-row">
        <div className="relative flex w-full flex-col items-center justify-center overflow-hidden bg-zinc-900 p-8 text-white dark:bg-black md:w-2/5">
          <div className="absolute inset-0 bg-gradient-to-br from-primary-600/20 to-transparent" />
          <div className="relative z-10 flex flex-col items-center text-center">
            <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-2xl bg-primary-600 shadow-lg">
              <QrCode className="h-8 w-8 text-white" />
            </div>
            <h2 className="mb-2 text-2xl font-bold">{t('QR login')}</h2>
            <p className="mb-8 max-w-[200px] text-sm text-zinc-400">
              Open the desktop app and scan this code to continue without typing credentials.
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
              <h1 className="mb-2 text-3xl font-black tracking-tight text-zinc-900 dark:text-white">
                {t(copy.title)}
              </h1>
              <p className="text-zinc-500 dark:text-zinc-400">{t(copy.description)}</p>
            </div>

            <form className="space-y-5" onSubmit={handleSubmit}>
              {mode === 'register' ? (
                <div>
                  <Label className="mb-1.5 block text-zinc-700 dark:text-zinc-300">{t('Name')}</Label>
                  <LeadingIconInput
                    icon={<User className="h-5 w-5" />}
                    inputClassName="h-10 pr-3"
                    onChange={(event) => setName(event.target.value)}
                    placeholder="Workspace owner"
                    required
                    type="text"
                    value={name}
                  />
                </div>
              ) : null}

              <div>
                <Label className="mb-1.5 block text-zinc-700 dark:text-zinc-300">{t('Email')}</Label>
                <LeadingIconInput
                  autoComplete="email"
                  icon={<Mail className="h-5 w-5" />}
                  inputClassName="h-10 pr-3"
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder="name@example.com"
                  required
                  type="email"
                  value={email}
                />
              </div>

              {mode !== 'forgot-password' ? (
                <div>
                  <div className="mb-1.5 flex items-center justify-between">
                    <Label className="text-zinc-700 dark:text-zinc-300">{t('Password')}</Label>
                    {mode === 'login' ? (
                      <Button
                        className="h-auto rounded-none p-0 text-sm font-medium text-primary-600 shadow-none hover:bg-transparent hover:text-primary-500"
                        onClick={() => navigate(withRedirect('/forgot-password', { email }))}
                        type="button"
                        variant="ghost"
                      >
                        {t('Forgot password?')}
                      </Button>
                    ) : null}
                  </div>
                  <LeadingIconInput
                    autoComplete={mode === 'register' ? 'new-password' : 'current-password'}
                    icon={<Lock className="h-5 w-5" />}
                    inputClassName="h-10 pr-3"
                    onChange={(event) => setPassword(event.target.value)}
                    placeholder={mode === 'register' ? 'Create a password' : 'Enter your password'}
                    required
                    type="password"
                    value={password}
                  />
                </div>
              ) : null}

              <Button className="h-auto w-full rounded-xl py-3 font-bold shadow-sm" disabled={submitting} type="submit">
                {submitting ? t('Loading...') : t(copy.submitLabel)}
                <ArrowRight className="h-4 w-4" />
              </Button>
            </form>

            {showDevCredentials ? (
              <p className="mt-4 text-sm font-medium text-zinc-500 dark:text-zinc-400">
                {t('Local dev credentials are prefilled: {email} / {password}.', DEV_PORTAL_CREDENTIALS)}
              </p>
            ) : null}

            {feedback ? (
              <p className="mt-4 text-sm font-medium text-rose-500">{feedback}</p>
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
                    type="button"
                    variant="secondary"
                  >
                    <Github className="h-5 w-5" />
                    GitHub
                  </Button>
                  <Button
                    className="h-auto w-full rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-700 shadow-sm transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800"
                    type="button"
                    variant="secondary"
                  >
                    <Chrome className="h-5 w-5" />
                    Google
                  </Button>
                </div>
              </div>
            ) : null}

            <div className="mt-8 text-center text-sm text-zinc-600 dark:text-zinc-400">
              {mode === 'login' ? (
                <>
                  {t('No account?')}{' '}
                  <Button
                    className="h-auto rounded-none p-0 font-bold text-primary-600 shadow-none hover:bg-transparent hover:text-primary-500"
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
                    className="h-auto rounded-none p-0 font-bold text-primary-600 shadow-none hover:bg-transparent hover:text-primary-500"
                    onClick={() => navigate(withRedirect(copy.alternatePath))}
                    type="button"
                    variant="ghost"
                  >
                    {t(copy.alternateLabel)}
                  </Button>
                </>
              ) : (
                <Button
                  className="mx-auto h-auto gap-1 rounded-none p-0 font-bold text-primary-600 shadow-none hover:bg-transparent hover:text-primary-500"
                  onClick={() => navigate(withRedirect('/login', { email }))}
                  type="button"
                  variant="ghost"
                >
                  <ArrowRight className="h-4 w-4 rotate-180" />
                  {t('Back to login')}
                </Button>
              )}
            </div>

            {mode === 'forgot-password' ? (
              <div className="mt-4 text-center">
                <Button
                  className="h-auto rounded-none p-0 text-sm font-medium text-primary-600 shadow-none hover:bg-transparent hover:text-primary-500"
                  onClick={() => navigate(withRedirect('/register'))}
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
