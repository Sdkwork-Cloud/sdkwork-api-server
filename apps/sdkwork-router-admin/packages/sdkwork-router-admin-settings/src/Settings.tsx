import {
  LayoutPanelLeft,
  Monitor,
  PanelsTopLeft,
  Search,
  ShieldCheck,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';

import { adminRoutes, useAdminAppStore, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { AppearanceSettings } from './AppearanceSettings';
import { GeneralSettings } from './GeneralSettings';
import { NavigationSettings } from './NavigationSettings';
import { SettingsNavButton } from './Shared';
import { WorkspaceSettings } from './WorkspaceSettings';

const SETTINGS_TABS = [
  {
    id: 'general',
    label: 'General',
    detail: 'control plane framing, operator continuity, and shell summary',
    icon: ShieldCheck,
  },
  {
    id: 'appearance',
    label: 'Appearance',
    detail: 'theme mode, accent color, and shell visual snapshot',
    icon: Monitor,
  },
  {
    id: 'navigation',
    label: 'Navigation',
    detail: 'sidebar visibility, collapse posture, and rail preview',
    icon: LayoutPanelLeft,
  },
  {
    id: 'workspace',
    label: 'Workspace',
    detail: 'content region rules, persistence, and shell posture',
    icon: PanelsTopLeft,
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
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, themeColor, themeMode } =
    useAdminAppStore();
  const { sessionUser, status } = useAdminWorkbench();
  const activeTab = resolveTab(searchParams.get('tab'));
  const activeSettingsTab = SETTINGS_TABS.find((tab) => tab.id === activeTab) ?? SETTINGS_TABS[0];
  const visibleRoutes = adminRoutes.filter(
    (route) => route.key !== 'settings' && !hiddenSidebarItems.includes(route.key),
  ).length;

  const filteredTabs = useMemo(
    () =>
      SETTINGS_TABS.filter((tab) => {
        const haystack = `${tab.label} ${tab.detail}`.toLowerCase();
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
    <div className="admin-shell-settings">
      <aside className="admin-shell-settings-nav">
        <div className="admin-shell-settings-nav-head">
          <span>Settings Center</span>
          <strong>Control plane workspace</strong>
          <p>
            Claw-studio-aligned settings center for shell appearance, navigation behavior, and
            workspace continuity.
          </p>
        </div>

        <div className="admin-shell-settings-nav-summary">
          <div className="admin-shell-settings-nav-summary-head">
            <span>Live shell</span>
            <strong>Claw-aligned posture</strong>
            <p>
              Theme, rail state, and the right content canvas stay synchronized with the live admin
              workspace.
            </p>
          </div>

          <div className="admin-shell-settings-nav-summary-grid">
            <div>
              <span>Theme</span>
              <strong>
                {themeMode} / {themeColor}
              </strong>
            </div>
            <div>
              <span>Rail</span>
              <strong>{isSidebarCollapsed ? 'collapsed' : `${sidebarWidth}px`}</strong>
            </div>
            <div>
              <span>Visible routes</span>
              <strong>{visibleRoutes}</strong>
            </div>
            <div>
              <span>Operator</span>
              <strong>{sessionUser?.display_name ?? 'Control plane operator'}</strong>
            </div>
          </div>

          <div className="admin-shell-settings-nav-summary-status" title={status}>
            {status}
          </div>

          <div className="admin-shell-settings-nav-shell-preview" aria-hidden="true">
            <div
              className={`admin-shell-settings-nav-shell-preview-rail ${
                isSidebarCollapsed ? 'is-collapsed' : ''
              }`.trim()}
            >
              <span />
              <span />
              <span />
              <div className="admin-shell-settings-nav-shell-preview-profile">
                <span className="admin-shell-settings-nav-shell-preview-avatar" />
                {!isSidebarCollapsed ? (
                  <div className="admin-shell-settings-nav-shell-preview-copy">
                    <span />
                    <span />
                  </div>
                ) : null}
              </div>
            </div>

            <div className="admin-shell-settings-nav-shell-preview-canvas">
              <span className="admin-shell-settings-nav-shell-preview-header" />
              <div className="admin-shell-settings-nav-shell-preview-grid">
                <span />
                <span />
                <span />
                <span />
              </div>
            </div>
          </div>
        </div>

        <label className="admin-shell-settings-search">
          <span>Search settings</span>
          <div className="admin-shell-settings-search-input">
            <Search />
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="appearance, sidebar, workspace"
            />
          </div>
        </label>

        {/* data-settings-tab is applied by SettingsNavButton to keep each tab addressable. */}
        <div className="admin-shell-settings-tabs">
          {filteredTabs.map((tab) => {
            const Icon = tab.icon;

            return (
              <SettingsNavButton
                key={tab.id}
                tabId={tab.id}
                active={activeTab === tab.id}
                icon={<Icon />}
                label={tab.label}
                detail={tab.detail}
                onClick={() => {
                  const nextSearchParams = new URLSearchParams(searchParams);
                  nextSearchParams.set('tab', tab.id);
                  setSearchParams(nextSearchParams, { replace: true });
                }}
              />
            );
          })}
          {!filteredTabs.length ? (
            <div className="admin-shell-settings-empty">
              No settings sections match the current filter.
            </div>
          ) : null}
        </div>
      </aside>

      <section className="admin-shell-settings-panel">
        <div className="admin-shell-settings-panel-stage">
          <div className="admin-shell-settings-panel-stage-head">
            <div>
              <span>{activeSettingsTab.label}</span>
              <strong>{activeSettingsTab.detail}</strong>
              <p>
                The settings stage keeps live theme, navigation posture, and the single right-side
                canvas aligned with the current admin shell.
              </p>
            </div>

            <div className="admin-shell-settings-panel-stage-pills">
              <span className="admin-shell-settings-panel-pill">{themeColor}</span>
              <span className="admin-shell-settings-panel-pill">
                {isSidebarCollapsed ? 'collapsed rail' : `${sidebarWidth}px rail`}
              </span>
            </div>
          </div>

          <div className="admin-shell-settings-stage-metrics">
            <div>
              <span>Stage</span>
              <strong>{activeSettingsTab.label}</strong>
            </div>
            <div>
              <span>Theme mode</span>
              <strong>{themeMode}</strong>
            </div>
            <div>
              <span>Rail posture</span>
              <strong>{isSidebarCollapsed ? 'collapsed' : 'expanded'}</strong>
            </div>
            <div>
              <span>Content surface</span>
              <strong>Right canvas</strong>
            </div>
          </div>

          <div className="admin-shell-settings-panel-stage-body">{renderActivePanel()}</div>
        </div>
      </section>
    </div>
  );
}
