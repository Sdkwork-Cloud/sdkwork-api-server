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
  ToolbarSearchField,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CreatedGatewayApiKey } from 'sdkwork-router-admin-types';

type ApiKeyDraft = {
  tenant_id: string;
  project_id: string;
  environment: string;
  label: string;
  notes: string;
  expires_at_ms: string;
};

type TenantDirectoryRow = {
  id: string;
  name: string;
  projectCount: number;
  projectSummary: string;
  portalUserCount: number;
  apiKeyCount: number;
  activeApiKeyCount: number;
  environmentSummary: string;
  requestCount: number;
  tokenCount: number;
  canIssueApiKey: boolean;
  searchHaystack: string;
};

export function TenantsPage({
  snapshot,
  onSaveTenant,
  onSaveProject,
  onCreateApiKey,
  onDeleteTenant,
}: AdminPageProps & {
  onSaveTenant: (input: { id: string; name: string }) => Promise<void>;
  onSaveProject: (input: { tenant_id: string; id: string; name: string }) => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    label?: string;
    notes?: string;
    expires_at_ms?: number | null;
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
  const defaultTenantId = snapshot.tenants[0]?.id ?? 'tenant_local_demo';
  const defaultProjectId = snapshot.projects[0]?.id ?? 'project_local_demo';
  const createApiKeyDraft = (overrides: Partial<ApiKeyDraft> = {}): ApiKeyDraft => ({
    tenant_id: defaultTenantId,
    project_id: defaultProjectId,
    environment: 'production',
    label: '',
    notes: '',
    expires_at_ms: '',
    ...overrides,
  });
  const [apiKeyDraft, setApiKeyDraft] = useState<ApiKeyDraft>(createApiKeyDraft());
  const [search, setSearch] = useState('');
  const [isTenantDialogOpen, setIsTenantDialogOpen] = useState(false);
  const [isProjectDialogOpen, setIsProjectDialogOpen] = useState(false);
  const [isApiKeyDialogOpen, setIsApiKeyDialogOpen] = useState(false);
  const [revealedApiKey, setRevealedApiKey] = useState<CreatedGatewayApiKey | null>(null);
  const [pendingDelete, setPendingDelete] = useState<{ id: string; label: string } | null>(null);

  const selectedProjectUsage = snapshot.usageSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectTokens = snapshot.usageRecords
    .filter((record) => record.project_id === projectDraft.id)
    .reduce((sum, record) => sum + record.total_tokens, 0);
  const normalizedSearch = search.trim().toLowerCase();
  const tenantRows: TenantDirectoryRow[] = snapshot.tenants
    .map((tenant) => {
      const projects = snapshot.projects.filter((project) => project.tenant_id === tenant.id);
      const projectIds = new Set(projects.map((project) => project.id));
      const portalUsers = snapshot.portalUsers.filter((user) => user.workspace_tenant_id === tenant.id);
      const tenantApiKeys = snapshot.apiKeys.filter(
        (key) => key.tenant_id === tenant.id || projectIds.has(key.project_id),
      );
      const activeApiKeyCount = tenantApiKeys.filter((key) => key.active).length;
      const environmentSummary = Array.from(new Set(tenantApiKeys.map((key) => key.environment)))
        .sort()
        .join(', ');
      const requestCount = snapshot.usageRecords
        .filter((record) => projectIds.has(record.project_id))
        .reduce((sum, record) => sum + 1, 0);
      const tokenCount = snapshot.usageRecords
        .filter((record) => projectIds.has(record.project_id))
        .reduce((sum, record) => sum + record.total_tokens, 0);
      const projectSummary = projects.length
        ? projects.slice(0, 2).map((project) => project.name).join(', ')
        : 'No projects';

      return {
        id: tenant.id,
        name: tenant.name,
        projectCount: projects.length,
        projectSummary,
        portalUserCount: portalUsers.length,
        apiKeyCount: tenantApiKeys.length,
        activeApiKeyCount,
        environmentSummary: environmentSummary || 'No keys',
        requestCount,
        tokenCount,
        canIssueApiKey: projects.length > 0,
        searchHaystack: [
          tenant.id,
          tenant.name,
          ...projects.flatMap((project) => [project.id, project.name]),
          ...tenantApiKeys.flatMap((key) => [key.project_id, key.label, key.environment, key.notes ?? '']),
        ].join(' ').toLowerCase(),
      };
    })
    .filter((tenant) => !normalizedSearch || tenant.searchHaystack.includes(normalizedSearch))
    .sort((left, right) => (
      left.name.localeCompare(right.name)
      || left.id.localeCompare(right.id)
    ));

  function resetTenantDialog() {
    setTenantDraft({ id: '', name: '' });
    setIsTenantDialogOpen(false);
  }

  function resetProjectDialog() {
    setProjectDraft({
      tenant_id: defaultTenantId,
      id: '',
      name: '',
    });
    setIsProjectDialogOpen(false);
  }

  function resetApiKeyDialog() {
    setApiKeyDraft(createApiKeyDraft());
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
    const normalizedLabel = apiKeyDraft.label.trim();
    const normalizedNotes = apiKeyDraft.notes.trim();
    const normalizedExpiresAt = apiKeyDraft.expires_at_ms.trim();
    const parsedExpiresAt =
      normalizedExpiresAt === '' ? undefined : Number(normalizedExpiresAt);
    const created = await onCreateApiKey({
      tenant_id: apiKeyDraft.tenant_id,
      project_id: apiKeyDraft.project_id,
      environment: apiKeyDraft.environment,
      label: normalizedLabel || undefined,
      notes: normalizedNotes || undefined,
      expires_at_ms:
        parsedExpiresAt !== undefined
        && Number.isFinite(parsedExpiresAt)
        && Number.isInteger(parsedExpiresAt)
          ? parsedExpiresAt
          : undefined,
    });
    setRevealedApiKey(created);
    resetApiKeyDialog();
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    await onDeleteTenant(pendingDelete.id);
    setPendingDelete(null);
  }

  const availableApiKeyProjects = snapshot.projects.filter(
    (project) => project.tenant_id === apiKeyDraft.tenant_id,
  );

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
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
                    <FormField label="Key label">
                      <input
                        value={apiKeyDraft.label}
                        onChange={(event) => setApiKeyDraft((current) => ({ ...current, label: event.target.value }))}
                        placeholder="Production App Key"
                      />
                    </FormField>
                    <FormField label="Notes">
                      <textarea
                        value={apiKeyDraft.notes}
                        onChange={(event) => setApiKeyDraft((current) => ({ ...current, notes: event.target.value }))}
                        rows={3}
                        placeholder="Retained for admin inventory"
                      />
                    </FormField>
                    <FormField label="Expires at (ms)">
                      <input
                        type="number"
                        inputMode="numeric"
                        min="0"
                        step="1"
                        value={apiKeyDraft.expires_at_ms}
                        onChange={(event) => setApiKeyDraft((current) => ({ ...current, expires_at_ms: event.target.value }))}
                        placeholder="4102444800000"
                      />
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
        <ToolbarSearchField
          label="Search tenants"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="tenant, project, environment, key label"
        />
      </PageToolbar>

      <DataTable
        columns={[
          {
            key: 'tenant',
            label: 'Tenant',
            render: (tenant) => (
              <div className="adminx-table-cell-stack">
                <strong>{tenant.name}</strong>
                <span>{tenant.id}</span>
              </div>
            ),
          },
          {
            key: 'projects',
            label: 'Projects',
            render: (tenant) => (
              <div className="adminx-table-cell-stack">
                <strong>{tenant.projectCount}</strong>
                <span>{tenant.projectSummary}</span>
              </div>
            ),
          },
          {
            key: 'portal-users',
            label: 'Portal users',
            render: (tenant) => tenant.portalUserCount,
          },
          {
            key: 'api-keys',
            label: 'Api keys',
            render: (tenant) => (
              <div className="adminx-table-cell-stack">
                <strong>{tenant.activeApiKeyCount} active / {tenant.apiKeyCount} total</strong>
                <span>{tenant.environmentSummary}</span>
              </div>
            ),
          },
          {
            key: 'traffic',
            label: 'Traffic',
            render: (tenant) => (
              <div className="adminx-table-cell-stack">
                <strong>{tenant.requestCount} requests</strong>
                <span>{tenant.tokenCount} tokens</span>
              </div>
            ),
          },
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
                  onClick={() => {
                    setProjectDraft({
                      tenant_id: tenant.id,
                      id: '',
                      name: '',
                    });
                    setIsProjectDialogOpen(true);
                  }}
                >
                  New project
                </InlineButton>
                <InlineButton
                  disabled={!tenant.canIssueApiKey}
                  onClick={() => {
                    const firstProjectId = snapshot.projects.find(
                      (project) => project.tenant_id === tenant.id,
                    )?.id ?? '';
                    setApiKeyDraft(createApiKeyDraft({
                      tenant_id: tenant.id,
                      project_id: firstProjectId,
                    }));
                    setIsApiKeyDialogOpen(true);
                  }}
                >
                  Issue gateway key
                </InlineButton>
                <InlineButton
                  tone="danger"
                  disabled={tenant.projectCount > 0 || tenant.portalUserCount > 0}
                  onClick={() =>
                    setPendingDelete({
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
        rows={tenantRows}
        empty="No tenants available."
        getKey={(tenant) => tenant.id}
      />

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
            detail="Store this secret now. The control plane persists both hashed_key and raw_key in the canonical ai_app_api_keys table."
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
                        {revealedApiKey.label}
                        {' | '}
                        hashed: {revealedApiKey.hashed}
                      </p>
                    </div>
                    {revealedApiKey.notes ? (
                      <div className="adminx-note">
                        <strong>Notes</strong>
                        <p>{revealedApiKey.notes}</p>
                      </div>
                    ) : null}
                    <div className="adminx-note">
                      <strong>Expires at (ms)</strong>
                      <p>{revealedApiKey.expires_at_ms ?? 'never'}</p>
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
