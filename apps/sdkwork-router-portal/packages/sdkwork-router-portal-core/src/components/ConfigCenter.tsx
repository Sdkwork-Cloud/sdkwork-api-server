import { useEffect, useMemo, useState, type ReactNode } from 'react';
import {
  Check,
  Laptop,
  Moon,
  Palette,
  PanelLeft,
  RotateCcw,
  Search,
  SlidersHorizontal,
  Sun,
  UserRound,
  type LucideIcon,
} from 'lucide-react';
import { Checkbox, Dialog, DialogContent, DialogDescription, DialogTitle, cn } from 'sdkwork-router-portal-commons';
import type { PortalThemeColor, PortalThemeMode } from 'sdkwork-router-portal-types';

import {
  PORTAL_THEME_COLOR_OPTIONS,
  PORTAL_THEME_MODE_OPTIONS,
} from '../lib/portalPreferences';
import { portalRoutes } from '../routes';
import { usePortalAuthStore } from '../store/usePortalAuthStore';
import { usePortalShellStore } from '../store/usePortalShellStore';

type ConfigCenterSectionId = 'appearance' | 'navigation' | 'workspace';

const THEME_MODE_ICONS: Record<PortalThemeMode, LucideIcon> = {
  light: Sun,
  dark: Moon,
  system: Laptop,
};

const CONFIG_CENTER_SECTIONS: Array<{
  id: ConfigCenterSectionId;
  label: string;
  description: string;
  icon: LucideIcon;
}> = [
  {
    id: 'appearance',
    label: 'Appearance',
    description: 'Theme mode, Theme color, and Theme preview',
    icon: Palette,
  },
  {
    id: 'navigation',
    label: 'Navigation',
    description: 'Sidebar behavior and visible workspace modules',
    icon: PanelLeft,
  },
  {
    id: 'workspace',
    label: 'Workspace',
    description: 'Workspace profile and reset controls',
    icon: UserRound,
  },
];

function SettingsNavButton({
  active,
  icon: Icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: LucideIcon;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex w-full items-center gap-3 rounded-xl border px-3 py-2.5 text-[14px] font-medium transition-all duration-200',
        active
          ? 'border-zinc-200/50 bg-white text-primary-600 shadow-sm dark:border-zinc-700/50 dark:bg-zinc-800 dark:text-primary-400'
          : 'border-transparent text-zinc-600 hover:bg-zinc-200/50 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800/50 dark:hover:text-zinc-100',
      )}
      type="button"
    >
      <Icon
        className={cn(
          'h-4 w-4',
          active ? 'text-primary-500 dark:text-primary-400' : 'text-zinc-400 dark:text-zinc-500',
        )}
      />
      {label}
    </button>
  );
}

function SettingsPageHeader({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <div>
      <h2 className="mb-1 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
        {title}
      </h2>
      <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>
    </div>
  );
}

function SettingsSection({
  title,
  description,
  actions,
  children,
}: {
  title: string;
  description?: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="overflow-hidden rounded-[1.5rem] border border-zinc-200/80 bg-white shadow-sm transition-shadow duration-300 hover:shadow-md dark:border-zinc-800/80 dark:bg-zinc-900">
      <div className="border-b border-zinc-100 bg-zinc-50/50 px-6 py-5 dark:border-zinc-800/80 dark:bg-zinc-900/50">
        <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div className="space-y-1">
            <h3 className="text-[15px] font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
              {title}
            </h3>
            {description ? (
              <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>
            ) : null}
          </div>
          {actions ? <div className="flex flex-wrap gap-3">{actions}</div> : null}
        </div>
      </div>
      <div className="p-6">{children}</div>
    </div>
  );
}

function ThemeOptionButton({
  active,
  icon: Icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: LucideIcon;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex flex-col items-center justify-center gap-3 rounded-xl border-2 p-4 transition-all ${
        active
          ? 'border-primary-500 bg-primary-50/50 dark:bg-primary-500/10'
          : 'border-zinc-200 bg-white hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700'
      }`}
      type="button"
    >
      <Icon
        className={`h-6 w-6 ${
          active ? 'text-primary-500 dark:text-primary-400' : 'text-zinc-500 dark:text-zinc-400'
        }`}
      />
      <span
        className={`text-sm font-medium ${
          active
            ? 'text-primary-700 dark:text-primary-300'
            : 'text-zinc-700 dark:text-zinc-300'
        }`}
      >
        {label}
      </span>
    </button>
  );
}

function ThemeColorButton({
  active,
  color,
  label,
  onClick,
  previewClassName,
}: {
  active: boolean;
  color: PortalThemeColor;
  label: string;
  onClick: () => void;
  previewClassName: string;
}) {
  return (
    <button onClick={onClick} className="group relative flex flex-col items-center gap-2" type="button">
      <div
        className={`flex h-10 w-10 items-center justify-center rounded-full ${previewClassName} shadow-sm ring-2 ring-offset-2 transition-all dark:ring-offset-zinc-950 ${
          active
            ? 'scale-110 ring-zinc-900 dark:ring-zinc-100'
            : 'ring-transparent hover:scale-105'
        }`}
      >
        {active ? <Check className="h-5 w-5 text-white" /> : null}
      </div>
      <span
        className={`text-xs font-medium ${
          active
            ? 'text-zinc-900 dark:text-zinc-100'
            : 'text-zinc-500 group-hover:text-zinc-700 dark:text-zinc-400 dark:group-hover:text-zinc-300'
        }`}
      >
        {label}
      </span>
      <span className="sr-only">{color}</span>
    </button>
  );
}

function SettingsStatCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
        {label}
      </div>
      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">{value}</div>
      <div className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{detail}</div>
    </div>
  );
}

export function ConfigCenter({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const workspace = usePortalAuthStore((state) => state.workspace);
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    resetShellPreferences,
    setSidebarCollapsed,
    sidebarWidth,
    themeColor,
    themeMode,
    toggleSidebarItem,
    setThemeColor,
    setThemeMode,
  } = usePortalShellStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [activeSection, setActiveSection] = useState<ConfigCenterSectionId>('appearance');

  const filteredSections = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    if (!normalizedQuery) {
      return CONFIG_CENTER_SECTIONS;
    }

    return CONFIG_CENTER_SECTIONS.filter((section) =>
      `${section.id} ${section.label} ${section.description}`
        .toLowerCase()
        .includes(normalizedQuery),
    );
  }, [searchQuery]);

  useEffect(() => {
    if (!filteredSections.some((section) => section.id === activeSection)) {
      setActiveSection(filteredSections[0]?.id ?? 'appearance');
    }
  }, [activeSection, filteredSections]);

  const visibleRoutes = portalRoutes.filter((route) => !hiddenSidebarItems.includes(route.key));
  const previewRoutes = visibleRoutes.slice(0, 4);
  const workspaceName = workspace?.project.name ?? 'Portal Workspace';
  const workspaceEmail = workspace?.user.email ?? 'Awaiting workspace session';
  const tenantName = workspace?.tenant.name ?? 'Portal tenant';
  const operatorName = workspace?.user.display_name ?? 'Portal operator';
  const currentThemeMode = PORTAL_THEME_MODE_OPTIONS.find((option) => option.id === themeMode)?.label ?? themeMode;
  const currentThemeColor = PORTAL_THEME_COLOR_OPTIONS.find((option) => option.id === themeColor)?.label ?? themeColor;

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="max-h-[calc(100dvh-2rem)] w-[min(1320px,calc(100%-2rem))] overflow-hidden border-zinc-200/80 bg-white/95 p-0 shadow-[0_32px_80px_rgba(15,23,42,0.18)] dark:border-zinc-800/80 dark:bg-zinc-950/92">
        <div className="sr-only">
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>Workspace shell settings for theme, navigation, and portal ownership.</DialogDescription>
        </div>

        <div className="flex h-full min-h-[760px] bg-zinc-50/50 dark:bg-zinc-950/50">
          <div className="flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50/80 backdrop-blur-xl dark:border-zinc-800 dark:bg-zinc-900/80">
            <div className="p-6 pb-4">
              <div className="mb-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                Workspace shell
              </div>
              <h1 className="mb-6 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
                Settings
              </h1>
              <div className="relative">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-400 dark:text-zinc-500" />
                <input
                  type="text"
                  placeholder="Search settings..."
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                  className="flex h-11 w-full rounded-xl border border-zinc-200 bg-white py-2.5 pl-9 pr-4 text-[13px] text-zinc-950 outline-none transition placeholder:text-zinc-400 focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50 dark:placeholder:text-zinc-500"
                />
              </div>
            </div>

            <nav className="scrollbar-hide flex-1 space-y-1.5 overflow-y-auto px-4 pb-6">
              {filteredSections.length > 0 ? (
                filteredSections.map((section) => (
                  <SettingsNavButton
                    active={activeSection === section.id}
                    icon={section.icon}
                    key={section.id}
                    label={section.label}
                    onClick={() => setActiveSection(section.id)}
                  />
                ))
              ) : (
                <div className="px-3 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400">
                  No settings found.
                </div>
              )}
            </nav>
          </div>

          <div className="scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto">
            <div className="mx-auto w-full max-w-5xl p-8 md:p-12">
              {activeSection === 'appearance' ? (
                <div className="space-y-6">
                  <SettingsPageHeader
                    title="Appearance"
                    description="Keep Portal on the same visual system as claw-studio while preserving Portal-specific work surfaces."
                  />

                  <SettingsSection
                    title="Theme mode"
                    description="Theme mode stays synchronized across header, sidebar, content surfaces, and dialogs."
                  >
                    <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                      {PORTAL_THEME_MODE_OPTIONS.map((option) => {
                        const Icon = THEME_MODE_ICONS[option.id];
                        return (
                          <ThemeOptionButton
                            active={themeMode === option.id}
                            icon={Icon}
                            key={option.id}
                            label={option.label}
                            onClick={() => setThemeMode(option.id)}
                          />
                        );
                      })}
                    </div>
                  </SettingsSection>

                  <SettingsSection
                    title="Theme color"
                    description="Theme color changes accent surfaces without changing the claw-style layout contract."
                  >
                    <div className="flex flex-wrap gap-4">
                      {PORTAL_THEME_COLOR_OPTIONS.map((option) => (
                        <ThemeColorButton
                          active={themeColor === option.id}
                          color={option.id}
                          key={option.id}
                          label={option.label}
                          onClick={() => setThemeColor(option.id)}
                          previewClassName={option.previewClassName}
                        />
                      ))}
                    </div>
                  </SettingsSection>

                  <SettingsSection
                    title="Theme preview"
                    description="Validate the rail, titlebar, and right-side content surfaces before closing settings."
                  >
                    <div className="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]">
                      <div className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70">
                        <div className="rounded-[24px] border border-zinc-200/80 bg-zinc-50/80 p-4 dark:border-zinc-800/80 dark:bg-zinc-900/70">
                          <div className="rounded-2xl bg-white/72 px-4 py-3 backdrop-blur-xl dark:bg-zinc-950/78">
                            <div className="flex items-center justify-between gap-3">
                              <div className="flex items-center gap-3">
                                <div className="flex h-7 w-7 items-center justify-center rounded-xl bg-primary-600 text-white">
                                  <Palette className="h-4 w-4" />
                                </div>
                                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                  SDKWork Router
                                </div>
                              </div>
                              <div className="hidden text-[10px] font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400 sm:block">
                                Current workspace
                              </div>
                            </div>
                          </div>

                          <div className="mt-4 grid gap-4 lg:grid-cols-[220px_minmax(0,1fr)]">
                            <div className="rounded-[24px] border border-zinc-900/90 bg-[linear-gradient(180deg,_#13151a_0%,_#0b0c10_100%)] p-4 text-zinc-300 shadow-[18px_0_50px_rgba(9,9,11,0.16)]">
                              <div className="grid gap-1">
                                <div className="text-[10px] font-semibold uppercase tracking-[0.22em] text-zinc-500">
                                  Workspace
                                </div>
                                <div className="truncate text-sm font-semibold text-white">
                                  {workspaceName}
                                </div>
                                <div className="truncate text-xs text-zinc-400">{workspaceEmail}</div>
                              </div>
                              <div className="mt-4 space-y-1">
                                {previewRoutes.map((route, index) => (
                                  <div
                                    key={route.key}
                                    className={cn(
                                      'rounded-2xl px-3 py-2.5 text-sm transition-colors',
                                      index === 0
                                        ? 'bg-white/[0.08] font-medium text-white'
                                        : 'text-zinc-400',
                                    )}
                                  >
                                    {route.label}
                                  </div>
                                ))}
                              </div>
                            </div>

                            <div className="space-y-4">
                              <div className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                  Right content region
                                </div>
                                <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                                  The right side remains the Portal content display area, but its shell treatment stays in the same family as claw-studio.
                                </p>
                              </div>
                              <div className="grid gap-4 md:grid-cols-3">
                                {[
                                  'Header glass',
                                  'Rail parity',
                                  'Theme-aware content',
                                ].map((label) => (
                                  <div
                                    key={label}
                                    className="rounded-[24px] border border-zinc-200 bg-zinc-50/90 p-4 dark:border-zinc-800 dark:bg-zinc-900/80"
                                  >
                                    <div className="text-xs uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
                                      Theme preview
                                    </div>
                                    <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                      {label}
                                    </div>
                                  </div>
                                ))}
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>

                      <div className="grid gap-4">
                        <SettingsStatCard
                          detail={`Mode: ${currentThemeMode}`}
                          label="Current theme"
                          value={currentThemeColor}
                        />
                        <SettingsStatCard
                          detail={`${sidebarWidth}px width`}
                          label="Sidebar state"
                          value={isSidebarCollapsed ? 'Collapsed' : 'Expanded'}
                        />
                        <SettingsStatCard
                          detail="Modules currently visible in the rail"
                          label="Visible modules"
                          value={`${visibleRoutes.length}`}
                        />
                      </div>
                    </div>
                  </SettingsSection>
                </div>
              ) : null}

              {activeSection === 'navigation' ? (
                <div className="space-y-6">
                  <SettingsPageHeader
                    title="Navigation"
                    description="Keep the sidebar display effect aligned with claw-studio while preserving Portal routes."
                  />

                  <SettingsSection
                    title="Sidebar behavior"
                    description="Sidebar behavior stays consistent across click collapse, expand, and drag resize."
                    actions={
                      <>
                        <button
                          type="button"
                          onClick={() => setSidebarCollapsed(!isSidebarCollapsed)}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-white px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
                        >
                          <SlidersHorizontal className="h-4 w-4" />
                          {isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
                        </button>
                        <button
                          type="button"
                          onClick={resetShellPreferences}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-zinc-50 px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
                        >
                          Restore defaults
                        </button>
                      </>
                    }
                  >
                    <div className="grid gap-4 md:grid-cols-3">
                      <SettingsStatCard
                        detail="Current shell rail display mode"
                        label="Sidebar state"
                        value={isSidebarCollapsed ? 'Collapsed' : 'Expanded'}
                      />
                      <SettingsStatCard
                        detail="Resizable rail width"
                        label="Sidebar width"
                        value={`${sidebarWidth}px`}
                      />
                      <SettingsStatCard
                        detail="Modules shown in the rail"
                        label="Visible modules"
                        value={`${visibleRoutes.length}`}
                      />
                    </div>
                  </SettingsSection>

                  <SettingsSection
                    title="Sidebar navigation"
                    description="Show or hide workspace modules while keeping the claw-style rail intact."
                  >
                    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                      {portalRoutes.map((route) => {
                        const visible = !hiddenSidebarItems.includes(route.key);

                        return (
                          <label
                            key={route.key}
                            className="flex cursor-pointer items-center gap-3 rounded-xl border border-zinc-200 p-3 transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
                          >
                            <Checkbox checked={visible} onChange={() => toggleSidebarItem(route.key)} />
                            <span className="grid gap-0.5">
                              <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                                {route.label}
                              </span>
                              <span className="text-xs text-zinc-500 dark:text-zinc-400">
                                {route.detail}
                              </span>
                            </span>
                          </label>
                        );
                      })}
                    </div>
                  </SettingsSection>
                </div>
              ) : null}

              {activeSection === 'workspace' ? (
                <div className="space-y-6">
                  <SettingsPageHeader
                    title="Workspace"
                    description="Keep workspace identity and shell reset controls in one place."
                  />

                  <SettingsSection
                    title="Workspace profile"
                    description="Portal workspace identity lives here while the shell stays visually aligned with claw-studio."
                  >
                    <div className="grid gap-4 md:grid-cols-2">
                      <SettingsStatCard
                        detail={tenantName}
                        label="Active workspace"
                        value={workspaceName}
                      />
                      <SettingsStatCard
                        detail={workspaceEmail}
                        label="Operator"
                        value={operatorName}
                      />
                    </div>
                  </SettingsSection>

                  <SettingsSection
                    title="Restore defaults"
                    description="Reset Theme mode, Theme color, and Sidebar behavior back to the default Portal shell."
                  >
                    <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                      <div className="flex items-start gap-3">
                        <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-primary-500/12 text-primary-600 dark:text-primary-300">
                          <RotateCcw className="h-4 w-4" />
                        </span>
                        <div className="min-w-0">
                          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            Restore defaults
                          </div>
                          <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                            Use this when the shell needs to snap back to the default Theme mode, Theme color, and Sidebar navigation contract.
                          </p>
                        </div>
                      </div>
                      <div className="mt-5 flex flex-wrap gap-3">
                        <button
                          type="button"
                          onClick={resetShellPreferences}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl bg-zinc-950 px-4 text-sm font-semibold text-white transition hover:bg-zinc-900 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200"
                        >
                          Restore defaults
                        </button>
                        <button
                          type="button"
                          onClick={() => onOpenChange(false)}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-white px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
                        >
                          Close settings
                        </button>
                      </div>
                    </div>
                  </SettingsSection>
                </div>
              ) : null}
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
