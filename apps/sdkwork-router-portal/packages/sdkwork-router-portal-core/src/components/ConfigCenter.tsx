import { Check, Laptop, Moon, SlidersHorizontal, Sun } from 'lucide-react';
import {
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  InlineButton,
  Pill,
} from 'sdkwork-router-portal-commons';
import type { PortalThemeMode } from 'sdkwork-router-portal-types';

import {
  PORTAL_THEME_COLOR_OPTIONS,
  PORTAL_THEME_MODE_OPTIONS,
} from '../lib/portalPreferences';
import { portalRoutes } from '../routes';
import { usePortalShellStore } from '../store/usePortalShellStore';

const THEME_MODE_ICONS: Record<
  PortalThemeMode,
  typeof Sun
> = {
  light: Sun,
  dark: Moon,
  system: Laptop,
};

export function ConfigCenter({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const {
    hiddenSidebarItems,
    themeColor,
    themeMode,
    toggleSidebarItem,
    setThemeColor,
    setThemeMode,
  } = usePortalShellStore();

  return (
      <Dialog onOpenChange={onOpenChange} open={open}>
        <DialogContent className="max-h-[calc(100dvh-2rem)] w-[min(1040px,calc(100%-2rem))] overflow-y-auto border-[color:var(--portal-border-color)] bg-[var(--portal-overlay-surface)] text-[var(--portal-text-primary)] shadow-[var(--portal-shadow-strong)]">
          <DialogHeader className="gap-4">
            <div className="flex flex-wrap items-center gap-3">
              <Pill tone="accent">Workspace shell</Pill>
              <Pill tone="default">{themeMode}</Pill>
              <Pill tone="default">{themeColor}</Pill>
            </div>
            <div className="grid gap-2">
              <DialogTitle>Config center</DialogTitle>
              <DialogDescription>
                Tune the portal shell with the same visual grammar as claw-studio: Appearance, theme color, and Sidebar navigation all stay aligned in one place.
              </DialogDescription>
            </div>
          </DialogHeader>

          <div className="grid gap-6 lg:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
            <section className="grid gap-6">
              <article className="grid gap-5 rounded-[28px] border border-[color:var(--portal-border-color)] bg-[var(--portal-surface-elevated)] p-5 shadow-[var(--portal-shadow-soft)]">
                <div className="flex items-center justify-between gap-3">
                  <div className="grid gap-1">
                    <strong className="text-base text-[var(--portal-text-primary)]">Appearance</strong>
                    <p className="text-sm text-[var(--portal-text-secondary)]">
                      Theme mode and palette stay locked to the shell so the portal feels consistent everywhere.
                    </p>
                  </div>
                  <Pill tone="accent">Appearance</Pill>
                </div>

                <div className="grid gap-4">
                  <div className="flex items-center justify-between gap-3">
                    <strong className="text-sm text-[var(--portal-text-primary)]">Theme mode</strong>
                    <Pill tone="default">{themeMode}</Pill>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-3">
                    {PORTAL_THEME_MODE_OPTIONS.map((option) => {
                      const Icon = THEME_MODE_ICONS[option.id];

                      return (
                        <button
                          className={`flex flex-col items-center justify-center gap-3 rounded-2xl border p-4 text-sm font-medium transition ${
                            themeMode === option.id
                              ? 'border-primary-500/35 bg-[color:rgb(var(--portal-accent-rgb)_/_0.12)] text-primary-700 dark:text-primary-100'
                              : 'border-[color:var(--portal-border-color)] bg-[var(--portal-surface-background)] text-[var(--portal-text-secondary)] hover:border-[color:var(--portal-border-strong)] hover:bg-[var(--portal-hover-surface)] hover:text-[var(--portal-text-primary)]'
                          }`}
                          key={option.id}
                          onClick={() => setThemeMode(option.id)}
                          type="button"
                        >
                          <Icon
                            className={`h-5 w-5 ${
                              themeMode === option.id
                                ? 'text-primary-500 dark:text-primary-300'
                                : 'text-[var(--portal-text-muted)]'
                            }`}
                          />
                          <span>{option.label}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>

                <div className="grid gap-4">
                  <div className="flex items-center justify-between gap-3">
                    <strong className="text-sm text-[var(--portal-text-primary)]">Theme color</strong>
                    <Pill tone="default">{themeColor}</Pill>
                  </div>
                  <div className="grid gap-4 sm:grid-cols-3">
                    {PORTAL_THEME_COLOR_OPTIONS.map((option) => (
                      <button
                        className="group flex flex-col items-center gap-3 rounded-2xl border border-[color:var(--portal-border-color)] bg-[var(--portal-surface-background)] p-4 transition hover:border-[color:var(--portal-border-strong)] hover:bg-[var(--portal-hover-surface)]"
                        key={option.id}
                        onClick={() => setThemeColor(option.id)}
                        type="button"
                      >
                        <div
                          className={`flex h-10 w-10 items-center justify-center rounded-full ${option.previewClassName} ${
                            themeColor === option.id
                              ? 'ring-2 ring-zinc-900 ring-offset-2 ring-offset-white dark:ring-white dark:ring-offset-zinc-950'
                              : ''
                            }`}
                        >
                          {themeColor === option.id ? <Check className="h-4 w-4 text-white" /> : null}
                        </div>
                        <span className="text-sm font-medium text-[var(--portal-text-secondary)]">
                          {option.label}
                        </span>
                      </button>
                    ))}
                  </div>
                </div>
              </article>
            </section>

            <section className="grid gap-4 rounded-[28px] border border-[color:var(--portal-border-color)] bg-[var(--portal-surface-elevated)] p-5 shadow-[var(--portal-shadow-soft)]">
              <div className="flex items-start justify-between gap-3">
                <div className="grid gap-1">
                  <strong className="text-base text-[var(--portal-text-primary)]">Sidebar navigation</strong>
                  <p className="text-sm text-[var(--portal-text-secondary)]">
                    Hide or restore individual workspace modules while keeping the shell structure and right-side content region intact.
                  </p>
                </div>
                <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-primary-50 text-primary-600 dark:bg-primary-500/10 dark:text-primary-300">
                  <SlidersHorizontal className="h-4 w-4" />
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-2">
                {portalRoutes.map((route) => {
                  const visible = !hiddenSidebarItems.includes(route.key);

                  return (
                    <label
                      className="flex items-start gap-3 rounded-2xl border border-[color:var(--portal-border-color)] bg-[var(--portal-surface-background)] p-4 text-sm text-[var(--portal-text-secondary)] transition-colors hover:bg-[var(--portal-hover-surface)]"
                      key={route.key}
                    >
                      <Checkbox checked={visible} onChange={() => toggleSidebarItem(route.key)} />
                      <span className="grid gap-1">
                        <strong className="text-sm text-[var(--portal-text-primary)]">{route.label}</strong>
                        <span className="text-xs leading-5 text-[var(--portal-text-muted)]">
                          {route.detail}
                        </span>
                      </span>
                    </label>
                  );
                })}
              </div>

              <div className="flex justify-end">
                <InlineButton onClick={() => onOpenChange(false)} tone="primary">
                  Close config center
                </InlineButton>
              </div>
            </section>
          </div>
        </DialogContent>
    </Dialog>
  );
}
