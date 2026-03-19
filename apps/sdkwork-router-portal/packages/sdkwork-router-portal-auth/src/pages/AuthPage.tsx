import { ArrowRight, Chrome, Github, Lock, Mail, QrCode, Smartphone, User } from 'lucide-react';
import { useEffect, useState, type FormEvent } from 'react';
import { useLocation, useNavigate, useSearchParams } from 'react-router-dom';
import { Button, Input, Label } from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import type { PortalAuthMode, PortalAuthPageProps } from '../types';

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
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const mode = resolveAuthMode(location.pathname);
  const redirectTarget = resolveRedirectTarget(searchParams.get('redirect'));
  const copy = authCopy(mode);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [name, setName] = useState('');
  const [feedback, setFeedback] = useState('');
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    const nextEmail = searchParams.get('email');
    if (nextEmail) {
      setEmail(nextEmail);
    }
  }, [searchParams]);

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
            <h2 className="mb-2 text-2xl font-bold">QR login</h2>
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
              <span>Open app to scan</span>
            </div>
          </div>
        </div>

        <div className="w-full p-8 md:w-3/5 md:p-12">
          <div className="mx-auto max-w-md">
            <div className="mb-8">
              <h1 className="mb-2 text-3xl font-black tracking-tight text-zinc-900 dark:text-white">
                {copy.title}
              </h1>
              <p className="text-zinc-500 dark:text-zinc-400">{copy.description}</p>
            </div>

            <form className="space-y-5" onSubmit={handleSubmit}>
              {mode === 'register' ? (
                <div>
                  <Label className="mb-1.5 block text-zinc-700 dark:text-zinc-300">Name</Label>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                      <User className="h-5 w-5 text-zinc-400" />
                    </div>
                    <Input
                      className="h-10 rounded-xl border-zinc-200 bg-white py-2.5 pl-10 pr-3 shadow-sm dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-100"
                      onChange={(event) => setName(event.target.value)}
                      placeholder="Workspace owner"
                      required
                      type="text"
                      value={name}
                    />
                  </div>
                </div>
              ) : null}

              <div>
                <Label className="mb-1.5 block text-zinc-700 dark:text-zinc-300">Email</Label>
                <div className="relative">
                  <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                    <Mail className="h-5 w-5 text-zinc-400" />
                  </div>
                  <Input
                    autoComplete="email"
                    className="h-10 rounded-xl border-zinc-200 bg-white py-2.5 pl-10 pr-3 shadow-sm dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-100"
                    onChange={(event) => setEmail(event.target.value)}
                    placeholder="name@example.com"
                    required
                    type="email"
                    value={email}
                  />
                </div>
              </div>

              {mode !== 'forgot-password' ? (
                <div>
                  <div className="mb-1.5 flex items-center justify-between">
                    <Label className="text-zinc-700 dark:text-zinc-300">Password</Label>
                    {mode === 'login' ? (
                      <button
                        className="text-sm font-medium text-primary-600 transition-colors hover:text-primary-500"
                        onClick={() => navigate(withRedirect('/forgot-password', { email }))}
                        type="button"
                      >
                        Forgot password?
                      </button>
                    ) : null}
                  </div>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                      <Lock className="h-5 w-5 text-zinc-400" />
                    </div>
                    <Input
                      autoComplete={mode === 'register' ? 'new-password' : 'current-password'}
                      className="h-10 rounded-xl border-zinc-200 bg-white py-2.5 pl-10 pr-3 shadow-sm dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-100"
                      onChange={(event) => setPassword(event.target.value)}
                      placeholder={mode === 'register' ? 'Create a password' : 'Enter your password'}
                      required
                      type="password"
                      value={password}
                    />
                  </div>
                </div>
              ) : null}

              <Button className="h-auto w-full rounded-xl py-3 font-bold shadow-sm" disabled={submitting} type="submit">
                {submitting ? 'Loading...' : copy.submitLabel}
                <ArrowRight className="h-4 w-4" />
              </Button>
            </form>

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
                      Continue with
                    </span>
                  </div>
                </div>

                <div className="mt-6 grid grid-cols-2 gap-3">
                  <button className="flex w-full items-center justify-center gap-2 rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-700 shadow-sm transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800" type="button">
                    <Github className="h-5 w-5" />
                    GitHub
                  </button>
                  <button className="flex w-full items-center justify-center gap-2 rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-700 shadow-sm transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800" type="button">
                    <Chrome className="h-5 w-5" />
                    Google
                  </button>
                </div>
              </div>
            ) : null}

            <div className="mt-8 text-center text-sm text-zinc-600 dark:text-zinc-400">
              {mode === 'login' ? (
                <>
                  No account?{' '}
                  <button
                    className="font-bold text-primary-600 transition-colors hover:text-primary-500"
                    onClick={() => navigate(withRedirect(copy.alternatePath))}
                    type="button"
                  >
                    {copy.alternateLabel}
                  </button>
                </>
              ) : mode === 'register' ? (
                <>
                  Already have an account?{' '}
                  <button
                    className="font-bold text-primary-600 transition-colors hover:text-primary-500"
                    onClick={() => navigate(withRedirect(copy.alternatePath))}
                    type="button"
                  >
                    {copy.alternateLabel}
                  </button>
                </>
              ) : (
                <button
                  className="mx-auto flex items-center justify-center gap-1 font-bold text-primary-600 transition-colors hover:text-primary-500"
                  onClick={() => navigate(withRedirect('/login', { email }))}
                  type="button"
                >
                  <ArrowRight className="h-4 w-4 rotate-180" />
                  Back to login
                </button>
              )}
            </div>

            {mode === 'forgot-password' ? (
              <div className="mt-4 text-center">
                <button
                  className="text-sm font-medium text-primary-600 transition-colors hover:text-primary-500"
                  onClick={() => navigate(withRedirect('/register'))}
                  type="button"
                >
                  Create account
                </button>
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}
