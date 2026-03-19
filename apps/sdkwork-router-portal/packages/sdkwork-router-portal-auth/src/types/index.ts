import type { ReactNode } from 'react';

export interface AuthShellStoryItem {
  title: string;
  detail: string;
}

export interface AuthShellPreviewItem {
  label: string;
  value: string;
  detail: string;
}

export type PortalAuthMode = 'login' | 'register' | 'forgot-password';

export interface PortalAuthPageProps {
  signIn: (credentials: { email: string; password: string }) => Promise<unknown>;
  register: (payload: { name: string; email: string; password: string }) => Promise<unknown>;
}

export interface AuthShellProps {
  eyebrow: string;
  title: string;
  detail: string;
  highlights: AuthShellStoryItem[];
  launchSteps: AuthShellStoryItem[];
  trustSignals: string[];
  status: string;
  previewTitle?: string;
  previewItems?: AuthShellPreviewItem[];
  children: ReactNode;
}
