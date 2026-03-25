import type { ComponentType } from 'react';
import { Check, Laptop, Moon, Sun } from 'lucide-react';

import { useAdminI18n } from 'sdkwork-router-admin-commons';
import { useAdminAppStore } from 'sdkwork-router-admin-core';

import { SettingsSection } from './Shared';

const THEME_COLORS = [
  { id: 'tech-blue', label: 'tech-blue', colorClass: 'bg-sky-500' },
  { id: 'lobster', label: 'lobster', colorClass: 'bg-red-500' },
  { id: 'green-tech', label: 'green-tech', colorClass: 'bg-emerald-500' },
  { id: 'zinc', label: 'zinc', colorClass: 'bg-zinc-500' },
  { id: 'violet', label: 'violet', colorClass: 'bg-violet-500' },
  { id: 'rose', label: 'rose', colorClass: 'bg-rose-500' },
] as const;

function ThemeModeCard({
  active,
  description,
  icon: Icon,
  label,
  onClick,
}: {
  active: boolean;
  description: string;
  icon: ComponentType<{ className?: string }>;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex flex-col items-center justify-center gap-3 rounded-xl border-2 p-4 transition-all ${
        active
          ? 'border-primary-500 bg-primary-50/50 dark:bg-primary-500/10'
          : 'border-zinc-200 bg-white hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700'
      }`}
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
      <span className="text-center text-xs leading-6 text-zinc-500 dark:text-zinc-400">
        {description}
      </span>
    </button>
  );
}

export function AppearanceSettings() {
  const { setThemeColor, setThemeMode, themeColor, themeMode } = useAdminAppStore();
  const { t } = useAdminI18n();

  return (
    <div className="space-y-8">
      <div>
        <h2 className="mb-1 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
          {t('Appearance')}
        </h2>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          {t('Theme mode and accent color stay synchronized across header, sidebar, and page surfaces.')}
        </p>
      </div>

      <div className="space-y-6">
        <SettingsSection
          eyebrow={t('Appearance')}
          title={t('Theme mode')}
          description={t('Choose how the shell follows light, dark, or system appearance.')}
        >
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <ThemeModeCard
              active={themeMode === 'light'}
              description={t('Bright shell with frosted content panes.')}
              icon={Sun}
              label={t('Light')}
              onClick={() => setThemeMode('light')}
            />
            <ThemeModeCard
              active={themeMode === 'dark'}
              description={t('Claw-style low-glare shell with higher contrast.')}
              icon={Moon}
              label={t('Dark')}
              onClick={() => setThemeMode('dark')}
            />
            <ThemeModeCard
              active={themeMode === 'system'}
              description={t('Follow the device preference automatically.')}
              icon={Laptop}
              label={t('System')}
              onClick={() => setThemeMode('system')}
            />
          </div>
        </SettingsSection>

        <SettingsSection
          eyebrow={t('Accent')}
          title={t('Theme color')}
          description={t('Theme color updates accent surfaces without changing the claw-style shell contract.')}
        >
          <div className="flex flex-wrap gap-4">
            {THEME_COLORS.map((color) => (
              <button
                key={color.id}
                type="button"
                onClick={() => setThemeColor(color.id)}
                className="group relative flex flex-col items-center gap-2"
              >
                <div
                  className={`flex h-10 w-10 items-center justify-center rounded-full ${color.colorClass} shadow-sm ring-2 ring-offset-2 transition-all dark:ring-offset-zinc-950 ${
                    themeColor === color.id
                      ? 'scale-110 ring-zinc-900 dark:ring-zinc-100'
                      : 'ring-transparent hover:scale-105'
                  }`}
                >
                  {themeColor === color.id ? <Check className="h-5 w-5 text-white" /> : null}
                </div>
                <span
                  className={`text-xs font-medium ${
                    themeColor === color.id
                      ? 'text-zinc-900 dark:text-zinc-100'
                      : 'text-zinc-500 group-hover:text-zinc-700 dark:text-zinc-400 dark:group-hover:text-zinc-300'
                  }`}
                >
                  {t(color.label)}
                </span>
              </button>
            ))}
          </div>
        </SettingsSection>
      </div>
    </div>
  );
}
