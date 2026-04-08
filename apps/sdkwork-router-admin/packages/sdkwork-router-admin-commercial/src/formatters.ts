import type { CommercePaymentEventRecord } from 'sdkwork-router-admin-types';
import type { CommercialLedgerTimelineRow } from './ledgerTimeline';
import type { CommercialOrderPaymentAuditRow } from './orderPaymentAudit';

export type CommercialFact = {
  label: string;
  value: string;
  detail: string;
  tone?: 'success' | 'warning' | 'secondary';
};

export type CommercialSummaryMetric = {
  label: string;
  value: string;
  description: string;
};

export function formatStatusLabel(value: string) {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length > 1
        ? `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`
        : segment.toUpperCase())
    .join(' ');
}

export function formatLedgerEntryTypeLabel(
  value: CommercialLedgerTimelineRow['entry_type'],
) {
  return formatStatusLabel(value);
}

export function formatOrderAuditEventLabel(
  row: CommercialOrderPaymentAuditRow,
) {
  if (row.event_type) {
    return formatStatusLabel(row.event_type);
  }

  return formatStatusLabel(row.order_status);
}

export function latestObservedPaymentEvent(
  paymentEvents: CommercePaymentEventRecord[],
) {
  return [...paymentEvents].sort((left, right) =>
    (right.processed_at_ms ?? right.received_at_ms)
    - (left.processed_at_ms ?? left.received_at_ms)
    || right.payment_event_id.localeCompare(left.payment_event_id),
  )[0] ?? null;
}
