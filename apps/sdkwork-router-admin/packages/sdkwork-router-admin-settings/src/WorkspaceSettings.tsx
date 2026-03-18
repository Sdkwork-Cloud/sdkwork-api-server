import { Monitor, PanelsTopLeft } from 'lucide-react';

import { useAdminAppStore } from 'sdkwork-router-admin-core';

import { SettingsSection } from './Shared';

export function WorkspaceSettings() {
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, themeColor, themeMode } =
    useAdminAppStore();

  return (
    <div className="admin-shell-settings-stack">
      <SettingsSection
        eyebrow="Workspace"
        title="shell posture"
        icon={<PanelsTopLeft className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-kpi-grid">
          <div className="admin-shell-settings-kpi">
            <span>Theme mode</span>
            <strong>{themeMode}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Theme color</span>
            <strong>{themeColor}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Sidebar width</span>
            <strong>{sidebarWidth}px</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Content region</span>
            <strong>right canvas</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Sidebar mode</span>
            <strong>{isSidebarCollapsed ? 'collapsed' : 'expanded'}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Hidden nav items</span>
            <strong>{hiddenSidebarItems.length}</strong>
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Continuity"
        title="workspace persistence"
        icon={<Monitor className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-copy">
          <p>
            Theme preferences, sidebar width, hidden entries, and collapse state are persisted so
            the control-plane workspace reopens with the same shell posture.
          </p>
          <p>
            The layout stays split into a claw-style left navigation rail and a single right content
            region, keeping product behavior and visual framing consistent.
          </p>
        </div>
      </SettingsSection>
    </div>
  );
}
