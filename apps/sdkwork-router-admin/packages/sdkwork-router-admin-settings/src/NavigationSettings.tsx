import { CheckSquare, LayoutPanelLeft } from 'lucide-react';

import { adminRoutes, useAdminAppStore } from 'sdkwork-router-admin-core';

import { SettingsSection } from './Shared';

export function NavigationSettings() {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    setSidebarCollapsed,
    sidebarWidth,
    toggleSidebarItem,
  } = useAdminAppStore();

  const sidebarRoutes = adminRoutes.filter((route) => route.key !== 'settings');
  const visibleSidebarItems = sidebarRoutes.length - hiddenSidebarItems.length;

  return (
    <div className="admin-shell-settings-stack">
      <SettingsSection
        eyebrow="Live rail"
        title="live rail posture"
        icon={<LayoutPanelLeft className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-kpi-grid">
          <div className="admin-shell-settings-kpi">
            <span>Visible routes</span>
            <strong>{visibleSidebarItems}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Rail state</span>
            <strong>{isSidebarCollapsed ? 'collapsed' : 'expanded'}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Rail width</span>
            <strong>{sidebarWidth}px</strong>
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Navigation"
        title="sidebar visibility"
        icon={<CheckSquare className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-sidebar-toggle-grid">
          {sidebarRoutes.map((route) => (
            <label key={route.key} className="admin-shell-sidebar-toggle">
              <input
                type="checkbox"
                checked={!hiddenSidebarItems.includes(route.key)}
                onChange={() => toggleSidebarItem(route.key)}
              />
              <div>
                <strong>{route.label}</strong>
                <span>{route.detail}</span>
              </div>
            </label>
          ))}
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Preview"
        title="sidebar preview"
        icon={<LayoutPanelLeft className="admin-shell-settings-card-icon" />}
        className="admin-shell-settings-preview"
      >
        <div className="admin-shell-sidebar-toggle-grid">
          <label className="admin-shell-sidebar-toggle">
            <input
              type="radio"
              checked={!isSidebarCollapsed}
              onChange={() => setSidebarCollapsed(false)}
            />
            <div>
              <strong>Expanded rail</strong>
              <span>Use the full claw-style left navigation rail with labels and badges.</span>
            </div>
          </label>
          <label className="admin-shell-sidebar-toggle">
            <input
              type="radio"
              checked={isSidebarCollapsed}
              onChange={() => setSidebarCollapsed(true)}
            />
            <div>
              <strong>Collapsed rail</strong>
              <span>Switch to the compact icon rail while keeping the right canvas untouched.</span>
            </div>
          </label>
        </div>

        <div className="admin-shell-settings-sidebar-preview">
          <div className={`admin-shell-settings-preview-rail ${isSidebarCollapsed ? 'is-collapsed' : ''}`}>
            <span />
            <span />
            <span />
            <span />
            <div className="admin-shell-settings-preview-sidebar-footer">
              <span className="admin-shell-settings-preview-sidebar-avatar" />
              {!isSidebarCollapsed ? (
                <>
                  <div className="admin-shell-settings-preview-sidebar-user">
                    <span />
                    <span />
                  </div>
                  <span className="admin-shell-settings-preview-sidebar-action" />
                </>
              ) : (
                <span className="admin-shell-settings-preview-sidebar-action" />
              )}
            </div>
          </div>
          <div className="admin-shell-settings-preview-body">
            <strong>{isSidebarCollapsed ? 'Collapsed' : 'Expanded'} sidebar</strong>
            <span>{visibleSidebarItems} visible routes in the live rail</span>
            <p>
              The live shell keeps the left rail synchronized with claw-studio while the right
              content canvas remains the only page-rendering surface.
            </p>
          </div>
        </div>
      </SettingsSection>
    </div>
  );
}
