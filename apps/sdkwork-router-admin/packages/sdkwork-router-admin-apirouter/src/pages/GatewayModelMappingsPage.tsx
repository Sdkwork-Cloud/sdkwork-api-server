import { useDeferredValue, useMemo, useState } from 'react';
import type { FormEvent } from 'react';

import {
  AdminDialog,
  ConfirmDialog,
  DataTable,
  Dialog,
  DialogContent,
  DialogFooter,
  FormField,
  Input,
  InlineButton,
  PageToolbar,
  Pill,
  Select,
  Surface,
  Textarea,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import {
  createGatewayModelMapping,
  deleteGatewayModelMapping,
  listGatewayModelMappings,
  updateGatewayModelMapping,
  updateGatewayModelMappingStatus,
  type GatewayModelMappingRecord,
  type GatewayModelMappingStatus,
} from '../services/gatewayOverlayStore';
import { buildGatewayModelCatalog } from '../services/gatewayViewService';

type MappingRuleDraft = {
  id: string;
  source_value: string;
  target_value: string;
};

type MappingDraft = {
  name: string;
  description: string;
  status: GatewayModelMappingStatus;
  effective_from: string;
  effective_to: string;
  rules: MappingRuleDraft[];
};

function createRuleDraft(sourceValue = '', targetValue = ''): MappingRuleDraft {
  return {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    source_value: sourceValue,
    target_value: targetValue,
  };
}

function todayDateValue(): string {
  return new Date().toISOString().slice(0, 10);
}

function emptyDraft(catalogValues: string[]): MappingDraft {
  const defaultValue = catalogValues[0] ?? '';
  return {
    name: '',
    description: '',
    status: 'active',
    effective_from: todayDateValue(),
    effective_to: '',
    rules: [createRuleDraft(defaultValue, defaultValue)],
  };
}

function draftFromMapping(mapping: GatewayModelMappingRecord): MappingDraft {
  return {
    name: mapping.name,
    description: mapping.description,
    status: mapping.status,
    effective_from: mapping.effective_from,
    effective_to: mapping.effective_to ?? '',
    rules: mapping.rules.map((rule) =>
      createRuleDraft(
        `${rule.source_channel_id}::${rule.source_model_id}`,
        `${rule.target_channel_id}::${rule.target_model_id}`,
      ),
    ),
  };
}

function formatDateLabel(value?: string | null): string {
  if (!value) {
    return 'open-ended';
  }

  return value;
}

export function GatewayModelMappingsPage({ snapshot }: AdminPageProps) {
  const catalog = useMemo(() => buildGatewayModelCatalog(snapshot), [snapshot]);
  const catalogValues = catalog.map((item) => item.value);
  const catalogByValue = useMemo(
    () => new Map(catalog.map((item) => [item.value, item])),
    [catalog],
  );
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | GatewayModelMappingStatus>('all');
  const [selectedMappingId, setSelectedMappingId] = useState<string | null>(null);
  const [editingMappingId, setEditingMappingId] = useState<string | null>(null);
  const [mappingDraft, setMappingDraft] = useState<MappingDraft>(() => emptyDraft(catalogValues));
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<GatewayModelMappingRecord | null>(null);
  const [overlayVersion, setOverlayVersion] = useState(0);
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const mappings = useMemo(
    () => listGatewayModelMappings(),
    [overlayVersion],
  );
  const filteredMappings = mappings.filter((mapping) => {
    if (statusFilter !== 'all' && mapping.status !== statusFilter) {
      return false;
    }

    if (!deferredSearch) {
      return true;
    }

    const haystack = [
      mapping.name,
      mapping.description,
      ...mapping.rules.flatMap((rule) => [
        rule.source_channel_name,
        rule.source_model_name,
        rule.source_model_id,
        rule.target_channel_name,
        rule.target_model_name,
        rule.target_model_id,
      ]),
    ]
      .join(' ')
      .toLowerCase();

    return haystack.includes(deferredSearch);
  });
  const selectedMapping =
    mappings.find((mapping) => mapping.id === selectedMappingId) ?? null;
  function refreshMappings(): void {
    setOverlayVersion((value) => value + 1);
  }

  function resetEditor(): void {
    setEditingMappingId(null);
    setMappingDraft(emptyDraft(catalogValues));
    setIsEditorOpen(false);
  }

  function openCreateDialog(): void {
    setEditingMappingId(null);
    setMappingDraft(emptyDraft(catalogValues));
    setIsEditorOpen(true);
  }

  function openEditDialog(mapping: GatewayModelMappingRecord): void {
    setEditingMappingId(mapping.id);
    setMappingDraft(draftFromMapping(mapping));
    setIsEditorOpen(true);
  }

  function addRule(): void {
    const defaultValue = catalogValues[0] ?? '';
    setMappingDraft((current) => ({
      ...current,
      rules: [...current.rules, createRuleDraft(defaultValue, defaultValue)],
    }));
  }

  function removeRule(ruleId: string): void {
    setMappingDraft((current) => {
      const nextRules = current.rules.filter((rule) => rule.id !== ruleId);
      return {
        ...current,
        rules: nextRules.length ? nextRules : [createRuleDraft(catalogValues[0] ?? '', catalogValues[0] ?? '')],
      };
    });
  }

  function updateRule(
    ruleId: string,
    field: 'source_value' | 'target_value',
    value: string,
  ): void {
    setMappingDraft((current) => ({
      ...current,
      rules: current.rules.map((rule) =>
        rule.id === ruleId ? { ...rule, [field]: value } : rule,
      ),
    }));
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();

    const rules = mappingDraft.rules
      .map((rule) => {
        const source = catalogByValue.get(rule.source_value);
        const target = catalogByValue.get(rule.target_value);
        if (!source || !target) {
          return null;
        }

        return {
          id: rule.id,
          source_channel_id: source.channel_id,
          source_channel_name: source.channel_name,
          source_model_id: source.model_id,
          source_model_name: source.model_name,
          target_channel_id: target.channel_id,
          target_channel_name: target.channel_name,
          target_model_id: target.model_id,
          target_model_name: target.model_name,
        };
      })
      .filter((rule): rule is NonNullable<typeof rule> => Boolean(rule));

    if (!rules.length) {
      return;
    }

    if (editingMappingId) {
      updateGatewayModelMapping(editingMappingId, {
        name: mappingDraft.name,
        description: mappingDraft.description,
        status: mappingDraft.status,
        effective_from: mappingDraft.effective_from,
        effective_to: mappingDraft.effective_to || null,
        rules,
      });
    } else {
      createGatewayModelMapping({
        name: mappingDraft.name,
        description: mappingDraft.description,
        effective_from: mappingDraft.effective_from,
        effective_to: mappingDraft.effective_to || null,
        rules,
      });
    }

    refreshMappings();
    resetEditor();
  }

  async function confirmDelete(): Promise<void> {
    if (!pendingDelete) {
      return;
    }

    deleteGatewayModelMapping(pendingDelete.id);
    refreshMappings();
    setPendingDelete(null);
    if (selectedMappingId === pendingDelete.id) {
      setSelectedMappingId(null);
    }
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <>
            <InlineButton tone="primary" onClick={openCreateDialog}>
              New model mapping
            </InlineButton>
            <InlineButton onClick={refreshMappings}>Refresh overlay</InlineButton>
          </>
        )}
      >
        <ToolbarInline>
          <ToolbarSearchField
            label="Search mappings"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="name, model, channel..."
          />
          <ToolbarField label="Mapping status">
            <Select
              value={statusFilter}
              onChange={(event) =>
                setStatusFilter(event.target.value as 'all' | GatewayModelMappingStatus)
              }
            >
              <option value="all">All mappings</option>
              <option value="active">Active mappings</option>
              <option value="disabled">Disabled mappings</option>
            </Select>
          </ToolbarField>
        </ToolbarInline>
      </PageToolbar>

      <DataTable
        columns={[
          {
            key: 'name',
            label: 'Mapping',
            render: (mapping) => (
              <div className="adminx-table-cell-stack">
                <strong>{mapping.name}</strong>
                <span>{mapping.description || 'No description'}</span>
              </div>
            ),
          },
          {
            key: 'status',
            label: 'Status',
            render: (mapping) => (
              <Pill tone={mapping.status === 'active' ? 'live' : 'danger'}>
                {mapping.status}
              </Pill>
            ),
          },
          {
            key: 'window',
            label: 'Effective window',
            render: (mapping) =>
              `${formatDateLabel(mapping.effective_from)} -> ${formatDateLabel(mapping.effective_to)}`,
          },
          {
            key: 'rules',
            label: 'Rule rows',
            render: (mapping) => mapping.rules.length,
          },
          {
            key: 'actions',
            label: 'Actions',
            render: (mapping) => (
              <div className="adminx-row">
                <InlineButton
                  onClick={() => {
                    setSelectedMappingId(mapping.id);
                  }}
                >
                  View rules
                </InlineButton>
                <InlineButton onClick={() => openEditDialog(mapping)}>Edit</InlineButton>
                <InlineButton
                  onClick={() => {
                    updateGatewayModelMappingStatus(
                      mapping.id,
                      mapping.status === 'active' ? 'disabled' : 'active',
                    );
                    refreshMappings();
                  }}
                >
                  {mapping.status === 'active' ? 'Disable' : 'Enable'}
                </InlineButton>
                <InlineButton tone="danger" onClick={() => setPendingDelete(mapping)}>
                  Delete
                </InlineButton>
              </div>
            ),
          },
        ]}
        rows={filteredMappings}
        empty="No model mapping overlays match the current filter."
        getKey={(mapping) => mapping.id}
      />

      <Dialog open={isEditorOpen} onOpenChange={(nextOpen) => (nextOpen ? setIsEditorOpen(true) : resetEditor())}>
        <DialogContent size="large">
          <AdminDialog
            title={editingMappingId ? 'Edit model mapping' : 'New model mapping'}
            detail="Model mapping keeps the admin API gateway compatible with claw-style model translation while the router remains on its existing catalog schema."
          >
            <form className="adminx-form-grid" onSubmit={(event) => void handleSubmit(event)}>
              <FormField label="Mapping name">
                <Input
                  value={mappingDraft.name}
                  onChange={(event) =>
                    setMappingDraft((current) => ({ ...current, name: event.target.value }))
                  }
                  required
                />
              </FormField>
              <FormField label="Description">
                <Textarea
                  rows={3}
                  value={mappingDraft.description}
                  onChange={(event) =>
                    setMappingDraft((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }
                />
              </FormField>
              <FormField label="Status">
                <Select
                  value={mappingDraft.status}
                  onChange={(event) =>
                    setMappingDraft((current) => ({
                      ...current,
                      status: event.target.value as GatewayModelMappingStatus,
                    }))
                  }
                >
                  <option value="active">Active</option>
                  <option value="disabled">Disabled</option>
                </Select>
              </FormField>
              <FormField label="Effective from">
                <Input
                  type="date"
                  value={mappingDraft.effective_from}
                  onChange={(event) =>
                    setMappingDraft((current) => ({
                      ...current,
                      effective_from: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField label="Effective to">
                <Input
                  type="date"
                  value={mappingDraft.effective_to}
                  onChange={(event) =>
                    setMappingDraft((current) => ({
                      ...current,
                      effective_to: event.target.value,
                    }))
                  }
                />
              </FormField>

              <Surface
                title="Rule builder"
                detail="Choose a source model and a target model for each translation rule."
                actions={<InlineButton onClick={addRule}>Add rule</InlineButton>}
              >
                <div className="adminx-page-grid">
                  {mappingDraft.rules.map((rule, index) => (
                    <div
                      key={rule.id}
                      className="rounded-[24px] border border-zinc-200/80 bg-zinc-50/80 p-5 dark:border-zinc-800/80 dark:bg-zinc-900/70"
                    >
                      <div className="flex flex-col gap-4 border-b border-zinc-200/80 pb-4 dark:border-zinc-800/80 md:flex-row md:items-start md:justify-between">
                        <div className="space-y-1">
                          <h3 className="text-base font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                            Rule {index + 1}
                          </h3>
                          <p className="text-sm text-zinc-500 dark:text-zinc-400">
                            Map one client-facing model shape onto a target channel model.
                          </p>
                        </div>
                        <div className="flex flex-wrap gap-2.5">
                          <InlineButton
                            tone="danger"
                            onClick={() => removeRule(rule.id)}
                            disabled={mappingDraft.rules.length === 1}
                          >
                            Remove
                          </InlineButton>
                        </div>
                      </div>
                      <div className="adminx-form-grid pt-4">
                        <FormField label="Source model">
                          <Select
                            value={rule.source_value}
                            onChange={(event) => updateRule(rule.id, 'source_value', event.target.value)}
                          >
                            {catalog.map((item) => (
                              <option key={item.value} value={item.value}>
                                {item.label}
                              </option>
                            ))}
                          </Select>
                        </FormField>
                        <FormField label="Target model">
                          <Select
                            value={rule.target_value}
                            onChange={(event) => updateRule(rule.id, 'target_value', event.target.value)}
                          >
                            {catalog.map((item) => (
                              <option key={item.value} value={item.value}>
                                {item.label}
                              </option>
                            ))}
                          </Select>
                        </FormField>
                      </div>
                    </div>
                  ))}
                </div>
              </Surface>

              <DialogFooter>
                <InlineButton onClick={resetEditor}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  {editingMappingId ? 'Save mapping' : 'Create mapping'}
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={Boolean(selectedMapping)}
        onOpenChange={(nextOpen) => (nextOpen ? null : setSelectedMappingId(null))}
      >
        <DialogContent size="large">
          <AdminDialog
            title={selectedMapping ? `Mapping rules: ${selectedMapping.name}` : 'Mapping rules'}
            detail={
              selectedMapping
                ? selectedMapping.description || 'Review the current claw-style source-to-target model translation rules.'
                : 'Review the current claw-style source-to-target model translation rules.'
            }
          >
            {selectedMapping ? (
              <div className="adminx-page-grid">
                {selectedMapping.rules.map((rule, index) => (
                  <article
                    key={rule.id}
                    className="rounded-[24px] border border-zinc-200/80 bg-zinc-50/80 p-5 dark:border-zinc-800/80 dark:bg-zinc-900/70"
                  >
                    <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                      <div className="space-y-1">
                        <h3 className="text-base font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                          Rule {index + 1}
                        </h3>
                        <p className="text-sm text-zinc-500 dark:text-zinc-400">
                          Source-to-target translation stays readable without opening a second table.
                        </p>
                      </div>
                      <Pill tone="seed">Model translation</Pill>
                    </div>

                    <div className="mt-4 grid gap-4 md:grid-cols-2">
                      <div className="rounded-[20px] border border-zinc-200/80 bg-white/90 p-4 dark:border-zinc-800/80 dark:bg-zinc-950/70">
                        <div className="space-y-1">
                          <span className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                            Source model
                          </span>
                          <strong className="block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {rule.source_model_name}
                          </strong>
                          <p className="text-sm text-zinc-500 dark:text-zinc-400">
                            {rule.source_channel_name} / {rule.source_model_id}
                          </p>
                        </div>
                      </div>

                      <div className="rounded-[20px] border border-zinc-200/80 bg-white/90 p-4 dark:border-zinc-800/80 dark:bg-zinc-950/70">
                        <div className="space-y-1">
                          <span className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                            Target model
                          </span>
                          <strong className="block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {rule.target_model_name}
                          </strong>
                          <p className="text-sm text-zinc-500 dark:text-zinc-400">
                            {rule.target_channel_name} / {rule.target_model_id}
                          </p>
                        </div>
                      </div>
                    </div>
                  </article>
                ))}
              </div>
            ) : null}
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title="Delete model mapping"
        detail={
          pendingDelete
            ? `Delete ${pendingDelete.name}. Any Api key overlay using this mapping will be detached automatically.`
            : ''
        }
        confirmLabel="Delete mapping"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
