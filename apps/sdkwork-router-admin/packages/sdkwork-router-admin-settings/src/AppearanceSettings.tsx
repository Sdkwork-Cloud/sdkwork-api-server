import { Check, Laptop, Moon, Sparkles, Sun, SwatchBook } from 'lucide-react';

import { useAdminAppStore, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { SettingsSection } from './Shared';

const THEME_COLORS = [
  { id: 'tech-blue', label: 'tech-blue' },
  { id: 'lobster', label: 'lobster' },
  { id: 'green-tech', label: 'green-tech' },
  { id: 'zinc', label: 'zinc' },
  { id: 'violet', label: 'violet' },
  { id: 'rose', label: 'rose' },
] as const;

export function AppearanceSettings() {
  const { setThemeColor, setThemeMode, themeColor, themeMode } = useAdminAppStore();
  const { status } = useAdminWorkbench();

  return (
    <div className="admin-shell-settings-stack">
      <SettingsSection
        eyebrow="Appearance"
        title="theme mode"
        icon={<Sun className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-theme-mode-grid">
          <button
            type="button"
            className={themeMode === 'light' ? 'is-active' : ''}
            onClick={() => setThemeMode('light')}
          >
            <Sun />
            <strong>Light</strong>
            <span>Bright shell with frosted content panes.</span>
          </button>
          <button
            type="button"
            className={themeMode === 'dark' ? 'is-active' : ''}
            onClick={() => setThemeMode('dark')}
          >
            <Moon />
            <strong>Dark</strong>
            <span>Claw-style low-glare shell with higher contrast.</span>
          </button>
          <button
            type="button"
            className={themeMode === 'system' ? 'is-active' : ''}
            onClick={() => setThemeMode('system')}
          >
            <Laptop />
            <strong>System</strong>
            <span>Follow the device preference automatically.</span>
          </button>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Accent"
        title="theme color"
        icon={<SwatchBook className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-theme-color-grid">
          {THEME_COLORS.map((color) => (
            <button
              key={color.id}
              type="button"
              data-theme-color={color.id}
              className={themeColor === color.id ? 'is-active' : ''}
              onClick={() => setThemeColor(color.id)}
            >
              <div className="admin-shell-theme-color-copy">
                <strong>{color.label}</strong>
                <span>{color.id === 'tech-blue' ? 'default' : 'accent preset'}</span>
              </div>
              {themeColor === color.id ? <Check /> : null}
            </button>
          ))}
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Preview"
        title="shell visual snapshot"
        icon={<Sparkles className="admin-shell-settings-card-icon" />}
        className="admin-shell-settings-preview"
      >
        <div className="admin-shell-settings-preview-shell">
          <div className="admin-shell-settings-preview-sidebar">
            <span />
            <span />
            <span />
            <div className="admin-shell-settings-preview-sidebar-footer">
              <span className="admin-shell-settings-preview-sidebar-avatar" />
              <div className="admin-shell-settings-preview-sidebar-user">
                <span />
                <span />
              </div>
              <span className="admin-shell-settings-preview-sidebar-action" />
            </div>
          </div>
          <div className="admin-shell-settings-preview-canvas">
            <div className="admin-shell-settings-preview-header" />
            <div className="admin-shell-settings-preview-grid">
              <span />
              <span />
              <span />
              <span />
            </div>
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        eyebrow="Signature"
        title="theme signature"
        icon={<Sparkles className="admin-shell-settings-card-icon" />}
      >
        <div className="admin-shell-settings-kpi-grid">
          <div className="admin-shell-settings-kpi">
            <span>Live theme</span>
            <strong>{themeMode}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Accent preset</span>
            <strong>{themeColor}</strong>
          </div>
          <div className="admin-shell-settings-kpi">
            <span>Shell status</span>
            <strong>{status}</strong>
          </div>
        </div>
      </SettingsSection>
    </div>
  );
}
