import { useState } from 'react';
import type { FormEvent } from 'react';

import {
  AdminDialog,
  ConfirmDialog,
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
import type { AdminPageProps, CreatedGatewayApiKey } from 'sdkwork-router-admin-types';

export function TenantsPage({
  snapshot,
  onSaveTenant,
  onSaveProject,
  onCreateApiKey,
  onUpdateApiKeyStatus,
  onDeleteApiKey,
  onDeleteTenant,
  onDeleteProject,
}: AdminPageProps & {
  onSaveTenant: (input: { id: string; name: string }) => Promise<void>;
  onSaveProject: (input: { tenant_id: string; id: string; name: string }) => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
  }) => Promise<CreatedGatewayApiKey>;
  onUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  onDeleteApiKey: (hashedKey: string) => Promise<void>;
  onDeleteTenant: (tenantId: string) => Promise<void>;
  onDeleteProject: (projectId: string) => Promise<void>;
}) {
  const [tenantDraft, setTenantDraft] = useState({ id: '', name: '' });
  const [projectDraft, setProjectDraft] = useState({
    tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
    id: '',
    name: '',
  });
  const [apiKeyDraft, setApiKeyDraft] = useState({
    tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
    project_id: snapshot.projects[0]?.id ?? 'project_local_demo',
    environment: 'production',
  });
  const [isTenantDialogOpen, setIsTenantDialogOpen] = useState(false);
  const [isProjectDialogOpen, setIsProjectDialogOpen] = useState(false);
  const [isApiKeyDialogOpen, setIsApiKeyDialogOpen] = useState(false);
  const [revealedApiKey, setRevealedApiKey] = useState<CreatedGatewayApiKey | null>(null);
  const [pendingDelete, setPendingDelete] = useState<
    | { kind: 'tenant'; id: string; label: string }
    | { kind: 'project'; id: string; label: string }
    | { kind: 'key'; id: string; label: string }
    | null
  >(null);

  const selectedProjectUsage = snapshot.usageSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectTokens = snapshot.usageRecords
    .filter((record) => record.project_id === projectDraft.id)
    .reduce((sum, record) => sum + record.total_tokens, 0);
  const portalProjects = new Set(
    snapshot.portalUsers
      .map((user) => user.workspace_project_id)
      .filter((projectId): projectId is string => Boolean(projectId)),
  );
  const portalTenants = new Set(
    snapshot.portalUsers
      .map((user) => user.workspace_tenant_id)
      .filter((tenantId): tenantId is string => Boolean(tenantId)),
  );

  function resetTenantDialog() {
    setTenantDraft({ id: '', name: '' });
    setIsTenantDialogOpen(false);
  }

  function resetProjectDialog() {
    setProjectDraft({
      tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
      id: '',
      name: '',
    });
    setIsProjectDialogOpen(false);
  }

  function resetApiKeyDialog() {
    setApiKeyDraft({
      tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
      project_id: snapshot.projects[0]?.id ?? 'project_local_demo',
      environment: 'production',
    });
    setIsApiKeyDialogOpen(false);
  }

  async function handleTenantSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveTenant(tenantDraft);
    resetTenantDialog();
  }

  async function handleProjectSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveProject(projectDraft);
    resetProjectDialog();
  }

  async function handleApiKeySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const created = await onCreateApiKey(apiKeyDraft);
    setRevealedApiKey(created);
    setIsApiKeyDialogOpen(false);
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    if (pendingDelete.kind === 'tenant') {
      await onDeleteTenant(pendingDelete.id);
    } else if (pendingDelete.kind === 'project') {
      await onDeleteProject(pendingDelete.id);
    } else {
      await onDeleteApiKey(pendingDelete.id);
    }

    setPendingDelete(null);
  }

  const availableApiKeyProjects = snapshot.projects.filter(
    (project) => project.tenant_id === apiKeyDraft.tenant_id,
  );

  return (
    <div className="adminx-page-grid">
      <section className="adminx-stat-grid">
        <StatCard
          label="Tenants"
          value={String(snapshot.tenants.length)}
          detail="Distinct tenant workspaces managed by the router."
        />
        <StatCard
          label="Projects"
          value={String(snapshot.projects.length)}
          detail="Projects linked to tenants and routing or billing posture."
        />
        <StatCard
          label="Gateway keys"
          value={String(snapshot.apiKeys.length)}
          detail="Issued gateway keys across environments."
        />
      </section>

      <PageToolbar
        title="Workspace operations"
        detail="Keep the registries readable, then open a focused dialog when you need to create a tenant, create a project, or mint a gateway credential."
        actions={(
          <>
            <Dialog open={isTenantDialogOpen} onOpenChange={setIsTenantDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton tone="primary" onClick={() => setIsTenantDialogOpen(true)}>
                  New tenant
                </InlineButton>
              </DialogTrigger>
              <DialogContent size="medium">
                <AdminDialog
                  title={tenantDraft.id ? 'Edit tenant' : 'New tenant'}
                  detail="Tenant creation and editing happen in a dedicated dialog so the registry stays primary on the page."
                >
                  <form className="adminx-form-grid" onSubmit={(event) => void handleTenantSubmit(event)}>
                    <FormField label="Tenant id">
                      <input
                        value={tenantDraft.id}
                        onChange={(event) => setTenantDraft((current) => ({ ...current, id: event.target.value }))}
                        required
                      />
                    </FormField>
                    <FormField label="Tenant name">
                      <input
                        value={tenantDraft.name}
                        onChange={(event) => setTenantDraft((current) => ({ ...current, name: event.target.value }))}
                        required
                      />
                    </FormField>
                    <DialogFooter>
                      <InlineButton onClick={resetTenantDialog}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">
                        {tenantDraft.id ? 'Save tenant' : 'Create tenant'}
                      </InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>

            <Dialog open={isProjectDialogOpen} onOpenChange={setIsProjectDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton onClick={() => setIsProjectDialogOpen(true)}>
                  New project
                </InlineButton>
              </DialogTrigger>
              <DialogContent size="medium">
                <AdminDialog
                  title={projectDraft.id ? 'Edit project' : 'New project'}
                  detail="Projects remain the routing, usage, and billing ownership boundary, so edits belong in their own dialog."
                >
                  <form className="adminx-form-grid" onSubmit={(event) => void handleProjectSubmit(event)}>
                    <FormField label="Tenant id">
                      {snapshot.tenants.length ? (
                        <select
                          value={projectDraft.tenant_id}
                          onChange={(event) => setProjectDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                        >
                          {snapshot.tenants.map((tenant) => (
                            <option key={tenant.id} value={tenant.id}>
                              {tenant.name} ({tenant.id})
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          value={projectDraft.tenant_id}
                          onChange={(event) => setProjectDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                          required
                        />
                      )}
                    </FormField>
                    <FormField label="Project id">
                      <input
                        value={projectDraft.id}
                        onChange={(event) => setProjectDraft((current) => ({ ...current, id: event.target.value }))}
                        required
                      />
                    </FormField>
                    <FormField label="Project name">
                      <input
                        value={projectDraft.name}
                        onChange={(event) => setProjectDraft((current) => ({ ...current, name: event.target.value }))}
                        required
                      />
                    </FormField>
                    <div className="adminx-note">
                      <strong>Selected project posture</strong>
                      <p>
                        Requests: {selectedProjectUsage?.request_count ?? 0}
                        {' | '}
                        Usage units: {selectedProjectBilling?.used_units ?? 0}
                        {' | '}
                        Tokens: {selectedProjectTokens}
                      </p>
                    </div>
                    <DialogFooter>
                      <InlineButton onClick={resetProjectDialog}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">
                        {projectDraft.id ? 'Save project' : 'Create project'}
                      </InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>

            <Dialog open={isApiKeyDialogOpen} onOpenChange={setIsApiKeyDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton onClick={() => setIsApiKeyDialogOpen(true)}>
                  Issue gateway key
                </InlineButton>
              </DialogTrigger>
              <DialogContent size="medium">
                <AdminDialog
                  title="Issue gateway key"
                  detail="Mint a project-scoped API key in a focused dialog, then reveal the plaintext once for secure handoff."
                >
                  <form className="adminx-form-grid" onSubmit={(event) => void handleApiKeySubmit(event)}>
                    <FormField label="Tenant">
                      {snapshot.tenants.length ? (
                        <select
                          value={apiKeyDraft.tenant_id}
                          onChange={(event) => {
                            const nextTenantId = event.target.value;
                            setApiKeyDraft((current) => ({
                              ...current,
                              tenant_id: nextTenantId,
                              project_id: snapshot.projects.find((project) => project.tenant_id === nextTenantId)?.id ?? '',
                            }));
                          }}
                        >
                          {snapshot.tenants.map((tenant) => (
                            <option key={tenant.id} value={tenant.id}>
                              {tenant.name} ({tenant.id})
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          value={apiKeyDraft.tenant_id}
                          onChange={(event) => setApiKeyDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                          required
                        />
                      )}
                    </FormField>
                    <FormField label="Project">
                      {availableApiKeyProjects.length ? (
                        <select
                          value={apiKeyDraft.project_id}
                          onChange={(event) => setApiKeyDraft((current) => ({ ...current, project_id: event.target.value }))}
                        >
                          {availableApiKeyProjects.map((project) => (
                            <option key={project.id} value={project.id}>
                              {project.name} ({project.id})
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          value={apiKeyDraft.project_id}
                          onChange={(event) => setApiKeyDraft((current) => ({ ...current, project_id: event.target.value }))}
                          required
                        />
                      )}
                    </FormField>
                    <FormField label="Environment">
                      <select
                        value={apiKeyDraft.environment}
                        onChange={(event) => setApiKeyDraft((current) => ({ ...current, environment: event.target.value }))}
                      >
                        <option value="production">Production</option>
                        <option value="staging">Staging</option>
                        <option value="development">Development</option>
                      </select>
                    </FormField>
                    <DialogFooter>
                      <InlineButton onClick={resetApiKeyDialog}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">
                        Issue gateway key
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
            <strong>Registry-first workflow</strong>
            <p>Review tenants, projects, and gateway keys in the registries first, then open dialogs only when you need to mutate state.</p>
          </div>
          <div className="adminx-note">
            <strong>Plaintext key ready</strong>
            <p>Plaintext gateway keys are revealed in a separate dialog after issuance and should be copied into a secure vault immediately.</p>
          </div>
          <div className="adminx-note">
            <strong>Deletion guardrails</strong>
            <p>Tenant deletion stays blocked while projects or portal users still depend on the workspace. Project deletion is blocked while portal identities are still bound.</p>
          </div>
        </div>
      </PageToolbar>

      <Surface
        title="Deletion posture"
        detail="Project deletion retires gateway keys and quota policies for the project. Tenant deletion is only allowed after its projects are cleared and portal users are re-bound."
      >
        <div className="adminx-note">
          <strong>Operational safety</strong>
          <p>Portal users must be reassigned before deleting the tenant or project they are bound to. Usage and billing history remain available as audit data even after workspace ownership records are retired.</p>
        </div>
      </Surface>

      <Surface title="Tenant registry" detail="Live tenant catalog from the admin API.">
        <DataTable
          columns={[
            { key: 'id', label: 'Tenant id', render: (tenant) => <strong>{tenant.id}</strong> },
            { key: 'name', label: 'Name', render: (tenant) => tenant.name },
            {
              key: 'actions',
              label: 'Actions',
              render: (tenant) => (
                <div className="adminx-row">
                  <InlineButton
                    onClick={() => {
                      setTenantDraft({ id: tenant.id, name: tenant.name });
                      setIsTenantDialogOpen(true);
                    }}
                  >
                    Edit tenant
                  </InlineButton>
                  <InlineButton
                    tone="danger"
                    disabled={
                      snapshot.projects.some((project) => project.tenant_id === tenant.id)
                      || portalTenants.has(tenant.id)
                    }
                    onClick={() =>
                      setPendingDelete({
                        kind: 'tenant',
                        id: tenant.id,
                        label: `${tenant.name} (${tenant.id})`,
                      })}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.tenants}
          empty="No tenants available."
          getKey={(tenant) => tenant.id}
        />
      </Surface>

      <Surface
        title="Project registry"
        detail="Projects are the routing, billing, and usage ownership boundary."
      >
        <DataTable
          columns={[
            { key: 'id', label: 'Project id', render: (project) => <strong>{project.id}</strong> },
            { key: 'tenant', label: 'Tenant', render: (project) => project.tenant_id },
            { key: 'name', label: 'Name', render: (project) => project.name },
            {
              key: 'actions',
              label: 'Actions',
              render: (project) => (
                <div className="adminx-row">
                  <InlineButton
                    onClick={() => {
                      setProjectDraft({
                        tenant_id: project.tenant_id,
                        id: project.id,
                        name: project.name,
                      });
                      setIsProjectDialogOpen(true);
                    }}
                  >
                    Edit project
                  </InlineButton>
                  <InlineButton
                    onClick={() => {
                      setApiKeyDraft({
                        tenant_id: project.tenant_id,
                        project_id: project.id,
                        environment: 'production',
                      });
                      setIsApiKeyDialogOpen(true);
                    }}
                  >
                    Issue gateway key
                  </InlineButton>
                  <InlineButton
                    tone="danger"
                    disabled={portalProjects.has(project.id)}
                    onClick={() =>
                      setPendingDelete({
                        kind: 'project',
                        id: project.id,
                        label: `${project.name} (${project.id})`,
                      })}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.projects}
          empty="No projects available."
          getKey={(project) => project.id}
        />
      </Surface>

      <Surface title="Gateway key inventory" detail="Keys are presented as hashed records only.">
        <DataTable
          columns={[
            { key: 'project', label: 'Project', render: (key) => key.project_id },
            { key: 'tenant', label: 'Tenant', render: (key) => key.tenant_id },
            { key: 'environment', label: 'Environment', render: (key) => key.environment },
            {
              key: 'active',
              label: 'Status',
              render: (key) => (
                <Pill tone={key.active ? 'live' : 'danger'}>
                  {key.active ? 'active' : 'revoked'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (key) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => void onUpdateApiKeyStatus(key.hashed_key, !key.active)}>
                    {key.active ? 'Revoke' : 'Restore'}
                  </InlineButton>
                  <InlineButton
                    tone="danger"
                    onClick={() =>
                      setPendingDelete({
                        kind: 'key',
                        id: key.hashed_key,
                        label: `${key.project_id} / ${key.environment}`,
                      })}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.apiKeys}
          empty="No gateway keys available."
          getKey={(key) => key.hashed_key}
        />
      </Surface>

      <Dialog
        open={Boolean(revealedApiKey)}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) {
            setRevealedApiKey(null);
          }
        }}
      >
        <DialogContent size="medium">
          <AdminDialog
            title="Plaintext key ready"
            detail="Store this secret now. After the dialog closes, the inventory only retains the hashed record."
          >
            {revealedApiKey ? (
              <div className="adminx-form-grid">
                <div className="adminx-note">
                  <strong>Issued scope</strong>
                  <p>
                    {revealedApiKey.project_id}
                    {' | '}
                    {revealedApiKey.environment}
                    {' | '}
                    hashed: {revealedApiKey.hashed}
                  </p>
                </div>
                <div className="adminx-note">
                  <strong>Plaintext key</strong>
                  <code>{revealedApiKey.plaintext}</code>
                </div>
                <DialogFooter>
                  <InlineButton
                    tone="primary"
                    onClick={() => {
                      if (navigator.clipboard) {
                        void navigator.clipboard.writeText(revealedApiKey.plaintext);
                      }
                    }}
                  >
                    Copy key
                  </InlineButton>
                  <InlineButton onClick={() => setRevealedApiKey(null)}>Close</InlineButton>
                </DialogFooter>
              </div>
            ) : null}
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title="Delete workspace resource"
        detail={
          pendingDelete
            ? `Delete ${pendingDelete.label}. This permanently removes the selected resource from the workspace registry.`
            : ''
        }
        confirmLabel="Delete now"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
