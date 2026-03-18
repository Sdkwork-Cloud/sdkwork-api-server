import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

export interface PortalUserPageProps {
  workspace: PortalWorkspaceSummary | null;
  onNavigate: (route: PortalRouteKey) => void;
}

export interface UserFactItem {
  id: string;
  title: string;
  value: string;
  detail: string;
}

export interface UserChecklistItem {
  id: string;
  title: string;
  detail: string;
  complete: boolean;
}

export interface UserRecoverySignal {
  id: string;
  title: string;
  detail: string;
}

export interface PasswordPolicyItem {
  id: string;
  label: string;
  met: boolean;
}

export interface PortalUserViewModel {
  profile_facts: UserFactItem[];
  personal_security_checklist: UserChecklistItem[];
  recovery_signals: UserRecoverySignal[];
  password_policy: PasswordPolicyItem[];
  can_submit_password: boolean;
}
