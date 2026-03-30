import {
  InlineButton,
  Pill,
  Surface,
} from 'sdkwork-router-portal-commons';

import type {
  GatewayLaunchReadinessSummary,
  GatewayModeCard,
  GatewayPostureCard,
  GatewayReadinessAction,
  GatewayRuntimeControl,
  GatewayTopologyPlaybook,
} from '../types';

export function GatewayPostureGrid({ cards }: { cards: GatewayPostureCard[] }) {
  return (
    <div className="grid gap-4 xl:grid-cols-4">
      {cards.map((card) => (
        <article
          key={card.id}
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
        >
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
            {card.label}
          </p>
          <strong className="mt-3 block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {card.value}
          </strong>
          <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {card.detail}
          </p>
        </article>
      ))}
    </div>
  );
}

export function GatewayModeGrid({ cards }: { cards: GatewayModeCard[] }) {
  return (
    <div className="grid gap-4">
      {cards.map((card) => (
        <article
          key={card.id}
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
        >
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {card.title}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                {card.summary}
              </p>
            </div>
            <Pill tone="positive">{card.title}</Pill>
          </div>
          <pre className="mt-4 overflow-x-auto rounded-2xl bg-zinc-950 p-4 text-sm leading-6 text-zinc-300">
            <code>{card.command}</code>
          </pre>
          <ul className="mt-4 space-y-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {card.notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        </article>
      ))}
    </div>
  );
}

export function GatewayTopologyGrid({ playbooks }: { playbooks: GatewayTopologyPlaybook[] }) {
  return (
    <div className="grid gap-4">
      {playbooks.map((playbook) => (
        <article
          key={playbook.id}
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
        >
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
            {playbook.topology}
          </p>
          <strong className="mt-3 block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {playbook.title}
          </strong>
          <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {playbook.detail}
          </p>
          <pre className="mt-4 overflow-x-auto rounded-2xl bg-zinc-950 p-4 text-sm leading-6 text-zinc-300">
            <code>{playbook.command}</code>
          </pre>
        </article>
      ))}
    </div>
  );
}

export function GatewayReadinessGrid({
  actions,
  onNavigate,
}: {
  actions: GatewayReadinessAction[];
  onNavigate: (route: GatewayReadinessAction['route']) => void;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-3">
      {actions.map((action) => (
        <article
          key={action.id}
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
        >
          <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {action.title}
          </strong>
          <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {action.detail}
          </p>
          <div className="mt-4">
            <InlineButton
              onClick={() => onNavigate(action.route)}
              tone={action.tone ?? 'secondary'}
            >
              {action.cta}
            </InlineButton>
          </div>
        </article>
      ))}
    </div>
  );
}

function launchReadinessTone(status: GatewayLaunchReadinessSummary['status']) {
  if (status === 'ready') {
    return 'positive';
  }

  if (status === 'watch') {
    return 'accent';
  }

  return 'warning';
}

export function GatewayLaunchReadinessPanel({
  readiness,
}: {
  readiness: GatewayLaunchReadinessSummary;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-[0.72fr_1.28fr]">
      <article className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
          Launch readiness
        </p>
        <div className="mt-4 flex items-center justify-between gap-3">
          <strong className="text-5xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
            {readiness.score}
          </strong>
          <Pill tone={launchReadinessTone(readiness.status)}>
            {readiness.status === 'ready'
              ? 'Ready'
              : readiness.status === 'watch'
                ? 'Watch'
                : 'Blocked'}
          </Pill>
        </div>
        <strong className="mt-4 block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
          {readiness.headline}
        </strong>
        <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
          {readiness.detail}
        </p>
      </article>

      <div className="grid gap-4 xl:grid-cols-2">
        <article className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
          <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {readiness.blockersHeading}
          </strong>
          <ul className="mt-4 space-y-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {readiness.blockers.length ? (
              readiness.blockers.map((entry) => <li key={entry}>{entry}</li>)
            ) : (
              <li>No hard blockers are currently holding the launch posture.</li>
            )}
          </ul>
        </article>

        <article className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
          <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {readiness.watchpointsHeading}
          </strong>
          <ul className="mt-4 space-y-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {readiness.watchpoints.length ? (
              readiness.watchpoints.map((entry) => <li key={entry}>{entry}</li>)
            ) : (
              <li>No watchpoints are currently diluting the launch posture.</li>
            )}
          </ul>
        </article>
      </div>
    </div>
  );
}

export function GatewayRuntimeControlsGrid({
  controls,
  onAction,
  busyAction,
}: {
  controls: GatewayRuntimeControl[];
  onAction: (action: GatewayRuntimeControl['action']) => void;
  busyAction?: GatewayRuntimeControl['action'] | null;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-2">
      {controls.map((control) => (
        <article
          key={control.id}
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
        >
          <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {control.title}
          </strong>
          <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {control.detail}
          </p>
          <div className="mt-4">
            <InlineButton
              disabled={!control.enabled || busyAction === control.action}
              onClick={() => onAction(control.action)}
              tone={control.tone ?? 'secondary'}
            >
              {busyAction === control.action ? 'Restarting desktop runtime...' : control.cta}
            </InlineButton>
          </div>
        </article>
      ))}
    </div>
  );
}

export { Surface };
