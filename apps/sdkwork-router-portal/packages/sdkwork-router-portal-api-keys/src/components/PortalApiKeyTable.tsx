import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

function formatDate(value?: number | null): string {
  if (value === null || value === undefined) {
    return 'Never';
  }

  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(new Date(value));
}

function formatLastUsed(value?: number | null): string {
  if (value === null || value === undefined) {
    return 'Not yet';
  }

  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function maskValue(value: string): string {
  if (value.length <= 14) {
    return value;
  }

  return `${value.slice(0, 10)}********${value.slice(-4)}`;
}

function describeUsage(item: GatewayApiKeyRecord): string {
  if (!item.active) {
    return 'Revoked from gateway traffic';
  }

  if (item.last_used_at_ms) {
    return 'Gateway traffic observed';
  }

  return 'Ready for first authenticated request';
}

export function PortalApiKeyTable({
  items,
  latestCreatedKey,
  mutatingKey,
  onCopyLatestPlaintext,
  onDelete,
  onOpenUsage,
  onToggleStatus,
}: {
  items: GatewayApiKeyRecord[];
  latestCreatedKey: CreatedGatewayApiKey | null;
  mutatingKey: string | null;
  onCopyLatestPlaintext: () => void;
  onDelete: (item: GatewayApiKeyRecord) => void;
  onOpenUsage: (item: GatewayApiKeyRecord) => void;
  onToggleStatus: (item: GatewayApiKeyRecord) => void;
}) {
  if (!items.length) {
    return (
      <div className="rounded-[32px] border border-dashed border-zinc-300 bg-white/80 p-10 text-center shadow-[0_18px_48px_rgba(15,23,42,0.06)] dark:border-zinc-800 dark:bg-zinc-950/50">
        <div className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">No API keys yet</div>
        <p className="mt-2 text-sm text-zinc-500 dark:text-zinc-400">
          Create your first key to connect a client or service to the SDKWork Router gateway.
        </p>
      </div>
    );
  }

  return (
    <div
      data-slot="portal-api-key-table"
      className="overflow-hidden rounded-[32px] border border-zinc-200/80 bg-white/92 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70"
    >
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-zinc-200 dark:divide-zinc-800">
          <thead className="bg-zinc-50/90 dark:bg-zinc-900/80">
            <tr className="text-left text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
              <th className="px-5 py-4">Name</th>
              <th className="px-5 py-4">API key</th>
              <th className="px-5 py-4">Source</th>
              <th className="px-5 py-4">Environment</th>
              <th className="px-5 py-4">Usage</th>
              <th className="px-5 py-4">Expires at</th>
              <th className="px-5 py-4">Status</th>
              <th className="px-5 py-4">Created at</th>
              <th className="px-5 py-4">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
            {items.map((item) => {
              const isLatestCreatedKey = latestCreatedKey?.hashed === item.hashed_key;
              const displayValue = isLatestCreatedKey
                ? latestCreatedKey?.plaintext ?? item.hashed_key
                : item.hashed_key;

              return (
                <tr key={item.hashed_key} className="align-top">
                  <td className="px-5 py-5">
                    <div className="min-w-[16rem]">
                      <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {item.label}
                      </div>
                      {item.notes ? (
                        <div className="mt-2 max-w-[24rem] text-xs leading-6 text-zinc-500 dark:text-zinc-400">
                          {item.notes}
                        </div>
                      ) : null}
                    </div>
                  </td>

                  <td className="px-5 py-5">
                    <div className="flex min-w-[14rem] items-start gap-3">
                      <div className="flex-1 rounded-2xl border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200">
                        {maskValue(displayValue)}
                      </div>
                      {isLatestCreatedKey ? (
                        <button
                          type="button"
                          onClick={onCopyLatestPlaintext}
                          className="inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-white px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
                        >
                          Copy key
                        </button>
                      ) : (
                        <span className="inline-flex h-9 items-center justify-center rounded-2xl border border-primary-500/15 bg-primary-500/10 px-3 text-xs font-semibold text-primary-600 dark:border-primary-500/20 dark:text-primary-300">
                          Write-only
                        </span>
                      )}
                    </div>
                  </td>

                  <td className="px-5 py-5">
                    <span className="inline-flex min-w-[8rem] items-center justify-center rounded-full border border-primary-500/15 bg-primary-500/10 px-3 py-1 text-xs font-semibold text-primary-600 dark:border-primary-500/20 dark:text-primary-300">
                      Portal managed
                    </span>
                  </td>

                  <td className="px-5 py-5">
                    <span className="inline-flex min-w-[8rem] items-center justify-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
                      {item.environment}
                    </span>
                  </td>

                  <td className="px-5 py-5">
                    <div className="min-w-[11rem]">
                      <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatLastUsed(item.last_used_at_ms)}
                      </div>
                      <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
                        Last authenticated use
                      </div>
                      <div className="mt-2 text-xs font-semibold text-primary-500">
                        {describeUsage(item)}
                      </div>
                    </div>
                  </td>

                  <td className="px-5 py-5 text-sm text-zinc-600 dark:text-zinc-300">
                    {formatDate(item.expires_at_ms)}
                  </td>

                  <td className="px-5 py-5">
                    <span
                      className={
                        item.active
                          ? 'inline-flex items-center rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs font-semibold text-emerald-700 dark:text-emerald-300'
                          : 'inline-flex items-center rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-semibold text-amber-700 dark:text-amber-300'
                      }
                    >
                      {item.active ? 'Active' : 'Inactive'}
                    </span>
                  </td>

                  <td className="px-5 py-5 text-sm text-zinc-600 dark:text-zinc-300">
                    {formatDate(item.created_at_ms)}
                  </td>

                  <td className="px-5 py-5">
                    <div className="flex min-w-[17rem] flex-wrap gap-2">
                      <button
                        type="button"
                        onClick={() => onOpenUsage(item)}
                        className="inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-white px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
                      >
                        Usage method
                      </button>
                      <button
                        type="button"
                        disabled={mutatingKey === item.hashed_key}
                        onClick={() => onToggleStatus(item)}
                        className="inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-zinc-50 px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 disabled:opacity-60 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
                      >
                        {item.active ? 'Disable' : 'Enable'}
                      </button>
                      <button
                        type="button"
                        disabled={mutatingKey === item.hashed_key}
                        onClick={() => onDelete(item)}
                        className="inline-flex h-9 items-center justify-center rounded-2xl border border-rose-200 bg-rose-50 px-3 text-sm font-medium text-rose-600 transition hover:bg-rose-100 disabled:opacity-60 dark:border-rose-500/20 dark:bg-rose-500/10 dark:text-rose-300 dark:hover:bg-rose-500/15"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
