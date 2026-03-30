import type { FormEvent } from 'react';
import { KeyRound, Shield, Sparkles } from 'lucide-react';
import {
  Button,
  FormField,
  InlineButton,
  Input,
  Select,
  Textarea,
} from 'sdkwork-router-portal-commons';

import type {
  PortalApiKeyCreateFormState,
  PortalApiKeyCreateMode,
} from '../types';

const environmentOptions = [
  { value: 'live', label: 'live' },
  { value: 'staging', label: 'staging' },
  { value: 'test', label: 'test' },
  { value: 'custom', label: 'Custom environment' },
] as const;

const keyModeOptions: Array<{
  id: PortalApiKeyCreateMode;
  title: string;
  detail: string;
  icon: typeof Sparkles;
}> = [
  {
    id: 'system-generated',
    title: 'System generated',
    detail: 'Let Portal create a one-time plaintext secret that is stored in write-only mode.',
    icon: Sparkles,
  },
  {
    id: 'custom',
    title: 'Custom key',
    detail: 'Provide an exact plaintext value when rollout coordination requires a predefined credential.',
    icon: KeyRound,
  },
];

function SelectionCard({
  title,
  detail,
  selected,
  icon: Icon,
  onClick,
}: {
  title: string;
  detail: string;
  selected: boolean;
  icon?: typeof Sparkles;
  onClick: () => void;
}) {
  return (
    <Button
      type="button"
      onClick={onClick}
      className={
        selected
          ? 'h-auto w-full justify-start whitespace-normal rounded-[24px] border border-primary-500/35 bg-primary-500/8 p-4 text-left shadow-[0_12px_30px_rgba(59,130,246,0.10)] hover:bg-primary-500/10'
          : 'h-auto w-full justify-start whitespace-normal rounded-[24px] border border-zinc-200 bg-white p-4 text-left shadow-none hover:border-zinc-300 hover:bg-white dark:border-zinc-800 dark:bg-zinc-950 dark:hover:border-zinc-700 dark:hover:bg-zinc-950'
      }
      variant="ghost"
    >
      <div className="flex items-start gap-3">
        {Icon ? (
          <span
            className={
              selected
                ? 'inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-primary-500 text-white'
                : 'inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-zinc-100 text-zinc-600 dark:bg-zinc-900 dark:text-zinc-300'
            }
          >
            <Icon className="h-4 w-4" />
          </span>
        ) : null}
        <div>
          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">{title}</div>
          <p className="mt-1 text-xs leading-6 text-zinc-500 dark:text-zinc-400">{detail}</p>
        </div>
      </div>
    </Button>
  );
}

export function PortalApiKeyCreateForm({
  formState,
  onChange,
  onSubmit,
  submitting,
}: {
  formState: PortalApiKeyCreateFormState;
  onChange: (updater: (current: PortalApiKeyCreateFormState) => PortalApiKeyCreateFormState) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  submitting: boolean;
}) {
  const isSystemGenerated = formState.keyMode === 'system-generated';

  return (
    <form className="space-y-6" onSubmit={onSubmit}>
      <section className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
        <div className="grid gap-5 lg:grid-cols-2">
          <FormField
            hint="Keep labels auditable for incident review, ownership review, and future rotation."
            label="Key label"
          >
            <Input
              placeholder="Production rollout"
              value={formState.label}
              onChange={(event) =>
                onChange((current) => ({ ...current, label: event.target.value }))
              }
            />
          </FormField>

          <FormField
            hint="Choose which workspace boundary this key should protect."
            label="Environment boundary"
          >
            <Select
              value={formState.environment}
              onChange={(event) =>
                onChange((current) => ({ ...current, environment: event.target.value }))
              }
            >
              {environmentOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </Select>
          </FormField>

          {formState.environment === 'custom' ? (
            <div className="lg:col-span-2">
              <FormField
                hint="Examples: canary, partner, sandbox-eu"
                label="Custom environment"
              >
                <Input
                  placeholder="Custom environment"
                  value={formState.customEnvironment}
                  onChange={(event) =>
                    onChange((current) => ({ ...current, customEnvironment: event.target.value }))
                  }
                />
              </FormField>
            </div>
          ) : null}

          <div className="space-y-2 lg:col-span-2">
            <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
              Gateway key mode
            </div>
            <p className="text-xs leading-6 text-zinc-500 dark:text-zinc-400">
              Choose whether Portal generates the secret or stores a custom plaintext value for this workspace boundary.
            </p>
            <div className="grid gap-3 md:grid-cols-2">
              {keyModeOptions.map((option) => (
                <SelectionCard
                  key={option.id}
                  title={option.title}
                  detail={option.detail}
                  icon={option.icon}
                  selected={formState.keyMode === option.id}
                  onClick={() =>
                    onChange((current) => ({
                      ...current,
                      keyMode: option.id,
                      customKey: option.id === current.keyMode ? current.customKey : '',
                    }))
                  }
                />
              ))}
            </div>
          </div>

          {isSystemGenerated ? (
            <div className="lg:col-span-2 rounded-[24px] border border-primary-500/15 bg-primary-500/8 p-4 dark:border-primary-500/20 dark:bg-primary-500/10">
              <div className="flex items-start gap-3">
                <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-primary-500 text-white">
                  <Shield className="h-4 w-4" />
                </span>
                <div className="min-w-0">
                  <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    Portal-managed key
                  </div>
                  <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                    Portal will generate a one-time plaintext secret, persist only the hashed value, and reveal the plaintext once after creation.
                  </p>
                  <div className="mt-3 rounded-2xl border border-dashed border-primary-500/25 bg-white/70 px-3 py-3 text-sm text-zinc-600 dark:border-primary-500/20 dark:bg-zinc-950/50 dark:text-zinc-300">
                    A one-time plaintext key will be revealed after creation.
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="lg:col-span-2">
              <FormField
                hint="Paste the exact plaintext value that should be stored in write-only mode."
                label="API key"
              >
                <Input
                  autoComplete="off"
                  className="font-mono"
                  placeholder="skw_live_custom_portal_secret"
                  spellCheck={false}
                  value={formState.customKey}
                  onChange={(event) =>
                    onChange((current) => ({ ...current, customKey: event.target.value }))
                  }
                />
              </FormField>
            </div>
          )}

          <FormField
            hint="Optional. Leave empty to keep this key active until you revoke it."
            label="Expires at"
          >
            <Input
              type="date"
              value={formState.expiresAt}
              onChange={(event) =>
                onChange((current) => ({ ...current, expiresAt: event.target.value }))
              }
            />
          </FormField>

          <FormField
            hint="Add operator context, ownership, or rollout details for future review."
            label="Notes"
          >
            <Textarea
              rows={5}
              placeholder="Operator-managed migration key"
              value={formState.notes}
              onChange={(event) =>
                onChange((current) => ({ ...current, notes: event.target.value }))
              }
            />
          </FormField>
        </div>
      </section>

      <div className="flex flex-wrap justify-end gap-3">
        <InlineButton disabled={submitting} tone="primary" type="submit">
          {submitting ? 'Creating API key...' : 'Create API key'}
        </InlineButton>
      </div>
    </form>
  );
}
