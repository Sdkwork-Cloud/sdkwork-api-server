import { rootSections } from 'sdkwork-api-core';

export function WorkspaceDashboard() {
  return (
    <section>
      <h2>{rootSections[0]?.title ?? 'Workspace'}</h2>
      <p>Tenant, project, member, and API key controls will appear here.</p>
    </section>
  );
}

