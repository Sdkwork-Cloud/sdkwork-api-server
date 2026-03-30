import { Pill } from 'sdkwork-router-portal-commons';

type RoutingCardTone = 'default' | 'accent' | 'positive' | 'warning' | 'seed';

export interface RoutingCardItem {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone?: RoutingCardTone;
}

export function RoutingCardGrid({
  items,
  columns = 'xl:grid-cols-4',
}: {
  items: RoutingCardItem[];
  columns?: string;
}) {
  return (
    <div className={`grid gap-4 md:grid-cols-2 ${columns}`}>
      {items.map((item) => (
        <article
          key={item.id}
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
        >
          <div className="flex flex-wrap items-start justify-between gap-3">
            <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
              {item.label}
            </span>
            {item.tone ? <Pill tone={item.tone}>{item.value}</Pill> : null}
          </div>
          <strong className="mt-3 block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {item.value}
          </strong>
          <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            {item.detail}
          </p>
        </article>
      ))}
    </div>
  );
}
