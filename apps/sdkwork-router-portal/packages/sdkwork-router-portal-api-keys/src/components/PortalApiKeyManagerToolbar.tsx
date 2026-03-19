import { Plus, RefreshCw, Search } from 'lucide-react';

import type { PortalApiKeyEnvironmentOption } from '../types';

export function PortalApiKeyManagerToolbar({
  environment,
  environmentOptions,
  onEnvironmentChange,
  onOpenCreate,
  onRefresh,
  onSearchChange,
  searchQuery,
}: {
  environment: string;
  environmentOptions: PortalApiKeyEnvironmentOption[];
  onEnvironmentChange: (value: string) => void;
  onOpenCreate: () => void;
  onRefresh: () => void;
  onSearchChange: (value: string) => void;
  searchQuery: string;
}) {
  return (
    <section
      data-slot="portal-api-key-manager"
      className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
    >
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex flex-wrap items-center gap-3">
          <button
            type="button"
            onClick={onOpenCreate}
            className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl bg-zinc-950 px-4 text-sm font-semibold text-white transition hover:bg-zinc-900 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200"
          >
            <Plus className="h-4 w-4" />
            Create API key
          </button>

          <button
            type="button"
            onClick={onRefresh}
            className="inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-white px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
          >
            <RefreshCw className="h-4 w-4" />
            Refresh
          </button>
        </div>

        <div className="flex flex-col gap-3 sm:flex-row lg:w-[min(100%,52rem)] lg:justify-end">
          <div className="relative flex-1 lg:max-w-[24rem]">
            <Search className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-400 dark:text-zinc-500" />
            <input
              value={searchQuery}
              onChange={(event) => onSearchChange(event.target.value)}
              placeholder="Search API keys"
              className="h-11 w-full rounded-2xl border border-zinc-200 bg-white pl-11 pr-4 text-sm text-zinc-950 outline-none transition placeholder:text-zinc-400 focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50 dark:placeholder:text-zinc-500"
            />
          </div>

          <div className="w-full sm:w-[15rem]">
            <select
              aria-label="All environments"
              value={environment}
              onChange={(event) => onEnvironmentChange(event.target.value)}
              className="h-11 w-full rounded-2xl border border-zinc-200 bg-white px-4 text-sm text-zinc-950 outline-none transition focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50"
            >
              {environmentOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>
    </section>
  );
}
