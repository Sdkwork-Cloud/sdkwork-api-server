import { useDeferredValue, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';

import {
  AdminDialog,
  DataTable,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogTrigger,
  FormField,
  InlineButton,
  PageToolbar,
  Pill,
  ToolbarDisclosure,
  ToolbarField,
  ToolbarSearchField,
  useAdminI18n,
} from 'sdkwork-router-admin-commons';
import type {
  AdminPageProps,
  ProviderHealthSnapshot,
  RuntimeReloadReport,
  RuntimeStatusRecord,
} from 'sdkwork-router-admin-types';

type ViewMode = 'providers' | 'runtimes';

type ProviderHealthRow = ProviderHealthSnapshot & {
  kind: 'providers';
};

type RuntimeStatusRow = RuntimeStatusRecord & {
  kind: 'runtimes';
};

type OperationsTableRow = ProviderHealthRow | RuntimeStatusRow;

function rowKey(row: OperationsTableRow): string {
  if (row.kind === 'providers') {
    return `${row.provider_id}:${row.observed_at_ms}`;
  }

  return `${row.runtime}:${row.extension_id}:${row.instance_id ?? 'global'}`;
}

export function OperationsPage({
  snapshot,
  onReloadRuntimes,
}: AdminPageProps & {
  onReloadRuntimes: (input?: {
    extension_id?: string;
    instance_id?: string;
  }) => Promise<RuntimeReloadReport>;
}) {
  const { t } = useAdminI18n();
  const [reloadDraft, setReloadDraft] = useState({
    extension_id: '',
    instance_id: '',
  });
  const [search, setSearch] = useState('');
  const [viewMode, setViewMode] = useState<ViewMode>('runtimes');
  const [isReloadDialogOpen, setIsReloadDialogOpen] = useState(false);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  async function handleReload(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onReloadRuntimes({
      extension_id: reloadDraft.extension_id.trim() || undefined,
      instance_id: reloadDraft.instance_id.trim() || undefined,
    });
    setIsReloadDialogOpen(false);
  }

  function resetReloadDialog() {
    setIsReloadDialogOpen(false);
    setReloadDraft({
      extension_id: '',
      instance_id: '',
    });
  }

  const providerRows: ProviderHealthRow[] = snapshot.providerHealth
    .filter((item) => {
      if (!deferredQuery) {
        return true;
      }

      return [
        item.provider_id,
        item.status,
        item.message ?? '',
        item.healthy ? 'healthy' : 'attention',
      ]
        .join(' ')
        .toLowerCase()
        .includes(deferredQuery);
    })
    .map((item) => ({
      ...item,
      kind: 'providers' as const,
    }));

  const runtimeRows: RuntimeStatusRow[] = snapshot.runtimeStatuses
    .filter((runtime) => {
      if (!deferredQuery) {
        return true;
      }

      return [
        runtime.display_name,
        runtime.runtime,
        runtime.instance_id ?? '',
        runtime.extension_id,
        runtime.message ?? '',
        runtime.healthy ? 'healthy' : 'attention',
        runtime.running ? 'running' : 'stopped',
      ]
        .join(' ')
        .toLowerCase()
        .includes(deferredQuery);
    })
    .map((runtime) => ({
      ...runtime,
      kind: 'runtimes' as const,
    }));

  let rows: OperationsTableRow[] = runtimeRows;
  let columns: Array<{ key: string; label: string; render: (row: OperationsTableRow) => ReactNode }> = [
    { key: 'display', label: 'Runtime', render: (row) => row.kind === 'runtimes' ? <strong>{row.display_name}</strong> : '-' },
    { key: 'family', label: 'Family', render: (row) => row.kind === 'runtimes' ? row.runtime : '-' },
    { key: 'instance', label: 'Instance', render: (row) => row.kind === 'runtimes' ? row.instance_id ?? row.extension_id : '-' },
    { key: 'running', label: 'Running', render: (row) => row.kind === 'runtimes' ? String(row.running) : '-' },
    {
      key: 'healthy',
      label: 'Healthy',
      render: (row) => row.kind === 'runtimes' ? (
        <Pill tone={row.healthy ? 'live' : 'danger'}>
          {row.healthy ? 'healthy' : 'attention'}
        </Pill>
      ) : '-',
    },
    { key: 'message', label: 'Message', render: (row) => row.kind === 'runtimes' ? row.message ?? '-' : '-' },
  ];
  let empty = t('No runtime statuses available.');

  if (viewMode === 'providers') {
    rows = providerRows;
    columns = [
      { key: 'provider', label: 'Provider', render: (row) => row.kind === 'providers' ? <strong>{row.provider_id}</strong> : '-' },
      { key: 'status', label: 'Status', render: (row) => row.kind === 'providers' ? row.status : '-' },
      {
        key: 'healthy',
        label: 'Healthy',
        render: (row) => row.kind === 'providers' ? (
          <Pill tone={row.healthy ? 'live' : 'danger'}>
            {row.healthy ? 'healthy' : 'attention'}
          </Pill>
        ) : '-',
      },
      { key: 'message', label: 'Message', render: (row) => row.kind === 'providers' ? row.message ?? '-' : '-' },
    ];
    empty = t('No provider health data available.');
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <>
            <InlineButton
              tone="primary"
              onClick={() => void onReloadRuntimes()}
            >
              Reload runtimes
            </InlineButton>
            <Dialog
              open={isReloadDialogOpen}
              onOpenChange={(nextOpen) => {
                if (!nextOpen) {
                  resetReloadDialog();
                  return;
                }
                setIsReloadDialogOpen(true);
              }}
            >
              <DialogTrigger asChild>
                <InlineButton onClick={() => setIsReloadDialogOpen(true)}>
                  Targeted reload
                </InlineButton>
              </DialogTrigger>
              <DialogContent size="medium">
                <AdminDialog
                  title="Targeted reload"
                  detail="Use a narrow runtime reload when you need a smaller blast radius than the full control-plane refresh."
                >
                  <form className="adminx-form-grid" onSubmit={(event) => void handleReload(event)}>
                    <FormField label="Extension id" hint="Leave blank to keep the reload scope open.">
                      <input
                        value={reloadDraft.extension_id}
                        onChange={(event) =>
                          setReloadDraft((current) => ({
                            ...current,
                            extension_id: event.target.value,
                          }))}
                        placeholder="optional extension id"
                      />
                    </FormField>
                    <FormField label="Instance id" hint="Use this only when one runtime instance needs intervention.">
                      <input
                        value={reloadDraft.instance_id}
                        onChange={(event) =>
                          setReloadDraft((current) => ({
                            ...current,
                            instance_id: event.target.value,
                          }))}
                        placeholder="optional instance id"
                      />
                    </FormField>
                    <DialogFooter>
                      <InlineButton onClick={resetReloadDialog}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">
                        Run targeted reload
                      </InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>
          </>
        )}
      >
        <>
          <ToolbarSearchField
            label={t('Search operations')}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="provider, runtime, instance, message"
          />
          <ToolbarDisclosure>
            <div className="adminx-form-grid">
              <ToolbarField label={t('View mode')}>
                <select
                  value={viewMode}
                  onChange={(event) => setViewMode(event.target.value as ViewMode)}
                >
                  <option value="runtimes">{t('Managed runtimes')}</option>
                  <option value="providers">{t('Provider health')}</option>
                </select>
              </ToolbarField>
            </div>
          </ToolbarDisclosure>
        </>
      </PageToolbar>

      <DataTable
        columns={columns}
        rows={rows}
        empty={empty}
        getKey={(row) => rowKey(row)}
      />
    </div>
  );
}
