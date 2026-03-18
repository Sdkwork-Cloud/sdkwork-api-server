import { formatDateTime } from 'sdkwork-router-portal-commons';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

export function UserProfileFacts({
  workspace,
}: {
  workspace: PortalWorkspaceSummary | null;
}) {
  if (!workspace) {
    return null;
  }

  return (
    <ul className="portalx-fact-list">
      <li>
        <strong>Name</strong>
        <span>{workspace.user.display_name}</span>
      </li>
      <li>
        <strong>Email</strong>
        <span>{workspace.user.email}</span>
      </li>
      <li>
        <strong>Workspace</strong>
        <span>{workspace.tenant.name} / {workspace.project.name}</span>
      </li>
      <li>
        <strong>Created</strong>
        <span>{formatDateTime(workspace.user.created_at_ms)}</span>
      </li>
    </ul>
  );
}
