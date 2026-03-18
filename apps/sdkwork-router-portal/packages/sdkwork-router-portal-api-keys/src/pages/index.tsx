import { useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import {
  copyText,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  EmptyState,
  FormField,
  formatDateTime,
  InlineButton,
  Input,
  MetricCard,
  Pill,
  Select,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

import { ApiKeyEnvironmentSummaryGrid } from '../components';
import {
  issuePortalApiKey,
  loadPortalApiKeys,
  removePortalApiKey,
  setPortalApiKeyActive,
} from '../repository';
import { buildPortalApiKeysViewModel } from '../services';
import type { PortalApiKeysPageProps } from '../types';

function parseExpiryInput(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Date.parse(trimmed);
  return Number.isNaN(parsed) ? null : parsed;
}

function formatTimestamp(value?: number | null): string {
  return value === null || value === undefined ? 'Not yet' : formatDateTime(value);
}

function formatExpiry(value?: number | null): string {
  return value === null || value === undefined ? 'No expiry' : formatDateTime(value);
}

export function PortalApiKeysPage({ onNavigate }: PortalApiKeysPageProps) {
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [environment, setEnvironment] = useState('live');
  const [label, setLabel] = useState('Production rollout');
  const [expiresAt, setExpiresAt] = useState('');
  const [createdKey, setCreatedKey] = useState<CreatedGatewayApiKey | null>(null);
  const [status, setStatus] = useState('Loading issued keys...');
  const [submitting, setSubmitting] = useState(false);
  const [mutatingKey, setMutatingKey] = useState<string | null>(null);
  const [copyStatus, setCopyStatus] = useState('Plaintext keys are only shown once at creation time.');
  const [dialogOpen, setDialogOpen] = useState(false);

  async function refresh() {
    const keys = await loadPortalApiKeys();
    setApiKeys(keys);
  }

  useEffect(() => {
    let cancelled = false;

    void refresh()
      .then(() => {
        if (!cancelled) {
          setStatus('Credential inventory is synced with the latest project key state.');
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!label.trim()) {
      setStatus('Key label is required so credentials remain auditable after creation.');
      return;
    }

    const expiresAtMs = parseExpiryInput(expiresAt);
    if (expiresAt && expiresAtMs === null) {
      setStatus('Expires at must be a valid date-time value.');
      return;
    }

    setSubmitting(true);
    setStatus(`Issuing a ${environment} key for this workspace...`);

    try {
      const nextKey = await issuePortalApiKey({
        environment,
        label,
        expires_at_ms: expiresAtMs,
      });
      await refresh();
      setCreatedKey(nextKey);
      setCopyStatus('Copy the plaintext secret now. It will not be shown again after you leave this screen.');
      setStatus(`Key issued for ${environment}. Copy the plaintext secret before leaving this page.`);
      setDialogOpen(false);
      setExpiresAt('');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleCopyPlaintext() {
    if (!createdKey) {
      return;
    }

    const copied = await copyText(createdKey.plaintext);
    setCopyStatus(copied ? 'Plaintext key copied to clipboard.' : 'Clipboard copy is unavailable in this browser context.');
  }

  async function handleKeyStatusChange(key: GatewayApiKeyRecord, active: boolean) {
    setMutatingKey(key.hashed_key);
    setStatus(`${active ? 'Restoring' : 'Revoking'} ${key.label}...`);

    try {
      await setPortalApiKeyActive(key.hashed_key, active);
      await refresh();
      setStatus(
        active
          ? `${key.label} is active again and can authenticate gateway traffic.`
          : `${key.label} has been revoked and will no longer authenticate requests.`,
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setMutatingKey(null);
    }
  }

  async function handleDeleteKey(key: GatewayApiKeyRecord) {
    setMutatingKey(key.hashed_key);
    setStatus(`Deleting ${key.label}...`);

    try {
      await removePortalApiKey(key.hashed_key);
      await refresh();
      if (createdKey?.hashed === key.hashed_key) {
        setCreatedKey(null);
      }
      setStatus(`${key.label} was deleted from this workspace.`);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setMutatingKey(null);
    }
  }

  const viewModel = useMemo(
    () => buildPortalApiKeysViewModel(apiKeys, createdKey),
    [apiKeys, createdKey],
  );

  const activeKeys = apiKeys.filter((item) => item.active).length;
  const environmentsCovered = viewModel.environment_summaries.filter((item) => item.active > 0).length;
  const expiringSoon = apiKeys.filter((item) => item.expires_at_ms && item.expires_at_ms < Date.now() + 1000 * 60 * 60 * 24 * 30).length;

  return (
    <>
      <div className="portalx-status-row">
        <Pill tone="accent">Credential inventory</Pill>
        <span className="portal-shell-status-copy text-sm">{status}</span>
        <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
          Open usage
        </InlineButton>
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <button className="inline-flex h-10 items-center justify-center rounded-2xl bg-primary-600 px-4 text-sm font-medium text-white shadow-[0_16px_30px_rgb(var(--portal-accent-rgb)_/_0.22)] transition hover:bg-primary-500" type="button">
              Create key
            </button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create key dialog</DialogTitle>
              <DialogDescription>
                Issue a scoped credential without forcing the full creation form to live inline on the page.
              </DialogDescription>
            </DialogHeader>
            <form className="grid gap-4" onSubmit={handleSubmit}>
              <FormField hint="Keep labels auditable for incident review and rotation." label="Key label">
                <Input
                  onChange={(event) => setLabel(event.target.value)}
                  placeholder="Production rollout"
                  value={label}
                />
              </FormField>
              <FormField label="Environment">
                <Select value={environment} onChange={(event) => setEnvironment(event.target.value)}>
                  <option value="live">live</option>
                  <option value="staging">staging</option>
                  <option value="test">test</option>
                </Select>
              </FormField>
              <FormField hint="Optional bounded lifetime for safer credential hygiene." label="Expires at">
                <Input
                  onChange={(event) => setExpiresAt(event.target.value)}
                  type="datetime-local"
                  value={expiresAt}
                />
              </FormField>
              <DialogFooter>
                <InlineButton onClick={() => setDialogOpen(false)} tone="ghost" type="button">
                  Cancel
                </InlineButton>
                <InlineButton tone="primary" type="submit">
                  {submitting ? 'Issuing key...' : 'Create key'}
                </InlineButton>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </div>

        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <MetricCard detail="Issued gateway credentials currently visible in this workspace." label="Total keys" value={String(apiKeys.length)} />
        <MetricCard detail="Keys that can still authenticate live traffic." label="Active keys" value={String(activeKeys)} />
        <MetricCard detail="Environments currently protected by at least one active key." label="Environments covered" value={String(environmentsCovered)} />
        <MetricCard detail="Keys expiring in the next 30 days." label="Expiring soon" value={String(expiringSoon)} />
      </div>

      {createdKey ? (
        <Surface
          actions={
            <InlineButton disabled={!createdKey} onClick={handleCopyPlaintext} tone="secondary">
              Copy plaintext
            </InlineButton>
          }
          detail={copyStatus}
          title="Latest plaintext key"
        >
          <div className="grid gap-3 rounded-2xl border border-emerald-400/20 bg-emerald-400/8 p-4">
            <code className="portal-shell-code-surface text-sm text-emerald-100">{createdKey.plaintext}</code>
            <div className="grid gap-2 text-sm text-[var(--portal-text-secondary)] md:grid-cols-3">
              <span>Key label: {createdKey.label}</span>
              <span>Environment: {createdKey.environment}</span>
              <span>Expires at: {formatExpiry(createdKey.expires_at_ms)}</span>
            </div>
          </div>
        </Surface>
      ) : null}

      <Tabs defaultValue="inventory">
        <TabsList>
          <TabsTrigger value="inventory">Credential inventory</TabsTrigger>
          <TabsTrigger value="coverage">Environment strategy</TabsTrigger>
          <TabsTrigger value="rotation">Rotation checklist</TabsTrigger>
        </TabsList>

        <TabsContent value="inventory">
          <Surface
            detail="Write-only key secrets never reappear in list responses, so the inventory is optimized for governance, not secret replay."
            title="Issued keys"
          >
            {viewModel.keys.length ? (
              <DataTable
                columns={[
                  {
                    key: 'label',
                    label: 'Key label',
                    render: (row) => row.label,
                  },
                  {
                    key: 'environment',
                    label: 'Environment',
                    render: (row) => row.environment,
                  },
                  {
                    key: 'created_at_ms',
                    label: 'Created',
                    render: (row) => formatTimestamp(row.created_at_ms),
                  },
                  {
                    key: 'last_used_at_ms',
                    label: 'Last used',
                    render: (row) => formatTimestamp(row.last_used_at_ms),
                  },
                  {
                    key: 'expires_at_ms',
                    label: 'Expires at',
                    render: (row) => formatExpiry(row.expires_at_ms),
                  },
                  {
                    key: 'hashed',
                    label: 'Hashed Key',
                    render: (row) => <span className="portal-shell-status-copy text-xs">{row.hashed_key}</span>,
                  },
                  {
                    key: 'status',
                    label: 'Status',
                    render: (row) => <Pill tone={row.active ? 'positive' : 'warning'}>{row.active ? 'Active' : 'Inactive'}</Pill>,
                  },
                  {
                    key: 'actions',
                    label: 'Actions',
                    render: (row) => (
                      <div className="flex flex-wrap gap-2">
                        <InlineButton
                          disabled={mutatingKey === row.hashed_key}
                          onClick={() => handleKeyStatusChange(row, !row.active)}
                          tone="secondary"
                        >
                          {row.active ? 'Revoke' : 'Restore'}
                        </InlineButton>
                        <InlineButton
                          disabled={mutatingKey === row.hashed_key}
                          onClick={() => handleDeleteKey(row)}
                          tone="ghost"
                        >
                          Delete
                        </InlineButton>
                      </div>
                    ),
                  },
                ]}
                empty="No gateway keys issued yet."
                getKey={(row) => row.hashed_key}
                rows={viewModel.keys}
              />
            ) : (
              <EmptyState
                detail="Create your first key to connect a client or service to the SDKWork Router gateway."
                title="No API keys yet"
              />
            )}
          </Surface>
        </TabsContent>

        <TabsContent value="coverage">
          <div className="grid gap-6">
            {viewModel.environment_summaries.length ? (
              <Surface detail="A quick audit of key posture across environments." title="Environment coverage">
                <ApiKeyEnvironmentSummaryGrid summaries={viewModel.environment_summaries} />
              </Surface>
            ) : null}

            <div className="grid gap-6 xl:grid-cols-2">
              <Surface
                detail="A recommended rollout order for test, staging, and live credentials based on the current workspace posture."
                title="Environment strategy"
              >
                <div className="grid gap-3">
                  {viewModel.environment_strategy.map((item) => (
                    <article className="portal-shell-info-card" key={item.environment}>
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <strong className="portal-shell-info-title">{item.environment}</strong>
                          <p className="portal-shell-info-copy mt-2 text-sm">{item.detail}</p>
                        </div>
                        <Pill tone={item.recommended ? 'accent' : item.status === 'Covered' ? 'positive' : 'warning'}>
                          {item.recommended ? 'Recommended' : item.status}
                        </Pill>
                      </div>
                    </article>
                  ))}
                </div>
              </Surface>

              <Surface
                detail="The portal keeps secret lifecycle guidance visible so new keys turn into safe operational habits."
                title="Key handling guardrails"
              >
                <div className="grid gap-3">
                  {viewModel.guardrails.map((guardrail) => (
                    <article className="portal-shell-info-card" key={guardrail.id}>
                      <div className="flex items-center justify-between gap-3">
                        <strong className="portal-shell-info-title">{guardrail.title}</strong>
                        <Pill tone={guardrail.tone}>{guardrail.tone}</Pill>
                      </div>
                      <p className="portal-shell-info-copy mt-2 text-sm">{guardrail.detail}</p>
                    </article>
                  ))}
                </div>
              </Surface>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="rotation">
          <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
            <Surface
              detail="Use this flow after each new key issuance so environment cutovers stay deliberate and reversible."
              title="Rotation checklist"
            >
              <ol className="grid gap-3">
                {viewModel.rotation_checklist.map((item) => (
                  <li className="portal-shell-info-card" key={item.id}>
                    <strong className="portal-shell-info-title">{item.title}</strong>
                    <p className="portal-shell-info-copy mt-2 text-sm">{item.detail}</p>
                  </li>
                ))}
              </ol>
            </Surface>

            <div className="grid gap-6">
              {viewModel.quickstart_snippet ? (
                <Surface detail="A first authenticated call using the freshly created key." title="Quickstart snippet">
                  <div className="portal-shell-code-surface">
                    <code className="text-sm text-sky-100">{viewModel.quickstart_snippet}</code>
                  </div>
                </Surface>
              ) : null}

              <Surface
                detail="The best next action depends on whether you want to validate traffic, review global posture, or protect quota before launch."
                title="Recommended next move"
              >
                <div className="grid gap-3">
                  <article className="portal-shell-info-card">
                    <strong className="portal-shell-info-title">Validate the new credential with live telemetry</strong>
                    <p className="portal-shell-info-copy mt-2 text-sm">After issuing a key, send a small authenticated request so Usage can confirm model, provider, and token-unit visibility.</p>
                    <div className="mt-4">
                      <InlineButton onClick={() => onNavigate('usage')} tone="primary">
                        Open usage
                      </InlineButton>
                    </div>
                  </article>
                  <article className="portal-shell-info-card">
                    <strong className="portal-shell-info-title">Return to workspace posture</strong>
                    <p className="portal-shell-info-copy mt-2 text-sm">Go back to Dashboard to confirm keys, traffic, and launch readiness stay aligned.</p>
                    <div className="mt-4">
                      <InlineButton onClick={() => onNavigate('dashboard')} tone="secondary">
                        Open dashboard
                      </InlineButton>
                    </div>
                  </article>
                  <article className="portal-shell-info-card">
                    <strong className="portal-shell-info-title">Protect runway before production rollout</strong>
                    <p className="portal-shell-info-copy mt-2 text-sm">If key issuance is happening right before a launch window, confirm credits posture before promoting traffic.</p>
                    <div className="mt-4">
                      <InlineButton onClick={() => onNavigate('credits')} tone="ghost">
                        Review credits
                      </InlineButton>
                    </div>
                  </article>
                </div>
              </Surface>
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </>
  );
}
