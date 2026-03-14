import { useEffect, useState } from 'react';
import { getBillingSummary, getUsageSummary } from 'sdkwork-api-admin-sdk';
import type { BillingSummary, ProjectBillingSummary, UsageSummary } from 'sdkwork-api-types';

interface UsageSnapshot {
  usage: UsageSummary;
  billing: BillingSummary;
}

const emptySnapshot: UsageSnapshot = {
  usage: {
    total_requests: 0,
    project_count: 0,
    model_count: 0,
    provider_count: 0,
    projects: [],
    providers: [],
    models: [],
  },
  billing: {
    total_entries: 0,
    project_count: 0,
    total_units: 0,
    total_amount: 0,
    active_quota_policy_count: 0,
    exhausted_project_count: 0,
    projects: [],
  },
};

function formatAmount(amount: number): string {
  return amount.toFixed(2);
}

function billingPosture(project: ProjectBillingSummary): string {
  if (project.quota_limit_units === undefined) {
    return `${project.used_units} units / no active quota`;
  }

  return `${project.used_units} of ${project.quota_limit_units} units / ${project.exhausted ? 'exhausted' : `${project.remaining_units ?? 0} remaining`}`;
}

export function RequestExplorerPage() {
  const [snapshot, setSnapshot] = useState<UsageSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Loading request telemetry...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([getUsageSummary(), getBillingSummary()])
      .then(([usage, billing]) => {
        if (cancelled) {
          return;
        }

        setSnapshot({ usage, billing });
        setStatus('Backend-owned usage and billing summaries are streaming from admin APIs.');
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Usage explorer is in offline mode.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Usage Ledger</p>
          <h2>Recent requests and booked cost</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Usage Requests</span>
          <strong>{snapshot.usage.total_requests}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Ledger Entries</span>
          <strong>{snapshot.billing.total_entries}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Booked Amount</span>
          <strong>{formatAmount(snapshot.billing.total_amount)}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Active Quota Policies</span>
          <strong>{snapshot.billing.active_quota_policy_count}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Exhausted Projects</span>
          <strong>{snapshot.billing.exhausted_project_count}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Tracked Models</span>
          <strong>{snapshot.usage.model_count}</strong>
        </article>
      </div>

      <div className="detail-grid">
        <article className="detail-card">
          <h3>Usage By Project</h3>
          <ul className="compact-list">
            {snapshot.usage.projects.map((project) => (
              <li key={project.project_id}>
                <strong>{project.project_id}</strong>
                <span>{project.request_count} requests</span>
              </li>
            ))}
            {!snapshot.usage.projects.length && (
              <li className="empty">No gateway requests have been recorded yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Usage By Provider</h3>
          <ul className="compact-list">
            {snapshot.usage.providers.map((provider) => (
              <li key={provider.provider}>
                <strong>{provider.provider}</strong>
                <span>
                  {provider.request_count} requests / {provider.project_count} projects
                </span>
              </li>
            ))}
            {!snapshot.usage.providers.length && (
              <li className="empty">No provider traffic has been recorded yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Usage By Model</h3>
          <ul className="compact-list">
            {snapshot.usage.models.map((model) => (
              <li key={model.model}>
                <strong>{model.model}</strong>
                <span>
                  {model.request_count} requests / {model.provider_count} providers
                </span>
              </li>
            ))}
            {!snapshot.usage.models.length && (
              <li className="empty">No model traffic has been recorded yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Billing Posture</h3>
          <ul className="compact-list">
            {snapshot.billing.projects.map((project) => (
              <li key={project.project_id}>
                <strong>{project.project_id}</strong>
                <span>{billingPosture(project)}</span>
              </li>
            ))}
            {!snapshot.billing.projects.length && (
              <li className="empty">No billing posture is available yet.</li>
            )}
          </ul>
        </article>
      </div>
    </section>
  );
}
