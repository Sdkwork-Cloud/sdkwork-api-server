import { formatDateTime } from 'sdkwork-router-portal-commons/format-core';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import type {
  PasswordPolicyItem,
  PortalUserViewModel,
  UserChecklistItem,
  UserFactItem,
  UserRecoverySignal,
} from '../types';

export function passwordsMatch(left: string, right: string): boolean {
  return left === right;
}

function hasUppercase(value: string): boolean {
  return /[A-Z]/.test(value);
}

function hasLowercase(value: string): boolean {
  return /[a-z]/.test(value);
}

function hasNumber(value: string): boolean {
  return /\d/.test(value);
}

export function buildPasswordPolicy(
  nextPassword: string,
  confirmPassword: string,
): PasswordPolicyItem[] {
  return [
    {
      id: 'length',
      label: 'At least 12 characters',
      met: nextPassword.length >= 12,
    },
    {
      id: 'mixed-case',
      label: 'Include uppercase and lowercase letters',
      met: hasUppercase(nextPassword) && hasLowercase(nextPassword),
    },
    {
      id: 'number',
      label: 'Include at least one number',
      met: hasNumber(nextPassword),
    },
    {
      id: 'confirm',
      label: 'Confirmation matches the new password',
      met: Boolean(nextPassword) && passwordsMatch(nextPassword, confirmPassword),
    },
  ];
}

function buildProfileFacts(workspace: PortalWorkspaceSummary | null): UserFactItem[] {
  if (!workspace) {
    return [
      {
        id: 'pending',
        title: 'Profile facts',
        value: 'Loading',
        detail: 'User profile facts will appear after workspace identity finishes loading.',
      },
    ];
  }

  return [
    {
      id: 'identity',
      title: 'Profile facts',
      value: workspace.user.display_name,
      detail: 'Personal profile remains distinct from financial account posture and operator tooling.',
    },
    {
      id: 'email',
      title: 'Sign-in email',
      value: workspace.user.email,
      detail: 'This is the identity anchor for personal access, password recovery, and self-service sign-in.',
    },
    {
      id: 'workspace',
      title: 'Workspace access',
      value: `${workspace.tenant.name} / ${workspace.project.name}`,
      detail: `User access was created ${formatDateTime(workspace.user.created_at_ms)} and is currently ${workspace.user.active ? 'active' : 'restricted'}.`,
    },
  ];
}

function buildPersonalSecurityChecklist(
  workspace: PortalWorkspaceSummary | null,
  passwordPolicy: PasswordPolicyItem[],
): UserChecklistItem[] {
  return [
    {
      id: 'active',
      title: 'User access is active',
      detail: workspace?.user.active
        ? 'This user can currently authenticate and manage personal profile actions.'
        : 'Restore access before expecting password rotation or self-service recovery to succeed.',
      complete: Boolean(workspace?.user.active),
    },
    {
      id: 'password',
      title: 'Password rotation meets the visible policy',
      detail: 'The password bar should make entropy and confirmation rules obvious before submission.',
      complete: passwordPolicy.every((item) => item.met),
    },
    {
      id: 'boundary',
      title: 'Personal identity stays separate from financial account posture',
      detail: 'Use the User module for profile and password work, and keep money, credits, and ledger posture in Account.',
      complete: true,
    },
  ];
}

function buildRecoverySignals(workspace: PortalWorkspaceSummary | null): UserRecoverySignal[] {
  return [
    {
      id: 'password-loss',
      title: 'Password loss should not erase workspace continuity',
      detail: 'Keep API keys and deployment secrets outside the browser so sign-in recovery never becomes a production outage.',
    },
    {
      id: 'user-risk',
      title: workspace?.user.active ? 'Personal access is currently healthy' : 'Personal access needs recovery',
      detail: 'User recovery should restore profile access first, then return to routing, keys, and financial review with confidence.',
    },
    {
      id: 'next-step',
      title: 'Password rotation should lead back into active workspace work',
      detail: 'After a trust change, validate routing posture, key ownership, and account runway before the next launch window.',
    },
  ];
}

export function buildPortalUserViewModel(
  workspace: PortalWorkspaceSummary | null,
  nextPassword: string,
  confirmPassword: string,
): PortalUserViewModel {
  const password_policy = buildPasswordPolicy(nextPassword, confirmPassword);

  return {
    profile_facts: buildProfileFacts(workspace),
    personal_security_checklist: buildPersonalSecurityChecklist(workspace, password_policy),
    recovery_signals: buildRecoverySignals(workspace),
    password_policy,
    can_submit_password: password_policy.every((item) => item.met),
  };
}
