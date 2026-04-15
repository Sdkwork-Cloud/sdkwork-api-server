import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  InlineAlert,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

type UsersDetailPanelProps = {
  user: ManagedUser | null;
  userBilling:
    | AdminPageProps['snapshot']['billingSummary']['projects'][number]
    | null
    | undefined;
  userProject: AdminPageProps['snapshot']['projects'][number] | null | undefined;
  userTraffic:
    | AdminPageProps['snapshot']['usageSummary']['projects'][number]
    | null
    | undefined;
};

export function UsersDetailPanel({
  user,
  userBilling,
  userProject,
  userTraffic,
}: UsersDetailPanelProps) {
  const { formatNumber, t } = useAdminI18n();

  if (!user) {
    return null;
  }

  return (
    <div className="space-y-4">
      <div className="grid gap-3 text-sm text-[var(--sdk-color-text-secondary)] sm:grid-cols-3">
        <div className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] p-4">
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Requests')}
          </div>
          <div className="mt-2 text-xl font-semibold text-[var(--sdk-color-text-primary)]">
            {formatNumber(user.request_count)}
          </div>
        </div>
        <div className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] p-4">
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Tokens')}
          </div>
          <div className="mt-2 text-xl font-semibold text-[var(--sdk-color-text-primary)]">
            {formatNumber(user.total_tokens)}
          </div>
        </div>
        <div className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] p-4">
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Usage units')}
          </div>
          <div className="mt-2 text-xl font-semibold text-[var(--sdk-color-text-primary)]">
            {formatNumber(user.usage_units)}
          </div>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('User information')}</CardTitle>
          <CardDescription>
            {t('Basic identity and access information for the selected user.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 text-sm sm:grid-cols-2">
          <div className="space-y-1">
            <div className="text-[var(--sdk-color-text-muted)]">{t('Display Name')}</div>
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {user.display_name}
            </div>
          </div>
          <div className="space-y-1">
            <div className="text-[var(--sdk-color-text-muted)]">{t('Email')}</div>
            <div className="font-medium text-[var(--sdk-color-text-primary)]">{user.email}</div>
          </div>
          <div className="space-y-1">
            <div className="text-[var(--sdk-color-text-muted)]">{t('Role')}</div>
            <div>
              <StatusBadge
                showIcon
                status={user.role}
                variant={user.role === 'operator' ? 'success' : 'secondary'}
              />
            </div>
          </div>
          <div className="space-y-1">
            <div className="text-[var(--sdk-color-text-muted)]">{t('Status')}</div>
            <div>
              <StatusBadge
                showIcon
                status={user.active ? 'active' : 'disabled'}
                variant={user.active ? 'success' : 'danger'}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('Workspace')}</CardTitle>
          <CardDescription>
            {t('Workspace attribution and scope for the selected user.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 text-sm sm:grid-cols-2">
          <div className="space-y-1">
            <div className="text-[var(--sdk-color-text-muted)]">{t('Tenant')}</div>
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {user.workspace_tenant_id ?? t('control-plane')}
            </div>
          </div>
          <div className="space-y-1">
            <div className="text-[var(--sdk-color-text-muted)]">{t('Project')}</div>
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {user.workspace_project_id ?? t('shared operator context')}
            </div>
          </div>
        </CardContent>
      </Card>

      {user.role === 'portal' ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t('Project attribution')}</CardTitle>
            <CardDescription>
              {t('Live usage and billing posture for the selected portal user.')}
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 text-sm sm:grid-cols-2">
            <div className="space-y-1">
              <div className="text-[var(--sdk-color-text-muted)]">{t('Project name')}</div>
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {userProject?.name ?? t('Unassigned workspace')}
              </div>
            </div>
            <div className="space-y-1">
              <div className="text-[var(--sdk-color-text-muted)]">{t('Project requests')}</div>
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {formatNumber(userTraffic?.request_count ?? 0)}
              </div>
            </div>
            <div className="space-y-1">
              <div className="text-[var(--sdk-color-text-muted)]">{t('Used units')}</div>
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {formatNumber(userBilling?.used_units ?? 0)}
              </div>
            </div>
          </CardContent>
        </Card>
      ) : (
        <InlineAlert
          description={t(
            'The operator attached to the current session cannot be removed from this console. Any broader identity protection must be enforced by the backend.',
          )}
          showIcon
          title={t('Current session guard')}
          tone="info"
        />
      )}
    </div>
  );
}
