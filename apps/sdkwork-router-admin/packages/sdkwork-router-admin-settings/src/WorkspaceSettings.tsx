import { useAdminI18n } from 'sdkwork-router-admin-commons';
import { useAdminAppStore } from 'sdkwork-router-admin-core';

import { SettingsInfoCard, SettingsSection } from './Shared';

export function WorkspaceSettings() {
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, themeColor, themeMode } =
    useAdminAppStore();
  const { t } = useAdminI18n();

  return (
    <div className="space-y-8">
      <div>
        <h2 className="mb-1 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
          {t('Workspace')}
        </h2>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          {t('Shell posture, persistence, and canvas continuity for the control plane workspace.')}
        </p>
      </div>

      <div className="space-y-6">
        <SettingsSection
          eyebrow={t('Workspace')}
          title={t('shell posture')}
          description={t('Keep the left navigation rail and the right canvas in a single consistent shell contract.')}
        >
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <SettingsInfoCard label={t('Theme mode')} value={t(themeMode)} />
            <SettingsInfoCard label={t('Theme color')} value={t(themeColor)} />
            <SettingsInfoCard label={t('Sidebar width')} value={`${sidebarWidth}px`} />
            <SettingsInfoCard
              label={t('Sidebar mode')}
              value={isSidebarCollapsed ? t('collapsed') : t('expanded')}
            />
            <SettingsInfoCard
              label={t('Hidden nav items')}
              value={hiddenSidebarItems.length}
            />
            <SettingsInfoCard label={t('Content region')} value={t('right canvas')} />
          </div>
        </SettingsSection>

        <SettingsSection
          eyebrow={t('Continuity')}
          title={t('workspace persistence')}
          description={t('Theme, sidebar, and hidden-navigation preferences reopen with the same shell posture on the next launch.')}
        >
          <div className="space-y-3 text-sm leading-7 text-zinc-500 dark:text-zinc-400">
            <p>
              {t(
                'Theme preferences, sidebar width, hidden entries, and collapse state are persisted so the control-plane workspace reopens with the same shell posture.',
              )}
            </p>
            <p>
              {t(
                'The layout stays split into a claw-style left navigation rail and a single right content region, keeping product behavior and visual framing consistent.',
              )}
            </p>
          </div>
        </SettingsSection>
      </div>
    </div>
  );
}
