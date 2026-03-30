import { Button, DataTable } from 'sdkwork-router-portal-commons';
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

const secondaryButtonClassName =
  'inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-white px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 disabled:opacity-60 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50';

const subtleButtonClassName =
  'inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-zinc-50 px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 disabled:opacity-60 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-zinc-50';

const dangerButtonClassName =
  'inline-flex h-9 items-center justify-center rounded-2xl border border-rose-200 bg-rose-50 px-3 text-sm font-medium text-rose-600 transition hover:bg-rose-100 disabled:opacity-60 dark:border-rose-500/20 dark:bg-rose-500/10 dark:text-rose-300 dark:hover:bg-rose-500/15';

export function PortalApiKeyTable({
  items,
  latestCreatedKey,
  mutatingKey,
  onCopyLatestPlaintext,
  onCopyPlaintext,
  onDelete,
  onOpenUsage,
  resolvePlaintext,
  onToggleStatus,
}: {
  items: GatewayApiKeyRecord[];
  latestCreatedKey: CreatedGatewayApiKey | null;
  mutatingKey: string | null;
  onCopyLatestPlaintext: () => void;
  onCopyPlaintext: (item: GatewayApiKeyRecord) => void;
  onDelete: (item: GatewayApiKeyRecord) => void;
  onOpenUsage: (item: GatewayApiKeyRecord) => void;
  resolvePlaintext: (item: GatewayApiKeyRecord) => string | null;
  onToggleStatus: (item: GatewayApiKeyRecord) => void;
}) {
  return (
    <div data-slot="portal-api-key-table">
      <DataTable
        columns={[
          {
            key: 'name',
            label: 'Name',
            render: (item) => (
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
            ),
          },
          {
            key: 'key',
            label: 'API key',
            render: (item) => {
              const isLatestCreatedKey = latestCreatedKey?.hashed === item.hashed_key;
              const plaintext = resolvePlaintext(item);
              const hasVisiblePlaintext = Boolean(plaintext);
              const displayValue = plaintext ?? item.hashed_key;

              return (
                <div className="flex min-w-[14rem] items-start gap-3">
                  <div className="flex-1 rounded-2xl border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200">
                    {maskValue(displayValue)}
                  </div>
                  {hasVisiblePlaintext ? (
                    <Button
                      className={secondaryButtonClassName}
                      onClick={() =>
                        isLatestCreatedKey
                          ? onCopyLatestPlaintext()
                          : onCopyPlaintext(item)
                      }
                      type="button"
                      variant="secondary"
                    >
                      Copy key
                    </Button>
                  ) : (
                    <span className="inline-flex h-9 items-center justify-center rounded-2xl border border-primary-500/15 bg-primary-500/10 px-3 text-xs font-semibold text-primary-600 dark:border-primary-500/20 dark:text-primary-300">
                      Write-only
                    </span>
                  )}
                </div>
              );
            },
          },
          {
            key: 'source',
            label: 'Source',
            render: () => (
              <span className="inline-flex min-w-[8rem] items-center justify-center rounded-full border border-primary-500/15 bg-primary-500/10 px-3 py-1 text-xs font-semibold text-primary-600 dark:border-primary-500/20 dark:text-primary-300">
                Portal managed
              </span>
            ),
          },
          {
            key: 'environment',
            label: 'Environment',
            render: (item) => (
              <span className="inline-flex min-w-[8rem] items-center justify-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
                {item.environment}
              </span>
            ),
          },
          {
            key: 'usage',
            label: 'Usage',
            render: (item) => (
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
            ),
          },
          {
            key: 'expires_at',
            label: 'Expires at',
            render: (item) => formatDate(item.expires_at_ms),
          },
          {
            key: 'status',
            label: 'Status',
            render: (item) => (
              <span
                className={
                  item.active
                    ? 'inline-flex items-center rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs font-semibold text-emerald-700 dark:text-emerald-300'
                    : 'inline-flex items-center rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-semibold text-amber-700 dark:text-amber-300'
                }
              >
                {item.active ? 'Active' : 'Inactive'}
              </span>
            ),
          },
          {
            key: 'created_at',
            label: 'Created at',
            render: (item) => formatDate(item.created_at_ms),
          },
          {
            key: 'actions',
            label: 'Actions',
            render: (item) => (
              <div className="flex min-w-[17rem] flex-wrap gap-2">
                <Button
                  className={secondaryButtonClassName}
                  onClick={() => onOpenUsage(item)}
                  type="button"
                  variant="secondary"
                >
                  Usage method
                </Button>
                <Button
                  className={subtleButtonClassName}
                  disabled={mutatingKey === item.hashed_key}
                  onClick={() => onToggleStatus(item)}
                  type="button"
                  variant="secondary"
                >
                  {item.active ? 'Disable' : 'Enable'}
                </Button>
                <Button
                  className={dangerButtonClassName}
                  disabled={mutatingKey === item.hashed_key}
                  onClick={() => onDelete(item)}
                  type="button"
                  variant="destructive"
                >
                  Delete
                </Button>
              </div>
            ),
          },
        ]}
        empty={(
          <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
            <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
              No API keys yet
            </strong>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              Create your first key to connect a client or service to the SDKWork Router gateway.
            </p>
          </div>
        )}
        getKey={(item) => item.hashed_key}
        rows={items}
      />
    </div>
  );
}
