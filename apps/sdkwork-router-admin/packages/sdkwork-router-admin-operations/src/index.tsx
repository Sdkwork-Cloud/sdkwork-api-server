import { useState } from 'react';
import type { FormEvent } from 'react';

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
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, RuntimeReloadReport } from 'sdkwork-router-admin-types';

function formatTimestamp(timestamp: number): string {
  if (!timestamp) {
    return '-';
  }

  return new Date(timestamp).toLocaleString();
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
  const healthyProviders = snapshot.providerHealth.filter((snapshotItem) => snapshotItem.healthy).length;
  const healthyRuntimes = snapshot.runtimeStatuses.filter((runtime) => runtime.healthy).length;
  const [reloadDraft, setReloadDraft] = useState({
    extension_id: '',
    instance_id: '',
  });
  const [isReloadDialogOpen, setIsReloadDialogOpen] = useState(false);
  const [lastReloadReport, setLastReloadReport] = useState<RuntimeReloadReport | null>(null);

  async function handleReload(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const report = await onReloadRuntimes({
      extension_id: reloadDraft.extension_id.trim() || undefined,
      instance_id: reloadDraft.instance_id.trim() || undefined,
    });
    setLastReloadReport(report);
    setIsReloadDialogOpen(false);
  }

  function resetReloadDialog() {
    setIsReloadDialogOpen(false);
    setReloadDraft({
      extension_id: '',
      instance_id: '',
    });
  }

  return (
    <div className="adminx-page-grid">
      <section className="adminx-stat-grid">
        <StatCard
          label="Provider health snapshots"
          value={String(snapshot.providerHealth.length)}
          detail="Recent live provider-health records."
        />
        <StatCard
          label="Healthy providers"
          value={String(healthyProviders)}
          detail="Providers currently marked healthy."
        />
        <StatCard
          label="Runtime statuses"
          value={String(snapshot.runtimeStatuses.length)}
          detail="Managed runtime and connector status records."
        />
        <StatCard
          label="Healthy runtimes"
          value={String(healthyRuntimes)}
          detail="Runtimes reporting healthy state."
        />
      </section>

      <PageToolbar
        title="Runtime intervention workbench"
        detail="Leave monitoring on the canvas, and open a focused dialog only when a targeted reload needs a specific extension or instance scope."
        actions={(
          <>
            <InlineButton
              tone="primary"
              onClick={() => void onReloadRuntimes().then(setLastReloadReport)}
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
        <div className="adminx-form-grid">
          <div className="adminx-note">
            <strong>Latest reload report</strong>
            <p>
              {lastReloadReport
                ? `Scope: ${lastReloadReport.scope} | Active runtimes: ${lastReloadReport.active_runtime_count} | Loadable packages: ${lastReloadReport.loadable_package_count} | Reloaded: ${formatTimestamp(lastReloadReport.reloaded_at_ms)}`
                : 'Last reload report is not available yet.'}
            </p>
          </div>
          <div className="adminx-note">
            <strong>Runtime reload behavior</strong>
            <p>Use the targeted dialog for narrow intervention, and use the toolbar reload action when the whole runtime mesh should be refreshed.</p>
          </div>
        </div>
      </PageToolbar>

      <div className="adminx-users-grid">
        <Surface
          title="Runtime posture"
          detail="Runtime states are refreshed after every reload so operators can immediately confirm outcome."
        >
          <div className="adminx-card-grid">
            {snapshot.runtimeStatuses.map((runtime) => (
              <article
                key={`${runtime.runtime}:${runtime.extension_id}:${runtime.instance_id ?? 'global'}`}
                className="adminx-mini-card"
              >
                <div className="adminx-row">
                  <strong>{runtime.display_name}</strong>
                  <Pill tone={runtime.healthy ? 'live' : 'danger'}>
                    {runtime.healthy ? 'healthy' : 'attention'}
                  </Pill>
                </div>
                <p>{runtime.extension_id}</p>
                <p>
                  Running: {String(runtime.running)}
                  {' | '}
                  Instance: {runtime.instance_id ?? 'global'}
                </p>
              </article>
            ))}
          </div>
        </Surface>

        <Surface
          title="Intervention guidance"
          detail="Focused dialog actions keep runtime monitoring stable while you decide whether to escalate."
        >
          <div className="adminx-card-grid">
            <article className="adminx-mini-card">
              <div className="adminx-row">
                <strong>Prefer narrow reloads first</strong>
                <Pill tone="seed">safe</Pill>
              </div>
              <p>Start with extension or instance scope when only one runtime looks degraded.</p>
            </article>
            <article className="adminx-mini-card">
              <div className="adminx-row">
                <strong>Read health before intervening</strong>
                <Pill tone="live">observe</Pill>
              </div>
              <p>Provider and runtime tables remain primary so the operator can compare signals before taking action.</p>
            </article>
          </div>
        </Surface>
      </div>

      <Surface title="Provider health" detail="Latest provider-health snapshots from the admin API.">
        <DataTable
          columns={[
            { key: 'provider', label: 'Provider', render: (item) => <strong>{item.provider_id}</strong> },
            { key: 'status', label: 'Status', render: (item) => item.status },
            {
              key: 'healthy',
              label: 'Healthy',
              render: (item) => (
                <Pill tone={item.healthy ? 'live' : 'danger'}>
                  {item.healthy ? 'healthy' : 'attention'}
                </Pill>
              ),
            },
            { key: 'message', label: 'Message', render: (item) => item.message ?? '-' },
          ]}
          rows={snapshot.providerHealth}
          empty="No provider health data available."
          getKey={(item) => `${item.provider_id}:${item.observed_at_ms}`}
        />
      </Surface>

      <Surface title="Managed runtimes" detail="Runtime status and extension-health view.">
        <DataTable
          columns={[
            { key: 'display', label: 'Runtime', render: (runtime) => <strong>{runtime.display_name}</strong> },
            { key: 'family', label: 'Family', render: (runtime) => runtime.runtime },
            { key: 'instance', label: 'Instance', render: (runtime) => runtime.instance_id ?? runtime.extension_id },
            { key: 'running', label: 'Running', render: (runtime) => String(runtime.running) },
            {
              key: 'healthy',
              label: 'Healthy',
              render: (runtime) => (
                <Pill tone={runtime.healthy ? 'live' : 'danger'}>
                  {runtime.healthy ? 'healthy' : 'attention'}
                </Pill>
              ),
            },
            { key: 'message', label: 'Message', render: (runtime) => runtime.message ?? '-' },
          ]}
          rows={snapshot.runtimeStatuses}
          empty="No runtime statuses available."
          getKey={(runtime) => `${runtime.runtime}:${runtime.extension_id}:${runtime.instance_id ?? 'global'}`}
        />
      </Surface>
    </div>
  );
}
