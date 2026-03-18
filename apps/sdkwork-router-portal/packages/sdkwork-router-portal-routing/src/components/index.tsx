import { InlineButton, Pill } from 'sdkwork-router-portal-commons';
import type { PortalRoutingDecisionLog, PortalRoutingProviderOption } from 'sdkwork-router-portal-types';

import type { RoutingEvidenceItem, RoutingPresetCard } from '../types';

export function RoutingPresetGrid({
  presets,
  onApply,
}: {
  presets: RoutingPresetCard[];
  onApply: (presetId: string) => void;
}) {
  return (
    <div className="portalx-checklist-grid">
      {presets.map((preset) => (
        <article className="portalx-checklist-card" key={preset.id}>
          <div className="portalx-status-row">
            <strong>{preset.title}</strong>
            <Pill tone={preset.active ? 'positive' : 'default'}>
              {preset.active ? 'Active' : 'Available'}
            </Pill>
          </div>
          <p>{preset.detail}</p>
          <InlineButton onClick={() => onApply(preset.id)} tone="ghost">
            Apply preset
          </InlineButton>
        </article>
      ))}
    </div>
  );
}

export function RoutingProviderList({
  providers,
  onMove,
  onDefault,
}: {
  providers: PortalRoutingProviderOption[];
  onMove: (providerId: string, direction: 'up' | 'down') => void;
  onDefault: (providerId: string) => void;
}) {
  return (
    <div className="portalx-checklist-grid">
      {providers.map((provider, index) => (
        <article className="portalx-checklist-card" key={provider.provider_id}>
          <div className="portalx-status-row">
            <strong>{provider.display_name}</strong>
            <Pill tone={provider.default_provider ? 'accent' : provider.preferred ? 'positive' : 'default'}>
              {provider.default_provider ? 'Default' : provider.preferred ? 'Ordered' : provider.channel_id}
            </Pill>
          </div>
          <p>{provider.provider_id}</p>
          <div className="portalx-form-actions">
            <InlineButton disabled={index === 0} onClick={() => onMove(provider.provider_id, 'up')} tone="ghost">
              Move up
            </InlineButton>
            <InlineButton
              disabled={index === providers.length - 1}
              onClick={() => onMove(provider.provider_id, 'down')}
              tone="ghost"
            >
              Move down
            </InlineButton>
            <InlineButton onClick={() => onDefault(provider.provider_id)} tone="secondary">
              Set default
            </InlineButton>
          </div>
        </article>
      ))}
    </div>
  );
}

export function RoutingEvidenceList({
  evidence,
  logs,
}: {
  evidence: RoutingEvidenceItem[];
  logs: PortalRoutingDecisionLog[];
}) {
  return (
    <div className="portalx-guardrail-list">
      {evidence.map((item, index) => (
        <article className="portalx-guardrail-card" key={item.id}>
          <div className="portalx-status-row">
            <strong>{item.title}</strong>
            <Pill tone="default">{item.timestamp_label}</Pill>
          </div>
          <p>{item.detail}</p>
          {logs[index] ? (
            <small>
              {logs[index].selection_reason ?? 'Selection evidence is available from the current decision log.'}
            </small>
          ) : null}
        </article>
      ))}
    </div>
  );
}
