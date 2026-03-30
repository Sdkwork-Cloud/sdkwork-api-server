import {
  Check,
  Laptop,
  Moon,
  Palette,
  PanelLeft,
  RotateCcw,
  Sun,
  UserRound,
  type LucideIcon,
} from 'lucide-react';
import { motion } from 'motion/react';
import { useEffect, useMemo, useState, type ReactNode } from 'react';

import {
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  FormField,
  PORTAL_LOCALE_OPTIONS,
  SearchInput,
  Select,
  cn,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
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
    description: 'Theme mode and Theme color',
    icon: Palette,
  },
  {
    id: 'navigation',
    label: 'Navigation',
    description: 'Sidebar behavior and Sidebar navigation',
    icon: PanelLeft,
  },
  {
    id: 'workspace',
    label: 'Workspace',
    description: 'Language and workspace preferences',
    icon: UserRound,
  },
];

function ConfigNavButton({
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
    <Button
      onClick={onClick}
      className={cn(
        'h-auto w-full justify-start gap-3 rounded-xl border px-3 py-2.5 text-[14px] font-medium shadow-none transition-all duration-200',
        active
          ? 'border-zinc-200/50 bg-white text-primary-600 shadow-sm dark:border-zinc-700/50 dark:bg-zinc-800 dark:text-primary-400'
          : 'border-transparent text-zinc-600 hover:bg-zinc-200/50 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800/50 dark:hover:text-zinc-100',
      )}
      type="button"
      variant="ghost"
    >
      <Icon
        className={cn(
          'h-4 w-4',
          active ? 'text-primary-500 dark:text-primary-400' : 'text-zinc-400 dark:text-zinc-500',
        )}
      />
      {label}
    </Button>
  );
}

function ConfigBlock({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <div className="overflow-hidden rounded-[1.5rem] border border-zinc-200/80 bg-white shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
      <div className="border-b border-zinc-100 bg-zinc-50/50 px-6 py-5 dark:border-zinc-800/80 dark:bg-zinc-900/50">
        <div className="space-y-1">
          <h3 className="text-[15px] font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
            {title}
          </h3>
          {description ? (
            <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>
          ) : null}
        </div>
      </div>
      <div className="p-6">{children}</div>
    </div>
  );
}

function ModeChoice({
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
    <Button
      type="button"
      onClick={onClick}
      className={cn(
        'h-auto w-full flex-col items-center justify-center gap-3 whitespace-normal rounded-xl border-2 p-4 text-center shadow-none transition-all',
        active
          ? 'border-primary-500 bg-primary-50/50 dark:bg-primary-500/10'
          : 'border-zinc-200 bg-white hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700',
      )}
      variant="ghost"
    >
      <Icon
        className={cn(
          'h-6 w-6',
          active ? 'text-primary-500 dark:text-primary-400' : 'text-zinc-500 dark:text-zinc-400',
        )}
      />
      <span
        className={cn(
          'text-sm font-medium',
          active
            ? 'text-primary-700 dark:text-primary-300'
            : 'text-zinc-700 dark:text-zinc-300',
        )}
      >
        {label}
      </span>
    </Button>
  );
}

function ColorSwatch({
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
    <Button
      type="button"
      onClick={onClick}
      className="group relative h-auto flex-col items-center gap-2 whitespace-normal rounded-none p-0 shadow-none hover:bg-transparent"
      variant="ghost"
    >
      <div
        className={cn(
          'flex h-10 w-10 items-center justify-center rounded-full shadow-sm ring-2 ring-offset-2 transition-all dark:ring-offset-zinc-950',
          previewClassName,
          active ? 'scale-110 ring-zinc-900 dark:ring-zinc-100' : 'ring-transparent hover:scale-105',
        )}
      >
        {active ? <Check className="h-5 w-5 text-white" /> : null}
      </div>
      <span
        className={cn(
          'text-xs font-medium',
          active
            ? 'text-zinc-900 dark:text-zinc-100'
            : 'text-zinc-500 group-hover:text-zinc-700 dark:text-zinc-400 dark:group-hover:text-zinc-300',
        )}
        >
          {label}
        </span>
      <span className="sr-only">{color}</span>
    </Button>
  );
}

export function ConfigCenter({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { locale, setLocale, t } = usePortalI18n();
  const workspace = usePortalAuthStore((state) => state.workspace);
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    resetShellPreferences,
    setSidebarCollapsed,
    themeColor,
    themeMode,
    toggleSidebarItem,
    setThemeColor,
    setThemeMode,
  } = usePortalShellStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [activeSection, setActiveSection] = useState<ConfigCenterSectionId>('appearance');
  const configSections = useMemo(
    () =>
      CONFIG_CENTER_SECTIONS.map((section) => ({
        ...section,
        label: t(section.label),
        description: t(section.description),
      })),
    [t],
  );

  const filteredSections = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    if (!normalizedQuery) {
      return configSections;
    }

    return configSections.filter((section) =>
      `${section.id} ${section.label} ${section.description}`
        .toLowerCase()
        .includes(normalizedQuery),
    );
  }, [configSections, searchQuery]);

  useEffect(() => {
    if (!filteredSections.some((section) => section.id === activeSection)) {
      setActiveSection(filteredSections[0]?.id ?? 'appearance');
    }
  }, [activeSection, filteredSections]);

  const workspaceName = workspace?.project.name ?? t('Portal workspace');
  const workspaceEmail = workspace?.user.email ?? t('Awaiting workspace session');
  const tenantName = workspace?.tenant.name ?? t('Portal tenant');
  const operatorName = workspace?.user.display_name ?? t('Portal operator');

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="max-h-[calc(100dvh-2rem)] w-[min(1180px,calc(100%-2rem))] overflow-hidden border-zinc-200/80 bg-white/95 p-0 shadow-[0_32px_80px_rgba(15,23,42,0.18)] dark:border-zinc-800/80 dark:bg-zinc-950/92">
        <div className="sr-only">
          <DialogTitle>{t('Settings')}</DialogTitle>
          <DialogDescription>{t('Portal workspace settings')}</DialogDescription>
        </div>

        <div className="flex h-full min-h-[760px] bg-zinc-50/50 dark:bg-zinc-950/50">
          <div className="flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50/80 backdrop-blur-xl dark:border-zinc-800 dark:bg-zinc-900/80">
            <div className="p-6 pb-4">
              <h1 className="mb-6 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
                {t('Settings')}
              </h1>
              <SearchInput
                placeholder={t('Search settings...')}
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                inputClassName="h-10 pr-4 text-[13px]"
              />
            </div>

            <nav className="scrollbar-hide flex-1 space-y-1.5 overflow-y-auto px-4 pb-6">
              {filteredSections.length ? (
                filteredSections.map((section) => (
                  <ConfigNavButton
                    key={section.id}
                    active={activeSection === section.id}
                    icon={section.icon}
                    label={section.label}
                    onClick={() => setActiveSection(section.id)}
                  />
                ))
              ) : (
                <div className="px-3 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400">
                  {t('No settings found.')}
                </div>
              )}
            </nav>
          </div>

          <div className="scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto">
            <div className="mx-auto w-full max-w-5xl p-8 md:p-12">
              <motion.div
                key={activeSection}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.2 }}
                className="w-full"
              >
                {activeSection === 'appearance' ? (
                  <div className="space-y-6">
                    <ConfigBlock
                      title={t('Theme mode')}
                      description={t('Theme mode stays synchronized across header, sidebar, content surfaces, and dialogs.')}
                    >
                      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                        {PORTAL_THEME_MODE_OPTIONS.map((option) => {
                          const Icon = THEME_MODE_ICONS[option.id];
                          return (
                            <ModeChoice
                              key={option.id}
                              active={themeMode === option.id}
                              icon={Icon}
                              label={t(option.label)}
                              onClick={() => setThemeMode(option.id)}
                            />
                          );
                        })}
                      </div>
                    </ConfigBlock>

                    <ConfigBlock
                      title={t('Theme color')}
                      description={t('Theme color updates the accent surfaces without changing the claw-style shell contract.')}
                    >
                      <div className="flex flex-wrap gap-4">
                        {PORTAL_THEME_COLOR_OPTIONS.map((option) => (
                          <ColorSwatch
                            key={option.id}
                            active={themeColor === option.id}
                            color={option.id}
                            label={t(option.label)}
                            onClick={() => setThemeColor(option.id)}
                            previewClassName={option.previewClassName}
                          />
                        ))}
                      </div>
                    </ConfigBlock>
                  </div>
                ) : null}

                {activeSection === 'navigation' ? (
                  <div className="space-y-6">
                    <ConfigBlock
                      title={t('Sidebar behavior')}
                      description={t('Keep the left rail aligned with claw-studio while preserving the portal route set.')}
                    >
                      <div className="flex flex-wrap gap-3">
                        <Button
                          type="button"
                          onClick={() => setSidebarCollapsed(!isSidebarCollapsed)}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-white px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
                          variant="secondary"
                        >
                          <PanelLeft className="h-4 w-4" />
                          {isSidebarCollapsed ? t('Expand sidebar') : t('Collapse sidebar')}
                        </Button>
                        <Button
                          type="button"
                          onClick={resetShellPreferences}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-zinc-50 px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
                          variant="secondary"
                        >
                          <RotateCcw className="h-4 w-4" />
                          {t('Reset shell preferences')}
                        </Button>
                      </div>
                    </ConfigBlock>

                    <ConfigBlock
                      title={t('Navigation')}
                      description={t('Show or hide workspace modules while keeping the left rail compact and stable.')}
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
                                  {t(route.label)}
                                </span>
                                <span className="text-xs text-zinc-500 dark:text-zinc-400">
                                  {t(route.detail)}
                                </span>
                              </span>
                            </label>
                          );
                        })}
                      </div>
                    </ConfigBlock>
                  </div>
                ) : null}

                {activeSection === 'workspace' ? (
                  <div className="space-y-6">
                    <ConfigBlock
                      title={t('Language and locale')}
                      description={t('Choose the portal workspace language. Shared shell copy and locale-aware formatting update immediately.')}
                    >
                      <div className="grid gap-4 md:grid-cols-2">
                        <FormField label={t('Language')}>
                          <Select
                            value={locale}
                            onChange={(event) => setLocale(event.target.value as typeof locale)}
                          >
                            {PORTAL_LOCALE_OPTIONS.map((option) => (
                              <option key={option.id} value={option.id}>
                                {t(option.label)}
                              </option>
                            ))}
                          </Select>
                        </FormField>
                      </div>
                    </ConfigBlock>

                    <ConfigBlock
                      title={t('Workspace preferences')}
                      description={t('Keep workspace identity and shell reset controls in one place.')}
                    >
                      <div className="grid gap-4 md:grid-cols-2">
                        <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                          <div className="text-xs uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
                            {t('Workspace')}
                          </div>
                          <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                            {workspaceName}
                          </div>
                          <div className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                            {tenantName}
                          </div>
                        </div>
                        <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                          <div className="text-xs uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
                            {t('Operator')}
                          </div>
                          <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                            {operatorName}
                          </div>
                          <div className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                            {workspaceEmail}
                          </div>
                        </div>
                      </div>

                      <div className="mt-5 flex flex-wrap gap-3">
                        <Button
                          type="button"
                          onClick={resetShellPreferences}
                          className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl bg-zinc-950 px-4 text-sm font-semibold text-white transition hover:bg-zinc-900 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200"
                          variant="default"
                        >
                          {t('Reset shell preferences')}
                        </Button>
                      </div>
                    </ConfigBlock>
                  </div>
                ) : null}
              </motion.div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
