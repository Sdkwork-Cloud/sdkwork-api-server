import {
  ADMIN_LOCALE_OPTIONS,
  Select,
  useAdminI18n,
} from 'sdkwork-router-admin-commons';

import { useAdminAppStore, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { SettingsInfoCard, SettingsSection } from './Shared';

export function GeneralSettings() {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    sidebarWidth,
    themeColor,
    themeMode,
  } = useAdminAppStore();
  const { sessionUser, status } = useAdminWorkbench();
  const { locale, setLocale, t } = useAdminI18n();

  return (
    <div className="space-y-8">
      <div>
        <h2 className="mb-1 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
          {t('General')}
        </h2>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          {t('Operator workspace language, shell posture, and persistence defaults.')}
        </p>
      </div>

      <div className="space-y-6">
        <SettingsSection
          eyebrow={t('Language')}
          title={t('Language and locale')}
          description={t(
            'Choose the operator workspace language. Dates, numbers, and shared shell copy follow this setting immediately.',
          )}
        >
          <div className="grid gap-4 md:grid-cols-2">
            <label className="grid gap-2">
              <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                {t('Language')}
              </span>
              <Select
                value={locale}
                onChange={(event) => setLocale(event.target.value as typeof locale)}
              >
                {ADMIN_LOCALE_OPTIONS.map((option) => (
                  <option key={option.id} value={option.id}>
                    {t(option.label)}
                  </option>
                ))}
              </Select>
            </label>
          </div>
        </SettingsSection>

        <SettingsSection
          eyebrow={t('Workspace')}
          title={t('Workspace posture')}
          description={t('Current shell posture for the control plane workspace.')}
        >
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <SettingsInfoCard
              label={t('Operator')}
              value={sessionUser?.display_name ?? t('Control plane operator')}
              detail={sessionUser?.email ?? t(status)}
            />
            <SettingsInfoCard label={t('Theme mode')} value={t(themeMode)} />
            <SettingsInfoCard label={t('Theme color')} value={t(themeColor)} />
            <SettingsInfoCard
              label={t('Sidebar mode')}
              value={isSidebarCollapsed ? t('collapsed') : t('expanded')}
            />
            <SettingsInfoCard label={t('Sidebar width')} value={`${sidebarWidth}px`} />
            <SettingsInfoCard
              label={t('Hidden nav items')}
              value={hiddenSidebarItems.length}
            />
          </div>
        </SettingsSection>
      </div>
    </div>
  );
}
