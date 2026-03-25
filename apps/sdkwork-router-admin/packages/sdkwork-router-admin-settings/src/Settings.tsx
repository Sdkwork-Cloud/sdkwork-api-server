import { LayoutPanelLeft, Monitor, PanelsTopLeft, Search, ShieldCheck } from 'lucide-react';
import { motion } from 'motion/react';
import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';

import { Input, useAdminI18n } from 'sdkwork-router-admin-commons';

import { AppearanceSettings } from './AppearanceSettings';
import { GeneralSettings } from './GeneralSettings';
import { NavigationSettings } from './NavigationSettings';
import { SettingsNavButton } from './Shared';
import { WorkspaceSettings } from './WorkspaceSettings';

const SETTINGS_TABS = [
  {
    id: 'general',
    label: 'General',
    icon: ShieldCheck,
    keywords: 'workspace operator language',
  },
  {
    id: 'appearance',
    label: 'Appearance',
    icon: Monitor,
    keywords: 'theme mode theme color',
  },
  {
    id: 'navigation',
    label: 'Navigation',
    icon: LayoutPanelLeft,
    keywords: 'sidebar visibility routing',
  },
  {
    id: 'workspace',
    label: 'Workspace',
    icon: PanelsTopLeft,
    keywords: 'persistence canvas locale',
  },
] as const;

type SettingsTab = (typeof SETTINGS_TABS)[number]['id'];

function resolveTab(requestedTab: string | null): SettingsTab {
  if (SETTINGS_TABS.some((tab) => tab.id === requestedTab)) {
    return requestedTab as SettingsTab;
  }

  return 'general';
}

export function SettingsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [search, setSearch] = useState('');
  const { t } = useAdminI18n();
  const activeTab = resolveTab(searchParams.get('tab'));

  const filteredTabs = useMemo(
    () =>
      SETTINGS_TABS.filter((tab) => {
        const haystack = `${tab.label} ${tab.keywords}`.toLowerCase();
        return haystack.includes(search.toLowerCase());
      }),
    [search],
  );

  useEffect(() => {
    if (filteredTabs.length && !filteredTabs.some((tab) => tab.id === activeTab)) {
      const nextSearchParams = new URLSearchParams(searchParams);
      nextSearchParams.set('tab', filteredTabs[0].id);
      setSearchParams(nextSearchParams, { replace: true });
    }
  }, [activeTab, filteredTabs, searchParams, setSearchParams]);

  const renderActivePanel = () => {
    switch (activeTab) {
      case 'appearance':
        return <AppearanceSettings />;
      case 'navigation':
        return <NavigationSettings />;
      case 'workspace':
        return <WorkspaceSettings />;
      case 'general':
      default:
        return <GeneralSettings />;
    }
  };

  return (
    <div className="flex h-full bg-zinc-50/50 dark:bg-zinc-950/50">
      <div className="flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50/80 backdrop-blur-xl dark:border-zinc-800 dark:bg-zinc-900/80">
        <div className="p-6 pb-4">
          <h1 className="mb-6 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
            {t('Settings')}
          </h1>
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-400 dark:text-zinc-500" />
            <Input
              type="text"
              placeholder={t('Search settings')}
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              className="py-2.5 pl-9 pr-4 text-[13px]"
            />
          </div>
        </div>

        <nav className="scrollbar-hide flex-1 space-y-1.5 overflow-y-auto px-4 pb-6">
          {filteredTabs.length ? (
            filteredTabs.map((tab) => {
              const Icon = tab.icon;

              return (
                // data-settings-tab is applied by SettingsNavButton so the left-nav shell stays query-driven.
                <SettingsNavButton
                  key={tab.id}
                  tabId={tab.id}
                  active={activeTab === tab.id}
                  icon={Icon}
                  label={t(tab.label)}
                  onClick={() => {
                    const nextSearchParams = new URLSearchParams(searchParams);
                    nextSearchParams.set('tab', tab.id);
                    setSearchParams(nextSearchParams, { replace: true });
                  }}
                />
              );
            })
          ) : (
            <div className="px-3 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400">
              {t('No settings sections match the current filter.')}
            </div>
          )}
        </nav>
      </div>

      <div className="scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto">
        <div className="mx-auto w-full max-w-5xl p-8 md:p-12">
          <motion.div
            key={activeTab}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.2 }}
            className="w-full"
          >
            {renderActivePanel()}
          </motion.div>
        </div>
      </div>
    </div>
  );
}
