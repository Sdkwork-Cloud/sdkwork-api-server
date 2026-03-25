import type { FormEvent } from 'react';
import { KeyRound, Link2 } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  InlineButton,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

import type { PortalApiKeyCreateFormState, PortalApiKeyUsagePreview } from '../types';
import type {
  ApiKeyQuickSetupPlan,
  ApiKeySetupClientId,
  ApiKeySetupInstance,
} from '../services/quickSetup';
import { PortalApiKeyCreateForm } from './PortalApiKeyCreateForm';

export function PortalApiKeyDialogs({
  createFormState,
  createOpen,
  createdKey,
  onChangeForm,
  onCloseCreate,
  onCloseUsage,
  onCopyPlaintext,
  onCreate,
  onApplySetup,
  onChangeInstanceSelection,
  onSelectClient,
  submitting,
  applyingClientId,
  gatewayBaseUrl,
  loadingInstances,
  openClawInstances,
  quickSetupPlans,
  selectedClientId,
  selectedInstanceIds,
  selectedPlan,
  usagePlaintext,
  usageStatus,
  usageKey,
  usagePreview,
}: {
  createFormState: PortalApiKeyCreateFormState;
  createOpen: boolean;
  createdKey: CreatedGatewayApiKey | null;
  onChangeForm: (updater: (current: PortalApiKeyCreateFormState) => PortalApiKeyCreateFormState) => void;
  onCloseCreate: () => void;
  onCloseUsage: () => void;
  onCopyPlaintext: () => void;
  onCreate: (event: FormEvent<HTMLFormElement>) => void;
  onApplySetup: () => void;
  onChangeInstanceSelection: (nextValue: string[]) => void;
  onSelectClient: (clientId: ApiKeySetupClientId) => void;
  submitting: boolean;
  applyingClientId: ApiKeySetupClientId | null;
  gatewayBaseUrl: string;
  loadingInstances: boolean;
  openClawInstances: ApiKeySetupInstance[];
  quickSetupPlans: ApiKeyQuickSetupPlan[];
  selectedClientId: ApiKeySetupClientId;
  selectedInstanceIds: string[];
  selectedPlan: ApiKeyQuickSetupPlan | null;
  usagePlaintext: string | null;
  usageStatus: string;
  usageKey: GatewayApiKeyRecord | null;
  usagePreview: PortalApiKeyUsagePreview | null;
}) {
  const isLatestUsageKey = createdKey && usageKey ? createdKey.hashed === usageKey.hashed_key : false;

  return (
    <>
      <Dialog onOpenChange={(open) => !open && onCloseCreate()} open={createOpen}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>Create API key</DialogTitle>
            <DialogDescription>
              Recommended key setup starts with Key label ownership, any needed
              Custom environment override, and the Lifecycle policy that matches
              the rollout plan.
            </DialogDescription>
          </DialogHeader>

          <PortalApiKeyCreateForm
            formState={createFormState}
            onChange={onChangeForm}
            onSubmit={onCreate}
            submitting={submitting}
          />
        </DialogContent>
      </Dialog>

      <Dialog onOpenChange={(open) => !open && onCloseUsage()} open={Boolean(usageKey)}>
        <DialogContent className="max-w-5xl">
          <DialogHeader>
            <DialogTitle>{usagePreview?.title ?? 'Usage method'}</DialogTitle>
            <DialogDescription>{usagePreview?.detail}</DialogDescription>
          </DialogHeader>

          {usageKey && usagePreview ? (
            <div className="space-y-6">
              <section className="rounded-[28px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                  <div>
                    <div className="text-xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                      {usageKey.label}
                    </div>
                    <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                      Use this key for the {usageKey.environment} environment boundary and keep
                      rollout verification inside the same workspace posture.
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <span className="inline-flex items-center rounded-full border border-primary-500/20 bg-primary-500/10 px-3 py-1 text-xs font-semibold text-primary-600 dark:text-primary-300">
                      {usageKey.environment}
                    </span>
                    <span
                      className={
                        usageKey.active
                          ? 'inline-flex items-center rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs font-semibold text-emerald-700 dark:text-emerald-300'
                          : 'inline-flex items-center rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-semibold text-amber-700 dark:text-amber-300'
                      }
                    >
                      {usageKey.active ? 'Active' : 'Inactive'}
                    </span>
                    {usagePlaintext || isLatestUsageKey ? (
                      <InlineButton onClick={onCopyPlaintext} tone="secondary">
                        Copy plaintext
                      </InlineButton>
                    ) : null}
                  </div>
                </div>
              </section>

              <div className="grid gap-5 xl:grid-cols-3">
                <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="flex items-center gap-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    <Link2 className="h-4 w-4 text-primary-500" />
                    Portal endpoint
                  </div>
                  <div className="mt-3 break-all text-sm text-zinc-600 dark:text-zinc-300">
                    {gatewayBaseUrl}/v1/models
                  </div>
                </article>

                <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="flex items-center gap-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    <KeyRound className="h-4 w-4 text-primary-500" />
                    Authorization header
                  </div>
                  <div className="mt-3 break-all text-sm text-zinc-600 dark:text-zinc-300">
                    {usagePreview.authorizationHeader ??
                      'Plaintext unavailable. Rotate this key to obtain a new one-time secret.'}
                  </div>
                </article>

                <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    Expires at
                  </div>
                  <div className="mt-3 text-sm text-zinc-600 dark:text-zinc-300">
                    {usageKey.expires_at_ms
                      ? `This credential expires on ${new Intl.DateTimeFormat('en-US', {
                          year: 'numeric',
                          month: 'short',
                          day: 'numeric',
                        }).format(new Date(usageKey.expires_at_ms))}.`
                      : 'This credential has no expiry. Keep revocation ownership explicit.'}
                  </div>
                </article>
              </div>

              <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  How to use this key
                </div>
                <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  Use this key for the {usageKey.environment} environment boundary and keep rollout
                  verification inside the same workspace posture. If the plaintext is no longer
                  visible, create a replacement instead of depending on the UI as secret storage.
                </p>
              </article>

              <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  Quick setup
                </div>
                <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  Apply setup directly on this device for Codex, Claude Code, OpenCode, Gemini,
                  or OpenClaw, or copy the generated snippets into your preferred environment.
                </p>

                <Tabs className="mt-4" value={selectedClientId} onValueChange={(value) => onSelectClient(value as ApiKeySetupClientId)}>
                  <TabsList>
                    {quickSetupPlans.map((plan) => (
                      <TabsTrigger key={plan.id} value={plan.id}>
                        {plan.label}
                      </TabsTrigger>
                    ))}
                  </TabsList>

                  {quickSetupPlans.map((plan) => (
                    <TabsContent key={plan.id} value={plan.id}>
                      <div className="space-y-4">
                        <div className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {plan.label}
                          </div>
                          <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                            {plan.description}
                          </p>
                        </div>

                        {plan.requiresInstances ? (
                          <div className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                            <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                              OpenClaw instances
                            </div>
                            {loadingInstances ? (
                              <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                                Loading local instances...
                              </p>
                            ) : openClawInstances.length ? (
                              <div className="mt-3 grid gap-2">
                                {openClawInstances.map((instance) => (
                                  <label key={instance.id} className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300">
                                    <input
                                      checked={selectedInstanceIds.includes(instance.id)}
                                      onChange={(event) =>
                                        onChangeInstanceSelection(
                                          event.target.checked
                                            ? [...selectedInstanceIds, instance.id]
                                            : selectedInstanceIds.filter((item) => item !== instance.id),
                                        )
                                      }
                                      type="checkbox"
                                    />
                                    <span>
                                      {instance.label}
                                      {instance.detail ? ` · ${instance.detail}` : ''}
                                    </span>
                                  </label>
                                ))}
                              </div>
                            ) : (
                              <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                                No OpenClaw instances were detected on this machine.
                              </p>
                            )}
                          </div>
                        ) : null}

                        {plan.snippets.map((snippet) => (
                          <div
                            key={snippet.id}
                            className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60"
                          >
                            <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                              {snippet.title}
                            </div>
                            <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
                              {snippet.target}
                            </p>
                            <pre className="mt-3 overflow-x-auto rounded-2xl bg-zinc-950 p-4 text-sm leading-6 text-zinc-300">
                              <code>{snippet.content}</code>
                            </pre>
                          </div>
                        ))}
                      </div>
                    </TabsContent>
                  ))}
                </Tabs>

                <DialogFooter className="mt-4">
                  {usagePlaintext || isLatestUsageKey ? (
                    <InlineButton onClick={onCopyPlaintext} tone="secondary">
                      Copy plaintext
                    </InlineButton>
                  ) : null}
                  <InlineButton
                    disabled={
                      !selectedPlan ||
                      !usagePlaintext ||
                      applyingClientId === selectedClientId ||
                      (selectedPlan.requiresInstances && !selectedInstanceIds.length)
                    }
                    onClick={onApplySetup}
                  >
                    {applyingClientId === selectedClientId ? 'Applying...' : 'Apply setup'}
                  </InlineButton>
                </DialogFooter>
              </article>

              {usageKey.notes ? (
                <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    Notes
                  </div>
                  <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {usageKey.notes}
                  </p>
                </article>
              ) : null}

              {usageStatus ? (
                <article className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    Status
                  </div>
                  <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {usageStatus}
                  </p>
                </article>
              ) : null}

              {usagePreview.curlExample ? (
                <article className="rounded-[24px] border border-zinc-200 bg-zinc-950 p-5 dark:border-zinc-800">
                  <div className="text-sm font-semibold text-zinc-100">Quickstart snippet</div>
                  <pre className="mt-4 overflow-x-auto text-sm leading-6 text-zinc-300">
                    <code>{usagePreview.curlExample}</code>
                  </pre>
                </article>
              ) : null}
            </div>
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  );
}
