import { formatCurrency, formatUnits } from 'sdkwork-router-portal-commons';
import type { PortalWorkspaceSummary, ProjectBillingSummary } from 'sdkwork-router-portal-types';

export function AccountBalanceFacts({
  workspace,
  summary,
}: {
  workspace: PortalWorkspaceSummary | null;
  summary: ProjectBillingSummary | null;
}) {
  if (!workspace || !summary) {
    return null;
  }

  return (
    <ul className="portalx-fact-list">
      <li>
        <strong>Project</strong>
        <span>{workspace.project.name}</span>
      </li>
      <li>
        <strong>Tenant</strong>
        <span>{workspace.tenant.name}</span>
      </li>
      <li>
        <strong>Remaining units</strong>
        <span>
          {summary.remaining_units === null || summary.remaining_units === undefined
            ? 'Unlimited'
            : formatUnits(summary.remaining_units)}
        </span>
      </li>
      <li>
        <strong>Booked amount</strong>
        <span>{formatCurrency(summary.booked_amount)}</span>
      </li>
    </ul>
  );
}
