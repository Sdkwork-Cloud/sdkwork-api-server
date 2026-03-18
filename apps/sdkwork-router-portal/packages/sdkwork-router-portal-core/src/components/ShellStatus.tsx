import { Pill } from 'sdkwork-router-portal-commons';
import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { portalRoutes } from '../routes';

export function ShellStatus({
  activeRoute,
  pulseDetail,
  pulseStatus,
  pulseTitle,
  pulseTone,
  workspace,
}: {
  activeRoute: PortalRouteKey;
  pulseDetail: string;
  pulseStatus: string;
  pulseTitle: string;
  pulseTone: 'accent' | 'positive' | 'warning';
  workspace: PortalWorkspaceSummary | null;
}) {
  const routeDefinition = portalRoutes.find((route) => route.key === activeRoute) ?? portalRoutes[0];

  return (
    <section className="grid gap-4 rounded-[28px] border border-[color:var(--portal-contrast-border)] [background:var(--portal-surface-contrast)] p-6 shadow-[var(--portal-shadow-strong)]">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="grid gap-2">
          <span className="text-xs font-semibold uppercase tracking-[0.24em] text-[var(--portal-text-muted-on-contrast)]">
            {routeDefinition.eyebrow}
          </span>
          <h1 className="text-3xl font-semibold tracking-tight text-[var(--portal-text-on-contrast)]">
            {routeDefinition.label}
          </h1>
          <p className="max-w-3xl text-sm leading-6 text-[var(--portal-text-muted-on-contrast)]">
            {routeDefinition.detail}
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <Pill tone="default">{workspace?.project.name ?? 'Workspace'}</Pill>
          <Pill tone={pulseTone}>{pulseTitle}</Pill>
        </div>
      </div>

      <div className="grid gap-3 lg:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
        <article className="rounded-2xl border border-[color:var(--portal-contrast-border)] bg-[var(--portal-contrast-soft)] p-4">
          <strong className="text-sm font-semibold text-[var(--portal-text-on-contrast)]">Workspace pulse</strong>
          <p className="mt-2 text-sm text-[var(--portal-text-muted-on-contrast)]">{pulseDetail}</p>
        </article>
        <article className="rounded-2xl border border-[color:var(--portal-contrast-border)] bg-[var(--portal-contrast-soft)] p-4">
          <strong className="text-sm font-semibold text-[var(--portal-text-on-contrast)]">Workspace status</strong>
          <p className="mt-2 text-sm text-[var(--portal-text-muted-on-contrast)]">{pulseStatus}</p>
        </article>
      </div>
    </section>
  );
}
