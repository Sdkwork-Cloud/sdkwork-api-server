import type { FormEvent, ReactNode } from 'react';
import { KeyRound, Shield, Sparkles } from 'lucide-react';
import { FormField, InlineButton } from 'sdkwork-router-portal-commons';

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

function TextInput({
  type = 'text',
  value,
  placeholder,
  className = '',
  autoComplete,
  spellCheck,
  onChange,
}: {
  type?: string;
  value: string;
  placeholder?: string;
  className?: string;
  autoComplete?: string;
  spellCheck?: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <input
      type={type}
      value={value}
      placeholder={placeholder}
      autoComplete={autoComplete}
      spellCheck={spellCheck}
      onChange={(event) => onChange(event.target.value)}
      className={`h-11 w-full rounded-2xl border border-zinc-200 bg-white px-4 text-sm text-zinc-950 outline-none transition placeholder:text-zinc-400 focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50 dark:placeholder:text-zinc-500 ${className}`.trim()}
    />
  );
}

function SelectInput({
  value,
  className = '',
  onChange,
  children,
}: {
  value: string;
  className?: string;
  onChange: (value: string) => void;
  children: ReactNode;
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className={`h-11 w-full rounded-2xl border border-zinc-200 bg-white px-4 text-sm text-zinc-950 outline-none transition focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50 ${className}`.trim()}
    >
      {children}
    </select>
  );
}

function TextArea({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
}) {
  return (
    <textarea
      rows={5}
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      className="w-full rounded-2xl border border-zinc-200 bg-white px-4 py-3 text-sm text-zinc-950 outline-none transition placeholder:text-zinc-400 focus:border-primary-500/35 focus:ring-2 focus:ring-primary-500/20 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50 dark:placeholder:text-zinc-500"
    />
  );
}

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
    <button
      type="button"
      onClick={onClick}
      className={
        selected
          ? 'rounded-[24px] border border-primary-500/35 bg-primary-500/8 p-4 text-left transition shadow-[0_12px_30px_rgba(59,130,246,0.10)]'
          : 'rounded-[24px] border border-zinc-200 bg-white p-4 text-left transition hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-950 dark:hover:border-zinc-700'
      }
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
    </button>
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
            <TextInput
              placeholder="Production rollout"
              value={formState.label}
              onChange={(value) =>
                onChange((current) => ({ ...current, label: value }))
              }
            />
          </FormField>

          <FormField
            hint="Choose which workspace boundary this key should protect."
            label="Environment boundary"
          >
            <SelectInput
              value={formState.environment}
              onChange={(value) =>
                onChange((current) => ({ ...current, environment: value }))
              }
            >
              {environmentOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </SelectInput>
          </FormField>

          {formState.environment === 'custom' ? (
            <div className="lg:col-span-2">
              <FormField
                hint="Examples: canary, partner, sandbox-eu"
                label="Custom environment"
              >
                <TextInput
                  placeholder="Custom environment"
                  value={formState.customEnvironment}
                  onChange={(value) =>
                    onChange((current) => ({ ...current, customEnvironment: value }))
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
                <TextInput
                  autoComplete="off"
                  className="font-mono"
                  placeholder="skw_live_custom_portal_secret"
                  spellCheck={false}
                  value={formState.customKey}
                  onChange={(value) =>
                    onChange((current) => ({ ...current, customKey: value }))
                  }
                />
              </FormField>
            </div>
          )}

          <FormField
            hint="Optional. Leave empty to keep this key active until you revoke it."
            label="Expires at"
          >
            <TextInput
              type="date"
              value={formState.expiresAt}
              onChange={(value) =>
                onChange((current) => ({ ...current, expiresAt: value }))
              }
            />
          </FormField>

          <FormField
            hint="Add operator context, ownership, or rollout details for future review."
            label="Notes"
          >
            <TextArea
              placeholder="Operator-managed migration key"
              value={formState.notes}
              onChange={(value) =>
                onChange((current) => ({ ...current, notes: value }))
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
