import { Pill, StatCard, Surface } from 'sdkwork-router-admin-commons';
import type { AdminPageProps, AdminRouteKey } from 'sdkwork-router-admin-types';

import { buildAdminOverviewViewModel } from './view-model';

export function OverviewPage({
  snapshot,
  onNavigate: _onNavigate,
}: AdminPageProps & { onNavigate: (route: AdminRouteKey) => void }) {
  const viewModel = buildAdminOverviewViewModel(snapshot);
  const rankedUsers = viewModel.rankedUsers;
  const rankedProjects = viewModel.rankedProjects;

  return (
    <div className="adminx-page-grid">
      <section className="adminx-stat-grid">
        {viewModel.metrics.map((metric) => (
          <StatCard
            key={metric.label}
            label={metric.label}
            value={metric.value}
            detail={metric.detail}
          />
        ))}
      </section>

      <Surface
        title="Operator alerts"
        detail="Alerts are generated from live billing, runtime, catalog, and workspace health signals from the control plane."
      >
        <div className="adminx-card-grid">
          {viewModel.alerts.map((alert) => (
            <article key={alert.id} className="adminx-mini-card">
              <div className="adminx-row">
                <strong>{alert.title}</strong>
                <Pill tone={alert.severity === 'high' ? 'danger' : 'default'}>
                  {alert.severity}
                </Pill>
              </div>
              <p>{alert.detail}</p>
            </article>
          ))}
        </div>
      </Surface>

      <div className="adminx-users-grid">
        <Surface
          title="Top portal users"
          detail="Portal identities ranked by request count, then token consumption and metered usage."
        >
          <div className="adminx-card-grid">
            {rankedUsers.map((user) => (
              <article key={user.id} className="adminx-mini-card">
                <div className="adminx-row">
                  <strong>{user.display_name}</strong>
                  <Pill tone={user.active ? 'live' : 'danger'}>
                    {user.active ? 'active' : 'disabled'}
                  </Pill>
                </div>
                <p>{user.email}</p>
                <p>
                  Requests: {user.request_count}
                  {' | '}
                  Tokens: {user.total_tokens}
                  {' | '}
                  Units: {user.usage_units}
                </p>
              </article>
            ))}
          </div>
        </Surface>

        <Surface
          title="Hottest projects"
          detail="Projects with the highest traffic and spend signals across usage and billing summaries."
        >
          <div className="adminx-card-grid">
            {rankedProjects.map((project) => (
              <article key={project.id} className="adminx-mini-card">
                <div className="adminx-row">
                  <strong>{project.name}</strong>
                  <Pill tone="default">{project.id}</Pill>
                </div>
                <p>{project.tenant_id}</p>
                <p>
                  Requests: {project.request_count}
                  {' | '}
                  Tokens: {project.total_tokens}
                  {' | '}
                  Amount: {project.booked_amount.toFixed(2)}
                </p>
              </article>
            ))}
          </div>
        </Surface>
      </div>
    </div>
  );
}
