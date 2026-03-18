import { LayoutPanelLeft, Monitor, PanelsTopLeft, ShieldCheck } from 'lucide-react';

import { adminRoutes, useAdminAppStore, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { SettingsSection } from './Shared';

export function GeneralSettings() {
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, themeColor, themeMode } =
    useAdminAppStore();
  const { sessionUser, status } = useAdminWorkbench();
  const visibleSidebarItems = adminRoutes.filter((route) => route.key !== 'settings').length
    - hiddenSidebarItems.length;

  return (
    <div className="admin-shell-settings-stack">
      <SettingsSection
        eyebrow="General"
        title="control plane settings center"
        icon={<ShieldCheck className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-copy">
          <p>
            This workspace keeps operator preferences, shell posture, and control plane continuity
            aligned with claw-studio while preserving router-admin workflows.
          </p>
          <p>
            The left rail remains the navigation source of truth and the right canvas remains the
            only content display region for every admin page.
          </p>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Workspace"
        title="live shell summary"
        icon={<PanelsTopLeft className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-kpi-grid">
          <div className="admin-shell-settings-kpi">
            <span>Operator</span>
            <strong>{sessionUser?.display_name ?? 'Control plane operator'}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Theme posture</span>
            <strong>{themeMode}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Theme color</span>
            <strong>{themeColor}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Shell status</span>
            <strong>{status}</strong>
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Navigation"
        title="sidebar and canvas posture"
        icon={<LayoutPanelLeft className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-kpi-grid">
          <div className="admin-shell-settings-kpi">
            <span>Sidebar state</span>
            <strong>{isSidebarCollapsed ? 'collapsed' : 'expanded'}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Sidebar width</span>
            <strong>{sidebarWidth}px</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Visible routes</span>
            <strong>{visibleSidebarItems}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Content region</span>
            <strong>right canvas</strong>
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Appearance"
        title="shell continuity"
        icon={<Monitor className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-copy">
          <p>
            Appearance, navigation, and workspace sections now live in a real settings center
            instead of a standalone preferences panel.
          </p>
          <p>
            Every shell preference persists so the control plane reopens with the same workspace and
            operator posture.
          </p>
        </div>
      </SettingsSection>
    </div>
  );
}
