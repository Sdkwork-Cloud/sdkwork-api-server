import { useDeferredValue, useEffect, useState } from 'react';
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
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

const bootstrapOperatorEmail = 'admin@sdkwork.local';
const bootstrapPortalEmail = 'portal@sdkwork.local';

type SaveOperatorUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  active: boolean;
};

type SavePortalUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

type UsersPageProps = AdminPageProps & {
  onSaveOperatorUser: (input: SaveOperatorUserInput) => Promise<void> | void;
  onSavePortalUser: (input: SavePortalUserInput) => Promise<void> | void;
  onToggleOperatorUser: (userId: string, active: boolean) => Promise<void> | void;
  onTogglePortalUser: (userId: string, active: boolean) => Promise<void> | void;
  onDeleteOperatorUser: (userId: string) => Promise<void> | void;
  onDeletePortalUser: (userId: string) => Promise<void> | void;
};

type OperatorDraft = {
  id?: string;
  email: string;
  display_name: string;
  password: string;
  active: boolean;
};

type PortalDraft = {
  id?: string;
  email: string;
  display_name: string;
  password: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

type PendingDelete =
  | { kind: 'operator'; user: ManagedUser }
  | { kind: 'portal'; user: ManagedUser }
  | null;

function defaultTenantId(snapshot: AdminPageProps['snapshot']): string {
  return snapshot.tenants[0]?.id ?? 'tenant_local_demo';
}

function defaultProjectId(
  snapshot: AdminPageProps['snapshot'],
  tenantId: string,
): string {
  return (
    snapshot.projects.find((project) => project.tenant_id === tenantId)?.id
    ?? snapshot.projects[0]?.id
    ?? 'project_local_demo'
  );
}

function emptyOperatorDraft(): OperatorDraft {
  return {
    email: '',
    display_name: '',
    password: '',
    active: true,
  };
}

function emptyPortalDraft(snapshot: AdminPageProps['snapshot']): PortalDraft {
  const tenantId = defaultTenantId(snapshot);
  return {
    email: '',
    display_name: '',
    password: '',
    workspace_tenant_id: tenantId,
    workspace_project_id: defaultProjectId(snapshot, tenantId),
    active: true,
  };
}

function matchesFilters(
  user: ManagedUser,
  deferredQuery: string,
  statusFilter: 'all' | 'active' | 'disabled',
): boolean {
  const statusMatches = statusFilter === 'all'
    || (statusFilter === 'active' && user.active)
    || (statusFilter === 'disabled' && !user.active);

  if (!statusMatches) {
    return false;
  }

  const haystack = [
    user.display_name,
    user.email,
    user.workspace_tenant_id ?? '',
    user.workspace_project_id ?? '',
  ]
    .join(' ')
    .toLowerCase();

  return haystack.includes(deferredQuery);
}

export function UsersPage({
  snapshot,
  onSaveOperatorUser,
  onSavePortalUser,
  onToggleOperatorUser,
  onTogglePortalUser,
  onDeleteOperatorUser,
  onDeletePortalUser,
}: UsersPageProps) {
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'disabled'>('all');
  const [operatorDraft, setOperatorDraft] = useState<OperatorDraft>(() => emptyOperatorDraft());
  const [portalDraft, setPortalDraft] = useState<PortalDraft>(() => emptyPortalDraft(snapshot));
  const [isOperatorDialogOpen, setIsOperatorDialogOpen] = useState(false);
  const [isPortalDialogOpen, setIsPortalDialogOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<PendingDelete>(null);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  useEffect(() => {
    setPortalDraft((current) => {
      const nextTenantId = current.workspace_tenant_id || defaultTenantId(snapshot);
      const availableProjects = snapshot.projects.filter(
        (project) => project.tenant_id === nextTenantId,
      );
      const nextProjectId = availableProjects.some(
        (project) => project.id === current.workspace_project_id,
      )
        ? current.workspace_project_id
        : defaultProjectId(snapshot, nextTenantId);

      if (
        nextTenantId === current.workspace_tenant_id
        && nextProjectId === current.workspace_project_id
      ) {
        return current;
      }

      return {
        ...current,
        workspace_tenant_id: nextTenantId,
        workspace_project_id: nextProjectId,
      };
    });
  }, [snapshot.projects, snapshot.tenants]);

  const filteredOperators = snapshot.operatorUsers.filter((user) =>
    matchesFilters(user, deferredQuery, statusFilter));
  const filteredPortalUsers = snapshot.portalUsers.filter((user) =>
    matchesFilters(user, deferredQuery, statusFilter));
  const availableProjects = snapshot.projects.filter(
    (project) => project.tenant_id === portalDraft.workspace_tenant_id,
  );
  const selectedProject = snapshot.projects.find(
    (project) => project.id === portalDraft.workspace_project_id,
  );
  const selectedProjectTraffic = snapshot.usageSummary.projects.find(
    (project) => project.project_id === portalDraft.workspace_project_id,
  );
  const selectedProjectBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === portalDraft.workspace_project_id,
  );
  const selectedProjectTokens = snapshot.usageRecords
    .filter((record) => record.project_id === portalDraft.workspace_project_id)
    .reduce((sum, record) => sum + record.total_tokens, 0);
  const totalUsageUnits = snapshot.portalUsers.reduce((sum, user) => sum + user.usage_units, 0);
  const totalPortalTokens = snapshot.portalUsers.reduce((sum, user) => sum + user.total_tokens, 0);
  const disabledUsers = snapshot.operatorUsers
    .concat(snapshot.portalUsers)
    .filter((user) => !user.active).length;

  function resetOperatorDialog() {
    setIsOperatorDialogOpen(false);
    setOperatorDraft(emptyOperatorDraft());
  }

  function resetPortalDialog() {
    setIsPortalDialogOpen(false);
    setPortalDraft(emptyPortalDraft(snapshot));
  }

  function openOperatorDialog(user?: ManagedUser) {
    setOperatorDraft(
      user
        ? {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            password: '',
            active: user.active,
          }
        : emptyOperatorDraft(),
    );
    setIsOperatorDialogOpen(true);
  }

  function openPortalDialog(user?: ManagedUser) {
    setPortalDraft(
      user
        ? {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            password: '',
            workspace_tenant_id: user.workspace_tenant_id ?? defaultTenantId(snapshot),
            workspace_project_id:
              user.workspace_project_id
              ?? defaultProjectId(
                snapshot,
                user.workspace_tenant_id ?? defaultTenantId(snapshot),
              ),
            active: user.active,
          }
        : emptyPortalDraft(snapshot),
    );
    setIsPortalDialogOpen(true);
  }

  async function handleOperatorSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveOperatorUser({
      id: operatorDraft.id,
      email: operatorDraft.email.trim(),
      display_name: operatorDraft.display_name.trim(),
      password: operatorDraft.password.trim() || undefined,
      active: operatorDraft.active,
    });
    resetOperatorDialog();
  }

  async function handlePortalSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSavePortalUser({
      id: portalDraft.id,
      email: portalDraft.email.trim(),
      display_name: portalDraft.display_name.trim(),
      password: portalDraft.password.trim() || undefined,
      workspace_tenant_id: portalDraft.workspace_tenant_id,
      workspace_project_id: portalDraft.workspace_project_id,
      active: portalDraft.active,
    });
    resetPortalDialog();
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    if (pendingDelete.kind === 'operator') {
      await onDeleteOperatorUser(pendingDelete.user.id);
    } else {
      await onDeletePortalUser(pendingDelete.user.id);
    }

    setPendingDelete(null);
  }

  return (
    <div className="adminx-page-grid">
      <section className="adminx-stat-grid">
        <StatCard
          label="Operator users"
          value={String(snapshot.operatorUsers.length)}
          detail="Super-admin and support operators with direct control-plane access."
        />
        <StatCard
          label="Portal users"
          value={String(snapshot.portalUsers.length)}
          detail="End-user identities bound to tenant and project scopes."
        />
        <StatCard
          label="Metered units"
          value={String(totalUsageUnits)}
          detail="Usage units attributed back to active portal workspaces."
        />
        <StatCard
          label="Portal tokens"
          value={String(totalPortalTokens)}
          detail="Prompt and completion tokens consumed across portal identities."
        />
        <StatCard
          label="Disabled users"
          value={String(disabledUsers)}
          detail="Accounts waiting for manual restore or retirement decisions."
        />
      </section>

      <PageToolbar
        title="User directory workbench"
        detail="Keep the roster readable, preserve filters while you work, and move create or edit into dedicated dialogs with explicit confirmation."
        actions={(
          <>
            <Dialog
              open={isOperatorDialogOpen}
              onOpenChange={(nextOpen) => {
                if (!nextOpen) {
                  resetOperatorDialog();
                  return;
                }
                setIsOperatorDialogOpen(true);
              }}
            >
              <DialogTrigger asChild>
                <InlineButton tone="primary" onClick={() => openOperatorDialog()}>
                  New operator
                </InlineButton>
              </DialogTrigger>
              <DialogContent size="medium">
                <AdminDialog
                  title={operatorDraft.id ? 'Edit operator' : 'Create operator'}
                  detail="Operators manage catalog, traffic, and runtime posture. Keep this population tightly controlled and only rotate passwords when needed."
                >
                  <form className="adminx-form-grid" onSubmit={(event) => void handleOperatorSubmit(event)}>
                    <FormField label="Display name">
                      <input
                        value={operatorDraft.display_name}
                        onChange={(event) =>
                          setOperatorDraft((current) => ({
                            ...current,
                            display_name: event.target.value,
                          }))}
                        required
                      />
                    </FormField>
                    <FormField label="Email">
                      <input
                        value={operatorDraft.email}
                        onChange={(event) =>
                          setOperatorDraft((current) => ({
                            ...current,
                            email: event.target.value,
                          }))}
                        type="email"
                        required
                      />
                    </FormField>
                    <FormField
                      label={operatorDraft.id ? 'New password' : 'Password'}
                      hint={operatorDraft.id ? 'Leave blank to preserve the current password.' : 'Set a strong operator password.'}
                    >
                      <input
                        value={operatorDraft.password}
                        onChange={(event) =>
                          setOperatorDraft((current) => ({
                            ...current,
                            password: event.target.value,
                          }))}
                        type="password"
                        required={!operatorDraft.id}
                      />
                    </FormField>
                    <FormField label="Status">
                      <select
                        value={operatorDraft.active ? 'active' : 'disabled'}
                        onChange={(event) =>
                          setOperatorDraft((current) => ({
                            ...current,
                            active: event.target.value === 'active',
                          }))}
                      >
                        <option value="active">Active</option>
                        <option value="disabled">Disabled</option>
                      </select>
                    </FormField>
                    <DialogFooter>
                      <InlineButton onClick={resetOperatorDialog}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">
                        {operatorDraft.id ? 'Save operator' : 'Create operator'}
                      </InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>

            <Dialog
              open={isPortalDialogOpen}
              onOpenChange={(nextOpen) => {
                if (!nextOpen) {
                  resetPortalDialog();
                  return;
                }
                setIsPortalDialogOpen(true);
              }}
            >
              <DialogTrigger asChild>
                <InlineButton onClick={() => openPortalDialog()}>
                  New portal user
                </InlineButton>
              </DialogTrigger>
              <DialogContent size="large">
                <AdminDialog
                  title={portalDraft.id ? 'Edit portal user' : 'Create portal user'}
                  detail="Portal identities are scoped to a tenant and project so usage, billing, and request posture remain attributable after every change."
                >
                  <form className="adminx-form-grid" onSubmit={(event) => void handlePortalSubmit(event)}>
                    <FormField label="Display name">
                      <input
                        value={portalDraft.display_name}
                        onChange={(event) =>
                          setPortalDraft((current) => ({
                            ...current,
                            display_name: event.target.value,
                          }))}
                        required
                      />
                    </FormField>
                    <FormField label="Email">
                      <input
                        value={portalDraft.email}
                        onChange={(event) =>
                          setPortalDraft((current) => ({
                            ...current,
                            email: event.target.value,
                          }))}
                        type="email"
                        required
                      />
                    </FormField>
                    <FormField
                      label={portalDraft.id ? 'New password' : 'Password'}
                      hint={portalDraft.id ? 'Leave blank to keep the current secret.' : 'Set an initial portal password.'}
                    >
                      <input
                        value={portalDraft.password}
                        onChange={(event) =>
                          setPortalDraft((current) => ({
                            ...current,
                            password: event.target.value,
                          }))}
                        type="password"
                        required={!portalDraft.id}
                      />
                    </FormField>
                    <FormField label="Workspace tenant">
                      {snapshot.tenants.length ? (
                        <select
                          value={portalDraft.workspace_tenant_id}
                          onChange={(event) => {
                            const nextTenantId = event.target.value;
                            setPortalDraft((current) => ({
                              ...current,
                              workspace_tenant_id: nextTenantId,
                              workspace_project_id: defaultProjectId(snapshot, nextTenantId),
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
                          value={portalDraft.workspace_tenant_id}
                          onChange={(event) =>
                            setPortalDraft((current) => ({
                              ...current,
                              workspace_tenant_id: event.target.value,
                            }))}
                        />
                      )}
                    </FormField>
                    <FormField label="Workspace project">
                      {availableProjects.length ? (
                        <select
                          value={portalDraft.workspace_project_id}
                          onChange={(event) =>
                            setPortalDraft((current) => ({
                              ...current,
                              workspace_project_id: event.target.value,
                            }))}
                        >
                          {availableProjects.map((project) => (
                            <option key={project.id} value={project.id}>
                              {project.name} ({project.id})
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          value={portalDraft.workspace_project_id}
                          onChange={(event) =>
                            setPortalDraft((current) => ({
                              ...current,
                              workspace_project_id: event.target.value,
                            }))}
                        />
                      )}
                    </FormField>
                    <FormField label="Status">
                      <select
                        value={portalDraft.active ? 'active' : 'disabled'}
                        onChange={(event) =>
                          setPortalDraft((current) => ({
                            ...current,
                            active: event.target.value === 'active',
                          }))}
                      >
                        <option value="active">Active</option>
                        <option value="disabled">Disabled</option>
                      </select>
                    </FormField>
                    <div className="adminx-note">
                      <strong>Selected workspace posture</strong>
                      <p>
                        {selectedProject?.name ?? 'Unassigned workspace'}
                        {' | '}
                        Requests: {selectedProjectTraffic?.request_count ?? 0}
                        {' | '}
                        Usage units: {selectedProjectBilling?.used_units ?? 0}
                        {' | '}
                        Tokens: {selectedProjectTokens}
                      </p>
                    </div>
                    <DialogFooter>
                      <InlineButton onClick={resetPortalDialog}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">
                        {portalDraft.id ? 'Save portal user' : 'Create portal user'}
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
          <FormField label="Search users">
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="name, email, tenant, project"
            />
          </FormField>
          <FormField label="Status">
            <select
              value={statusFilter}
              onChange={(event) =>
                setStatusFilter(event.target.value as 'all' | 'active' | 'disabled')}
            >
              <option value="all">All users</option>
              <option value="active">Active only</option>
              <option value="disabled">Disabled only</option>
            </select>
          </FormField>
          <div className="adminx-note">
            <strong>Identity guardrails</strong>
            <p>
              Passwords are only required on first creation. Protected bootstrap accounts stay
              undeletable, and editing with a blank password preserves the current secret.
            </p>
          </div>
        </div>
      </PageToolbar>

      <Surface
        title="Operator roster"
        detail="Manage super-admin and support operator accounts with controlled activation and explicit password rotation."
        actions={<Pill tone="default">{filteredOperators.length} visible</Pill>}
      >
        <DataTable
          columns={[
            {
              key: 'operator',
              label: 'Operator',
              render: (user) => (
                <div className="adminx-table-cell-stack">
                  <strong>{user.display_name}</strong>
                  <span>{user.id}</span>
                </div>
              ),
            },
            { key: 'email', label: 'Email', render: (user) => user.email },
            {
              key: 'role',
              label: 'Role',
              render: () => <Pill tone="live">operator</Pill>,
            },
            {
              key: 'status',
              label: 'Status',
              render: (user) => (
                <Pill tone={user.active ? 'live' : 'danger'}>
                  {user.active ? 'active' : 'disabled'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (user) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => openOperatorDialog(user)}>Edit operator</InlineButton>
                  <InlineButton onClick={() => void onToggleOperatorUser(user.id, !user.active)}>
                    {user.active ? 'Disable' : 'Restore'}
                  </InlineButton>
                  <InlineButton
                    tone="danger"
                    disabled={
                      user.email === bootstrapOperatorEmail
                      || user.id === snapshot.sessionUser?.id
                    }
                    onClick={() => setPendingDelete({ kind: 'operator', user })}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={filteredOperators}
          empty="No operator users match the current filter."
          getKey={(user) => user.id}
        />
      </Surface>

      <Surface
        title="Portal roster"
        detail="Inspect portal identities with workspace binding, request load, token usage, and metered units before intervening."
        actions={<Pill tone="default">{filteredPortalUsers.length} visible</Pill>}
      >
        <DataTable
          columns={[
            {
              key: 'portal-user',
              label: 'Portal user',
              render: (user) => (
                <div className="adminx-table-cell-stack">
                  <strong>{user.display_name}</strong>
                  <span>{user.email}</span>
                </div>
              ),
            },
            {
              key: 'workspace',
              label: 'Workspace',
              render: (user) => (
                <div className="adminx-table-cell-stack">
                  <strong>{user.workspace_tenant_id ?? '-'}</strong>
                  <span>{user.workspace_project_id ?? '-'}</span>
                </div>
              ),
            },
            { key: 'requests', label: 'Requests', render: (user) => user.request_count },
            { key: 'tokens', label: 'Tokens', render: (user) => user.total_tokens },
            { key: 'units', label: 'Metered units', render: (user) => user.usage_units },
            {
              key: 'status',
              label: 'Status',
              render: (user) => (
                <Pill tone={user.active ? 'live' : 'danger'}>
                  {user.active ? 'active' : 'disabled'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (user) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => openPortalDialog(user)}>Edit portal user</InlineButton>
                  <InlineButton onClick={() => void onTogglePortalUser(user.id, !user.active)}>
                    {user.active ? 'Disable' : 'Restore'}
                  </InlineButton>
                  <InlineButton
                    tone="danger"
                    disabled={user.email === bootstrapPortalEmail}
                    onClick={() => setPendingDelete({ kind: 'portal', user })}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={filteredPortalUsers}
          empty="No portal users match the current filter."
          getKey={(user) => user.id}
        />
      </Surface>

      <Surface
        title="Intervention guidance"
        detail="Keep admin actions aligned with the product expectations of a mature backend: fast scanning, focused mutation, and explicit state changes."
      >
        <div className="adminx-card-grid">
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Bootstrap accounts stay protected</strong>
              <Pill tone="default">safety</Pill>
            </div>
            <p>Primary bootstrap identities remain visible in the roster but cannot be deleted from this surface.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Workspace binding stays visible</strong>
              <Pill tone="live">context</Pill>
            </div>
            <p>Portal user dialogs keep tenant and project posture visible so support changes stay attributable.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Filters survive mutations</strong>
              <Pill tone="seed">focus</Pill>
            </div>
            <p>Search and status filters remain in place after closing dialogs, making back-to-back interventions faster.</p>
          </article>
        </div>
      </Surface>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title={
          pendingDelete?.kind === 'operator'
            ? 'Delete operator account'
            : 'Delete portal account'
        }
        detail={
          pendingDelete
            ? `Remove ${pendingDelete.user.display_name} (${pendingDelete.user.email}) from the directory. This action cannot be undone from this workbench.`
            : ''
        }
        confirmLabel="Delete now"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
