import { useEffect, useState } from 'react';
import { listProviderHealthSnapshots } from 'sdkwork-api-admin-sdk';
import type { ProviderHealthSnapshot, RuntimeMode } from 'sdkwork-api-types';

const activeMode: RuntimeMode = 'embedded';

function formatObservedAt(observedAtMs: number): string {
  return new Date(observedAtMs).toLocaleString();
}

function latestSnapshotsByProvider(snapshots: ProviderHealthSnapshot[]): ProviderHealthSnapshot[] {
  const latest = new Map<string, ProviderHealthSnapshot>();
  for (const snapshot of snapshots) {
    if (!latest.has(snapshot.provider_id)) {
      latest.set(snapshot.provider_id, snapshot);
    }
  }
  return Array.from(latest.values());
}

export function RuntimeStatusPage() {
  const [snapshots, setSnapshots] = useState<ProviderHealthSnapshot[]>([]);
  const [status, setStatus] = useState('Loading persisted runtime health snapshots...');

  useEffect(() => {
    let cancelled = false;

    void listProviderHealthSnapshots()
      .then((result) => {
        if (cancelled) {
          return;
        }

        setSnapshots(result);
        setStatus('Runtime supervision is feeding persisted provider health evidence into routing.');
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Runtime page is showing static deployment posture only.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const latestByProvider = latestSnapshotsByProvider(snapshots);
  const healthyProviders = latestByProvider.filter((snapshot) => snapshot.healthy).length;
  const runtimeFamilies = new Set(snapshots.map((snapshot) => snapshot.runtime)).size;
  const lastObservation = snapshots[0]?.observed_at_ms;

  return (
    <section className="panel panel-accent">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Runtime</p>
          <h2>Host mode, packaging posture, and persisted health telemetry</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Active Mode</span>
          <strong>{activeMode}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Preferred Local Store</span>
          <strong>SQLite</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Desktop Host</span>
          <strong>Tauri</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Healthy Providers</span>
          <strong>{healthyProviders}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Runtime Families</span>
          <strong>{runtimeFamilies || 'n/a'}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Last Observation</span>
          <strong>{lastObservation ? formatObservedAt(lastObservation) : 'n/a'}</strong>
        </article>
      </div>

      <div className="detail-grid">
        <article className="detail-card">
          <h3>Latest Provider Health</h3>
          <ul className="compact-list">
            {latestByProvider.map((snapshot) => (
              <li key={snapshot.provider_id}>
                <strong>{snapshot.provider_id}</strong>
                <span>
                  {snapshot.healthy ? 'healthy' : 'unhealthy'} / {snapshot.runtime}
                </span>
              </li>
            ))}
            {!latestByProvider.length && (
              <li className="empty">No persisted runtime health snapshots have been captured yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Recent Snapshot History</h3>
          <ul className="compact-list">
            {snapshots.slice(0, 6).map((snapshot, index) => (
              <li key={`${snapshot.provider_id}:${snapshot.observed_at_ms}:${index}`}>
                <strong>{snapshot.instance_id ?? snapshot.provider_id}</strong>
                <span>
                  {formatObservedAt(snapshot.observed_at_ms)} / {snapshot.running ? 'running' : 'stopped'}
                  {' / '}
                  {snapshot.message ?? (snapshot.healthy ? 'healthy' : 'unhealthy')}
                </span>
              </li>
            ))}
            {!snapshots.length && (
              <li className="empty">
                Snapshot history will appear after the standalone supervisor or embedded host captures runtime state.
              </li>
            )}
          </ul>
        </article>
      </div>
    </section>
  );
}
