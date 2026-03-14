import { useEffect, useState } from 'react';
import { simulateRoute } from 'sdkwork-api-admin-sdk';
import type { RoutingSimulationResult } from 'sdkwork-api-types';

const defaultSimulation: RoutingSimulationResult = {
  selected_provider_id: 'n/a',
  candidate_ids: [],
  assessments: [],
};

export function RouteSimulationPage() {
  const [simulation, setSimulation] = useState<RoutingSimulationResult>(defaultSimulation);
  const [status, setStatus] = useState('Running default route simulation for gpt-4.1...');

  useEffect(() => {
    let cancelled = false;

    void simulateRoute('chat_completion', 'gpt-4.1', 11)
      .then((result) => {
        if (!cancelled) {
          setSimulation(result);
          setStatus('Current simulation resolved from catalog-backed routing with a fixed seed.');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Route simulation requires the control plane.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="panel panel-highlight">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Routing</p>
          <h2>Default simulation for `chat_completion:gpt-4.1`</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Selected Provider</span>
          <strong>{simulation.selected_provider_id}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Candidate Count</span>
          <strong>{simulation.candidate_ids.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Strategy</span>
          <strong>{simulation.strategy ?? 'static_fallback'}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Selection Seed</span>
          <strong>{simulation.selection_seed ?? 'n/a'}</strong>
        </article>
      </div>

      <article className="detail-card">
        <h3>Decision Reason</h3>
        <p>{simulation.selection_reason ?? 'No routing explanation returned yet.'}</p>
      </article>

      <article className="detail-card">
        <h3>Candidate Providers</h3>
        <ul className="compact-list">
          {simulation.assessments.map((assessment) => (
            <li key={assessment.provider_id}>
              <div>
                <strong>{assessment.provider_id}</strong>
                <span>
                  {assessment.provider_id === simulation.selected_provider_id ? 'selected' : 'standby'}
                </span>
              </div>
              <div>
                <span>{assessment.available ? 'available' : 'unavailable'}</span>
                <span>{assessment.health}</span>
                <span>policy #{assessment.policy_rank + 1}</span>
                <span>weight {assessment.weight ?? 100}</span>
                {assessment.cost !== undefined ? <span>cost {assessment.cost}</span> : null}
                {assessment.latency_ms !== undefined ? (
                  <span>latency {assessment.latency_ms}ms</span>
                ) : null}
              </div>
              <div>
                {assessment.reasons.length ? assessment.reasons.join(', ') : 'No detailed reasons returned.'}
              </div>
            </li>
          ))}
          {!simulation.assessments.length && (
            <li className="empty">No candidates returned from the admin simulation endpoint.</li>
          )}
        </ul>
      </article>
    </section>
  );
}
