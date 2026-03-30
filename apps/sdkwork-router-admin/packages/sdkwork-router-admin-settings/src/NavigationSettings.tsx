import { Checkbox, Label, useAdminI18n } from 'sdkwork-router-admin-commons';
import { adminRoutes, useAdminAppStore } from 'sdkwork-router-admin-core';

import { SettingsInfoCard, SettingsSection } from './Shared';

export function NavigationSettings() {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    setSidebarCollapsed,
    sidebarWidth,
    toggleSidebarItem,
  } = useAdminAppStore();
  const { t } = useAdminI18n();

  const sidebarRoutes = adminRoutes.filter((route) => route.key !== 'settings');
  const visibleSidebarItems = sidebarRoutes.length - hiddenSidebarItems.length;

  return (
    <div className="space-y-8">
      <div>
        <h2 className="mb-1 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
          {t('Navigation')}
        </h2>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          {t('Sidebar visibility and left-rail posture stay aligned with claw-studio.')}
        </p>
      </div>

      <div className="space-y-6">
        <SettingsSection
          eyebrow={t('Behavior')}
          title={t('Sidebar behavior')}
          description={t('Keep the left rail expanded or collapse it into icon-only navigation.')}
        >
          <div className="grid gap-4 md:grid-cols-2">
            <Label className="flex cursor-pointer items-start gap-3 rounded-xl border border-zinc-200 bg-white p-4 transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:bg-zinc-800/60">
              <Checkbox
                checked={!isSidebarCollapsed}
                onChange={() => setSidebarCollapsed(false)}
              />
              <span className="grid gap-1">
                <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                  {t('Expanded sidebar')}
                </span>
                <span className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Keep labels visible across the full left rail.')}
                </span>
              </span>
            </Label>

            <Label className="flex cursor-pointer items-start gap-3 rounded-xl border border-zinc-200 bg-white p-4 transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:bg-zinc-800/60">
              <Checkbox
                checked={isSidebarCollapsed}
                onChange={() => setSidebarCollapsed(true)}
              />
              <span className="grid gap-1">
                <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                  {t('Collapsed sidebar')}
                </span>
                <span className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Reduce the rail to icon-only navigation without changing the canvas.')}
                </span>
              </span>
            </Label>
          </div>

          <div className="mt-4 grid gap-4 md:grid-cols-3">
            <SettingsInfoCard
              label={t('Visible routes')}
              value={visibleSidebarItems}
            />
            <SettingsInfoCard
              label={t('Sidebar mode')}
              value={isSidebarCollapsed ? t('collapsed') : t('expanded')}
            />
            <SettingsInfoCard label={t('Sidebar width')} value={`${sidebarWidth}px`} />
          </div>
        </SettingsSection>

        <SettingsSection
          eyebrow={t('Navigation')}
          title={t('sidebar visibility')}
          description={t('Show or hide modules while keeping the left navigation rail compact and stable.')}
        >
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            {sidebarRoutes.map((route) => (
              <Label
                key={route.key}
                className="flex cursor-pointer items-start gap-3 rounded-xl border border-zinc-200 p-3 transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
              >
                <Checkbox
                  checked={!hiddenSidebarItems.includes(route.key)}
                  onChange={() => toggleSidebarItem(route.key)}
                />
                <span className="grid gap-0.5">
                  <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                    {t(route.label)}
                  </span>
                  <span className="text-xs leading-6 text-zinc-500 dark:text-zinc-400">
                    {t(route.detail)}
                  </span>
                </span>
              </Label>
            ))}
          </div>
        </SettingsSection>
      </div>
    </div>
  );
}
